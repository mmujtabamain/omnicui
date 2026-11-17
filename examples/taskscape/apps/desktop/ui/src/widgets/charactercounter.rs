//! contract: omnicui/contracts/charactercounter.contract.yaml (id: ui.charactercounter). The note limit readout, "count/limit", in the muted tone and the danger tone once the count is past nearRatio of the limit.
use crate::omnicui::charactercounter::{parts, CharactercounterEvent, CharactercounterProps};
use crate::{part, rem, weight, Look};
use egui::{pos2, FontId, Rect, Role, Sense, Vec2, WidgetInfo};

/// Read once on crossing into the near state; the live region is empty while normal.
const NEAR_NOTICE: &str = "Approaching the character limit.";

pub struct CharacterCounter {
    p: CharactercounterProps,
}

impl CharacterCounter {
    pub fn new(p: CharactercounterProps) -> Self {
        Self { p }
    }

    /// Near when the count is strictly greater than nearRatio times the limit. The epsilon keeps a
    /// product that rounds a hair low from flipping the edge.
    pub fn near(&self) -> bool {
        self.p.count as f64 > self.p.nearRatio * self.p.limit as f64 + 1e-9
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, look: Look) -> Vec<CharactercounterEvent> {
        let t = look.t;
        let near = self.near();
        let color = if near { t.danger_500 } else { t.content_2 };
        let text = format!("{}/{}", self.p.count, self.p.limit);

        // text-xs (10.5px, regular) with tabular numbers: every digit takes the width of the widest
        // digit, so the readout holds still as the count changes. The separator keeps its own width.
        let font = FontId::new(rem(0.75), weight(400));
        let digit_w = ui.painter().layout_no_wrap("0".to_owned(), font.clone(), color).size().x;
        let glyphs: Vec<_> = text
            .chars()
            .map(|c| (c, ui.painter().layout_no_wrap(c.to_string(), font.clone(), color)))
            .collect();
        let slot = |c: char, w: f32| if c.is_ascii_digit() { digit_w } else { w };
        let height = glyphs.iter().map(|(_, g)| g.size().y).fold(0.0, f32::max);
        let width: f32 = glyphs.iter().map(|(c, g)| slot(*c, g.size().x)).sum();

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter();
        let mut x = rect.left();
        for (c, g) in &glyphs {
            let w = slot(*c, g.size().x);
            painter.galley(pos2(x + (w - g.size().x) / 2.0, rect.top()), g.clone(), color);
            x += w;
        }
        response.widget_info(|| WidgetInfo::labeled(Role::GenericContainer, true, &text));
        part(ui, &response, parts::READOUT);

        // The polite status: empty while normal, one sentence on the near side. It draws nothing and
        // has no box of its own, so it sits at the readout's start with zero size.
        let notice = if near { NEAR_NOTICE } else { "" };
        let status = ui.interact(
            Rect::from_min_size(rect.min, Vec2::ZERO),
            response.id.with("announcement"),
            Sense::hover(),
        );
        status.widget_info(|| WidgetInfo::labeled(Role::Status, true, notice));
        part(ui, &status, parts::ANNOUNCEMENT);

        vec![]
    }
}

crate::mounted!(CharacterCounter, CharactercounterProps);
