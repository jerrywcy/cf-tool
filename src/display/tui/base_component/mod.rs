use tuirealm::{tui::layout::Rect, Frame};

mod paragraph;
mod table;
mod tabs;

pub use paragraph::Paragraph;
pub use table::Table;
pub use tabs::Tabs;

pub trait BaseComponent {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect);
}
