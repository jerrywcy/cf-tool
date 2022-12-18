use tuirealm::{
    props::{Color, Style, TextSpan},
    tui::{
        text::{self, Span, Spans},
        widgets::Cell,
    },
};

#[derive(Clone, Debug)]
pub struct TextSpans(Vec<TextSpan>);

impl TextSpans {
    pub fn fg(self, color: Color) -> Self {
        Self(self.0.into_iter().map(|text| text.fg(color)).collect())
    }

    pub fn bg(self, color: Color) -> Self {
        Self(self.0.into_iter().map(|text| text.bg(color)).collect())
    }
}

impl From<&str> for TextSpans {
    fn from(text: &str) -> Self {
        TextSpans::from(String::from(text))
    }
}

impl From<String> for TextSpans {
    fn from(string: String) -> Self {
        Self(vec![TextSpan::new(string)])
    }
}

impl From<TextSpan> for TextSpans {
    fn from(text: TextSpan) -> Self {
        Self(vec![text])
    }
}

impl From<Vec<TextSpan>> for TextSpans {
    fn from(texts: Vec<TextSpan>) -> Self {
        Self(texts)
    }
}

impl<'a> Into<Spans<'a>> for TextSpans {
    fn into(self) -> Spans<'a> {
        Spans::from(
            self.0
                .into_iter()
                .map(|text| {
                    Span::styled(
                        text.content,
                        Style::default()
                            .fg(text.fg)
                            .bg(text.bg)
                            .add_modifier(text.modifiers),
                    )
                })
                .collect::<Vec<Span>>(),
        )
    }
}

impl<'a> Into<Spans<'a>> for &TextSpans {
    fn into(self) -> Spans<'a> {
        Spans::from(
            (*self)
                .0
                .iter()
                .map(|text| {
                    Span::styled(
                        text.content.clone(),
                        Style::default()
                            .fg(text.fg)
                            .bg(text.bg)
                            .add_modifier(text.modifiers),
                    )
                })
                .collect::<Vec<Span>>(),
        )
    }
}

impl<'a> Into<Cell<'a>> for TextSpans {
    fn into(self) -> Cell<'a> {
        Cell::from(Into::<Spans>::into(self))
    }
}

impl<'a> Into<Cell<'a>> for &TextSpans {
    fn into(self) -> Cell<'a> {
        Cell::from(Into::<Spans>::into(self))
    }
}

#[derive(Debug, Default, Clone)]
pub struct Text {
    pub lines: Vec<TextSpans>,
}

impl Text {
    pub fn push(&mut self, line: impl Into<TextSpans>) -> &mut Self {
        self.lines.push(line.into());
        self
    }
}

impl Text {
    pub fn height(&self) -> usize {
        self.lines.len()
    }
}

impl Text {
    pub fn fg(self, color: Color) -> Self {
        Self {
            lines: self.lines.into_iter().map(|text| text.fg(color)).collect(),
        }
    }

    pub fn bg(self, color: Color) -> Self {
        Self {
            lines: self.lines.into_iter().map(|text| text.bg(color)).collect(),
        }
    }
}

impl From<String> for Text {
    fn from(text: String) -> Self {
        let lines = text.split("\n").map(|line| TextSpans::from(line)).collect();
        Self { lines }
    }
}

impl From<&str> for Text {
    fn from(text: &str) -> Self {
        Self::from(String::from(text))
    }
}

impl From<TextSpan> for Text {
    fn from(text: TextSpan) -> Self {
        let lines: Vec<TextSpans> = text
            .content
            .split("\n")
            .map(|line| {
                TextSpan {
                    content: line.to_string(),
                    ..text
                }
                .into()
            })
            .collect();
        Self { lines }
    }
}

impl From<Vec<Text>> for Text {
    fn from(texts: Vec<Text>) -> Self {
        let mut lines: Vec<TextSpans> = vec![];
        texts
            .into_iter()
            .for_each(|mut text| lines.append(&mut text.lines));
        Self { lines }
    }
}

impl From<Vec<TextSpans>> for Text {
    fn from(text: Vec<TextSpans>) -> Self {
        Self { lines: text }
    }
}

impl<'a> Into<text::Text<'a>> for Text {
    fn into(self) -> text::Text<'a> {
        let lines: Vec<Spans> = self.lines.into_iter().map(|line| line.into()).collect();
        text::Text { lines }
    }
}
