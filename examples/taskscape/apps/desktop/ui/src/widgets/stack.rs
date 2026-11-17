//! contract: omnicui/contracts/stack.contract.yaml (id: ui.stack). A plain flex wrapper with no fill: lays its children out in a row or column, with justify, align, grow, gap and an optional rise entrance.
use crate::omnicui::stack::{parts, StackEvent, StackProps};
use crate::{part, rem, weight, Look, Mounted};
use egui::{FontId, Pos2, Rect, Role, Sense, Vec2, WidgetInfo};
use std::sync::Arc;

/// The two plain fixture children the harness renders, One then Two, with their part ids.
const FIXTURE: [(&str, &str); 2] = [(parts::FIRST, "One"), (parts::SECOND, "Two")];

/// Gap tokens in px: spacing tokens 1-8 are 2, 2.6, 3.5, 5.25, 7, 8.75, 12.25 and 17.5 px.
fn gap_px(gap: &str) -> f32 {
    match gap {
        "space-3" => 3.5,
        "space-4" => 5.25,
        "space-5" => 7.0,
        "space-6" => 8.75,
        _ => 0.0,
    }
}

pub struct Stack {
    p: StackProps,
}

impl Stack {
    pub fn new(p: StackProps) -> Self {
        Self { p }
    }

    /// One frame: lays the children (part id, text) out along the props' axis and returns the events raised.
    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look, children: &[(&str, &str)]) -> Vec<StackEvent> {
        let p = &self.p;
        let column = p.direction == "column";
        let fill = p.grow == "fill";
        let gap = gap_px(&p.gap);
        let t = look.t;
        let font = FontId::new(rem(1.0), weight(400));
        let galleys: Vec<Arc<egui::Galley>> = children
            .iter()
            .map(|(_, s)| ui.painter().layout_no_wrap(s.to_string(), font.clone(), t.content_1))
            .collect();
        let sizes: Vec<Vec2> = galleys.iter().map(|g| g.size()).collect();
        let n = sizes.len();
        let main_of = |v: Vec2| if column { v.y } else { v.x };
        let cross_of = |v: Vec2| if column { v.x } else { v.y };
        let content_main: f32 = sizes.iter().map(|v| main_of(*v)).sum::<f32>() + gap * n.saturating_sub(1) as f32;
        let content_cross: f32 = sizes.iter().map(|v| cross_of(*v)).fold(0.0, f32::max);

        // Block-level like the web div: full width; height is the content, or the free height
        // for a filling column.
        let height = if column && fill {
            ui.available_height()
        } else if column {
            content_main
        } else {
            content_cross
        };
        let response = ui.allocate_response(Vec2::new(ui.available_width(), height), Sense::hover());
        let rect = response.rect;
        response.widget_info(|| WidgetInfo::new(Role::GenericContainer));
        part(ui, &response, parts::ROOT);

        let main_len = if column { rect.height() } else { rect.width() };
        let cross_len = if column { rect.width() } else { rect.height() };
        let free = (main_len - content_main).max(0.0);
        let (mut offset, extra) = match p.justify.as_str() {
            "center" => (free / 2.0, 0.0),
            "end" => (free, 0.0),
            "between" if n > 1 => (0.0, free / (n - 1) as f32),
            _ => (0.0, 0.0),
        };

        // The rise entrance moves the body up into place; the settled layout is the same as none.
        let rise = if p.motion == "rise" {
            let k = ui.ctx().animate_bool_with_time(response.id.with("rise"), true, 0.28);
            (1.0 - k) * rem(0.3)
        } else {
            0.0
        };

        for (i, (id, text)) in children.iter().enumerate() {
            let main = main_of(sizes[i]);
            let own_cross = cross_of(sizes[i]);
            let (cross_off, child_cross) = match p.align.as_str() {
                "start" => (0.0, own_cross),
                "center" => ((cross_len - own_cross) / 2.0, own_cross),
                "end" => (cross_len - own_cross, own_cross),
                _ => (0.0, cross_len),
            };
            let (min, dims) = if column {
                (Pos2::new(rect.left() + cross_off, rect.top() + offset + rise), Vec2::new(child_cross, main))
            } else {
                (Pos2::new(rect.left() + offset, rect.top() + cross_off + rise), Vec2::new(main, child_cross))
            };
            let child_rect = Rect::from_min_size(min, dims);
            let child = ui.interact(child_rect, response.id.with(*id), Sense::hover());
            child.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, *text));
            part(ui, &child, *id);
            ui.painter().galley(child_rect.min, galleys[i].clone(), t.content_1);
            offset += main + gap + extra;
        }

        // The Stack emits no events.
        Vec::new()
    }
}

/// The desktop mount: the harness's two plain children, One then Two, around the widget.
pub struct StackMount {
    stack: Stack,
}

impl Mounted for StackMount {
    fn frame(&mut self, ui: &mut egui::Ui, look: Look, emit: &mut dyn FnMut(&'static str, serde_json::Value)) {
        for e in self.stack.ui(ui, look, &FIXTURE) {
            let (name, fields) = e.to_json();
            emit(name, fields);
        }
    }
}

/// Builds the widget from a test's props.
pub fn mount(props: serde_json::Value) -> Result<Box<dyn Mounted>, String> {
    let p: StackProps = serde_json::from_value(props).map_err(|e| format!("props: {e}"))?;
    Ok(Box::new(StackMount { stack: Stack::new(p) }))
}
