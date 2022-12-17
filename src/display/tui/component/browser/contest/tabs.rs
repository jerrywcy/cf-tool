use color_eyre::Result;

use tuirealm::{
    props::TextSpan,
    tui::{layout::Rect, style::Color},
    Frame,
};

use crate::display::tui::{
    base_component::{BaseComponent, Tabs},
    component::ComponentSender,
    event::AppEvent,
    msg::ComponentMsg,
    utils::{is_left_key, is_right_key},
    Component,
};

pub struct ContestBrowserTabs {
    sender: ComponentSender,
    component: Tabs,
}

impl Component for ContestBrowserTabs {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }

    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(evt) if is_right_key(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.index))?;
            }
            AppEvent::Key(evt) if is_left_key(evt) => {
                self.component.prev();
                self.send(ComponentMsg::ChangedTo(self.component.index))?;
            }
            _ => (),
        };
        Ok(())
    }
}

impl ContestBrowserTabs {
    pub fn new(sender: ComponentSender) -> Self {
        Self {
            sender,
            component: Tabs::new(vec![
                TextSpan::new("Problems").fg(Color::White),
                TextSpan::new("Standings").fg(Color::White),
                TextSpan::new("Submissions").fg(Color::White),
            ]),
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn selected(&self) -> usize {
        self.component.index
    }

    pub fn select(&mut self, index: usize) -> Result<()> {
        self.component.select(index)
    }
}
