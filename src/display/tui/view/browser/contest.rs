use std::sync::Arc;

use color_eyre::Result;


use tuirealm::tui::layout::{Constraint, Direction, Layout};

use crate::{
    api::objects::Contest,
    display::tui::{
        app::POPUP,
        component::{ContestBrowserTabs, ProblemsList, StandingsList, SubmissionsList},
        event::AppEvent,
        msg::{ComponentMsg, ViewMsg},
        utils::is_exit_key,
        Component, View,
    },
};

pub struct ContestBrowser {
    pub tabs: ContestBrowserTabs,
    pub problems_list: ProblemsList,
    pub standings_list: StandingsList,
    pub submissions_list: SubmissionsList,
    pub contest: Contest,
}

impl View for ContestBrowser {
    fn render(&mut self, frame: &mut tuirealm::Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(frame.size());
        self.tabs.render(frame, chunks[0]);
        match self.tabs.selected() {
            0 => self.problems_list.render(frame, chunks[1]),
            1 => self.standings_list.render(frame, chunks[1]),
            2 => self.submissions_list.render(frame, chunks[1]),
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
            0 => self.problems_list.on(event)?,
            1 => self.standings_list.on(event)?,
            2 => self.submissions_list.on(event)?,
            _ => unreachable!(),
        };
        match self.handle_msg(container_msg)? {
            ViewMsg::None => (),
            msg => return Ok(msg),
        }

        Ok(ViewMsg::None)
    }
}

impl ContestBrowser {
    pub fn new(contest: Contest) -> Result<Self> {
        Ok(Self {
            tabs: ContestBrowserTabs::default(),
            problems_list: ProblemsList::new(contest.clone())?,
            standings_list: StandingsList::new(contest.clone())?,
            submissions_list: SubmissionsList::new(contest.clone())?,
            contest,
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
