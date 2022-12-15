use tuirealm::{
    props::{Style, TextSpan},
    tui::{
        text::{Span, Spans},
        widgets::Cell,
    },
};

#[derive(Clone)]
pub struct TextSpans(Vec<TextSpan>);

impl TextSpans {
    pub fn height(&self) -> u16 {
        let mut height: u16 = 1;
        self.0.iter().for_each(|text| {
            height += text
                .content
                .chars()
                .filter(|ch| *ch == '\n')
                .count()
                .try_into()
                .unwrap_or(0);
        });
        height
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
