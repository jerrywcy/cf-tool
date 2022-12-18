use color_eyre::Result;

use tuirealm::{
    tui::{
        layout::{Constraint, Direction, Layout, Rect},
        widgets::Clear,
    },
    Frame,
};

use crate::display::tui::{
    component::{UpdatablePopup, UpdateFn},
    event::AppEvent,
    msg::{ChannelHandler, ComponentMsg, ViewMsg},
    types::{Text, TextSpans},
    utils::is_exit_key,
    view::ViewSender,
    Component, View,
};

pub fn get_chunk_with_ratio(vertical: (u32, u32, u32), horizontal: (u32, u32, u32)) -> GetChunkFn {
    Box::new(move |area| {
        let vertical_sum = vertical.0 + vertical.1 + vertical.2;
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Ratio(vertical.0, vertical_sum),
                    Constraint::Ratio(vertical.1, vertical_sum),
                    Constraint::Ratio(vertical.2, vertical_sum),
                ]
                .as_ref(),
            )
            .split(area);
        let horizontal_sum = horizontal.0 + horizontal.1 + horizontal.2;
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Ratio(horizontal.0, horizontal_sum),
                    Constraint::Ratio(horizontal.1, horizontal_sum),
                    Constraint::Ratio(horizontal.2, horizontal_sum),
                ]
                .as_ref(),
            )
            .split(chunks[1]);
        chunks[1]
    })
}

pub type GetChunkFn = Box<dyn Fn(Rect) -> Rect + Send + Sync>;

pub struct UpdatablePopupView {
    sender: ViewSender,
    handler: ChannelHandler<ComponentMsg>,
    get_chunk: GetChunkFn,
    component: UpdatablePopup,
}

impl View for UpdatablePopupView {
    fn render(&mut self, frame: &mut Frame<'_>) {
        let chunk = (self.get_chunk)(frame.size());
        frame.render_widget(Clear, chunk);
        self.component.render(frame, chunk);
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

impl UpdatablePopupView {
    pub fn new(
        sender: ViewSender,
        get_chunk: GetChunkFn,
        update: UpdateFn,
        title: impl Into<TextSpans>,
        text: impl Into<Text>,
    ) -> Self {
        let handler = ChannelHandler::new();
        let component = UpdatablePopup::new(handler.sender.clone(), update, title, text);
        Self {
            sender,
            handler,
            component,
            get_chunk,
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
