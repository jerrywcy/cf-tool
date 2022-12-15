use color_eyre::Result;
use tuirealm::Frame;

use super::{event::AppEvent, msg::ViewMsg};

mod browser;
mod popup;

pub use browser::{ContestBrowser, MainBrowser};
pub use popup::PopupView;

pub trait View {
    fn render(&mut self, frame: &mut Frame<'_>);
    fn handle_event(&mut self, event: &AppEvent) -> Result<ViewMsg>;
}
