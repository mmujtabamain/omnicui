//! contract: omnicui/contracts/scrollarea.contract.yaml (id: ui.scrollarea). A scroll region for the task list and the trash list: native wheel and keys, a drawn thumb, and the task list's drop target and blank-space press.
use crate::omnicui::scrollarea::{parts, ScrollareaEvent, ScrollareaProps};
use crate::{part, rem, weight, Look};
use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Key, Modifiers, Pos2, Rect, Role, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, WidgetInfo};

/// The fixture rows: the web harness's 16 tasks. The first and the last are the firstRow and lastRow parts.
const ROWS: usize = 16;
/// A list row is h-[2.5rem] tall, with mx-3 side margins and px-3 text padding. Horizontal rows are w-[10rem] by h-[13rem] with p-3.
const ROW_H: f32 = rem(2.5);
const COL_W: f32 = rem(10.0);
const COL_H: f32 = rem(13.0);
const ROW_MX: f32 = 12.0;
const PAD: f32 = 12.0;
const FONT: f32 = 14.0;
/// The native bar is w-2.5 (10px) on the far edge, its track inset by p-px (1px).
const BAR: f32 = 10.0;
const TRACK_PAD: f32 = 1.0;
/// Keyboard steps: a line as a browser scrolls one (40px), a page as 87.5% of the region.
const LINE: f32 = 40.0;
const PAGE: f32 = 0.875;
/// A scroll settles once it has been still this long (SETTLE_MS in the web), in seconds.
const SETTLE: f64 = 0.15;
/// The same settle counted in frames (9 frames is 150 ms at 60 Hz), for a target whose clock does not advance between frames.
const SETTLE_FRAMES: u64 = 9;
/// The band at each edge where a dragged row auto-scrolls the list (EDGE_PX and EDGE_STEP_PX in the web).
const EDGE: f32 = 35.0;
const EDGE_STEP: f32 = 12.0;

pub struct ScrollArea {
    p: ScrollareaProps,
    /// The offset the region is asked to show next frame.
    off: f32,
    /// The offset the region showed last frame.
    shown: f32,
    /// The content's length along the axis, from the last frame.
    content: f32,
    scrolling: bool,
    last_move: f64,
    last_frame: u64,
    at_bottom: bool,
    over: bool,
}

impl ScrollArea {
    pub fn new(p: ScrollareaProps) -> Self {
        Self {
            p,
            off: 0.0,
            shown: 0.0,
            content: 0.0,
            scrolling: false,
            last_move: 0.0,
            last_frame: 0,
            at_bottom: false,
            over: false,
        }
    }

    /// One frame: draws the region and returns the events it raised in this frame.
    pub fn ui(&mut self, ui: &mut Ui, look: Look) -> Vec<ScrollareaEvent> {
        let mut events = vec![];
        let t = look.t;
        let horizontal = self.p.orientation == "horizontal";
        let native = self.p.scrollbar != "hidden";
        let raised = self.p.tone == "raised";
        let rect = ui.available_rect_before_wrap();
        let view = along(horizontal, rect.size());
        let (pressed, pointer, now) = ui.input(|i| (i.pointer.primary_pressed(), i.pointer.latest_pos(), i.time));
        let frame = ui.ctx().cumulative_frame_nr();
        let start = self.shown;

        // The region is interacted before its content, so the rows and the thumb sit on top of it.
        let region_id = ui.make_persistent_id("region");
        let response = ui.interact(rect, region_id, Sense::click_and_drag());
        let label = self.p.label.clone();
        response.widget_info(|| {
            if label.is_empty() {
                WidgetInfo::new(Role::GenericContainer)
            } else {
                WidgetInfo::labeled(Role::GenericContainer, true, label.as_str())
            }
        });
        part(ui, &response, parts::REGION);

        // Keys while the region has focus (taken by a press on it, or by Tab once it overflows). They are consumed here, so the scroll does not act on them twice.
        if response.has_focus() {
            let max = (self.content - view).max(0.0);
            let keys = [
                Key::ArrowUp,
                Key::ArrowDown,
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::PageUp,
                Key::PageDown,
                Key::Home,
                Key::End,
            ];
            let [up, down, left, right, page_up, page_down, home, end] =
                ui.input_mut(|i| keys.map(|k| i.consume_key(Modifiers::NONE, k)));
            let (fwd, back) = if horizontal { (right || down, left || up) } else { (down, up) };
            if fwd {
                self.off += LINE;
            }
            if back {
                self.off -= LINE;
            }
            if page_down {
                self.off += view * PAGE;
            }
            if page_up {
                self.off -= view * PAGE;
            }
            if home {
                self.off = 0.0;
            }
            if end {
                self.off = max;
            }
            self.off = self.off.clamp(0.0, max);
        }

        // The background is reserved now and filled once the drop state is known.
        let bg = ui.painter().add(Shape::Noop);
        let ink = if raised { t.content_2 } else { t.content_1 };
        let mut row_rects: Vec<Rect> = Vec::with_capacity(ROWS);
        let scroll = if horizontal { egui::ScrollArea::horizontal() } else { egui::ScrollArea::vertical() };
        let out = scroll
            .id_salt("content")
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .scroll_source(egui::scroll_area::ScrollSource::MOUSE_WHEEL)
            .scroll_offset(if horizontal { vec2(self.off, 0.0) } else { vec2(0.0, self.off) })
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if horizontal {
                    let _ = ui.horizontal_top(|ui| rows(ui, true, ink, t.edge_1, &mut row_rects));
                } else {
                    rows(ui, false, ink, t.edge_1, &mut row_rects);
                }
            });

        // What the region showed this frame, and whether the wheel, keys, a drag or a drop moved it.
        self.shown = along(horizontal, out.state.clamped_offset());
        self.off = self.shown;
        self.content = along(horizontal, out.content_size);
        let content = self.content;
        let max = (content - view).max(0.0);
        let moved = (self.shown - start).abs() > 0.5;
        let overflowing = content > view + 1.0;

        // The drawn thumb of the native bar, on top of the content. Its box follows the offset.
        let bar = bar_rect(rect, horizontal);
        let track = bar.shrink(TRACK_PAD);
        let track_len = along(horizontal, track.size());
        let thumb_shown = native && overflowing && track_len > 0.0;
        let thumb_len = if overflowing { (track_len * view / content).min(track_len) } else { track_len };
        let at = if max > 0.0 { self.shown / max * (track_len - thumb_len) } else { 0.0 };
        let thumb = if horizontal {
            Rect::from_min_size(pos2(track.left() + at, track.top()), vec2(thumb_len, track.height()))
        } else {
            Rect::from_min_size(pos2(track.left(), track.top() + at), vec2(track.width(), thumb_len))
        };
        if thumb_shown {
            let tr = ui.interact(thumb, region_id.with("thumb"), Sense::drag());
            tr.widget_info(|| WidgetInfo::new(Role::GenericContainer));
            part(ui, &tr, parts::THUMB);
            if tr.dragged() {
                let d = along(horizontal, tr.drag_delta());
                self.off = (self.off + d * content / track_len).clamp(0.0, max);
            }
            ui.painter().rect_filled(thumb, CornerRadius::same(4), t.edge_3);
        }

        // A press on blank space: in the region, not on a row and not on the thumb. A press on a row focuses the region.
        let press = if pressed { pointer.filter(|p| rect.contains(*p)) } else { None };
        if let Some(p) = press {
            if !(thumb_shown && thumb.contains(p)) {
                response.request_focus();
                if !on_row(&row_rects, p) {
                    events.push(ScrollareaEvent::PressBlank { clearedSelection: true });
                }
            }
        }

        // Scroll state: scrolling starts on the first move and ends once it has been still for SETTLE.
        if moved {
            self.last_move = now;
            self.last_frame = frame;
            if !self.scrolling {
                self.scrolling = true;
                events.push(ScrollareaEvent::ScrollingChanged { scrolling: true });
            }
            let at_end = self.shown + view >= content - 1.0;
            if at_end != self.at_bottom {
                self.at_bottom = at_end;
                events.push(ScrollareaEvent::BottomChanged { atBottom: at_end });
            }
        }
        if self.scrolling {
            if now - self.last_move >= SETTLE || frame.saturating_sub(self.last_frame) >= SETTLE_FRAMES {
                self.scrolling = false;
                events.push(ScrollareaEvent::ScrollingChanged { scrolling: false });
            } else {
                // An immediate repaint, so the frames that count the still time run without a wait.
                ui.ctx().request_repaint();
            }
        }

        // Drop target: a row dragged over blank space; a drop there moves it to the root. Near an edge the list scrolls.
        let hover = self.p.dropTarget && response.dnd_hover_payload::<String>().is_some();
        let released = if self.p.dropTarget { response.dnd_release_payload::<String>() } else { None };
        let on_blank = pointer.is_some_and(|p| rect.contains(p) && !on_row(&row_rects, p));
        if hover && !horizontal {
            if let Some(y) = pointer.map(|p| p.y) {
                if y > rect.bottom() - EDGE {
                    self.off += EDGE_STEP;
                } else if y < rect.top() + EDGE {
                    self.off -= EDGE_STEP;
                }
                self.off = self.off.clamp(0.0, max);
                ui.ctx().request_repaint();
            }
        }
        let over = hover && on_blank && released.is_none();
        if over != self.over {
            self.over = over;
            events.push(ScrollareaEvent::DropOverChanged { over });
        }
        if let Some(task) = released {
            if on_blank {
                events.push(ScrollareaEvent::RootDropped { taskId: task.to_string() });
            }
        }

        // The fill: the drop-over state, else the raised tone.
        let fill = if self.over {
            Some(t.selection_1)
        } else if raised {
            Some(t.surface_2)
        } else {
            None
        };
        if let Some(c) = fill {
            ui.painter().set(bg, Shape::rect_filled(rect, CornerRadius::ZERO, c));
        }

        if response.has_focus() && overflowing {
            ui.painter().rect_stroke(rect, CornerRadius::ZERO, Stroke::new(2.0, t.focus_1), StrokeKind::Inside);
        }
        events
    }
}

/// The component of a size or vector along the scroll axis.
fn along(horizontal: bool, v: Vec2) -> f32 {
    if horizontal {
        v.x
    } else {
        v.y
    }
}

/// Whether a point is over one of the rows (a press or a drop there is not on blank space).
fn on_row(rows: &[Rect], p: Pos2) -> bool {
    rows.iter().any(|r| r.contains(p))
}

/// The native bar: along the far edge of the region.
fn bar_rect(rect: Rect, horizontal: bool) -> Rect {
    if horizontal {
        Rect::from_min_max(pos2(rect.left(), rect.bottom() - BAR), rect.right_bottom())
    } else {
        Rect::from_min_max(pos2(rect.right() - BAR, rect.top()), rect.right_bottom())
    }
}

/// The fixture rows, laid out along the axis. Each is a generic node with its task id as the drag payload;
/// the first and the last are tagged as the firstRow and lastRow parts. Their rects are kept in `rects`.
fn rows(ui: &mut Ui, horizontal: bool, ink: Color32, edge: Color32, rects: &mut Vec<Rect>) {
    let font = FontId::new(FONT, weight(400));
    for i in 0..ROWS {
        let name = format!("Task {}", i + 1);
        let task = format!("task-{}", i + 1);
        let cell = if horizontal {
            let (space, _) = ui.allocate_exact_size(vec2(COL_W, COL_H), Sense::hover());
            space
        } else {
            let (space, _) = ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::hover());
            space.shrink2(vec2(ROW_MX, 0.0))
        };
        let resp = ui.interact(cell, ui.scope_id().with(("task", i)), Sense::drag());
        resp.dnd_set_drag_payload(task.clone());
        resp.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, name.as_str()));
        if i == 0 {
            part(ui, &resp, parts::FIRSTROW);
        }
        if i + 1 == ROWS {
            part(ui, &resp, parts::LASTROW);
        }
        rects.push(cell);
        let p = ui.painter();
        if horizontal {
            p.text(cell.left_top() + vec2(PAD, PAD), Align2::LEFT_TOP, &name, font.clone(), ink);
            p.vline(cell.right(), cell.y_range(), Stroke::new(1.0, edge));
        } else {
            p.text(pos2(cell.left() + PAD, cell.center().y), Align2::LEFT_CENTER, &name, font.clone(), ink);
            p.hline(cell.x_range(), cell.bottom(), Stroke::new(1.0, edge));
        }
    }
}

crate::mounted!(ScrollArea, ScrollareaProps);
