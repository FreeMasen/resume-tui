use ratatui::widgets::{List, ListItem, Widget};

use crate::{data::source::DATABASE, Navigable};

#[derive(Debug, Clone)]
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
                .map(|e| ListItem::new(""))
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
