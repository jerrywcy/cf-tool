use color_eyre::Result;
use tuirealm::{
    tui::{
        layout::{Constraint, Direction, Layout},
        widgets::Clear,
    },
    Frame,
};

use crate::display::tui::{
    component::ContestParsePopup,
    event::AppEvent,
    msg::{ChannelHandler, ComponentMsg, ViewMsg},
    utils::is_exit_key,
    Component,
};

use super::{View, ViewSender};

pub struct ContestProblemParsePopupView {
    sender: ViewSender,
    handler: ChannelHandler<ComponentMsg>,
    component: ContestParsePopup,
}

impl View for ContestProblemParsePopupView {
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

impl ContestProblemParsePopupView {
    pub fn new(sender: ViewSender, contest_id: i32, problem_index: String) -> Self {
        let handler = ChannelHandler::new();
        let component = ContestParsePopup::new(handler.sender.clone(), contest_id, problem_index);
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
