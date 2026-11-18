//! contract: omnicui/contracts/richtexteditor.contract.yaml (id: ui.richtexteditor). The one rich-text body: a plain-text editor with a formatting bar (under the box on focus, or docked above it), @mention attachments as chips with an @ list, drop-to-attach, and the counter near its limit.
use crate::omnicui::charactercounter::CharactercounterProps;
use crate::omnicui::richtexteditor::{parts, RichtexteditorEvent, RichtexteditorProps};
use crate::widgets::charactercounter::CharacterCounter;
use crate::{icon, part, rem, weight, Look};
use egui::{
    pos2, vec2, Align2, Color32, CornerRadius, EventFilter, FontFamily, FontId, Id, Key, Modifiers, Rect, Role, Sense,
    Shape, Stroke, StrokeKind, WidgetInfo,
};

/// The formatting buttons in focus order: bold, italic, underline, strikethrough, bulleted list,
/// numbered list, then the link button. The first six are toggles.
const BUTTONS: [(&str, &str, &str); 7] = [
    ("format_bold", "Bold", parts::BOLD),
    ("format_italic", "Italic", parts::ITALIC),
    ("format_underlined", "Underline", parts::UNDERLINE),
    ("strikethrough_s", "Strikethrough", parts::STRIKETHROUGH),
    ("format_list_bulleted", "Bulleted list", parts::BULLETLIST),
    ("format_list_numbered", "Numbered list", parts::ORDEREDLIST),
    ("link", "Link", parts::LINK),
];
/// The floating bar's count ("n/limit"): no part of this widget's contract, named for a screen to rename.
pub const BAR_COUNT: &str = "ui.richtexteditor.barCount";

/// A toolbar button's square side: the web's Toggle sm, h-6 (21 px).
const BUTTON: f32 = rem(1.5);
/// The corner radius of a control, in px.
const RADIUS: u8 = 5;

#[derive(Clone, serde::Deserialize)]
struct Mention {
    id: String,
    name: String,
}

/// The open @ list: where the "@" is (a char index), the attachments that match, and the highlighted one.
#[derive(Clone)]
struct Menu {
    at: usize,
    items: Vec<Mention>,
    index: usize,
}

pub struct RichTextEditor {
    p: RichtexteditorProps,
    /// The plain text, paragraphs separated by '\n', soft breaks as SOFT_BREAK.
    text: String,
    /// The caret as a char index, as the editor reported it at the end of the last frame.
    cursor: usize,
    /// The char indices of the '\n' that are Shift+Enter line breaks rather than paragraph breaks.
    soft: Vec<usize>,
    /// Attachments inserted as mention chips, drawn in a row above the text.
    chips: Vec<Mention>,
    /// Where each chip was put in the text (a char index), so the text as read reads "…@name " there.
    chip_at: Vec<usize>,
    /// The attachments the @ list offers, parsed from the `mentions` JSON.
    mentions: Vec<Mention>,
    /// Bold, italic, underline, strikethrough, bulleted list, numbered list.
    marks: [bool; 6],
    menu: Option<Menu>,
    /// The "@" (char index) that Escape closed; the list stays closed for it.
    dismissed: Option<usize>,
    collapsed: bool,
    started: bool,
    /// Whether the widget had focus at the end of the last frame.
    focus_prev: bool,
    /// Where the @ list was drawn last frame: a press on it takes focus from the text for a moment,
    /// and the list must stay open until the press lands on a row.
    list_rect: Option<egui::Rect>,
    /// The formatting: a style per character, links, a kind per paragraph (the text and line breaks are
    /// `text` and `soft`). Read from the note's HTML, kept through edits, written back by `html()`.
    rich: crate::rich::Rich,
}

impl RichTextEditor {
    pub fn new(p: RichtexteditorProps) -> Self {
        let mentions = serde_json::from_str::<Vec<Mention>>(&p.mentions).unwrap_or_default();
        // A note may be the reference's HTML: its text is shown, its formatting kept.
        let rich = crate::rich::parse(&p.value);
        let text = rich.text.clone();
        let soft = rich.soft.clone();
        let cursor = text.chars().count();
        Self {
            p,
            text,
            cursor,
            soft,
            chips: vec![],
            chip_at: vec![],
            mentions,
            marks: [false; 6],
            menu: None,
            dismissed: None,
            collapsed: false,
            started: false,
            focus_prev: false,
            list_rect: None,
            rich,
        }
    }

    /// One frame: draws the editor and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<RichtexteditorEvent> {
        let mut events = vec![];
        if self.collapsed {
            return events;
        }
        let t = look.t;
        let disabled = self.p.disabled;
        let docked = self.p.toolbar == "docked";
        let max_len = self.p.maxLength.max(0) as usize;
        let base = ui.scope_id().with("richtexteditor");
        let editor_id = base.with("editor");
        let n = BUTTONS.len();

        if !self.started {
            self.started = true;
            if self.p.autoFocus && !disabled {
                ui.memory_mut(|m| m.request_focus(editor_id));
            }
        }

        // Before the editor reads the input: keys it must not see, and the drops.
        let focused = ui.memory(|m| m.has_focus(editor_id));
        let bar_focus = (0..n).find(|&i| ui.memory(|m| m.has_focus(button_id(base, i))));
        let shift_enter = focused && ui.input(|i| i.key_pressed(Key::Enter) && i.modifiers.shift);
        let cursor_before = self.cursor;
        let mut pick: Option<usize> = None;
        let mut inserted = false;

        if !disabled {
            // A file dropped on the widget becomes a mention chip, not text.
            let dropped: Vec<String> = ui.input(|i| {
                i.raw
                    .dropped_files
                    .iter()
                    .map(|f| {
                        // The attachment's file name is the last part of the dropped path.
                        let path = f.path();
                        path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
                    })
                    .collect()
            });
            if !dropped.is_empty() {
                for file in dropped {
                    self.chips.push(Mention { id: file.clone(), name: file.clone() });
                    self.chip_at.push(usize::MAX);
                    events.push(RichtexteditorEvent::AttachmentDropped { file });
                }
                ui.memory_mut(|m| m.request_focus(editor_id));
            }

            if focused {
                for (key, i) in [(Key::B, 0), (Key::I, 1), (Key::U, 2)] {
                    if take_cmd(ui, key) {
                        self.toggle_mark(ui.ctx(), editor_id, i);
                    }
                }
                if take_cmd(ui, Key::Enter) && self.p.submitOnModEnter {
                    events.push(RichtexteditorEvent::Submitted {});
                }

                // The @ list takes the arrows, Enter and Escape while it is open.
                let mut escape_done = false;
                if self.menu.is_some() {
                    let len = self.menu.as_ref().map_or(1, |m| m.items.len());
                    if take(ui, Modifiers::NONE, Key::ArrowDown) {
                        if let Some(m) = self.menu.as_mut() {
                            m.index = (m.index + 1) % len;
                        }
                    }
                    if take(ui, Modifiers::NONE, Key::ArrowUp) {
                        if let Some(m) = self.menu.as_mut() {
                            m.index = (m.index + len - 1) % len;
                        }
                    }
                    if take(ui, Modifiers::NONE, Key::Enter) {
                        pick = self.menu.as_ref().map(|m| m.index);
                    }
                    if take(ui, Modifiers::NONE, Key::Escape) {
                        if let Some(m) = self.menu.take() {
                            self.dismissed = Some(m.at);
                        }
                        escape_done = true;
                    }
                }
                if !escape_done && self.p.escapeCancels && take(ui, Modifiers::NONE, Key::Escape) {
                    events.push(RichtexteditorEvent::Cancelled {});
                }

                // Tab leaves the widget from the editor; Shift+Tab goes to the first button of the bar.
                if take(ui, Modifiers::SHIFT, Key::Tab) {
                    ui.memory_mut(|m| m.request_focus(button_id(base, 0)));
                } else if take(ui, Modifiers::NONE, Key::Tab) {
                    ui.memory_mut(|m| m.surrender_focus(editor_id));
                }

                // A chip is one unit: Backspace at the start of the text takes the last chip.
                if self.cursor == 0 && !self.chips.is_empty() && take(ui, Modifiers::NONE, Key::Backspace) {
                    self.chips.pop();
                    self.chip_at.pop();
                }
            }

            // Within the bar: arrows move between the buttons, Tab returns to the editor.
            if let Some(i) = bar_focus {
                if take(ui, Modifiers::NONE, Key::ArrowRight) {
                    ui.memory_mut(|m| m.request_focus(button_id(base, (i + 1) % n)));
                }
                if take(ui, Modifiers::NONE, Key::ArrowLeft) {
                    ui.memory_mut(|m| m.request_focus(button_id(base, (i + n - 1) % n)));
                }
                if take(ui, Modifiers::NONE, Key::Tab) || take(ui, Modifiers::SHIFT, Key::Tab) {
                    ui.memory_mut(|m| m.request_focus(editor_id));
                }
            }
        }

        // The floating bar is drawn while the editor (or its bar) has focus; a disabled one keeps its buttons.
        let show_bar = docked || focused || bar_focus.is_some() || disabled;
        let aria = self.p.ariaLabel.clone();

        ui.vertical(|ui| {
            if docked && show_bar {
                self.bar(ui, look, base, editor_id, disabled, docked, &mut events);
            }

            // The widget's top: the floating bar sits above it, clear of the chips.
            let widget_top = ui.cursor().top();
            // The attachments, as chips above the text.
            if !self.chips.is_empty() {
                let chips = self.chips.clone();
                ui.horizontal_wrapped(|ui| {
                    for chip in &chips {
                        let label = format!("@{}", chip.name);
                        let font = FontId::new(rem(0.875), weight(500));
                        let w = ui.painter().layout_no_wrap(label.clone(), font.clone(), t.accent_600).size().x + 16.0;
                        let (r, resp) = ui.allocate_exact_size(vec2(w, rem(1.714)), Sense::click());
                        let radius = CornerRadius::same((rem(1.714) / 2.0) as u8);
                        ui.painter().rect_filled(r, radius, t.selection_1);
                        if resp.has_focus() {
                            ui.painter().rect_stroke(r.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
                        }
                        ui.painter().text(
                            r.left_center() + vec2(8.0, 0.0),
                            Align2::LEFT_CENTER,
                            label.as_str(),
                            font,
                            t.accent_600,
                        );
                        resp.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, label.as_str()));
                        part(ui, &resp, parts::MENTIONCHIP);
                        if resp.clicked() {
                            events.push(RichtexteditorEvent::MentionOpened { id: chip.id.clone() });
                        }
                    }
                });
            }

            // The box: a TextEdit with its own frame off, drawn over a background set after it.
            // The web's heights: sm at least 3.5rem and md at least 6rem, both growing with the note; capped
            // 4rem growing to 10rem, then scrolling. Only capped scrolls: a growing box follows its content
            // (a scroll area would take the height its parent row offers, and egui's 64 px minimum).
            let (rows, max_h) = match self.p.size.as_str() {
                "md" => (4, f32::INFINITY),
                "capped" => (2, rem(10.0)),
                _ => (2, f32::INFINITY),
            };
            let scrolls = max_h.is_finite();
            let bg = ui.painter().add(Shape::Noop);
            let old_chars: Vec<char> = self.text.chars().collect();
            let scroll = egui::ScrollArea::vertical()
                .id_salt(base.with("scroll"))
                .max_height(max_h)
                .vscroll(scrolls)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    // The reference's px-3 py-2.5 padding (12 px across, 10 down; the web had px-2), laid out by hand
                    // so it holds whatever the TextEdit's margin does (#12487).
                    ui.horizontal(|ui| {
                        ui.add_space(12.0);
                        ui.vertical(|ui| {
                            // The top padding, laid out: a frameless TextEdit leaves its margin out (the
                            // text sat on the box's top edge, #12487).
                            ui.add_space(10.0);
                            let styles = self.rich.styles.clone();
                            let kinds = self.rich.kinds.clone();
                            let soft_now = self.soft.clone();
                            let typing = self.marks;
                            let ink = if disabled { t.content_2 } else { t.content_1 };
                            let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap: f32| {
                                let job = styled_job(buf.as_str(), &styles, &kinds, &soft_now, typing, ink, wrap);
                                ui.fonts_mut(|f| f.layout_job(job))
                            };
                            egui::TextEdit::multiline(&mut self.text)
                                .id(editor_id)
                                .layouter(&mut layouter)
                                .font(FontId::new(rem(0.9643), weight(400)))
                                .text_color(if disabled { t.content_2 } else { t.content_1 })
                                .frame(egui::Frame::NONE)
                                .margin(egui::Margin { left: 0, right: 0, top: 10, bottom: 10 })
                                .desired_width(ui.available_width() - 12.0)
                                .desired_rows(rows)
                                .char_limit(max_len)
                                .interactive(!disabled)
                                .show(ui)
                        })
                        .inner
                    })
                    .inner
                });
            let out = scroll.inner;
            // List items: their bullet or number in the indent the layout left before them.
            {
                let chars: Vec<char> = self.text.chars().collect();
                let mut starts = vec![0usize];
                for (i, c) in chars.iter().enumerate() {
                    if *c == '\n' && !self.soft.contains(&i) {
                        starts.push(i + 1);
                    }
                }
                let mut number = 0;
                for (pi, start) in starts.iter().enumerate() {
                    let kind = self.rich.kinds.get(pi).copied().unwrap_or(crate::rich::PARA);
                    number = if kind == crate::rich::NUMBER { number + 1 } else { 0 };
                    if kind == crate::rich::PARA {
                        continue;
                    }
                    let at = out.galley.pos_from_cursor(egui::text::CCursor::new(*start)).translate(out.galley_pos.to_vec2());
                    let mark = if kind == crate::rich::BULLET { "•".to_owned() } else { format!("{number}.") };
                    ui.painter().text(
                        pos2(at.left() - 9.0, at.center().y),
                        Align2::CENTER_CENTER,
                        mark,
                        FontId::new(rem(0.9643), weight(400)),
                        if disabled { t.content_2 } else { t.content_1 },
                    );
                }
            }
            // The box: the area's width, down to the text's bottom margin (a growing box has no scroll height).
            let frame_rect = if scrolls {
                scroll.inner_rect
            } else {
                Rect::from_min_max(scroll.inner_rect.min, pos2(scroll.inner_rect.max.x.max(out.response.rect.max.x), out.response.rect.max.y + 10.0))
            };
            let focus_now = out.response.has_focus();
            // The editor keeps Tab, the arrows and Escape: they are its keys, and the handling decides them.
            // Set after the TextEdit, which sets its own filter while it is shown.
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    editor_id,
                    EventFilter { horizontal_arrows: true, vertical_arrows: true, tab: true, escape: true },
                )
            });

            part(ui, &out.response, parts::EDITOR);
            let label = aria.clone();
            let read = self.read_text();
            ui.ctx().accesskit_node_builder(out.response.id, |node| {
                node.set_label(label.clone());
                // The text as a reader (and a test) sees it: each mention where it was put, as "@name ",
                // the way the web's inline chip reads.
                node.set_value(read.clone());
                if disabled {
                    node.set_disabled();
                }
            });

            let radius = CornerRadius::same(RADIUS);
            let shapes = vec![
                Shape::rect_filled(frame_rect, radius, if disabled { t.surface_2 } else { t.surface_3 }),
                // One ring: the border turns into the 2 px focus ring (a border plus a ring outside it read as
                // two rings, #12380).
                Shape::rect_stroke(
                    frame_rect,
                    radius,
                    if focus_now { Stroke::new(2.0, t.focus_1) } else { Stroke::new(1.0, t.edge_2) },
                    StrokeKind::Inside,
                ),
            ];
            ui.painter().set(bg, Shape::Vec(shapes));

            // The placeholder: only while the text is empty, so it is absent once typing starts.
            if self.text.is_empty() && !self.p.placeholder.is_empty() {
                let text = self.p.placeholder.clone();
                let origin = out.galley_pos;
                // Wrapped to the box, as the reference's placeholder wraps (it ran past the right edge, cut off),
                // on the note's 20 px line.
                let wrap = (frame_rect.right() - 12.0 - origin.x).max(0.0);
                let mut job = egui::text::LayoutJob::single_section(
                    text.clone(),
                    egui::TextFormat { font_id: FontId::new(rem(0.9643), weight(400)), color: t.content_3, line_height: Some(20.0), ..Default::default() },
                );
                job.wrap.max_width = wrap;
                let galley = ui.painter().layout_job(job);
                let area = Rect::from_min_size(origin, vec2(wrap, galley.size().y));
                let ph = ui.interact(area, base.with("placeholder"), Sense::hover());
                ph.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, text.as_str()));
                part(ui, &ph, parts::PLACEHOLDER);
                ui.painter().galley(origin, galley, t.content_3);
            }

            // The caret, the text change and the @ list.
            self.cursor = out.cursor_range.map_or(self.cursor, |r| r.primary.index.into());
            let changed = out.response.changed();
            if changed {
                let new_chars: Vec<char> = self.text.chars().collect();
                let (p, r, k) = diff(&old_chars, &new_chars);
                let soft_old = self.soft.clone();
                let typing = (0..4).filter(|&i| self.marks[i]).fold(0u8, |b, i| b | (1 << i));
                let soft_typed = shift_enter && new_chars.get(cursor_before) == Some(&'\n');
                crate::rich::splice(&mut self.rich, &old_chars, &new_chars, p, r, k, typing, &soft_old, soft_typed);
                shift_soft(&mut self.soft, p, r, k);
                if shift_enter && new_chars.get(cursor_before) == Some(&'\n') {
                    // The newline just typed by Shift+Enter is a line break, not a new paragraph.
                    self.soft.push(cursor_before);
                    self.soft.sort_unstable();
                    self.soft.dedup();
                }
                // One change per character typed, each with the text as it stood after that character.
                let steps = k.max(1);
                for j in 1..=steps {
                    let text: String = if k == 0 {
                        self.text.clone()
                    } else {
                        new_chars[..p + j].iter().chain(new_chars[p + k..].iter()).collect()
                    };
                    let below = self.soft.iter().filter(|&&s| s < text.chars().count()).count();
                    let paragraphs = paragraphs_of(&text, below);
                    events.push(RichtexteditorEvent::Changed { value: text, paragraphs });
                }
            }
            let on_list = self.list_rect.is_some_and(|r| {
                ui.input(|i| i.pointer.interact_pos().is_some_and(|p| r.contains(p)) && (i.pointer.any_down() || i.pointer.any_released()))
            });
            self.scan(focus_now || on_list);
            if let Some(i) = pick.take() {
                self.insert(i, ui.ctx(), editor_id);
                inserted = true;
            }

            // The floating bar is a popover 6 px above the box (the reference's), over whatever is there.
            if !docked && show_bar {
                let bar_h = BUTTON + 2.0 * rem(0.125);
                let at = pos2(frame_rect.left(), widget_top.min(frame_rect.top()) - 6.0 - bar_h);
                egui::Area::new(base.with("floating-bar")).order(egui::Order::Foreground).fixed_pos(at).show(ui.ctx(), |ui| {
                    self.bar(ui, look, base, editor_id, disabled, docked, &mut events);
                });
            }

            // The @ list: drawn only while open, so it is absent when closed.
            let mut list_pick: Option<usize> = None;
            if let Some(menu) = self.menu.clone() {
                let list_bg = ui.painter().add(Shape::Noop);
                let list = ui.scope(|ui| {
                    for (i, item) in menu.items.iter().enumerate() {
                        let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), rem(1.5)), Sense::click());
                        let selected = i == menu.index;
                        if selected {
                            ui.painter().rect_filled(r, CornerRadius::same(RADIUS), t.wash_2);
                        }
                        ui.painter().text(
                            pos2(r.left() + 8.0, r.center().y),
                            Align2::LEFT_CENTER,
                            item.name.as_str(),
                            FontId::new(rem(0.9643), weight(400)),
                            t.content_1,
                        );
                        resp.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, item.name.as_str()));
                        part(ui, &resp, parts::MENTIONOPTION);
                        if resp.clicked() {
                            list_pick = Some(i);
                        }
                    }
                });
                let list_rect = list.response.rect;
                self.list_rect = Some(list_rect);
                ui.painter().set(
                    list_bg,
                    Shape::Vec(vec![
                        Shape::rect_filled(list_rect, CornerRadius::same(RADIUS), t.surface_3),
                        Shape::rect_stroke(
                            list_rect,
                            CornerRadius::same(RADIUS),
                            Stroke::new(1.0, t.edge_2),
                            StrokeKind::Inside,
                        ),
                    ]),
                );
                list.response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Mentions"));
                part(ui, &list.response, parts::MENTIONLIST);
            }
            if self.menu.is_none() {
                self.list_rect = None;
            }
            if let Some(i) = list_pick {
                self.insert(i, ui.ctx(), editor_id);
                // The text takes focus back, with the caret after the mention.
                ui.memory_mut(|m| m.request_focus(editor_id));
                inserted = true;
            }

            // The counter, once the text is near its limit.
            let count = self.text.chars().count();
            if count as f64 > 0.9 * max_len as f64 {
                let json = serde_json::json!({ "count": count, "limit": max_len, "nearRatio": 0.9 });
                if let Ok(p) = serde_json::from_value::<CharactercounterProps>(json) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        CharacterCounter::new(p).ui(ui, look);
                    });
                }
            }
        });

        // The blur: focus left the whole widget (the bar included).
        let within = ui.memory(|m| m.has_focus(editor_id))
            || (0..n).any(|i| ui.memory(|m| m.has_focus(button_id(base, i))));
        if self.focus_prev && !within {
            events.push(RichtexteditorEvent::Blurred { value: self.value() });
            if self.p.collapseWhenEmpty && self.text.is_empty() {
                self.collapsed = true;
            }
            ui.ctx().request_repaint();
        }
        self.focus_prev = within;
        if inserted {
            ui.ctx().request_repaint();
        }
        events
    }

    /// The formatting bar: seven buttons, each a toggle or the link. Floating hangs under the box with
    /// a border; docked is a row with a top rule.
    #[allow(clippy::too_many_arguments)]
    fn bar(
        &mut self,
        ui: &mut egui::Ui,
        look: Look,
        base: Id,
        editor_id: Id,
        disabled: bool,
        docked: bool,
        events: &mut Vec<RichtexteditorEvent>,
    ) {
        let t = look.t;
        let n = BUTTONS.len() as f32;
        let (gap, pad_x, pad_y) = if docked {
            (rem(0.25), 8.0, rem(0.375))
        } else {
            (rem(0.125), rem(0.125), rem(0.125))
        };
        // The floating bar ends with the count, "n/limit", as the reference's.
        let count_text = (!docked && self.p.maxLength > 0).then(|| format!("{}/{}", self.text.chars().count(), self.p.maxLength));
        let count_w = count_text.as_ref().map_or(0.0, |c| ui.painter().layout_no_wrap(c.clone(), FontId::new(11.0, weight(500)), Color32::WHITE).size().x + 14.0);
        let width = if docked { ui.available_width() } else { n * BUTTON + (n - 1.0) * gap + 2.0 * pad_x + count_w };
        let height = BUTTON + 2.0 * pad_y;
        let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let bar = ui.interact(rect, base.with("bar"), Sense::hover());
        bar.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, "Formatting"));
        part(ui, &bar, parts::FORMATTING);
        if docked {
            ui.painter().hline(rect.x_range(), rect.top() + 0.5, Stroke::new(1.0, t.edge_1));
        } else {
            ui.painter().rect(
                rect,
                CornerRadius::same(7),
                t.surface_3,
                Stroke::new(1.0, t.edge_2),
                StrokeKind::Inside,
            );
        }

        let mut x = rect.left() + pad_x;
        for (i, &(icon_name, label, part_id)) in BUTTONS.iter().enumerate() {
            let r = Rect::from_min_size(pos2(x, rect.top() + pad_y), vec2(BUTTON, BUTTON));
            let id = button_id(base, i);
            let sense = if disabled { Sense::hover() } else { Sense::click() };
            let resp = ui.interact(r, id, sense);
            // Arrows and Tab are the bar's: egui must not move focus by them; the editor handles them.
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    EventFilter { horizontal_arrows: true, vertical_arrows: true, tab: true, escape: false },
                )
            });
            let on = self.marks.get(i).copied().unwrap_or(false);
            let hovered = resp.hovered() && !disabled;
            let fill = if on {
                t.wash_2
            } else if hovered {
                t.wash_1
            } else {
                Color32::TRANSPARENT
            };
            let ink = if disabled {
                t.content_3
            } else if on || hovered {
                t.content_1
            } else {
                t.content_2
            };
            ui.painter().rect_filled(r, CornerRadius::same(RADIUS), fill);
            ui.painter().text(
                r.center(),
                Align2::CENTER_CENTER,
                icon(icon_name, 15.0).text(),
                FontId::new(15.0, FontFamily::Name("Icons".into())),
                ink,
            );
            if resp.has_focus() {
                ui.painter().rect_stroke(
                    r.expand(2.0),
                    CornerRadius::same(RADIUS),
                    Stroke::new(2.0, t.focus_1),
                    StrokeKind::Outside,
                );
            }
            if i < 6 {
                resp.widget_info(|| WidgetInfo::selected(Role::Button, !disabled, on, label));
                ui.ctx().accesskit_node_builder(resp.id, |node| node.set_selected(on));
            } else {
                resp.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, label));
            }
            part(ui, &resp, part_id);

            // A press keeps focus in the editor, as the web's buttons do.
            if resp.clicked() && !disabled {
                match i {
                    // With text selected, the style goes on (or off) the selection; else it is what typing gets.
                    0..=3 => self.toggle_mark(ui.ctx(), editor_id, i),
                    4 => {
                        self.marks[4] = !self.marks[4];
                        if self.marks[4] {
                            self.marks[5] = false;
                        }
                        self.set_kind(if self.marks[4] { crate::rich::BULLET } else { crate::rich::PARA });
                    }
                    5 => {
                        self.marks[5] = !self.marks[5];
                        if self.marks[5] {
                            self.marks[4] = false;
                        }
                        self.set_kind(if self.marks[5] { crate::rich::NUMBER } else { crate::rich::PARA });
                    }
                    _ => events.push(RichtexteditorEvent::LinkRequested {}),
                }
                ui.memory_mut(|m| m.request_focus(editor_id));
            }
            x += BUTTON + gap;
        }
        if let Some(c) = count_text {
            ui.painter().vline(x + 2.0, (rect.top() + 6.0)..=(rect.bottom() - 6.0), Stroke::new(1.0, t.edge_2));
            let r = ui.painter().text(pos2(x + 9.0, rect.center().y), Align2::LEFT_CENTER, &c, FontId::new(11.0, weight(500)), t.content_3);
            let node = ui.interact(r, base.with("bar-count"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &c));
            part(ui, &node, BAR_COUNT);
        }
    }

    /// The text with each mention chip read in place as "@name " (chips dropped on the widget, which
    /// have no place in the text, come at the end).
    fn read_text(&self) -> String {
        let mut chars: Vec<char> = self.text.chars().collect();
        let mut placed: Vec<(usize, String)> = self.chips.iter().enumerate().map(|(i, c)| (self.chip_at.get(i).copied().unwrap_or(usize::MAX).min(chars.len()), format!("@{} ", c.name))).collect();
        placed.sort_by(|a, b| b.0.cmp(&a.0));
        for (at, label) in placed {
            let at = at.min(chars.len());
            chars.splice(at..at, label.chars());
        }
        chars.into_iter().collect()
    }

    /// The text as the value: paragraphs and soft breaks both as '\n'.
    fn value(&self) -> String {
        self.text.clone()
    }

    /// The note as the reference stores it: HTML with its formatting, lists, links and line breaks.
    pub fn html(&self) -> String {
        let mut r = self.rich.clone();
        r.text = self.text.clone();
        r.soft = self.soft.clone();
        crate::rich::to_html(&r)
    }

    /// Bold, italic, underline or strikethrough: on the selection when there is one, else for what is typed next.
    fn toggle_mark(&mut self, ctx: &egui::Context, editor_id: Id, i: usize) {
        let selection = egui::TextEdit::load_state(ctx, editor_id)
            .and_then(|st| st.cursor.char_range())
            .map(|r| {
                let (a, b): (usize, usize) = (r.primary.index.into(), r.secondary.index.into());
                (a.min(b), a.max(b))
            })
            .filter(|(a, b)| a < b);
        match selection {
            Some((a, b)) => self.marks[i] = crate::rich::toggle(&mut self.rich, a, b, 1 << i),
            None => self.marks[i] = !self.marks[i],
        }
    }

    /// Sets the kind (paragraph or list item) of the paragraph the caret is in.
    fn set_kind(&mut self, kind: u8) {
        let chars: Vec<char> = self.text.chars().collect();
        let para = chars[..self.cursor.min(chars.len())].iter().enumerate().filter(|(i, c)| **c == '\n' && !self.soft.contains(i)).count();
        if let Some(k) = self.rich.kinds.get_mut(para) {
            *k = kind;
        }
    }

    /// Finds an "@query" before the caret, and opens the list with the attachments that match it.
    fn scan(&mut self, focused: bool) {
        let prev = self.menu.take();
        let chars: Vec<char> = self.text.chars().collect();
        let c = self.cursor.min(chars.len());
        let mut at = None;
        let mut i = c;
        while i > 0 {
            let ch = chars[i - 1];
            if ch == '@' {
                at = Some(i - 1);
                break;
            }
            if ch.is_whitespace() {
                break;
            }
            i -= 1;
        }
        let Some(a) = at else {
            self.dismissed = None;
            return;
        };
        if !focused || (a > 0 && !chars[a - 1].is_whitespace()) || self.dismissed == Some(a) {
            return;
        }
        let needle: String = chars[a + 1..c].iter().collect::<String>().to_lowercase();
        let items: Vec<Mention> = self
            .mentions
            .iter()
            .filter(|m| m.name.to_lowercase().contains(&needle))
            .cloned()
            .collect();
        if items.is_empty() {
            return;
        }
        let index = match prev {
            Some(p) if p.at == a => p.index.min(items.len() - 1),
            _ => 0,
        };
        self.menu = Some(Menu { at: a, items, index });
    }

    /// Puts the attachment in as a chip and takes the "@query" out of the text.
    fn insert(&mut self, i: usize, ctx: &egui::Context, editor: Id) {
        let Some(menu) = self.menu.take() else {
            return;
        };
        let Some(item) = menu.items.get(i).cloned() else {
            return;
        };
        let from = byte_at(&self.text, menu.at);
        let to = byte_at(&self.text, self.cursor.max(menu.at));
        shift_soft(&mut self.soft, menu.at, self.cursor.max(menu.at) - menu.at, 0);
        self.text.replace_range(from..to, "");
        self.chips.push(item);
        self.chip_at.push(menu.at);
        self.dismissed = None;
        self.cursor = menu.at;
        set_cursor(ctx, editor, menu.at);
    }
}

crate::mounted!(RichTextEditor, RichtexteditorProps);

fn button_id(base: Id, i: usize) -> Id {
    base.with(("button", i))
}

/// Consumes a key press with these modifiers, so the editor and the focus navigation do not see it.
fn take(ui: &egui::Ui, modifiers: Modifiers, key: Key) -> bool {
    ui.input_mut(|i| i.consume_key(modifiers, key))
}

/// Consumes a press of `key` with the platform modifier: Command on macOS, Control elsewhere. The
/// modifier is read from the held state or from the event itself, whichever the platform reports.
fn take_cmd(ui: &egui::Ui, key: Key) -> bool {
    ui.input_mut(|i| {
        let held = i.modifiers.command || i.modifiers.ctrl || i.modifiers.mac_cmd;
        let found = i.events.iter().position(|e| {
            matches!(
                e,
                egui::Event::Key { key: k, pressed: true, modifiers, .. }
                    if *k == key && (held || modifiers.command || modifiers.ctrl || modifiers.mac_cmd)
            )
        });
        match found {
            Some(idx) => {
                i.events.remove(idx);
                true
            }
            None => false,
        }
    })
}

/// The shared prefix and suffix of two texts, as (prefix, removed from old, inserted into new).
fn diff(old: &[char], new: &[char]) -> (usize, usize, usize) {
    let max = old.len().min(new.len());
    let mut p = 0;
    while p < max && old[p] == new[p] {
        p += 1;
    }
    let mut s = 0;
    while s < max - p && old[old.len() - 1 - s] == new[new.len() - 1 - s] {
        s += 1;
    }
    (p, old.len() - p - s, new.len() - p - s)
}

/// Moves the soft-break indices across an edit that replaced `r` chars at `p` with `k` chars.
fn shift_soft(soft: &mut Vec<usize>, p: usize, r: usize, k: usize) {
    soft.retain_mut(|idx| {
        if *idx < p {
            true
        } else if *idx < p + r {
            false
        } else {
            *idx = *idx - r + k;
            true
        }
    });
}

/// One more than the paragraph breaks, less the soft breaks among them.
fn paragraphs_of(text: &str, soft_below: usize) -> i64 {
    1 + text.matches('\n').count() as i64 - soft_below as i64
}

/// The byte offset of a char index (the end of the string when it is past it).
fn byte_at(s: &str, index: usize) -> usize {
    s.char_indices().nth(index).map_or(s.len(), |(b, _)| b)
}

/// Moves the editor's caret to a char index.
fn set_cursor(ctx: &egui::Context, id: Id, index: usize) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    state
        .cursor
        .set_char_range(Some(egui::text_selection::CCursorRange::one(egui::text::CCursor::new(index))));
    state.store(ctx, id);
}

/// The text laid out with each character's style (bold, italic, underline, strikethrough), list items
/// indented. Characters past the known styles (typed this frame) take the typing style.
fn styled_job(text: &str, styles: &[u8], kinds: &[u8], soft: &[usize], typing: [bool; 6], ink: Color32, wrap: f32) -> egui::text::LayoutJob {
    use crate::rich::{BOLD, ITALIC, MENTION, PARA, STRIKE, UNDERLINE};
    let typed = (0..4).filter(|&i| typing[i]).fold(0u8, |b, i| b | (1 << i));
    let mut job = egui::text::LayoutJob::default();
    job.wrap.max_width = wrap;
    let chars: Vec<char> = text.chars().collect();
    let mut para = 0usize;
    let mut para_start = true;
    let mut i = 0;
    while i < chars.len() {
        let st = styles.get(i).copied().unwrap_or(typed);
        let mut j = i + 1;
        while j < chars.len() && styles.get(j).copied().unwrap_or(typed) == st && chars[j - 1] != '\n' {
            j += 1;
        }
        let run: String = chars[i..j].iter().collect();
        let kind = kinds.get(para).copied().unwrap_or(PARA);
        let mut format = egui::TextFormat {
            font_id: FontId::new(rem(0.9643), weight(if st & BOLD != 0 { 700 } else { 400 })),
            color: if st & MENTION != 0 { ink.gamma_multiply(0.85) } else { ink },
            italics: st & ITALIC != 0,
            // The reference's leading-5: a 20 px line.
            line_height: Some(20.0),
            ..Default::default()
        };
        if st & UNDERLINE != 0 {
            format.underline = Stroke::new(1.0, ink);
        }
        if st & STRIKE != 0 {
            format.strikethrough = Stroke::new(1.0, ink);
        }
        let lead = if para_start && kind != PARA { 18.0 } else { 0.0 };
        job.append(&run, lead, format);
        for (k, c) in chars[i..j].iter().enumerate() {
            if *c == '\n' && !soft.contains(&(i + k)) {
                para += 1;
            }
        }
        para_start = chars[j - 1] == '\n';
        i = j;
    }
    if chars.is_empty() {
        job.append("", 0.0, egui::TextFormat { font_id: FontId::new(rem(0.9643), weight(400)), color: ink, line_height: Some(20.0), ..Default::default() });
    }
    job
}