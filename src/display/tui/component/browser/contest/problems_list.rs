use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use lazy_static::lazy_static;
use tuirealm::{
    props::{Alignment, Color, TextSpan},
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
        utils::BASEURL,
    },
    display::tui::{
        app::POPUP,
        base_component::Table,
        event::AppEvent,
        msg::ComponentMsg,
        types::TextSpans,
        utils::{is_enter_key, is_scroll_down, is_scroll_up},
        view::PopupView,
        BaseComponent, Component,
    },
    settings::SETTINGS,
};

pub struct ProblemsList {
    contest: Contest,
    component: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    problems: Arc<Mutex<Vec<Problem>>>,
}

fn is_key(key: KeyEvent, code: KeyCode, modifiers: KeyModifiers) -> bool {
    key.code == code && key.modifiers == modifiers
}

fn is_up_key(evt: KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('k'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Up, KeyModifiers::NONE)
}

fn is_down_key(evt: KeyEvent) -> bool {
    is_key(evt, KeyCode::Char('j'), KeyModifiers::NONE)
        || is_key(evt, KeyCode::Down, KeyModifiers::NONE)
}

fn is_refresh_key(evt: KeyEvent) -> bool {
    is_key(evt, KeyCode::F(5), KeyModifiers::NONE)
}

impl Component for ProblemsList {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        let mut component = match self.component.try_lock() {
            Ok(component) => component,
            Err(_err) => return Ok(ComponentMsg::Locked),
        };
        let problems = match self.problems.try_lock() {
            Ok(problems) => problems,
            Err(_err) => return Ok(ComponentMsg::Locked),
        };
        match *event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                component.prev();
                Ok(ComponentMsg::ChangedTo(component.selected()))
            }
            AppEvent::Mouse(evt) if is_scroll_up(evt) => {
                component.prev();
                Ok(ComponentMsg::ChangedTo(component.selected()))
            }
            AppEvent::Key(evt) if is_down_key(evt) => {
                component.next();
                Ok(ComponentMsg::ChangedTo(component.selected()))
            }
            AppEvent::Mouse(evt) if is_scroll_down(evt) => {
                component.next();
                Ok(ComponentMsg::ChangedTo(component.selected()))
            }
            AppEvent::Key(evt) if is_refresh_key(evt) => {
                drop(component);
                drop(problems);
                self.update()?;
                Ok(ComponentMsg::Update)
            }
            AppEvent::Key(evt) if is_enter_key(evt) => {
                let index = component.selected();
                let index = problems
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}\nCommonly this is a problem of the application."
                    ))?
                    .index
                    .clone();
                drop(component);
                drop(problems);
                let contest_id = self.contest.id;
                let url = format!("{BASEURL}contest/{contest_id}/problem/{index}");
                webbrowser::open(url.as_str())?;
                Ok(ComponentMsg::Opened(url))
            }
            _ => Ok(ComponentMsg::None),
        }
    }

    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        match self.updating.try_lock() {
            Ok(updating) if *updating == false => match self.component.try_lock() {
                Ok(mut component) => component.render(frame, area),
                Err(_err) => self.render_loading(frame, area),
            },
            _ => self.render_loading(frame, area),
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

async fn update(
    contest_id: i32,
    table: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    problems: Arc<Mutex<Vec<Problem>>>,
) -> Result<()> {
    let results = contest_standings(contest_id, None, None, None, None, Some(true))
        .await?
        .problems;
    *problems.lock().unwrap() = results.clone();
    let mut problems: HashMap<String, (String, String)> = results
        .into_iter()
        .map(|problem| (problem.index, (problem.name, String::from(""))))
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
            match problems.get(&index) {
                Some((name, prev_status)) if prev_status == "" => {
                    problems.insert(index, (name.to_string(), status));
                }
                Some((name, prev_status)) if prev_status == "Rejected" => {
                    problems.insert(index, (name.to_string(), status));
                }
                _ => (),
            }
        }
    }
    let mut problems: Vec<(String, String, String)> = problems
        .into_iter()
        .map(|(index, (name, status))| (index, name, status))
        .collect();
    problems.sort_by_key(|problem| problem.0.clone());
    let items: Vec<Vec<TextSpans>> = problems
        .into_iter()
        .map(|(index, name, status)| {
            vec![
                TextSpan::new(index),
                TextSpan::new(name),
                if status == "Accepted" {
                    TextSpan::new("Accepted").fg(Color::Green)
                } else if status == "Rejected" {
                    TextSpan::new("Rejected").fg(Color::Red)
                } else {
                    TextSpan::new("Unrated").fg(Color::DarkGray)
                },
            ]
            .into_iter()
            .map(|text| text.into())
            .collect::<Vec<TextSpans>>()
        })
        .collect();

    table.lock().unwrap().set_items(items);
    *updating.lock().unwrap() = false;
    Ok(())
}

impl ProblemsList {
    pub fn new(contest: Contest) -> Result<Self> {
        let table = Table::new(
            DEFAULT_HEADER.clone(),
            DEFAULT_WIDTHS.clone(),
            contest.name.clone(),
        );
        let mut default = Self {
            contest,
            component: Arc::new(Mutex::new(table)),
            updating: Arc::new(Mutex::new(false)),
            problems: Arc::new(Mutex::new(vec![])),
        };
        default.update()?;
        Ok(default)
    }

    pub fn update(&mut self) -> Result<()> {
        match self.updating.try_lock() {
            Ok(mut updating) => *updating = true,
            Err(_) => return Ok(()),
        }
        let table = Arc::clone(&self.component);
        let updating = Arc::clone(&self.updating);
        let contest_id = self.contest.id;
        let problems = Arc::clone(&self.problems);
        let popup = Arc::clone(&POPUP);
        tokio::spawn(async move {
            if let Err(err) = update(contest_id, table, updating.clone(), problems).await {
                if let Ok(mut popup) = popup.lock() {
                    *popup = Some(PopupView::new(
                        TextSpan::new("Error from Problems").fg(Color::Red),
                        format!("{err:#}"),
                    ))
                }
            }
            if let Ok(mut updating) = updating.try_lock() {
                *updating = false;
            }
        });
        Ok(())
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
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
    }
}
