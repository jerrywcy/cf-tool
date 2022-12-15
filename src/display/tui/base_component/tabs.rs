use color_eyre::{eyre::bail, Result};
use lazy_static::lazy_static;
use tuirealm::{
    props::{BorderType, Color, Style},
    tui::{
        layout::Rect,
        style::Modifier,
        widgets::{self, Block, Borders},
    },
    Frame,
};

use crate::display::tui::types::TextSpans;

use super::BaseComponent;

pub struct Tabs {
    pub titles: Vec<TextSpans>,
    pub index: usize,
}

lazy_static! {
    pub static ref TABS_STYLE: Style = Style::default().fg(Color::Cyan);
    pub static ref TABS_HIGHLIGHT_STYLE: Style = Style::default().add_modifier(Modifier::REVERSED);
}

impl BaseComponent for Tabs {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let titles = self.titles.iter().map(|text| text.into()).collect();
        let tabs = widgets::Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .select(self.index)
            .style(TABS_STYLE.clone())
            .highlight_style(TABS_HIGHLIGHT_STYLE.clone());
        frame.render_widget(tabs, area);
    }
}

impl Tabs {
    pub fn new(titles: Vec<impl Into<TextSpans>>) -> Self {
        let titles = titles.into_iter().map(|text| text.into()).collect();
        Self { titles, index: 0 }
    }

    pub fn selected(&self) -> usize {
        self.index
    }

    pub fn select(&mut self, index: usize) -> Result<()> {
        if index < self.titles.len() {
            self.index = index
        } else {
            bail!("Index out of range: No tab numbered {index}!")
        }
        Ok(())
    }

    pub fn prev(&mut self) {
        if self.index == 0 {
            self.index = self.titles.len() - 1;
        } else {
            self.index -= 1;
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
}
