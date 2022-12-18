use std::{
    fs::{self, DirBuilder},
    sync::mpsc,
};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};

use tuirealm::{
    props::{Color, TextSpan},
    tui::layout::Rect,
    Frame,
};

use crate::{
    api::{parse::parse_testcase, utils::BASEURL},
    display::tui::{
        base_component::Paragraph,
        error::NoConfigItemError,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg},
        types::Text,
        utils::{is_down_key, is_scroll_down, is_scroll_up, is_up_key},
        BaseComponent,
    },
    settings::SETTINGS,
};

use super::{Component, ComponentSender};

struct UpdateResult {
    result: Text,
}

pub struct ContestParsePopup {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    component: Paragraph,
}

impl Component for ContestParsePopup {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                self.component.scroll_up();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                self.component.scroll_down();
                self.send(ComponentMsg::ChangeToTab(self.component.scroll.into()))?;
            }
            _ => (),
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        self.component.render(frame, area);
    }
}

async fn parse(
    sender: mpsc::Sender<UpdateResult>,
    contest_id: i32,
    problem_index: String,
) -> Result<()> {
    let home_dir = SETTINGS.home_dir.clone().ok_or(NoConfigItemError {
        item: "home_dir".to_string(),
    })?;
    let problem_dir = home_dir
        .join("Contests")
        .join(contest_id.to_string())
        .join(&problem_index);
    let open_dir_err = eyre!(
        "Failed to open directory when trying to test problem: {}",
        problem_dir.display()
    );
    DirBuilder::new()
        .recursive(true)
        .create(&problem_dir)
        .wrap_err(open_dir_err)?;

    let url = format!("{BASEURL}contest/{contest_id}/problem/{problem_index}");
    let test_cases = parse_testcase(url).await?;
    let mut text = Text::default();
    text.push(
        TextSpan::from(format!(
            "Parsed {} test cases for Problem {problem_index}",
            test_cases.len()
        ))
        .fg(Color::Green),
    );
    for (i, test_case) in test_cases.into_iter().enumerate() {
        let id = i + 1;
        let input_path = problem_dir.join(format!("in{id}.txt"));
        let answer_path = problem_dir.join(format!("ans{id}.txt"));
        fs::write(input_path, test_case.input)?;
        text.push(format!("Parsed in{id}.txt"));
        fs::write(answer_path, test_case.answer)?;
        text.push(format!("Parsed ans{id}.txt"));
    }
    sender.send(UpdateResult { result: text });
    Ok(())
}

impl ContestParsePopup {
    pub fn new(sender: ComponentSender, contest_id: i32, problem_index: String) -> Self {
        let handler = ChannelHandler::new();
        let update_sender = handler.sender.clone();
        let error_sender = handler.sender.clone();
        let title = format!("Parse Problem {problem_index}");
        let text = "Parsing...";
        let component = Paragraph::new(title, text);
        let executor = |update_sender: mpsc::Sender<UpdateResult>,
                        error_sender: mpsc::Sender<UpdateResult>| {
            tokio::spawn(async move {
                if let Err(err) = parse(update_sender, contest_id, problem_index).await {
                    error_sender.send(UpdateResult {
                        result: Text::from(format!("{err:#?}")).fg(Color::Red),
                    });
                }
            });
        };
        executor(update_sender, error_sender);

        Self {
            sender,
            handler,
            component,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { result } = result;
            self.component.set_text(result);
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
