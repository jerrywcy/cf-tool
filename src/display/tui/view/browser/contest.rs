use color_eyre::Result;

use tuirealm::tui::layout::{Constraint, Direction, Layout};

use crate::{
    api::objects::Contest,
    display::tui::{
        component::{ContestBrowserTabs, ProblemsList, StandingsList, SubmissionsList},
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewMsg},
        utils::is_exit_key,
        view::ViewSender,
        Component, View,
    },
};

pub struct ContestBrowser {
    pub sender: ViewSender,
    pub handler: ChannelHandler<ComponentMsg>,
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
    }

    fn handle_event(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Tick => {
                self.tick();
            }
            AppEvent::Key(evt) if is_exit_key(evt) => {
                self.send(ViewMsg::ExitCurrentView)?;
            }
            event => {
                self.tabs.on(event)?;
                match self.tabs.selected() {
                    0 => self.problems_list.on(event)?,
                    1 => self.standings_list.on(event)?,
                    2 => self.submissions_list.on(event)?,
                    _ => unreachable!(),
                };
            }
        }

        while let Ok(msg) = self.handler.try_next() {
            self.handle_msg(msg)?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        match self.tabs.selected() {
            0 => self.problems_list.tick(),
            1 => self.standings_list.tick(),
            2 => self.submissions_list.tick(),
            _ => unreachable!(),
        }
    }

    fn is_fullscreen(&self) -> bool {
        true
    }
}

impl ContestBrowser {
    pub fn new(sender: ViewSender, contest: Contest) -> Self {
        let handler = ChannelHandler::new();
        let tabs = ContestBrowserTabs::new(handler.sender.clone());
        let mut problems_list = ProblemsList::new(handler.sender.clone(), contest.clone());
        let mut standings_list = StandingsList::new(handler.sender.clone(), contest.clone());
        let mut submissions_list = SubmissionsList::new(handler.sender.clone(), contest.clone());

        problems_list.update();
        standings_list.update();
        submissions_list.update();

        Self {
            sender,
            handler,
            tabs,
            problems_list,
            standings_list,
            submissions_list,
            contest,
        }
    }

    fn send(&mut self, msg: ViewMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }

    fn handle_msg(&mut self, msg: ComponentMsg) -> Result<()> {
        match msg {
            ComponentMsg::AppClose => {
                self.send(ViewMsg::AppClose)?;
            }
            ComponentMsg::EnterNewView(constructor) => {
                self.send(ViewMsg::EnterNewView(constructor))?;
            }
            ComponentMsg::ExitCurrentView => {
                self.send(ViewMsg::ExitCurrentView)?;
            }
            ComponentMsg::ChangeToTab(index) => {
                self.tabs.select(index)?;
            }
            _ => (),
        };
        Ok(())
    }
}
