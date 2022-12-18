use color_eyre::Result;
use tuirealm::{
    tui::layout::{Constraint, Rect},
    Frame,
};

use crate::display::tui::{
    base_component::Table,
    component::ComponentSender,
    event::AppEvent,
    msg::{ComponentMsg},
    types::{Text, TextSpans},
    utils::{is_down_key, is_enter_key, is_scroll_down, is_scroll_up, is_up_key},
    BaseComponent, Component,
};

pub type HandleSelectionFn = Box<dyn Fn(usize) -> Result<()> + Send + Sync>;

pub struct SelectPopup {
    sender: ComponentSender,
    component: Table,
    handle_selection: HandleSelectionFn,
}

impl Component for SelectPopup {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                self.component.prev();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Key(evt) if is_enter_key(evt) => {
                let index = self.component.selected();
                self.send(ComponentMsg::ExitCurrentView)?;
                (self.handle_selection)(index)?;
            }
            _ => (),
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }
}

impl SelectPopup {
    pub fn new(
        sender: ComponentSender,
        handle_selection: HandleSelectionFn,
        title: impl Into<TextSpans>,
        header: Vec<impl Into<Text>>,
        widths: Vec<Constraint>,
        items: Vec<Vec<impl Into<Text>>>,
    ) -> Self {
        let mut table = Table::new(header, widths, title.into());
        table.set_items(items);
        Self {
            sender,
            handle_selection,
            component: table,
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
