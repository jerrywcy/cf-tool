use tuirealm::{
    props::{Alignment, BorderType},
    tui::{
        layout::Rect,
        widgets::{Block, Borders, Paragraph as TuiParagraph, Wrap},
    },
    Frame,
};

use crate::display::tui::types::{Text, TextSpans};

use super::BaseComponent;

#[derive(Clone)]
pub struct Paragraph {
    pub scroll: u16,
    title: TextSpans,
    text: Text,
}

impl BaseComponent for Paragraph {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let text = self.text.clone();
        let title = self.title.clone();
        let paragraph = TuiParagraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
            .scroll((self.scroll, 0))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}

impl Paragraph {
    pub fn new(title: impl Into<TextSpans>, text: impl Into<Text>) -> Self {
        Self {
            scroll: 0,
            title: title.into(),
            text: text.into(),
        }
    }

    pub fn set_text(&mut self, text: impl Into<Text>) -> &mut Self {
        self.text = text.into();
        self
    }

    pub fn push_line(&mut self, content: impl Into<TextSpans>) -> &mut Self {
        self.text.push(content);
        self
    }

    pub fn change_line(&mut self, index: usize, content: impl Into<TextSpans>) -> &mut Self {
        self.text.change(index, content);
        self
    }

    pub fn scroll_down(&mut self) {
        if usize::from(self.scroll + 1) < self.text.height() {
            self.scroll += 1
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll >= 1 {
            self.scroll -= 1;
        }
    }
}
