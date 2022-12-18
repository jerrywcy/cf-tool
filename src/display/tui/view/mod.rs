use std::sync::mpsc;

use color_eyre::Result;
use tuirealm::Frame;

use super::{event::AppEvent, msg::ViewMsg};

mod browser;
mod popup;
mod popup_test;

pub use browser::{ContestBrowser, MainBrowser};
pub use popup::PopupView;
pub use popup_test::TestPopupView;

pub trait View {
    fn render(&mut self, frame: &mut Frame<'_>);
    fn handle_event(&mut self, event: &AppEvent) -> Result<()>;
    fn tick(&mut self);
    fn is_fullscreen(&self) -> bool;
}

pub type ViewSender = mpsc::Sender<ViewMsg>;
