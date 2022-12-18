use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{read_dir, read_to_string, DirBuilder},
    path::PathBuf,
};

use color_eyre::{
    eyre::{bail, eyre, Context},
    Result,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use lazy_static::lazy_static;
use std::sync::mpsc;

use tuirealm::{
    props::{Alignment, BorderType, Color, TextSpan},
    tui::{
        layout::{Constraint, Rect},
        widgets::{Block, Borders, Paragraph},
    },
    Frame,
};

use crate::{
    api::{
        methods::{contest_standings, contest_status},
        objects::{Contest, Problem, SubmissionVerdict},
        parse::TestCase,
        utils::BASEURL,
    },
    display::tui::{
        base_component::Table,
        component::ComponentSender,
        error::NoConfigItemError,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewConstructor},
        types::Text,
        utils::{
            is_down_key, is_enter_key, is_key, is_refresh_key, is_scroll_down, is_scroll_up,
            is_up_key,
        },
        BaseComponent, Component,
    },
    settings::{Scripts, SETTINGS},
};

#[derive(Debug, Default)]
struct UpdateResult {
    problems: Vec<Problem>,
    items: Vec<Vec<Text>>,
}

pub struct ProblemsList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    contest: Contest,
    component: Table,
    updating: u32,
    problems: Vec<Problem>,
}

fn is_test_key(evt: &KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('t'), KeyModifiers::NONE)
}

fn get_test_cases(path: &PathBuf) -> Vec<TestCase> {
    let mut i = 1;
    let mut test_cases = vec![];
    loop {
        let input = match read_to_string(path.join(format!("in{i}.txt"))) {
            Ok(input) => input,
            Err(_) => break,
        };
        let answer = match read_to_string(path.join(format!("ans{i}.txt"))) {
            Ok(answer) => answer,
            Err(_) => break,
        };
        test_cases.push(TestCase { input, answer });
        i += 1;
    }
    return test_cases;
}

fn get_file_path_and_scripts(path: &PathBuf, file_name: &str) -> Result<(PathBuf, Scripts)> {
    for entry in read_dir(path)? {
        if let Ok(file) = entry {
            let file_path = file.path();
            if file_path.file_stem() == Some(&OsString::from(file_name)) {
                if let Some(file_ext) = file_path.extension() {
                    if let Some(file_ext) = file_ext.to_str() {
                        if let Some(scripts) = SETTINGS
                            .commands
                            .clone()
                            .ok_or(NoConfigItemError {
                                item: "commands".to_string(),
                            })?
                            .get(file_ext)
                        {
                            return Ok((file_path, scripts.clone()));
                        }
                    }
                }
            }
        }
    }
    bail!("Cannot find any code in {}", path.display());
}
impl Component for ProblemsList {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.prev();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                self.component.prev();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                self.component.next();
                self.send(ComponentMsg::ChangedTo(self.component.selected()))?;
            }
            AppEvent::Key(evt) if is_refresh_key(evt) => {
                self.update();
                self.send(ComponentMsg::Update)?;
            }
            AppEvent::Key(evt) if is_enter_key(evt) => {
                let index = self.component.selected();
                let index = self
                    .problems
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}\nCommonly this is a problem of the application."
                    ))?
                    .index
                    .clone();
                let contest_id = self.contest.id;
                let url = format!("{BASEURL}contest/{contest_id}/problem/{index}");
                webbrowser::open(&url)?;
                self.send(ComponentMsg::OpenedWebsite(url))?;
            }
            AppEvent::Key(evt) if is_test_key(evt) => {
                let home_dir = SETTINGS.home_dir.clone().ok_or(NoConfigItemError {
                    item: "home_dir".to_string(),
                })?;

                let contest_id = self.contest.id;
                let index = self.component.selected();
                let problem = self.problems.get(index).ok_or(eyre!(
                    "No such index: {index}\nCommonly this is a problem of the application."
                ))?;
                let problem_index = problem.index.clone();
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
                let test_cases = get_test_cases(&problem_dir);
                if test_cases.is_empty() {
                    bail!("Cannot find any test cases in {}", problem_dir.display());
                }
                let (file_path, scripts) = get_file_path_and_scripts(&problem_dir, &problem_index)?;

                self.send(ComponentMsg::EnterNewView(ViewConstructor::TestPopup(
                    scripts,
                    test_cases,
                    file_path,
                    format!("Test for Problem {problem_index}"),
                )))?;
            }
            _ => (),
        };
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        if self.updating == 0 {
            self.component.render(frame, area);
        } else {
            self.render_loading(frame, area);
        }
    }
}

lazy_static! {
    static ref DEFAULT_HEADER: Vec<String> = vec![
        String::from("#"),
        String::from("Name"),
        String::from("Status")
    ];
    static ref DEFAULT_WIDTHS: Vec<Constraint> = vec![
        Constraint::Length(2),
        Constraint::Percentage(90),
        Constraint::Percentage(10),
    ];
}

async fn update(sender: mpsc::Sender<UpdateResult>, contest_id: i32) -> Result<()> {
    let mut problems = contest_standings(contest_id, None, None, None, None, Some(true))
        .await?
        .problems;
    problems.sort_by_key(|problem| problem.index.clone());

    let mut index_to_problem: HashMap<String, (String, String)> = problems
        .iter()
        .map(|problem| {
            (
                problem.index.clone(),
                (problem.name.clone(), String::from("")),
            )
        })
        .collect();
    if let Some(handle) = SETTINGS.username.clone() {
        let submissions = contest_status(contest_id, Some(handle), None, None).await?;
        for submission in submissions {
            let index = submission.problem.index;
            let status = match submission.verdict {
                Some(SubmissionVerdict::OK) => String::from("Accepted"),
                Some(_) => String::from("Rejected"),
                None => continue,
            };
            match index_to_problem.get(&index) {
                Some((name, prev_status)) if prev_status == "" => {
                    index_to_problem.insert(index, (name.to_string(), status));
                }
                Some((name, prev_status)) if prev_status == "Rejected" => {
                    index_to_problem.insert(index, (name.to_string(), status));
                }
                _ => (),
            }
        }
    }
    let mut problems_tuple: Vec<(String, (String, String))> =
        index_to_problem.into_iter().collect();
    problems_tuple.sort_by_key(|(index, _)| index.clone());
    let items: Vec<Vec<Text>> = problems_tuple
        .into_iter()
        .map(|(index, (name, status))| {
            vec![
                TextSpan::new(index),
                TextSpan::new(name),
                if status == "Accepted" {
                    TextSpan::new("Accepted").fg(Color::Green)
                } else if status == "Rejected" {
                    TextSpan::new("Rejected").fg(Color::Red)
                } else {
                    TextSpan::new("Unrated").fg(Color::Gray)
                },
            ]
            .into_iter()
            .map(|text| text.into())
            .collect::<Vec<Text>>()
        })
        .collect();
    sender.send(UpdateResult { problems, items }).unwrap();

    Ok(())
}

impl ProblemsList {
    pub fn new(sender: ComponentSender, contest: Contest) -> Self {
        let table = Table::new(
            DEFAULT_HEADER.clone(),
            DEFAULT_WIDTHS.clone(),
            contest.name.clone(),
        );
        let handler = ChannelHandler::new();
        Self {
            sender,
            handler,
            contest,
            component: table,
            updating: 0,
            problems: vec![],
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { items, problems } = result;
            self.component.set_items(items);
            self.problems = problems;
            self.updating -= 1;
        }
    }

    pub fn update(&mut self) -> &mut Self {
        self.updating += 1;
        let update_sender = self.handler.sender.clone();
        let popup_sender = self.sender.clone();
        let error_sender = self.handler.sender.clone();
        let contest_id = self.contest.id;
        tokio::spawn(async move {
            if let Err(err) = update(update_sender, contest_id).await {
                error_sender.send(UpdateResult::default()).unwrap();
                popup_sender
                    .send(ComponentMsg::EnterNewView(ViewConstructor::ErrorPopup(
                        String::from("Error from Problems"),
                        format!("{err:#}"),
                    )))
                    .unwrap();
            }
        });
        self
    }

    fn render_loading(&self, frame: &mut Frame, area: Rect) {
        let loading_message = format!(
            "{}Loading...",
            (0..(area.height - 1) / 2)
                .into_iter()
                .map(|_| "\n")
                .collect::<String>()
        );
        let loading = Paragraph::new(loading_message)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
    }
}
