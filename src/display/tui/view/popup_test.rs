use std::path::PathBuf;

use color_eyre::Result;
use tuirealm::{
    tui::{
        layout::{Constraint, Direction, Layout},
        widgets::Clear,
    },
    Frame,
};

use crate::{
    api::parse::TestCase,
    display::tui::{
        component::TestPopup,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewMsg},
        utils::is_exit_key,
        Component,
    },
    settings::Scripts,
};

use super::{View, ViewSender};

pub struct TestPopupView {
    sender: ViewSender,
    handler: ChannelHandler<ComponentMsg>,
    component: TestPopup,
}

impl View for TestPopupView {
    fn render(&mut self, frame: &mut Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(3, 5),
                    Constraint::Ratio(1, 5),
                ]
                .as_ref(),
            )
            .split(frame.size());
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(1, 5),
                    Constraint::Ratio(3, 5),
                    Constraint::Ratio(1, 5),
                ]
                .as_ref(),
            )
            .split(chunks[1]);
        frame.render_widget(Clear, chunks[1]);
        self.component.render(frame, chunks[1]);
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
                self.component.on(event)?;
            }
        }

        while let Ok(msg) = self.handler.try_next() {
            self.handle_msg(msg)?;
        }
        Ok(())
    }

    fn tick(&mut self) {
        self.component.tick();
    }

    fn is_fullscreen(&self) -> bool {
        false
    }
}

impl TestPopupView {
    pub fn new(
        sender: ViewSender,
        scripts: Scripts,
        test_cases: Vec<TestCase>,
        file_path: PathBuf,
        title: String,
    ) -> Self {
        let handler = ChannelHandler::new();
        let component = TestPopup::new(
            handler.sender.clone(),
            scripts,
            test_cases,
            file_path,
            title,
        );
        Self {
            sender,
            handler,
            component,
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

            _ => (),
        };
        Ok(())
    }
}
