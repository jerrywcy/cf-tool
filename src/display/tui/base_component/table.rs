use lazy_static::lazy_static;
use tuirealm::{
    props::{BorderType, Color},
    tui::{
        layout::{Constraint, Rect},
        style::{Modifier, Style},
        widgets::{Block, Borders, Cell, Row, Table as TuiTable, TableState},
    },
    Frame,
};

use crate::display::tui::types::{Text, TextSpans};

use super::BaseComponent;

pub struct Table {
    pub state: TableState,
    pub items: Vec<Vec<Text>>,
    pub header: Vec<Text>,
    pub widths: Vec<Constraint>,
    pub title: TextSpans,
}

lazy_static! {
    static ref HEADER_STYLE: Style = Style::default().fg(Color::Black);
    static ref TABLE_HIGHLIGHT_STYLE: Style = Style::default().add_modifier(Modifier::REVERSED);
}

impl BaseComponent for Table {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let header: Vec<Cell> = self
            .header
            .iter()
            .map(|text| text.clone().bg(Color::Magenta).into())
            .collect();
        let header = Row::new(header)
            .style(Style::default().bg(Color::Magenta))
            .height(
                self.header
                    .iter()
                    .map(|text| text.height().try_into().unwrap_or(u16::MAX))
                    .max()
                    .unwrap_or(1),
            );

        let rows = self.items.iter().map(|texts| {
            let cells: Vec<Cell> = texts.iter().map(|text| text.clone().into()).collect();
            Row::new(cells).height(
                texts
                    .iter()
                    .map(|text| text.height().try_into().unwrap_or(u16::MAX))
                    .max()
                    .unwrap_or(1),
            )
        });
        let title = self.title.clone();
        let table = TuiTable::new(rows)
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
            .widths(&self.widths)
            .highlight_style(TABLE_HIGHLIGHT_STYLE.clone());
        frame.render_stateful_widget(table, area, &mut self.state);
    }
}

impl Table {
    pub fn new(
        header: Vec<impl Into<Text>>,
        widths: Vec<Constraint>,
        title: impl Into<TextSpans>,
    ) -> Self {
        let header = header.into_iter().map(|text| text.into()).collect();
        let title = title.into();
        Self {
            title,
            header,
            items: vec![],
            state: TableState::default(),
            widths,
        }
    }

    pub fn selected(&self) -> usize {
        match self.state.selected() {
            Some(i) => i,
            None => 0,
        }
    }

    pub fn select(&mut self, index: usize) {
        self.state.select(Some(index))
    }

    pub fn set_items(&mut self, items: Vec<Vec<impl Into<Text>>>) -> &mut Self {
        self.items = items
            .into_iter()
            .map(|texts| texts.into_iter().map(|text| text.into()).collect())
            .collect();
        self
    }

    pub fn set_header(&mut self, header: Vec<impl Into<Text>>) -> &mut Self {
        self.header = header.into_iter().map(|text| text.into()).collect();
        self
    }

    pub fn set_title(&mut self, title: impl Into<TextSpans>) -> &mut Self {
        self.title = title.into();
        self
    }

    pub fn set_widths(&mut self, widths: Vec<Constraint>) -> &mut Self {
        self.widths = widths;
        self
    }

    pub fn next(&mut self) {
        self.state.select(Some(match self.state.selected() {
            Some(i) => {
                if i + 1 < self.items.len() {
                    i + 1
                } else {
                    if self.items.len() == 0 {
                        0
                    } else {
                        self.items.len() - 1
                    }
                }
            }
            None => 0,
        }))
    }

    pub fn prev(&mut self) {
        self.state.select(Some(match self.state.selected() {
            Some(i) => {
                if i >= 1 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        }))
    }
}
