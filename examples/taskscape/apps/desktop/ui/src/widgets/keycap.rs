//! contract: omnicui/contracts/keycap.contract.yaml (id: ui.keycap). A button that shows one shortcut binding as keycap glyphs and, when activated, records the next key combo for that action; the reset control sits beside it when the binding is custom.
use crate::omnicui::button::{ButtonEvent, ButtonProps};
use crate::omnicui::hotkeyhint::HotkeyhintProps;
use crate::omnicui::keycap::{parts, KeycapEvent, KeycapProps};
use crate::widgets::button::Button;
use crate::widgets::hotkeyhint::Hotkeyhint;
use crate::{part, rem, weight, Look};
use egui::{vec2, Align2, CornerRadius, Event, FontId, Key, Modifiers, Rect, Role, Sense, Shape, Stroke, StrokeKind, WidgetInfo};

pub struct Keycap {
    p: KeycapProps,
    /// The mode, held here as the owning list would hold it: activation starts recording and cancel ends it.
    recording: bool,
    /// Recording was on at the end of the last frame. A keycap that starts recording takes focus, so the keys reach it.
    was_recording: bool,
    /// The keycap had keyboard focus at the end of the last frame. Losing it while recording cancels (Tab, or a click away).
    focused: bool,
    /// The refusal message of the last reserved combo, announced until the next key.
    refusal: Option<String>,
    /// The combo's spoken words ("Command N"), when its owner speaks it differently from the accelerator
    /// the glyphs are drawn from ("Meta+N"); else the accelerator's own words.
    pub spoken: Option<String>,
}

/// Accelerator notation: Control, Alt, Shift, Meta, then the key, joined with "+".
fn combo(m: Modifiers, key: Key) -> String {
    let mut mods: Vec<&str> = vec![];
    if m.ctrl || (m.command && !cfg!(target_os = "macos")) {
        mods.push("Control");
    }
    if m.alt {
        mods.push("Alt");
    }
    if m.shift {
        mods.push("Shift");
    }
    if m.mac_cmd {
        mods.push("Meta");
    }
    mods.push(key.name());
    mods.join("+")
}

impl Keycap {
    pub fn new(p: KeycapProps) -> Self {
        let recording = p.mode == "recording";
        Self { p, recording, was_recording: false, focused: false, refusal: None, spoken: None }
    }

    /// One frame: draws the keycap, and the reset control beside it when shown, and returns the events it raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<KeycapEvent> {
        let mut events = vec![];
        let unbound = self.p.binding == "unbound" || self.p.accel.is_empty();
        let show_reset = self.p.binding == "custom" && !unbound && !self.recording;
        let group = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = rem(0.25);
                self.keycap(ui, look, unbound, &mut events);
                if show_reset {
                    self.reset(ui, look, &mut events);
                }
            })
            .response;
        group.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &group, parts::GROUP);
        self.announcement(ui, group.rect);
        events
    }

    /// The keycap itself: the glyphs, the unbound label, or the prompt while recording.
    fn keycap(&mut self, ui: &mut egui::Ui, look: Look, unbound: bool, events: &mut Vec<KeycapEvent>) {
        let t = look.t;
        let disabled = self.p.disabled;
        let min_w = if self.recording { rem(7.214) } else { rem(3.5) };
        let min_h = rem(1.5);
        // The background is reserved before the content and filled once the hover state is known.
        let bg = ui.painter().add(Shape::Noop);
        let inner = ui.scope(|ui| {
            ui.set_min_size(vec2(min_w, min_h));
            if !self.recording && !unbound {
                let mut hint = Hotkeyhint::new(HotkeyhintProps {
                    accel: self.p.accel.clone(),
                    size: "sm".into(),
                    tone: "inherit".into(),
                    ..Default::default()
                });
                // px-1.5 on each side of the glyphs; the minimum width already includes it.
                ui.horizontal_centered(|ui| {
                    ui.add_space(rem(0.375));
                    hint.ui(ui, look);
                    ui.add_space(rem(0.375));
                });
            }
        });
        let rect = inner.response.rect;
        // Tab is taken by egui's focus navigation before the widget runs, so it is read from the raw input.
        // Tab while recording passes through: the recording ends and the keycap is not focusable this frame,
        // so focus is let go rather than wrapped back onto it.
        let tab = ui.input(|i| i.raw.events.iter().any(|e| matches!(e, Event::Key { key: Key::Tab, pressed: true, .. })));
        let passing_tab = tab && self.recording && self.focused;
        let sense = if disabled || passing_tab { Sense::hover() } else { Sense::click() };
        let response = ui.interact(rect, ui.scope_id().with("keycap"), sense);
        if passing_tab {
            self.cancel(events);
            ui.memory_mut(|m| m.surrender_focus(response.id));
        }

        // Activation: idle starts recording; recording cancels. Enter and Space arrive here as clicks.
        if !disabled && response.clicked() {
            if self.recording {
                self.cancel(events);
            } else {
                self.recording = true;
                events.push(KeycapEvent::StartRecording {});
            }
        }
        if self.recording && !disabled && !self.was_recording {
            response.request_focus();
        }
        self.was_recording = self.recording;

        // While recording, the keys are the new combo. A bare Enter or Space is not bound (the click cancels it),
        // and a bare Backspace or Delete unbinds the row.
        if self.recording && response.has_focus() {
            let presses: Vec<(Key, Modifiers)> = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|e| match e {
                        Event::Key { key, pressed: true, modifiers, .. } => Some((*key, *modifiers)),
                        _ => None,
                    })
                    .collect()
            });
            for (key, m) in presses {
                if !self.recording {
                    break;
                }
                let bare = !(m.ctrl || m.alt || m.shift || m.command || m.mac_cmd);
                match key {
                    Key::Tab => {}
                    Key::Enter | Key::Space if bare => {}
                    Key::Escape => {
                        ui.input_mut(|i| {
                            i.consume_key(m, key);
                        });
                        self.cancel(events);
                    }
                    Key::Backspace | Key::Delete if bare => {
                        ui.input_mut(|i| {
                            i.consume_key(m, key);
                        });
                        self.refusal = None;
                        events.push(KeycapEvent::Capture { accel: String::new() });
                    }
                    _ => {
                        ui.input_mut(|i| {
                            i.consume_key(m, key);
                        });
                        let accel = combo(m, key);
                        if self.p.reserved.split(',').map(str::trim).any(|r| !r.is_empty() && r == accel) {
                            self.refusal = Some(format!("{accel} is reserved"));
                            events.push(KeycapEvent::Refused { accel });
                        } else {
                            self.refusal = None;
                            events.push(KeycapEvent::Capture { accel });
                        }
                    }
                }
            }
        }
        // Focus left while recording (Tab, or a click elsewhere): the recording is cancelled.
        if self.recording && self.focused && !response.has_focus() {
            self.cancel(events);
        }
        self.focused = !passing_tab && response.has_focus();

        // Paint: the fill, the text while recording or unbound, the recording ring and the focus ring.
        let alpha = if disabled { 0.45 } else { 1.0 };
        let radius = CornerRadius::same(5);
        let hot = response.hovered() && !disabled;
        let fill = if !self.recording && hot { t.wash_2 } else { t.surface_0 };
        let spoken = self.spoken.clone().unwrap_or_else(|| self.p.accel.split('+').map(str::trim).collect::<Vec<_>>().join(" "));
        let value = if self.recording {
            self.p.prompt.clone()
        } else if unbound {
            self.p.unboundLabel.clone()
        } else {
            spoken
        };
        let painter = ui.painter();
        painter.set(bg, Shape::rect_filled(rect, radius, fill.gamma_multiply(alpha)));
        let font = FontId::new(rem(0.786), weight(500));
        if self.recording {
            painter.text(rect.center(), Align2::CENTER_CENTER, value.clone(), font, t.content_1.gamma_multiply(alpha));
        } else if unbound {
            painter.text(rect.center(), Align2::CENTER_CENTER, value.clone(), font, t.content_2.gamma_multiply(alpha));
        }
        if self.recording {
            painter.rect_stroke(rect, radius, Stroke::new(1.0, t.focus_1.gamma_multiply(alpha)), StrokeKind::Inside);
        }
        if response.has_focus() && !disabled {
            painter.rect_stroke(rect.expand(2.0), radius, Stroke::new(2.0, t.focus_1), StrokeKind::Outside);
        }

        // The name is the action and the spelled-out state; the text is the value on screen.
        let name = [self.p.actionName.clone(), value.clone()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        response.widget_info(|| {
            let mut info = WidgetInfo::labeled(Role::Button, !disabled, name.clone());
            info.current_text_value = Some(value.clone());
            info
        });
        part(ui, &response, parts::KEYCAP);
    }

    /// The reset control: a shared ghost icon button beside the keycap. Its node is the reset part.
    fn reset(&mut self, ui: &mut egui::Ui, look: Look, events: &mut Vec<KeycapEvent>) {
        // The button's own response is the first allocation in its frame, so its id is the next auto id.
        let id = ui.next_auto_id();
        let mut props = ButtonProps::default();
        props.shape = "icon".into();
        props.size = "sm".into();
        props.icon = "restart_alt".into();
        props.variant = "ghost".into();
        props.ariaLabel = format!("Reset {}", self.p.actionName).trim().to_owned();
        props.tabbable = true;
        props.disabled = self.p.disabled;
        let activated = Button::new(props).ui(ui, look).iter().any(|e| matches!(e, ButtonEvent::Activated { .. }));
        ui.ctx().accesskit_node_builder(id, |node| node.set_author_id(parts::RESET.to_owned()));
        if activated {
            events.push(KeycapEvent::Reset {});
        }
    }

    /// The polite live region: the prompt while recording, or the refusal message.
    fn announcement(&self, ui: &mut egui::Ui, at: Rect) {
        let text = if self.recording {
            self.refusal.clone().unwrap_or_else(|| self.p.prompt.clone())
        } else {
            String::new()
        };
        let node = ui.interact(Rect::from_min_size(at.min, vec2(1.0, 1.0)), ui.scope_id().with("announcement"), Sense::hover());
        node.widget_info(|| WidgetInfo::labeled(Role::Status, true, text.clone()));
        part(ui, &node, parts::ANNOUNCEMENT);
    }

    fn cancel(&mut self, events: &mut Vec<KeycapEvent>) {
        self.recording = false;
        self.refusal = None;
        events.push(KeycapEvent::Cancel {});
    }
}

crate::mounted!(Keycap, KeycapProps);
