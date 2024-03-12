use ratatui::widgets::ListState;

#[derive(Debug, Clone)]
pub struct ListStateWrapper {
    max: usize,
    state: ListState,
}

impl ListStateWrapper {
    pub fn new(max: usize) -> Self {
        Self {
            max,
            state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn increment(&mut self) {
        let Some(existing) = self.state.selected() else {
            self.state.select(Some(0));
            return;
        };
        let next = (existing + 1).min(self.max);
        self.state.select(Some(next));
    }

    pub fn decrement(&mut self) {
        let Some(existing) = self.state.selected() else {
            self.state.select(Some(self.max));
            return;
        };
        let next = existing.saturating_sub(1);
        self.state.select(Some(next));
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, new_idx: usize) {
        let new_idx = new_idx.min(self.max);
        self.state.select(Some(new_idx));
    }
}

impl AsMut<ListState> for ListStateWrapper {
    fn as_mut(&mut self) -> &mut ListState {
        &mut self.state
    }
}

impl AsRef<ListState> for ListStateWrapper {
    fn as_ref(&self) -> &ListState {
        &self.state
    }
}
