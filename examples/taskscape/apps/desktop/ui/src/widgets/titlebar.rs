//! contract: omnicui/contracts/titlebar.contract.yaml (id: ui.titlebar). The header strip of a window or pane: leading window controls, the project switcher and its menu, the title, the list tabs, a recording status and a trailing Discard button.
use crate::omnicui::button::ButtonProps;
use crate::omnicui::titlebar::{parts, TitlebarEvent, TitlebarProps};
use crate::widgets::button::Button;
use crate::{icon, part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{
    pos2, vec2, Align, Align2, Color32, CornerRadius, FontFamily, FontId, Galley, Key, Layout, Painter, Rect, Response, Role, Sense, Stroke, StrokeKind, UiBuilder, ViewportCommand, WidgetInfo,
};
use std::sync::Arc;

/// The project switcher's menu entries.
const PROJECTS: [&str; 3] = ["omnicui", "taskscape", "docs"];
/// The list tabs of the window scope.
const TABS: [&str; 4] = ["Tasks", "Notes", "Files", "History"];

pub struct TitleBar {
    p: TitlebarProps,
    /// The project menu is open: the props set it first, then the switcher and Escape.
    menu_open: bool,
    /// The window is maximised, as the last maximise toggle left it.
    maximized: bool,
    /// The selected list tab, an index into TABS.
    tab: usize,
    /// The Discard button of the trailing slot. None only if its constant props fail to build.
    trailing: Option<Button>,
}

impl TitleBar {
    pub fn new(p: TitlebarProps) -> Self {
        let menu_open = p.projectMenuOpen;
        let trailing = serde_json::from_value::<ButtonProps>(serde_json::json!({
            "label": "Discard",
            "variant": "outline",
            "tone": "strong",
            "width": "content",
        }))
        .ok()
        .map(Button::new);
        Self { p, menu_open, maximized: false, tab: 0, trailing }
    }

    /// One frame: draws the bar and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<TitlebarEvent> {
        let mut events = vec![];
        let p = self.p.clone();
        let t = look.t;
        let window = p.scope != "pane";
        let leading = p.controls == "leading";
        // Window chrome on desktop: the bar drags the window and a double-click maximises it.
        let drag = window && leading;
        let id = ui.id().with("titlebar");
        let sw_id = id.with("switcher");
        let font = FontId::new(13.0, weight(500));
        let gap = 6.0;
        let h = rem(3.286);

        // The bar first: its background, its bottom edge and its node. Its children come after, so they win clicks.
        let width = if ui.available_width().is_finite() { ui.available_width() } else { 1180.0 };
        let (bar, bar_resp) = ui.allocate_exact_size(vec2(width, h), Sense::click_and_drag());
        let cy = bar.center().y;
        ui.painter().rect_filled(bar, CornerRadius::same(0), t.surface_1);
        if p.scrolled {
            // The scrolled edge: a short shadow under the bar.
            for (k, a) in [0.4_f32, 0.25, 0.12, 0.05].into_iter().enumerate() {
                let y = bar.bottom() + k as f32;
                ui.painter().rect_filled(Rect::from_min_max(pos2(bar.left(), y), pos2(bar.right(), y + 1.0)), CornerRadius::same(0), t.edge_1.gamma_multiply(a));
            }
        }
        let border = if !window {
            t.edge_1
        } else if p.scrolled {
            t.edge_3
        } else {
            t.edge_2
        };
        ui.painter().line_segment([pos2(bar.left(), bar.bottom() - 0.5), pos2(bar.right(), bar.bottom() - 0.5)], Stroke::new(1.0, border));
        bar_resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &bar_resp, parts::BAR);

        // Window controls at the leading edge. A pane shows only the close light.
        let mut left = bar.left();
        let mut no_drag: Vec<Rect> = vec![];
        if leading {
            let lights: Vec<&str> = if window { vec!["close", "minimize", "maximize"] } else { vec!["close"] };
            let n = lights.len() as f32;
            let w = 12.0 + n * 12.0 + (n - 1.0) * 8.0 + 4.0;
            let group = Rect::from_min_size(bar.left_top(), vec2(w, h));
            let g = ui.interact(group, id.with("controls"), Sense::hover());
            g.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &g, parts::CONTROLS);
            no_drag.push(group);
            for (i, name) in lights.iter().enumerate() {
                let c = pos2(group.left() + 18.0 + i as f32 * 20.0, cy);
                let (label, light_id, tone) = match *name {
                    "close" => ("Close", parts::CLOSE, t.danger_500),
                    "minimize" => ("Minimize", parts::MINIMIZE, t.accent_500),
                    _ => ("Maximize", parts::MAXIMIZE, t.success_500),
                };
                let resp = ui.interact(Rect::from_center_size(c, vec2(12.0, 12.0)), id.with(*name), Sense::click());
                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, label));
                part(ui, &resp, light_id);
                let fill = if p.windowFocused { tone } else { t.content_3 };
                ui.painter().circle_filled(c, 6.0, fill);
                if resp.clicked() {
                    match *name {
                        "close" => ui.ctx().send_viewport_cmd(ViewportCommand::Close),
                        "minimize" => ui.ctx().send_viewport_cmd(ViewportCommand::Minimized(true)),
                        _ => self.toggle_maximize(ui.ctx(), &mut events),
                    }
                }
            }
            left = group.right();
        }

        // The trailing slot is right-aligned and 10 px in, so it is laid out right to left.
        // The recording status sits just left of it.
        let mut right = bar.right();
        if p.withTrailing {
            if let Some(b) = self.trailing.as_mut() {
                let slot = Rect::from_min_max(bar.left_top(), pos2(bar.right() - 10.0, bar.bottom()));
                let r = ui
                    .scope_builder(UiBuilder::new().max_rect(slot).layout(Layout::right_to_left(Align::Center)), |ui| {
                        ui.scope(|ui| {
                            b.ui(ui, look);
                        })
                        .response
                        .rect
                    })
                    .inner;
                let tr = ui.interact(r, id.with("trailing"), Sense::hover());
                tr.widget_info(|| WidgetInfo::labeled(Role::Button, true, "Discard"));
                part(ui, &tr, parts::TRAILING);
                no_drag.push(r);
                right = r.left();
            }
        }
        if p.recording {
            let rec_font = FontId::new(11.0, weight(500));
            let tw = ui.painter().layout_no_wrap("Recording".to_owned(), rec_font.clone(), t.danger_500).size().x;
            let w = 8.0 + 6.0 + 6.0 + tw + 8.0;
            let rec = Rect::from_min_size(pos2(right - w, bar.top()), vec2(w, h));
            let rs = ui.interact(rec, id.with("recording"), Sense::hover());
            rs.widget_info(|| WidgetInfo::labeled(Role::Status, true, "Recording"));
            part(ui, &rs, parts::RECORDINGSTATUS);
            ui.painter().circle_filled(pos2(rec.left() + 11.0, cy), 3.0, t.danger_500);
            ui.painter().text(pos2(rec.left() + 20.0, cy), Align2::LEFT_CENTER, "Recording", rec_font, t.danger_500);
            right = rec.left();
        }

        // The content slot fills the rest, with 10 px either side.
        let content = Rect::from_min_max(pos2(left, bar.top()), pos2(right.max(left), bar.bottom()));
        let cs = ui.interact(content, id.with("content"), Sense::hover());
        cs.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &cs, parts::CONTENTSLOT);
        let inner_l = left + 10.0;
        let inner_r = (right - 10.0).max(inner_l);

        // Widths of the items, so the title can give room to the switcher and the tabs.
        let sw_name = if p.projectName.is_empty() { "Project".to_owned() } else { p.projectName.clone() };
        let sw_w = if p.withProjectSwitcher { 8.0 + text_w(ui, &sw_name, &font) + 6.0 + 16.0 + 8.0 } else { 0.0 };
        let title_nat = text_w(ui, &p.title, &font);
        let tab_w: Vec<f32> = if p.withTabs {
            let u: &egui::Ui = ui;
            TABS.iter().map(|l| text_w(u, l, &font) + 24.0).collect()
        } else {
            vec![]
        };
        let tabs_nat: f32 = tab_w.iter().sum();
        let sw_gap = if sw_w > 0.0 { sw_w + gap } else { 0.0 };
        let tabs_gap = if p.withTabs { gap + tabs_nat } else { 0.0 };
        let room = (inner_r - inner_l - sw_gap - tabs_gap).max(0.0);
        let tw = title_nat.min(room);

        let mut x = inner_l;

        // The project switcher: a ghost button with the project's name and a chevron.
        let mut switcher: Option<Rect> = None;
        if p.withProjectSwitcher {
            let r = Rect::from_min_size(pos2(x, cy - 12.25), vec2(sw_w, 24.5));
            let sw = ui.interact(r, sw_id, Sense::click());
            sw.widget_info(|| WidgetInfo::labeled(Role::Button, true, &sw_name));
            part(ui, &sw, parts::PROJECTSWITCHER);
            if sw.clicked() {
                self.menu_open = !self.menu_open;
                events.push(TitlebarEvent::ProjectMenuChanged { open: self.menu_open });
            }
            let fill = if self.menu_open {
                Some(t.wash_2)
            } else if sw.hovered() {
                Some(t.wash_1)
            } else {
                None
            };
            let painter = ui.painter();
            if let Some(c) = fill {
                painter.rect_filled(r, CornerRadius::same(4), c);
            }
            painter.text(pos2(r.left() + 8.0, cy), Align2::LEFT_CENTER, sw_name.clone(), font.clone(), t.content_1);
            glyph(painter, "expand_more", pos2(r.right() - 16.0, cy), 16.0, t.content_1);
            if sw.has_focus() {
                painter.rect_stroke(r, CornerRadius::same(4), Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
            }
            no_drag.push(r);
            switcher = Some(r);
            x += sw_w + gap;
        }

        // The title: a heading when it is h1 or h2, else a generic box. It shrinks before the tabs do.
        let title_rect = Rect::from_min_size(pos2(x, cy - 7.0), vec2(tw, 14.0));
        let role = if p.titleAs == "h1" || p.titleAs == "h2" { Role::Heading } else { Role::GenericContainer };
        let tn = ui.interact(title_rect, id.with("title"), Sense::hover());
        tn.widget_info(|| WidgetInfo::labeled(role, true, &p.title));
        part(ui, &tn, parts::TITLE);
        if tw > 0.0 && !p.title.is_empty() {
            let g = line(ui, &p.title, font.clone(), t.content_1, tw);
            ui.painter().galley(pos2(x, cy - g.size().y / 2.0), g, t.content_1);
        }
        x += tw;

        // The list tabs: a row of tabs, the chosen one underlined in accent.
        if p.withTabs {
            x += gap;
            let region = Rect::from_min_max(pos2(x, bar.top()), pos2(inner_r.max(x), bar.bottom()));
            let tabs = ui.interact(region, id.with("tabs"), Sense::hover());
            tabs.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &tabs, parts::TABS);
            no_drag.push(region);

            let mut rects: Vec<Rect> = vec![];
            let mut tx = x;
            for w in &tab_w {
                rects.push(Rect::from_min_size(pos2(tx, bar.top()), vec2(*w, h)));
                tx += *w;
            }
            let resps: Vec<Response> = rects.iter().enumerate().map(|(i, r)| ui.interact(*r, id.with(("tab", i)), Sense::click())).collect();

            // Left and Right move focus along the tabs and choose the tab they reach.
            let (prev, next) = ui.input(|i| (i.key_pressed(Key::ArrowLeft), i.key_pressed(Key::ArrowRight)));
            if let Some(c) = resps.iter().position(|r| r.has_focus()) {
                let target = if next {
                    (c + 1).min(TABS.len() - 1)
                } else if prev {
                    c.saturating_sub(1)
                } else {
                    c
                };
                if target != c {
                    resps[target].request_focus();
                    self.tab = target;
                }
            }
            for (i, r) in resps.iter().enumerate() {
                if r.clicked() {
                    self.tab = i;
                    r.request_focus();
                }
            }

            for (i, r) in resps.iter().enumerate() {
                let rect = rects[i];
                let label = TABS[i];
                let selected = self.tab == i;
                r.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
                ui.ctx().accesskit_node_builder(r.id, |node| node.set_selected(selected));
                let color = if selected || r.hovered() { t.content_1 } else { t.content_2 };
                ui.painter().with_clip_rect(region).text(rect.center(), Align2::CENTER_CENTER, label, font.clone(), color);
                if selected {
                    let bar_rect = Rect::from_min_max(pos2(rect.left() + 8.0, rect.bottom() - 2.0), pos2(rect.right() - 8.0, rect.bottom()));
                    ui.painter().rect_filled(bar_rect, CornerRadius::same(1), t.accent_500);
                }
                if r.has_focus() {
                    ui.painter().rect_stroke(rect, CornerRadius::same(0), Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
                }
            }
        }

        // The project menu: a popup below the switcher. It shuts on a pick, on Escape and on a press outside.
        if self.menu_open {
            if let Some(sw_rect) = switcher {
                let mut picked = false;
                let mut menu_rect = Rect::NOTHING;
                let pos = sw_rect.left_bottom() + vec2(0.0, 4.0);
                egui::Area::new(id.with("menu")).order(egui::Order::Foreground).fixed_pos(pos).show(ui.ctx(), |m| {
                    let frame = egui::Frame::new()
                        .fill(t.surface_3)
                        .stroke(Stroke::new(1.0, t.edge_2))
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(egui::Margin::same(4))
                        .show(m, |m| {
                            let (head, _) = m.allocate_exact_size(vec2(146.0, 20.0), Sense::hover());
                            m.painter().text(pos2(head.left() + 8.0, head.center().y), Align2::LEFT_CENTER, "Projects", FontId::new(11.0, weight(500)), t.content_3);
                            for name in PROJECTS {
                                let (r, resp) = m.allocate_exact_size(vec2(146.0, 24.5), Sense::click());
                                if resp.hovered() {
                                    m.painter().rect_filled(r, CornerRadius::same(4), t.wash_1);
                                }
                                m.painter().text(pos2(r.left() + 8.0, r.center().y), Align2::LEFT_CENTER, name, font.clone(), t.content_1);
                                resp.widget_info(|| WidgetInfo::labeled(Role::Button, true, name));
                                if resp.clicked() {
                                    picked = true;
                                }
                            }
                        });
                    menu_rect = frame.response.rect;
                    let node = m.interact(menu_rect, id.with("menu_box"), Sense::hover());
                    node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(m, &node, parts::PROJECTMENU);
                });

                let outside = ui.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|pos| !menu_rect.contains(pos) && !sw_rect.contains(pos)));
                let escape = ui.input(|i| i.key_pressed(Key::Escape));
                if picked || escape || outside {
                    self.menu_open = false;
                    events.push(TitlebarEvent::ProjectMenuChanged { open: false });
                }
                // A pick or Escape returns focus to the switcher, as the popover does.
                if picked || escape {
                    ui.ctx().memory_mut(|m| m.request_focus(sw_id));
                }
            }
        }

        // Double-click maximises and a press drags the window, but only on the bar itself, not on its controls.
        let on_control = bar_resp.interact_pointer_pos().is_some_and(|pos| no_drag.iter().any(|r| r.contains(pos)));
        if drag && !on_control {
            if bar_resp.double_clicked() {
                self.toggle_maximize(ui.ctx(), &mut events);
            }
            if bar_resp.drag_started() {
                ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
            }
        }

        events
    }

    /// Flips the window between maximised and restored, and reports the new state.
    fn toggle_maximize(&mut self, ctx: &egui::Context, events: &mut Vec<TitlebarEvent>) {
        self.maximized = !self.maximized;
        ctx.send_viewport_cmd(ViewportCommand::Maximized(self.maximized));
        events.push(TitlebarEvent::MaximizeToggled { maximized: self.maximized });
    }
}

/// The width of one line of text, unwrapped.
fn text_w(ui: &egui::Ui, s: &str, font: &FontId) -> f32 {
    ui.painter().layout_no_wrap(s.to_owned(), font.clone(), Color32::WHITE).size().x
}

/// One line of text, cut with an ellipsis to fit max_w.
fn line(ui: &egui::Ui, s: &str, font: FontId, color: Color32, max_w: f32) -> Arc<Galley> {
    let mut job = LayoutJob::single_section(s.to_owned(), TextFormat { font_id: font, color, ..Default::default() });
    job.wrap.max_width = max_w;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    ui.painter().layout_job(job)
}

/// A Material Symbols glyph centred on a point.
fn glyph(painter: &Painter, name: &str, at: egui::Pos2, px: f32, color: Color32) {
    let g = icon(name, px);
    painter.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

crate::mounted!(TitleBar, TitlebarProps);
