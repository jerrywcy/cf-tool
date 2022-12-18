use std::{
    sync::mpsc::{self, RecvError, TryRecvError},
};

use tuirealm::props::{Color, TextSpan};

use crate::{
    api::{objects::Contest},
};

use super::{
    component::UpdateFn,
    types::{Text, TextSpans},
    view::{ContestBrowser, GetChunkFn, MainBrowser, PopupView, UpdatablePopupView},
    View,
};

pub enum ComponentMsg {
    AppClose,
    ChangedTo(usize),
    EnterNewView(ViewConstructor),
    ExitCurrentView,
    ChangeToTab(usize),
    OpenedWebsite(String),
    Locked,
    Update,
    None,
}

pub enum ViewConstructor {
    MainBrowser,
    ContestBrowser(Contest),
    ErrorPopup(String, String),
    UpdatablePopup(GetChunkFn, UpdateFn, TextSpans, Text),
}

impl ViewConstructor {
    pub fn construct(self, sender: &mpsc::Sender<ViewMsg>) -> Box<dyn View> {
        let sender = sender.clone();
        match self {
            ViewConstructor::MainBrowser => Box::new(MainBrowser::new(sender)),
            ViewConstructor::ContestBrowser(contest) => {
                Box::new(ContestBrowser::new(sender, contest))
            }
            ViewConstructor::ErrorPopup(title, text) => Box::new(PopupView::new(
                sender,
                TextSpan::new(title).fg(Color::Red),
                text,
            )),
            ViewConstructor::UpdatablePopup(get_chunk, update, title, text) => Box::new(
                UpdatablePopupView::new(sender, get_chunk, update, title, text),
            ),
        }
    }
}

pub enum ViewMsg {
    AppClose,
    EnterNewView(ViewConstructor),
    ExitCurrentView,
    None,
}

#[allow(dead_code)]
pub struct ChannelHandler<T> {
    pub sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T> ChannelHandler<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    pub fn try_next(&mut self) -> Result<T, TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn next(&mut self) -> Result<T, RecvError> {
        self.receiver.recv()
    }
}
