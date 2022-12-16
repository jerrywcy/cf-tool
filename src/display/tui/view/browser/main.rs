use color_eyre::Result;

use tuirealm::tui::layout::{Constraint, Direction, Layout};

use crate::display::tui::{
    component::{ContestList, MainBrowserTabs, ProblemsetList},
    event::AppEvent,
    msg::{ChannelHandler, ComponentMsg, ViewMsg},
    utils::is_exit_key,
    view::ViewSender,
    Component, View,
};

pub struct MainBrowser {
    pub sender: ViewSender,
    pub handler: ChannelHandler<ComponentMsg>,
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
    }

    fn handle_event(&mut self, event: &AppEvent) -> Result<()> {
        if let AppEvent::Key(evt) = event {
            if is_exit_key(*evt) {
                self.send(ViewMsg::ExitCurrentView)?;
                return Ok(());
            }
        }
        self.tabs.on(event)?;
        match self.tabs.selected() {
            0 => self.contest_list.on(event)?,
            1 => self.problemset_list.on(event)?,
            _ => unreachable!(),
        };
        while let Ok(msg) = self.handler.try_next() {
            self.handle_msg(msg)?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        match self.tabs.selected() {
            0 => self.contest_list.tick(),
            1 => self.problemset_list.tick(),
            _ => unreachable!(),
        }
    }

    fn is_fullscreen(&self) -> bool {
        true
    }
}

impl MainBrowser {
    pub fn new(sender: ViewSender) -> Self {
        let handler = ChannelHandler::new();
        let tabs = MainBrowserTabs::new(handler.sender.clone());
        let mut contest_list = ContestList::new(handler.sender.clone());
        let mut problemset_list = ProblemsetList::new(handler.sender.clone());

        contest_list.update();
        problemset_list.update();

        Self {
            sender,
            handler,
            tabs,
            contest_list,
            problemset_list,
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
