use color_eyre::Result;

use std::sync::mpsc;
use tuirealm::{tui::layout::Rect, Frame};

use super::{event::AppEvent, msg::ComponentMsg};

mod browser;
mod popup;
pub mod utils;

pub use browser::{ContestBrowserTabs, ProblemsList, StandingsList, SubmissionsList};
pub use browser::{ContestList, MainBrowserTabs, ProblemsetList};
pub use popup::Popup;

pub trait Component {
    fn on(&mut self, event: &AppEvent) -> Result<()>;
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect);
}

pub type ComponentSender = mpsc::Sender<ComponentMsg>;
