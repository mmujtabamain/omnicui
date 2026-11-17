//! contract: omnicui/contracts/tablist.contract.yaml (id: ui.tablist). A strip of list tabs, a title-bar row or a Settings column: the arrows, Home and End move focus and select the tab.
use crate::omnicui::tablist::{parts, TablistEvent, TablistProps};
use crate::{part, rem, weight, Look};
use egui::scroll_area::ScrollBarVisibility;
use egui::{pos2, vec2, Align, Align2, Color32, CornerRadius, FontId, Id, Key, Rect, Response, Role, ScrollArea, Sense, Stroke, StrokeKind, Vec2, WidgetInfo};

/// Tab text: Inter 13 px, medium.
const TAB_PX: f32 = 13.0;
const TAB_WEIGHT: u16 = 500;

pub struct TabList {
    p: TablistProps,
    /// The selected tab's id ("1".."N"), or "" for none.
    selected: String,
    /// Whether the focus came from the keyboard: a focus ring shows only then, as focus-visible does.
    keyboard: bool,
}

impl TabList {
    pub fn new(p: TablistProps) -> Self {
        let selected = p.value.clone();
        Self { p, selected, keyboard: true }
    }

    /// One frame: draws the strip and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TablistEvent> {
        let vertical = self.p.orientation == "vertical";
        let fit = self.p.overflow == "fit";
        let labels: Vec<String> = (1..=self.p.tabCount.max(0)).map(|i| format!("List {i}")).collect();
        let avail_w = ui.available_width();
        // The title-bar strip fills the bar's height, up to 36 px.
        let bar_h = if ui.available_height().is_finite() { ui.available_height().min(rem(2.571)) } else { rem(2.571) };
        let base = ui.scope_id().with("tablist");
        let (size, rects) = self.layout(ui, avail_w, bar_h, &labels);
        if fit {
            return self.strip(ui, look, &labels, &rects, size, base);
        }
        let scroll = if vertical { ScrollArea::vertical() } else { ScrollArea::horizontal() };
        scroll
            .id_salt(base)
            .auto_shrink([false, true])
            .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| self.strip(ui, look, &labels, &rects, size, base))
            .inner
    }

    /// The tab rectangles in the strip's own space, and the strip's size.
    fn layout(&self, ui: &egui::Ui, avail_w: f32, bar_h: f32, labels: &[String]) -> (Vec2, Vec<Rect>) {
        let n = labels.len();
        if self.p.orientation == "vertical" {
            // A column of 32 px rows with 4 px gaps, inset 2 px.
            let h = if n == 0 { 4.0 } else { 4.0 + n as f32 * 32.0 + (n - 1) as f32 * 4.0 };
            let rects: Vec<Rect> = (0..n)
                .map(|i| Rect::from_min_size(pos2(2.0, 2.0 + i as f32 * 36.0), vec2((avail_w - 4.0).max(0.0), 32.0)))
                .collect();
            return (vec2(avail_w, h), rects);
        }
        if self.p.overflow == "fit" {
            // Equal shares of the strip, text truncated.
            let w = avail_w / n.max(1) as f32;
            let rects: Vec<Rect> = (0..n).map(|i| Rect::from_min_size(pos2(i as f32 * w, 0.0), vec2(w, bar_h))).collect();
            return (vec2(avail_w, bar_h), rects);
        }
        // Scrolling row: each tab its text plus 12 px either side, no gaps.
        let font = FontId::new(TAB_PX, weight(TAB_WEIGHT));
        let mut x = 0.0_f32;
        let mut rects: Vec<Rect> = Vec::with_capacity(n);
        for label in labels {
            let text_w = ui.painter().layout_no_wrap(label.clone(), font.clone(), Color32::WHITE).size().x;
            let w = text_w + 24.0;
            rects.push(Rect::from_min_size(pos2(x, 0.0), vec2(w, bar_h)));
            x += w;
        }
        (vec2(x.max(avail_w), bar_h), rects)
    }

    /// Lays the strip out in the given space: the tablist node, the tabs, their input and their paint.
    fn strip(&mut self, ui: &mut egui::Ui, look: Look, labels: &[String], rects: &[Rect], size: Vec2, base: Id) -> Vec<TablistEvent> {
        let mut events = vec![];
        let t = look.t;
        let vertical = self.p.orientation == "vertical";
        let count = labels.len();

        // The strip: a generic container, named by its label.
        let (strip, list) = ui.allocate_exact_size(size, Sense::hover());
        let label = self.p.label.clone();
        list.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &label));
        part(ui, &list, parts::TABLIST);

        // The tabs, in screen space, each a click target.
        let tabs: Vec<Rect> = rects.iter().map(|r| r.translate(strip.min.to_vec2())).collect();
        let resps: Vec<Response> = tabs
            .iter()
            .enumerate()
            .map(|(i, r)| ui.interact(*r, base.with(("tab", i)), Sense::click()))
            .collect();
        for (i, r) in resps.iter().enumerate() {
            let slot = if i == 0 {
                Some(parts::FIRSTTAB)
            } else if i + 1 == count {
                Some(parts::LASTTAB)
            } else if i == 1 {
                Some(parts::SECONDTAB)
            } else {
                None
            };
            if let Some(id) = slot {
                part(ui, r, id);
            }
        }

        // Keys: the arrows of the strip's axis, Home and End. Tab leaves the strip.
        let (back, forward) = if vertical { (Key::ArrowUp, Key::ArrowDown) } else { (Key::ArrowLeft, Key::ArrowRight) };
        let (prev, next, home, end, tab) = ui.input(|i| {
            (
                i.key_pressed(back),
                i.key_pressed(forward),
                i.key_pressed(Key::Home),
                i.key_pressed(Key::End),
                i.key_pressed(Key::Tab),
            )
        });
        let cur = resps.iter().position(|r| r.has_focus());
        let wrap = self.p.r#loop;
        let mut moved: Option<usize> = None;
        if let Some(c) = cur {
            let last = count - 1;
            moved = if next {
                if c < last {
                    Some(c + 1)
                } else if wrap {
                    Some(0)
                } else {
                    None
                }
            } else if prev {
                if c > 0 {
                    Some(c - 1)
                } else if wrap {
                    Some(last)
                } else {
                    None
                }
            } else if home {
                Some(0)
            } else if end {
                Some(last)
            } else {
                None
            };
            if moved.is_some() {
                self.keyboard = true;
            }
            if tab {
                ui.memory_mut(|m| m.surrender_focus(resps[c].id));
            }
        }
        if let Some(n) = moved {
            resps[n].request_focus();
            ui.ctx().request_repaint();
            ui.scroll_to_rect(tabs[n], Some(Align::Center));
            self.select(n, &mut events);
        }
        // The focused tab keeps Tab and the arrows: egui's own focus navigation would otherwise move focus a second time.
        if let Some(f) = moved.or(cur) {
            keep_keys(ui, resps[f].id);
        }

        // A press selects the tab and takes focus (a press is not focus-visible).
        for (i, r) in resps.iter().enumerate() {
            if r.clicked() {
                self.keyboard = false;
                r.request_focus();
                self.select(i, &mut events);
            }
        }

        // Paint each tab: its node first, then its look.
        let font = FontId::new(TAB_PX, weight(TAB_WEIGHT));
        for (i, r) in resps.iter().enumerate() {
            let rect = tabs[i];
            let selected = self.selected == (i + 1).to_string();
            let name = labels[i].clone();
            r.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, &name));
            // The chosen tab carries selected on its node: the button role does not report it from the info alone.
            ui.ctx().accesskit_node_builder(r.id, |node| node.set_selected(selected));
            let hovered = r.hovered();
            let painter = ui.painter();
            if vertical {
                let fill = if selected {
                    Some(t.wash_2)
                } else if hovered {
                    Some(t.wash_1)
                } else {
                    None
                };
                if let Some(c) = fill {
                    painter.rect_filled(rect, CornerRadius::same(4), c);
                }
            }
            let color = if selected || hovered { t.content_1 } else { t.content_2 };
            let room = if vertical { rect.width() - 16.0 } else { rect.width() - 24.0 };
            let text_w = painter.layout_no_wrap(name.clone(), font.clone(), color).size().x;
            let clip = painter.with_clip_rect(rect);
            if vertical {
                clip.text(rect.left_center() + vec2(8.0, 0.0), Align2::LEFT_CENTER, name, font.clone(), color);
            } else if text_w <= room {
                clip.text(rect.center(), Align2::CENTER_CENTER, name, font.clone(), color);
            } else {
                clip.text(rect.left_center() + vec2(12.0, 0.0), Align2::LEFT_CENTER, name, font.clone(), color);
            }
            if !vertical && selected {
                let bar = Rect::from_min_max(pos2(rect.left() + 8.0, rect.bottom() - 2.0), pos2(rect.right() - 8.0, rect.bottom()));
                painter.rect_filled(bar, CornerRadius::same(1), t.accent_500);
            }
            if r.has_focus() && self.keyboard {
                let radius = if vertical { CornerRadius::same(4) } else { CornerRadius::same(0) };
                painter.rect_stroke(rect, radius, Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
            }
        }
        events
    }

    /// Selects tab `n` (index into the strip) and reports it, if it is not the selected tab already.
    fn select(&mut self, n: usize, events: &mut Vec<TablistEvent>) {
        let id = (n + 1).to_string();
        if id != self.selected {
            self.selected = id.clone();
            events.push(TablistEvent::ValueChanged { value: id });
        }
    }
}

/// Gives Tab and the horizontal and vertical arrows to the widget with this id, so egui's focus navigation leaves them alone.
fn keep_keys(ui: &egui::Ui, id: Id) {
    let filter = egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, ..Default::default() };
    ui.memory_mut(|m| m.set_focus_lock_filter(id, filter));
}

crate::mounted!(TabList, TablistProps);
