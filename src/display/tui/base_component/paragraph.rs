use tuirealm::{
    props::{Alignment, BorderType},
    tui::{
        layout::Rect,
        text::Spans,
        widgets::{Block, Borders, Paragraph as TuiParagraph, Wrap},
    },
    Frame,
};

use crate::display::tui::types::TextSpans;

use super::BaseComponent;

#[derive(Clone)]
pub struct Paragraph {
    pub scroll: u16,
    title: TextSpans,
    text: TextSpans,
}

impl BaseComponent for Paragraph {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let text: Spans = self.text.clone().into();
        let title: Spans = self.title.clone().into();
        let paragraph = TuiParagraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            )
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    }
}

impl Paragraph {
    pub fn new(title: impl Into<TextSpans>, text: impl Into<TextSpans>) -> Self {
        Self {
            scroll: 0,
            title: title.into(),
            text: text.into(),
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll >= 1 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll + 1 < self.text.height() {
            self.scroll += 1
        }
    }
}
