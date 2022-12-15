use color_eyre::Result;
use tuirealm::{
    tui::{
        layout::{Constraint, Direction, Layout},
        widgets::Clear,
    },
    Frame,
};

use crate::display::tui::{
    component::Popup, event::AppEvent, msg::ViewMsg, types::TextSpans, Component,
};

use super::View;

#[derive(Clone)]
pub struct PopupView {
    popup: Popup,
}

impl View for PopupView {
    fn render(&mut self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(2, 4),
                    Constraint::Ratio(1, 4),
                ]
                .as_ref(),
            )
            .split(frame.size());
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 4),
                    Constraint::Ratio(2, 4),
                    Constraint::Ratio(1, 4),
                ]
                .as_ref(),
            )
            .split(chunks[1]);
        frame.render_widget(Clear, chunks[1]);
        self.popup.render(frame, chunks[1]);
    }

    fn handle_event(&mut self, _event: &AppEvent) -> Result<ViewMsg> {
        Ok(ViewMsg::None)
    }
}

impl PopupView {
    pub fn new(title: impl Into<TextSpans>, text: impl Into<TextSpans>) -> Self {
        Self {
            popup: Popup::new(title, text),
        }
    }
}
