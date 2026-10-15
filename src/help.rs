use ratatui::{
    style::Stylize,
    symbols,
    text::{Line, Span, Text},
    widgets::{List, ListItem, Widget},
};

use crate::{data::source::DATABASE, Navigable, DEFAULT_STYLE};

#[derive(Debug, Clone, Default)]
pub struct Help;

impl Widget for Help {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        List::new([
            Text::raw("Controls").style(DEFAULT_STYLE.bold()),
            Text::raw("q         quit"),
            Text::raw("←|h       navigate left/back"),
            Text::raw("→|l       navigate right/forward"),
            Text::raw("↓|j       navigate down"),
            Text::raw("ꜛ|k       navigate up"),
            Text::raw("ꜛ|k       navigate up"),
            Text::raw("ctrl+p    save .pdf of this resume"),
            Text::raw("?         open this menu"),
        ])
        .render(area, buf);
    }
}
