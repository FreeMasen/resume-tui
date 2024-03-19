use ratatui::{
    symbols, text::{Line, Span, Text}, widgets::{List, ListItem, Widget}
};

use crate::{data::source::DATABASE, Navigable};

#[derive(Debug, Clone, Default)]
pub struct EduView;

impl Widget for EduView {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        List::new(
            DATABASE
                .education
                .iter()
                .map(|e| {
                    let mut items = vec![
                        Line::from(e.name),
                        Line::from(vec![Span::from("  "), e.desc.into()]),
                    ];
                    if let Some(grad) = e.graduated {
                        items.push(Line::from(vec![Span::from("  Graduated: "), grad.into()]));
                    }
                    items.push(Line::from(symbols::line::HORIZONTAL.repeat(area.width as _)));
                    ListItem::new(Text::from(items))
                })
                .collect::<Vec<_>>(),
        )
        .render(area, buf);
    }
}

impl Navigable for EduView {
    fn increment_selection(&mut self) {}

    fn decrement_selection(&mut self) {}

    fn handle_enter(&mut self) {}

    fn handle_left(&mut self) -> bool {
        false
    }
}
