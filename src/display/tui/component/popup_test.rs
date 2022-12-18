use std::{
    path::{PathBuf},
    process::Stdio,
    sync::mpsc,
    time::Duration,
};

use color_eyre::{eyre::eyre, Report, Result};

use similar::{ChangeTag, TextDiff};
use tokio::{io::AsyncWriteExt, process::Command, select, time::sleep};
use tuirealm::{
    props::{Color, TextSpan},
    tui::layout::Rect,
    Frame,
};

use crate::{
    api::parse::TestCase,
    display::tui::{
        base_component::Paragraph,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg},
        types::{Text, TextSpans},
        utils::{is_down_key, is_scroll_down, is_scroll_up, is_up_key},
        BaseComponent,
    },
    settings::Scripts,
};

use super::{Component, ComponentSender};

struct UpdateResult {
    id: usize,
    result: TestResult,
}

enum TestResult {
    Accepted,
    WrongAnswer(String, String, String),
    TimeLimitExceeded,
    Testing,
    Err(Report),
}

impl TestResult {
    fn format(&self, id: usize) -> Text {
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

pub struct TestPopup {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    component: Paragraph,
    results: Vec<TestResult>,
}

impl Component for TestPopup {
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

async fn test(
    sender: mpsc::Sender<UpdateResult>,
    id: usize,
    timeout: Duration,
    test_case: TestCase,
    test_commands: TestCommands,
) -> Result<()> {
    if let Some(mut command) = test_commands.before_command {
        command.spawn()?.wait().await?;
    }

    let mut command = test_commands.command;
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or(eyre!("Failed to write to Stdin"))?;
    stdin.write_all(test_case.input.as_bytes()).await?;
    let delay = sleep(timeout);
    select! {
        _ = delay => {
            sender.send(UpdateResult { id: id, result: TestResult::TimeLimitExceeded }).unwrap();
        }
        result = child.wait_with_output() => {
            match result {
                Ok(output) => {
                    let TestCase{input, answer} = test_case;
                    let output = String::from_utf8_lossy(&output.stdout);
                    let output = output.trim().to_string();
                    let answer = answer.trim().to_string();
                    if output != answer {
                        sender.send(UpdateResult { id, result: TestResult::WrongAnswer(input, output, answer) }).unwrap();
                    }
                    else {
                        sender.send(UpdateResult { id, result: TestResult::Accepted }).unwrap();
                    }
                }
                Err(err) => {
                    sender.send(UpdateResult { id, result: TestResult::Err(Report::from(err)) }).unwrap();
                }
            }
        }
    }

    Ok(())
}

static FULL_PATH_PLACE_HOLDER: &str = "<% full %>";
static PATH_PLACE_HOLDER: &str = "<% path %>";
static FILE_PLACE_HOLDER: &str = "<% file %>";

fn get_command(full_path: &PathBuf, script: &str) -> Result<Command> {
    let _full = full_path.display().to_string();
    let path = full_path
        .parent()
        .ok_or(eyre!("Code file has no parent!"))?
        .display()
        .to_string();
    let file = full_path
        .file_stem()
        .ok_or(eyre!("Code file has no file stem!"))?
        .to_string_lossy();
    let script = script
        .replace(FULL_PATH_PLACE_HOLDER, &full_path.display().to_string())
        .replace(PATH_PLACE_HOLDER, &path)
        .replace(FILE_PLACE_HOLDER, &file);
    let mut command = Command::from(execute::command(script));
    command.current_dir(path);
    Ok(command)
}

fn get_commands(file_path: &PathBuf, scripts: Scripts) -> Result<TestCommands> {
    let before_command = match &scripts.before_script {
        Some(script) => Some(get_command(&file_path, script)?),
        None => None,
    };
    let command = get_command(&file_path, &scripts.script)?;
    let after_command = match &scripts.after_script {
        Some(script) => Some(get_command(&file_path, script)?),
        None => None,
    };
    Ok(TestCommands {
        before_command,
        command,
        after_command,
    })
}

impl TestPopup {
    pub fn new(
        sender: ComponentSender,
        scripts: Scripts,
        test_cases: Vec<TestCase>,
        file_path: PathBuf,
        title: String,
    ) -> Self {
        let handler = ChannelHandler::new();
        let results: Vec<TestResult> = (0..test_cases.len()).map(|_| TestResult::Testing).collect();
        let update_sender = handler.sender.clone();
        let error_sender = handler.sender.clone();
        tokio::spawn(async move {
            for (i, test_case) in test_cases.into_iter().enumerate() {
                let update_sender = update_sender.clone();
                let error_sender = error_sender.clone();
                let commands = match get_commands(&file_path, scripts.clone()) {
                    Ok(commands) => commands,
                    Err(err) => {
                        error_sender.send(UpdateResult {
                            id: i,
                            result: TestResult::Err(err),
                        });
                        continue;
                    }
                };
                if let Err(err) = test(
                    update_sender,
                    i,
                    Duration::from_millis(1000),
                    test_case,
                    commands,
                )
                .await
                {
                    error_sender.send(UpdateResult {
                        id: i,
                        result: TestResult::Err(err),
                    });
                }
            }
        });

        let texts: Vec<Text> = results
            .iter()
            .enumerate()
            .map(|(id, result)| result.format(id + 1))
            .collect();
        let component = Paragraph::new(title, texts);
        Self {
            sender,
            handler,
            component,
            results,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { id, result } = result;
            self.results[id] = result;
            let text: Vec<Text> = self
                .results
                .iter()
                .enumerate()
                .map(|(id, result)| result.format(id + 1))
                .collect();
            self.component.set_text(text);
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }
}
