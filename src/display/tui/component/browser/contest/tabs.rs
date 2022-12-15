use color_eyre::Result;

use tuirealm::{
    props::TextSpan,
    tui::{layout::Rect, style::Color},
    Frame,
};

use crate::display::tui::{
    base_component::{BaseComponent, Tabs},
    event::AppEvent,
    msg::ComponentMsg,
    utils::{is_left_key, is_right_key},
    Component,
};

pub struct ContestBrowserTabs {
    component: Tabs,
}

impl Component for ContestBrowserTabs {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }

    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        match *event {
            AppEvent::Key(evt) if is_right_key(evt) => {
                self.component.next();
                Ok(ComponentMsg::ChangedTo(self.component.index))
            }
            AppEvent::Key(evt) if is_left_key(evt) => {
                self.component.prev();
                Ok(ComponentMsg::ChangedTo(self.component.index))
            }
            _ => Ok(ComponentMsg::None),
        }
    }
}

impl Default for ContestBrowserTabs {
    fn default() -> Self {
        Self {
            component: Tabs::new(vec![
                TextSpan::new("Problems").fg(Color::White),
                TextSpan::new("Standings").fg(Color::White),
                TextSpan::new("Submissions").fg(Color::White),
            ]),
        }
    }
}

impl ContestBrowserTabs {
    pub fn selected(&self) -> usize {
        self.component.index
    }

    pub fn select(&mut self, index: usize) -> Result<()> {
        self.component.select(index)
    }
}
