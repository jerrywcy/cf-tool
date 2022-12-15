use color_eyre::Result;
use tuirealm::{tui::layout::Rect, Frame};

use crate::display::tui::{
    base_component::Paragraph, event::AppEvent, msg::ComponentMsg, types::TextSpans, BaseComponent,
};

use super::Component;

#[derive(Clone)]
pub struct Popup {
    component: Paragraph,
}

impl Component for Popup {
    fn on(&mut self, _event: &AppEvent) -> Result<ComponentMsg> {
        Ok(ComponentMsg::None)
    }
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }
}

impl Popup {
    pub fn new(title: impl Into<TextSpans>, text: impl Into<TextSpans>) -> Self {
        Self {
            component: Paragraph::new(title, text),
        }
    }
}
