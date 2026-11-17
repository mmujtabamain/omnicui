//! contract: omnicui/contracts/inputgroup.contract.yaml (id: ui.inputgroup). The one chrome for text entry: a field or search well, in the well or bar shape, around one text input with a leading icon, a locked suffix, and clear and roll buttons.
use crate::omnicui::inputgroup::{parts, InputgroupEvent, InputgroupProps};
use crate::{icon, part, rem, text, weight, Look};
use egui::{Align, Align2, Color32, CornerRadius, FontFamily, FontId, Key, Layout, Painter, Pos2, Rect, Role, Sense, Stroke, StrokeKind, TextEdit, UiBuilder, Vec2, WidgetInfo};

/// One text well. The frame draws the chrome; the inner text input holds the typed value.
pub struct InputGroup {
    p: InputgroupProps,
    value: String,
}

impl InputGroup {
    pub fn new(p: InputgroupProps) -> Self {
        Self { p, value: String::new() }
    }

    /// One frame: draws the well and returns the events it raised in this frame (none).
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<InputgroupEvent> {
        let t = look.t;
        let md = self.p.size != "sm";
        let search = self.p.tone == "search";
        let bar = self.p.shape == "bar";
        let disabled = self.p.disabled;
        // A disabled well is drawn at the web's opacity-45.
        let alpha = if disabled { 0.45 } else { 1.0 };
        let fade = |c: Color32| c.gamma_multiply(alpha);

        let height = if md { rem(2.5) } else { rem(1.75) };
        let text_px = if md { rem(1.0) } else { rem(0.857) };
        let pad = if md { 8.0 } else { 6.0 };
        let gap = pad;
        let icon_px = if md { 16.0 } else { 14.0 };
        let btn = if md { rem(1.75) } else { rem(1.25) };
        let font = FontId::new(text_px, weight(400));

        // The web harness mounts the well with these: a search well is named "Search tasks" with a
        // "Search" placeholder, and a search or bar well shows the search icon in its leading slot.
        let leading_on = search || bar;
        let placeholder = if search { "Search" } else { "Name" };
        let input_label = if search { "Search tasks" } else { "Sheet name" };

        let suffix = self.p.suffix.clone();
        let suffix_on = !suffix.is_empty();
        let clear_on = !self.value.is_empty();
        let roll_on = self.p.sheet;
        let trailing_on = clear_on || roll_on;

        let full_w = ui.available_width();
        let suffix_w = if suffix_on {
            ui.painter().layout_no_wrap(suffix.clone(), font.clone(), Color32::WHITE).size().x
        } else {
            0.0
        };
        let leading_w = if leading_on { icon_px } else { 0.0 };
        let buttons = clear_on as u32 + roll_on as u32;
        let trailing_w = if trailing_on { buttons as f32 * btn + (buttons as f32 - 1.0).max(0.0) * 2.0 } else { 0.0 };
        let items = 1 + leading_on as u32 + suffix_on as u32 + trailing_on as u32;
        let input_w = (full_w - 2.0 * pad - leading_w - suffix_w - trailing_w - gap * (items - 1) as f32).max(0.0);

        let (outer, well) = ui.allocate_exact_size(Vec2::new(full_w, height), Sense::hover());
        let well_label = format!("{}{}", self.value, suffix);
        well.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, !disabled, well_label.as_str()));
        part(ui, &well, parts::WELL);
        let input_id = well.id.with("input");

        // The trailing slot ends the row, so the buttons sit at the right of the well.
        let mut button_rects: Vec<Rect> = Vec::new();
        {
            let mut x = outer.right() - pad - trailing_w;
            let y = outer.center().y - btn / 2.0;
            if clear_on {
                button_rects.push(Rect::from_min_size(Pos2::new(x, y), Vec2::splat(btn)));
                x += btn + 2.0;
            }
            if roll_on {
                button_rects.push(Rect::from_min_size(Pos2::new(x, y), Vec2::splat(btn)));
            }
        }
        // Typed text that arrives while nothing has focus goes to the input, so that the input already
        // has focus when the text is read. Nothing is focused at rest, so no ring is drawn at mount.
        let typed = ui.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Text(_))));
        if typed && !disabled && ui.ctx().memory(|m| m.focused().is_none()) {
            ui.ctx().memory_mut(|m| m.request_focus(input_id));
        }
        // A press anywhere in the well, other than on its buttons, focuses the input. It is done
        // before the input is drawn, so that the input already has focus when this frame's keys arrive.
        if self.p.focusInputOnPress && !disabled && ui.input(|i| i.pointer.primary_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if outer.contains(pos) && !button_rects.iter().any(|r| r.contains(pos)) {
                    ui.ctx().memory_mut(|m| m.request_focus(input_id));
                }
            }
        }

        let radius = if bar {
            CornerRadius::same(0)
        } else if search {
            CornerRadius::same((height / 2.0) as u8)
        } else {
            CornerRadius::same(4)
        };
        let painter: Painter = ui.painter().clone();
        let fill = if search { t.wash_2 } else { t.surface_1 };
        painter.rect_filled(outer, radius, fade(fill));
        if bar {
            let y = outer.bottom() - 0.5;
            painter.line_segment([Pos2::new(outer.left(), y), Pos2::new(outer.right(), y)], Stroke::new(1.0, fade(t.edge_1)));
        } else if !search {
            painter.rect_stroke(outer, radius, Stroke::new(1.0, fade(t.edge_2)), StrokeKind::Inside);
        }

        // The slots sit in a row inside the well, centred vertically: leading, input, suffix, trailing.
        let inner = outer.shrink2(Vec2::new(pad, 0.0));
        let mut value = std::mem::take(&mut self.value);
        let input = ui
            .scope_builder(UiBuilder::new().max_rect(inner).layout(Layout::left_to_right(Align::Center)), |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(gap, 0.0);
                if leading_on {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(icon_px, height), Sense::hover());
                    resp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(ui, &resp, parts::LEADING);
                    draw_icon(ui.painter(), r.center(), "search", icon_px, fade(t.content_3));
                }

                let input = ui.add_enabled(
                    !disabled,
                    TextEdit::singleline(&mut value)
                        .id(input_id)
                        .frame(egui::Frame::NONE)
                        .desired_width(input_w)
                        .font(font.clone())
                        .text_color(fade(t.content_1))
                        .hint_text(text(placeholder, text_px, 400).color(fade(t.content_3)))
                        .interactive(!disabled),
                );
                part(ui, &input, parts::INPUT);

                if suffix_on {
                    let (r, resp) = ui.allocate_exact_size(Vec2::new(suffix_w, height), Sense::hover());
                    resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, !disabled, suffix.as_str()));
                    part(ui, &resp, parts::SUFFIX);
                    ui.painter().text(r.left_center(), Align2::LEFT_CENTER, suffix.as_str(), font.clone(), fade(t.content_3));
                }

                if trailing_on {
                    let (tr, tresp) = ui.allocate_exact_size(Vec2::new(trailing_w, height), Sense::hover());
                    tresp.widget_info(|| WidgetInfo::new(Role::GenericContainer));
                    part(ui, &tresp, parts::TRAILING);
                    let sense = if disabled { Sense::hover() } else { Sense::click() };
                    let mut x = tr.left();
                    if clear_on {
                        let r = Rect::from_min_size(Pos2::new(x, tr.center().y - btn / 2.0), Vec2::splat(btn));
                        let resp = ui.interact(r, tresp.id.with("clear"), sense);
                        resp.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, "Clear"));
                        part(ui, &resp, parts::CLEAR);
                        if resp.hovered() && !disabled {
                            ui.painter().rect_filled(r, CornerRadius::same(7), fade(t.wash_2));
                        }
                        draw_icon(ui.painter(), r.center(), "close", icon_px, fade(t.content_2));
                        // Enter and Space on the focused button clear the field, as a press does.
                        let keyed = resp.has_focus() && ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space));
                        if (resp.clicked() || keyed) && !disabled {
                            // Clear empties the field and hands focus back to the input, as the web does.
                            value.clear();
                            ui.ctx().memory_mut(|m| m.request_focus(input_id));
                        }
                        x += btn + 2.0;
                    }
                    if roll_on {
                        let r = Rect::from_min_size(Pos2::new(x, tr.center().y - btn / 2.0), Vec2::splat(btn));
                        let resp = ui.interact(r, tresp.id.with("roll"), sense);
                        resp.widget_info(|| WidgetInfo::labeled(Role::Button, !disabled, "Roll dice"));
                        part(ui, &resp, parts::ROLL);
                        if resp.hovered() && !disabled {
                            ui.painter().rect_filled(r, CornerRadius::same(7), fade(t.wash_2));
                        }
                        draw_icon(ui.painter(), r.center(), "casino", icon_px, fade(t.content_2));
                    }
                }
                input
            })
            .inner;
        self.value = value;
        // The input's name is its aria label; the text edit only reports its hint.
        ui.ctx().accesskit_node_builder(input.id, |node| node.set_label(input_label));

        if input.has_focus() && !disabled {
            painter.rect_stroke(outer.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }
        Vec::new()
    }
}

/// Draws a Material Symbols glyph centred on `center`.
fn draw_icon(painter: &Painter, center: Pos2, name: &str, px: f32, color: Color32) {
    painter.text(center, Align2::CENTER_CENTER, icon(name, px).text(), FontId::new(px, FontFamily::Name("Icons".into())), color);
}

crate::mounted!(InputGroup, InputgroupProps);
