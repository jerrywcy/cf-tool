use std::sync::mpsc;

use color_eyre::Result;

use tuirealm::{tui::layout::Rect, Frame};

use crate::display::tui::{
    base_component::Paragraph,
    component::ComponentSender,
    event::AppEvent,
    msg::{ChannelHandler, ComponentMsg},
    types::{Text, TextSpans},
    utils::{is_down_key, is_scroll_down, is_scroll_up, is_up_key},
    BaseComponent, Component,
};

pub enum ContentUpdateCmd {
    Push(TextSpans),
    PushLines(Text),
    Change(usize, TextSpans),
    Set(Text),
}

pub type UpdateFn =
    Box<dyn FnOnce(mpsc::Sender<ContentUpdateCmd>, ComponentSender) -> () + Send + Sync>;

pub struct UpdatablePopup {
    sender: ComponentSender,
    handler: ChannelHandler<ContentUpdateCmd>,
    component: Paragraph,
}

impl Component for UpdatablePopup {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }

            _ => (),
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }
}

impl UpdatablePopup {
    pub fn new(
        sender: ComponentSender,
        update: UpdateFn,
        title: impl Into<TextSpans>,
        text: impl Into<Text>,
    ) -> Self {
        let handler = ChannelHandler::new();
        let update_sender = handler.sender.clone();
        let popup_sender = sender.clone();

        update(update_sender, popup_sender);

        let component = Paragraph::new(title, text);
        Self {
            sender,
            handler,
            component,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            match result {
                ContentUpdateCmd::Push(content) => {
                    self.component.push_line(content);
                }
                ContentUpdateCmd::Change(index, content) => {
                    self.component.change_line(index, content);
                }
                ContentUpdateCmd::Set(text) => {
                    self.component.set_text(text);
                }
                ContentUpdateCmd::PushLines(text) => {
                    for line in text.lines {
                        self.component.push_line(line);
                    }
                }
            };
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
