use color_eyre::Report;
use similar::{ChangeTag, TextDiff};
use tokio::process::Command;
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

    pub fn change(&mut self, index: usize, content: impl Into<TextSpans>) -> &mut Self {
        match self.lines.get_mut(index) {
            Some(line) => {
                *line = content.into();
            }
            None => {
                self.lines.append(
                    &mut (self.lines.len()..index)
                        .map(|_| TextSpans::from(""))
                        .collect(),
                );
                self.lines.push(content.into());
            }
        }
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

impl From<Vec<&str>> for Text {
    fn from(texts: Vec<&str>) -> Self {
        let lines = texts.into_iter().map(|line| line.into()).collect();
        Self { lines }
    }
}

impl From<Vec<String>> for Text {
    fn from(texts: Vec<String>) -> Self {
        let lines = texts.into_iter().map(|line| line.into()).collect();
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

impl Into<Vec<TextSpans>> for Text {
    fn into(self) -> Vec<TextSpans> {
        self.lines
    }
}

impl<'a> Into<text::Text<'a>> for Text {
    fn into(self) -> text::Text<'a> {
        let lines: Vec<Spans> = self.lines.into_iter().map(|line| line.into()).collect();
        text::Text { lines }
    }
}

pub enum TestResult {
    Accepted,
    WrongAnswer(String, String, String),
    TimeLimitExceeded,
    Testing,
    Err(Report),
}

impl TestResult {
    pub fn format(&self, id: usize) -> Text {
        match self {
            TestResult::Accepted => {
                Text::from(TextSpan::new(format!("Passed #{id}.")).fg(Color::Green))
            }
            TestResult::WrongAnswer(input, output, answer) => {
                let diff: Vec<TextSpans> = TextDiff::from_lines(output, answer)
                    .iter_all_changes()
                    .map(|line| {
                        match line.tag() {
                            ChangeTag::Equal => TextSpan::new(line.value()),
                            ChangeTag::Insert => TextSpan::new(line.value()).fg(Color::Green),
                            ChangeTag::Delete => TextSpan::new(line.value()).fg(Color::Red),
                        }
                        .into()
                    })
                    .collect();
                Text::from(vec![
                    Text::from(TextSpan::new(format!("Wrong Answer on Test #{id}")).fg(Color::Red)),
                    Text::from("--- Input ---"),
                    Text::from(input.clone()),
                    Text::from("--- Output ---"),
                    Text::from(output.clone()),
                    Text::from("--- Answer --- "),
                    Text::from(answer.clone()),
                    Text::from("--- Diff ---"),
                    Text::from(diff),
                ])
            }
            TestResult::TimeLimitExceeded => Text::from(
                TextSpan::new(format!("Time Limit Exceeded on Test #{id}")).fg(Color::Blue),
            ),
            TestResult::Err(err) => Text::from(
                TextSpan::new(format!("Error occured on Test #{id}: {err:#?}")).fg(Color::Red),
            ),
            TestResult::Testing => Text::from(format!("Testing #{id}...")),
        }
    }
}

#[derive(Debug)]
pub struct TestCommands {
    pub before_command: Option<Command>,
    pub command: Command,
    pub after_command: Option<Command>,
}
