use color_eyre::Result;
use tuirealm::{tui::layout::Rect, Frame};

use crate::display::tui::{
    base_component::Paragraph,
    event::AppEvent,
    msg::ComponentMsg,
    types::TextSpans,
    utils::{is_down_key, is_scroll_down, is_scroll_up, is_up_key},
    BaseComponent,
};

use super::{Component, ComponentSender};

#[derive(Clone)]
pub struct Popup {
    sender: ComponentSender,
    component: Paragraph,
}

impl Component for Popup {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangedTo(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangedTo(self.component.scroll.into()))?;
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangedTo(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangedTo(self.component.scroll.into()))?;
            }
            _ => (),
        };
        Ok(())
    }
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }
}

impl Popup {
    pub fn new(
        sender: ComponentSender,
        title: impl Into<TextSpans>,
        text: impl Into<TextSpans>,
    ) -> Self {
        Self {
            sender,
            component: Paragraph::new(title, text),
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
