//! contract: omnicui/contracts/heading.contract.yaml (id: ui.heading). A display-family semibold title in the primary tone: a level 1 to 3 heading, or a paragraph or span, drawn on one line with an ellipsis or wrapped.
use crate::omnicui::heading::{parts, HeadingEvent, HeadingProps};
use crate::{part, rem, weight, Look};
use egui::text::{LayoutJob, TextFormat};
use egui::{FontId, Role, Sense, WidgetInfo};

pub struct Heading {
    p: HeadingProps,
}

impl Heading {
    pub fn new(p: HeadingProps) -> Self {
        Self { p }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<HeadingEvent> {
        // Sizes: sm is 1rem (14px), md is 1.143rem (16px), both semibold at line-height 1.35.
        let md = self.p.size == "md";
        let size = if md { rem(1.143) } else { rem(1.0) };
        let wrap = self.p.overflow == "wrap";

        // The block fills the width it is given (max-w-full). Truncate lays out one row and lets the
        // overflow character (an ellipsis) end it; wrap breaks onto as many rows as the text needs.
        let mut job = LayoutJob::default();
        job.append(
            &self.p.content,
            0.0,
            TextFormat {
                font_id: FontId::new(size, weight(600)),
                color: look.t.content_1,
                line_height: Some(size * 1.35),
                ..Default::default()
            },
        );
        job.wrap.max_width = ui.available_width().max(0.0);
        job.wrap.max_rows = if wrap { usize::MAX } else { 1 };
        let galley = ui.painter().layout_job(job);

        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());

        // h1, h2 and h3 keep the heading role and their level; p and span report generic.
        let heading = !matches!(self.p.r#as.as_str(), "p" | "span");
        let role = if heading { Role::Heading } else { Role::GenericContainer };
        if heading {
            let level = match self.p.r#as.as_str() {
                "h1" => 1,
                "h3" => 3,
                _ => 2,
            };
            ui.ctx().accesskit_node_builder(response.id, |node| node.set_level(level));
        }
        let label = self.p.content.as_str();
        response.widget_info(|| WidgetInfo::labeled(role, true, label));
        part(ui, &response, parts::TITLE);

        ui.painter().galley(rect.min, galley, look.t.content_1);
        Vec::new()
    }
}

crate::mounted!(Heading, HeadingProps);
