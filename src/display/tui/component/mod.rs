use color_eyre::Result;

use tuirealm::{tui::layout::Rect, Frame};

use super::{event::AppEvent, msg::ComponentMsg};

mod browser;
mod popup;
pub mod utils;

pub use browser::{ContestBrowserTabs, ProblemsList, StandingsList, SubmissionsList};
pub use browser::{ContestList, MainBrowserTabs, ProblemsetList};
pub use popup::Popup;

pub trait Component {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg>;
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect);
}
