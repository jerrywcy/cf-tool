use color_eyre::Result;
use tuirealm::{
    tui::{layout::Constraint, widgets::Clear},
    Frame,
};

use crate::display::tui::{
    component::{HandleSelectionFn, SelectPopup},
    event::AppEvent,
    msg::{ChannelHandler, ComponentMsg, ViewMsg},
    types::{Text, TextSpans},
    utils::is_exit_key,
    view::ViewSender,
    Component, View,
};

use super::GetChunkFn;

pub struct SelectPopupView {
    sender: ViewSender,
    handler: ChannelHandler<ComponentMsg>,
    get_chunk: GetChunkFn,
    component: SelectPopup,
}

impl View for SelectPopupView {
    fn render(&mut self, frame: &mut Frame<'_>) {
        let chunk = (self.get_chunk)(frame.size());
        frame.render_widget(Clear, chunk);
        self.component.render(frame, chunk);
    }

    fn handle_event(&mut self, event: &AppEvent) -> Result<()> {
        match event {
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

    fn on_exit(self) {}

    fn is_fullscreen(&self) -> bool {
        false
    }
}

impl SelectPopupView {
    pub fn new(
        sender: ViewSender,
        get_chunk: GetChunkFn,
        handle_selection: HandleSelectionFn,
        title: impl Into<TextSpans>,
        header: Vec<impl Into<Text>>,
        widths: Vec<Constraint>,
        items: Vec<Vec<impl Into<Text>>>,
    ) -> Self {
        let handler = ChannelHandler::new();
        let component = SelectPopup::new(
            handler.sender.clone(),
            handle_selection,
            title,
            header,
            widths,
            items,
        );
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
