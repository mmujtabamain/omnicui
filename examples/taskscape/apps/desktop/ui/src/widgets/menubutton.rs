//! contract: omnicui/contracts/menubutton.contract.yaml (id: ui.menubutton). A button that opens a menu below it: a trigger with a label or an icon and an optional leading and trailing glyph; a press, Enter, Space or ArrowDown opens the menu and Escape closes it.
use crate::omnicui::menubutton::{parts, MenubuttonEvent, MenubuttonProps};
use crate::{icon, part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontFamily, FontId, Id, Key, Order, Pos2, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo};

/// The gap between the items of the trigger (the web's gap-space-4, 6px), the menu's gap below the trigger (4px), and
/// the edge of the window the menu keeps (4px, its collision padding).
const GAP: f32 = 6.0;
const OFFSET: f32 = 4.0;
const INSET: f32 = 4.0;
/// The menu's rows, as the harness renders them: plain rows of 26px with 13px text.
const ROWS: [&str; 3] = ["Home", "Work", "Errands"];

fn trigger_id() -> Id {
    Id::new(("ui.menubutton", "trigger"))
}

fn panel_id() -> Id {
    Id::new(("ui.menubutton", "panel"))
}

fn area_id() -> Id {
    Id::new(("ui.menubutton", "area"))
}

/// The keys a focused trigger or menu keeps for itself: the arrows and Escape, not egui's spatial focus moves.
fn keep_keys() -> egui::EventFilter {
    egui::EventFilter { tab: false, horizontal_arrows: true, vertical_arrows: true, escape: true }
}

/// The width of a text run in px, at a size and a weight.
fn measure(ctx: &egui::Context, s: &str, px: f32, w: u16) -> f32 {
    ctx.fonts_mut(|f| f.layout_no_wrap(s.to_owned(), FontId::new(px, weight(w)), Color32::WHITE).size().x)
}

/// Draws a Material Symbols glyph centred on a point.
fn glyph(painter: &egui::Painter, name: &str, px: f32, at: Pos2, color: Color32) {
    let g = icon(name, px);
    painter.text(at, Align2::CENTER_CENTER, g.text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

pub struct MenuButton {
    p: MenubuttonProps,
    /// The test set `open`, so the host owns the state: the trigger shows `p.open` and a press only asks.
    controlled: bool,
    /// The open state, when the widget owns it.
    open: bool,
    /// The focus to request at the end of the frame.
    pending: Option<Id>,
    /// The menu panel's rect as it was drawn, for a press outside it.
    panel_rect: Option<Rect>,
    /// The trigger got focus from a pointer press, so it shows no ring (the web's focus-visible).
    pointer_focus: bool,
}

impl MenuButton {
    pub fn new(p: MenubuttonProps, controlled: bool) -> Self {
        let open = p.open;
        Self { p, controlled, open, pending: None, panel_rect: None, pointer_focus: false }
    }

    fn is_open(&self) -> bool {
        if self.controlled {
            self.p.open
        } else {
            self.open
        }
    }

    /// One frame: draws the trigger and, while open, the menu. Returns the events it raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<MenubuttonEvent> {
        let mut events = vec![];
        let ctx = ui.ctx().clone();
        let t = look.t;
        let p = self.p.clone();
        let pill = p.variant == "pill";
        let icon_only = p.size == "icon";
        let disabled = p.disabled;
        let shown = if p.label.is_empty() { p.placeholder.clone() } else { p.label.clone() };
        let show_label = !icon_only && !shown.is_empty();
        let has_lead = !p.leadingIcon.is_empty();
        let has_trail = !p.trailingIcon.is_empty();

        // Sizes in px (the web's rem at 14px): sm is 21px high, the rest 28px. The pill and default labels are 14px, the rest 12px.
        let h = if p.size == "sm" { rem(1.5) } else { rem(2.0) };
        let text_px = if p.size == "default" || (pill && p.size == "lg") { 14.0 } else { 12.0 };
        let font_w: u16 = if pill { 600 } else { 500 };
        let px = if icon_only {
            0.0
        } else if p.size == "sm" {
            6.0
        } else if pill || p.size == "default" {
            8.0
        } else {
            10.0
        };
        let lead_w = if has_lead {
            if p.size == "sm" {
                14.0
            } else {
                16.0
            }
        } else {
            0.0
        };
        let trail_w = if has_trail { 15.0 } else { 0.0 };
        let label_w = if show_label { measure(&ctx, &shown, text_px, font_w) } else { 0.0 };
        let items = has_lead as usize + show_label as usize + has_trail as usize;
        let items_w = lead_w + label_w + trail_w + GAP * items.saturating_sub(1) as f32;
        // The pill is at least 6.072rem (85px) wide; an icon-only trigger is square.
        let w = if icon_only {
            h
        } else if pill {
            (2.0 * px + items_w).max(rem(6.072))
        } else {
            2.0 * px + items_w
        };

        let (rect, _) = ui.allocate_exact_size(vec2(w, h), Sense::hover());
        let sense = if disabled { Sense::hover() } else { Sense::click() };
        let response = ui.interact(rect, trigger_id(), sense);
        let hot = !disabled && response.hovered();
        let focused = !disabled && response.has_focus();

        // Keys and pointer state, read once. A press on the trigger sets the pointer focus; a key clears it.
        let (down, escape, tab) = ui.input(|i| (i.key_pressed(Key::ArrowDown), i.key_pressed(Key::Escape), i.key_pressed(Key::Tab)));
        let any_key = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Key { pressed: true, .. })));
        if response.is_pointer_button_down_on() || response.clicked() {
            self.pointer_focus = true;
        } else if any_key {
            self.pointer_focus = false;
        }
        if focused {
            ctx.memory_mut(|m| m.set_focus_lock_filter(trigger_id(), keep_keys()));
        }

        let open = self.is_open();
        let press = ui.input(|i| if i.pointer.primary_pressed() { i.pointer.interact_pos() } else { None });
        let outside = press.is_some_and(|pos| !rect.contains(pos) && !self.panel_rect.is_some_and(|r| r.contains(pos)));

        // A press (egui clicks a focused button on Enter and Space too), ArrowDown on the trigger, or Escape asks for the new state.
        let mut want: Option<bool> = None;
        let mut back = false;
        if !disabled && response.clicked() {
            want = Some(!open);
            back = true;
        } else if focused && down && !open {
            want = Some(true);
        } else if open && escape {
            want = Some(false);
            back = true;
        } else if open && (tab || outside) {
            want = Some(false);
        }
        if let Some(next) = want {
            if !self.controlled {
                self.open = next;
            }
            events.push(MenubuttonEvent::OpenChanged { open: next });
            // Opening moves focus into the menu; closing by a key or a press returns it to the trigger.
            self.pending = if next {
                Some(panel_id())
            } else if back {
                Some(trigger_id())
            } else {
                None
            };
        }
        let open = self.is_open();

        // Paint: the fill and text colour of the variant, the hover and open washes, the disabled fade and the focus ring.
        let a = if disabled { 0.45 } else { 1.0 };
        let fade = |c: Color32| c.gamma_multiply(a);
        let lit = hot || open;
        let (fill, fg) = if pill || p.variant == "neutral" {
            (Some(if lit { t.wash_2 } else { t.surface_0 }), t.content_1)
        } else {
            let fill = if open {
                Some(t.wash_2)
            } else if hot {
                Some(t.wash_1)
            } else {
                None
            };
            (fill, if lit { t.content_1 } else { t.content_2 })
        };
        let cr = CornerRadius::same(if pill || p.size == "default" { 7 } else { 4 });
        let painter = ui.painter().clone();
        if let Some(c) = fill {
            painter.rect_filled(rect, cr, fade(c));
        }
        if focused && !self.pointer_focus {
            painter.rect_stroke(rect, cr, Stroke::new(2.0, fade(t.focus_1)), StrokeKind::Outside);
        }

        // The items, centred in the trigger: the leading glyph, the label (a placeholder in content_3), the chevron (content_3).
        let cy = rect.center().y;
        let mut x = rect.center().x - items_w / 2.0;
        if has_lead {
            let r = Rect::from_min_size(pos2(x, cy - lead_w / 2.0), vec2(lead_w, lead_w));
            glyph(&painter, &p.leadingIcon, lead_w, r.center(), fade(fg));
            let node = ui.interact(r, trigger_id().with("leadingIcon"), Sense::hover());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::LEADINGICON);
            x += lead_w + GAP;
        }
        if show_label {
            let r = Rect::from_min_size(pos2(x, cy - h / 2.0), vec2(label_w, h));
            let col = if p.label.is_empty() { t.content_3 } else { fg };
            painter.text(pos2(x, cy), Align2::LEFT_CENTER, shown.as_str(), FontId::new(text_px, weight(font_w)), fade(col));
            let node = ui.interact(r, trigger_id().with("label"), Sense::hover());
            node.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, !disabled, &shown));
            part(ui, &node, parts::LABEL);
            x += label_w + GAP;
        }
        if has_trail {
            let r = Rect::from_min_size(pos2(x, cy - trail_w / 2.0), vec2(trail_w, trail_w));
            glyph(&painter, &p.trailingIcon, trail_w, r.center(), fade(t.content_3));
            let node = ui.interact(r, trigger_id().with("trailingIcon"), Sense::hover());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::TRAILINGICON);
        }

        // The trigger's name is the accessible name when one is set, else the visible text.
        let name = if p.ariaLabel.is_empty() { shown.clone() } else { p.ariaLabel.clone() };
        response.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, &name));
        part(ui, &response, parts::TRIGGER);

        if open {
            self.panel(&ctx, look, rect, pill);
        } else {
            self.panel_rect = None;
        }
        if let Some(id) = self.pending.take() {
            ctx.memory_mut(|m| m.request_focus(id));
        }
        events
    }

    /// The menu, below the trigger at its left edge (start alignment, 4px apart), kept inside the window.
    fn panel(&mut self, ctx: &egui::Context, look: Look, trig: Rect, pill: bool) {
        let t = look.t;
        let widest = ROWS.iter().map(|s| measure(ctx, s, 13.0, 400)).fold(0.0_f32, f32::max);
        let natural = widest + 2.0 * rem(0.75);
        let w = if pill { natural.max(rem(15.0)) } else { natural.max(trig.width()) };
        let screen = ctx.input(|i| i.content_rect());
        let x = trig.left().min(screen.right() - INSET - w).max(INSET);
        let y = trig.bottom() + OFFSET;
        if ctx.memory(|m| m.focused()) == Some(panel_id()) {
            ctx.memory_mut(|m| m.set_focus_lock_filter(panel_id(), keep_keys()));
        }
        let area = egui::Area::new(area_id()).order(Order::Foreground).fixed_pos(pos2(x, y)).show(ctx, |ui| {
            let frame = egui::Frame::popup(ui.style())
                .fill(t.surface_3)
                .stroke(Stroke::new(1.0, t.edge_1))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(0, 3));
            let f = frame.show(ui, |ui| {
                ui.set_width(w);
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                for name in ROWS {
                    let (r, _) = ui.allocate_exact_size(vec2(w, rem(1.857)), Sense::hover());
                    ui.painter().text(pos2(r.left() + rem(0.75), r.center().y), Align2::LEFT_CENTER, name, FontId::new(13.0, weight(400)), t.content_1);
                }
            });
            let rect = f.response.rect;
            // The panel takes focus when it opens, so the trigger does not keep it; it reads the arrows and Escape.
            let node = ui.interact(rect, panel_id(), Sense::focusable_noninteractive());
            node.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &node, parts::MENU);
            rect
        });
        self.panel_rect = Some(area.inner);
    }
}

/// The trigger as the harness mounts it: the trigger at the harness's padding, with its menu below it.
pub struct MenuButtonMount {
    mb: MenuButton,
}

impl crate::Mounted for MenuButtonMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        let mut events = Vec::new();
        ui.add_space(rem(1.25));
        ui.horizontal(|ui| {
            ui.add_space(rem(1.25));
            events = self.mb.ui(ui, look);
        });
        for e in events {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the menu button from a test's props. A test that sets `open` makes the host the owner of the open state.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn crate::Mounted>, String> {
    let controlled = props.get("open").is_some();
    let p: MenubuttonProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    if !["neutral", "ghost", "pill"].contains(&p.variant.as_str()) {
        return Err(format!("MenuButton: variant must be one of neutral, ghost, pill, got {}", p.variant));
    }
    if !["sm", "lg", "default", "icon"].contains(&p.size.as_str()) {
        return Err(format!("MenuButton: size must be one of sm, lg, default, icon, got {}", p.size));
    }
    Ok(Box::new(MenuButtonMount { mb: MenuButton::new(p, controlled) }))
}
