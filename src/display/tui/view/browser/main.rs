use std::sync::Arc;

use color_eyre::Result;

use tuirealm::tui::layout::{Constraint, Direction, Layout};

use crate::display::tui::{
    app::POPUP,
    component::{ContestList, MainBrowserTabs, ProblemsetList},
    event::AppEvent,
    msg::{ComponentMsg, ViewMsg},
    utils::is_exit_key,
    Component, View,
};

pub struct MainBrowser {
    pub tabs: MainBrowserTabs,
    pub contest_list: ContestList,
    pub problemset_list: ProblemsetList,
}

impl View for MainBrowser {
    fn render(&mut self, frame: &mut tuirealm::Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(frame.size());
        self.tabs.render(frame, chunks[0]);
        match self.tabs.selected() {
            0 => self.contest_list.render(frame, chunks[1]),
            1 => self.problemset_list.render(frame, chunks[1]),
            _ => unreachable!(),
        };

        let popup = Arc::clone(&POPUP);
        if let Ok(result) = popup.try_lock() {
            if let Some(popup) = result.clone() {
                popup.clone().render(frame);
            }
        };
    }

    fn handle_event(&mut self, event: &AppEvent) -> Result<ViewMsg> {
        if let AppEvent::Key(evt) = event {
            if is_exit_key(*evt) {
                return Ok(ViewMsg::ExitCurrentView);
            }
        }
        let tabs_msg = self.tabs.on(event)?;
        match self.handle_msg(tabs_msg)? {
            ViewMsg::None => (),
            msg => return Ok(msg),
        }
        let container_msg = match self.tabs.selected() {
            0 => self.contest_list.on(event)?,
            1 => self.problemset_list.on(event)?,
            _ => unreachable!(),
        };
        match self.handle_msg(container_msg)? {
            ViewMsg::None => (),
            msg => return Ok(msg),
        }

        Ok(ViewMsg::None)
    }
}

impl MainBrowser {
    pub fn new() -> Result<Self> {
        Ok(Self {
            tabs: MainBrowserTabs::default(),
            contest_list: ContestList::new()?,
            problemset_list: ProblemsetList::new()?,
        })
    }

    fn handle_msg(&mut self, msg: ComponentMsg) -> Result<ViewMsg> {
        match msg {
            ComponentMsg::AppClose => return Ok(ViewMsg::AppClose),
            ComponentMsg::EnterNewView(view) => return Ok(ViewMsg::EnterNewView(view)),
            ComponentMsg::ExitCurrentView => return Ok(ViewMsg::ExitCurrentView),
            ComponentMsg::ChangeToTab(index) => {
                self.tabs.select(index)?;
            }
            _ => (),
        }
        Ok(ViewMsg::None)
    }
}
