use std::sync::{Arc, Mutex};

use bytesize::ByteSize;
use chrono::prelude::*;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};

use lazy_static::lazy_static;
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
        methods::contest_status,
        objects::{Contest, Submission, SubmissionVerdict},
        utils::BASEURL,
    },
    display::tui::{
        app::POPUP,
        base_component::Table,
        event::AppEvent,
        msg::ComponentMsg,
        utils::{
            is_down_key, is_enter_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key,
        },
        view::PopupView,
        BaseComponent, Component,
    },
    settings::SETTINGS,
};

pub struct SubmissionsList {
    contest: Contest,
    component: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    submissions: Arc<Mutex<Vec<Submission>>>,
}

impl Component for SubmissionsList {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        let mut component = match self.component.try_lock() {
            Ok(component) => component,
            Err(_err) => return Ok(ComponentMsg::Locked),
        };
        let submissions = match self.submissions.try_lock() {
            Ok(submissions) => submissions,
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
                drop(submissions);
                self.update()?;
                Ok(ComponentMsg::Update)
            }
            AppEvent::Key(evt) if is_enter_key(evt) => {
                let index = component.selected();
                let id = submissions
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}\nCommonly this is a problem of the application."
                    ))?
                    .id
                    .clone();
                drop(component);
                drop(submissions);
                let contest_id = self.contest.id;
                let url = format!("{BASEURL}contest/{contest_id}/submission/{id}");
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
    pub static ref DEFAULT_HEADER: Vec<String> = vec![
        String::from("When"),
        String::from("Problem"),
        String::from("Verdict"),
        String::from("Time"),
        String::from("Memory"),
    ];
    pub static ref DEFAULT_WIDTHS: Vec<Constraint> = vec![
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
    ];
}

async fn update(
    contest_id: i32,
    table: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    submissions: Arc<Mutex<Vec<Submission>>>,
) -> Result<()> {
    if let None = SETTINGS.username {
        bail!(
            "No username configured.\
               Please configure your usename."
        );
    }
    let results = contest_status(contest_id, SETTINGS.username.clone(), None, None).await?;
    *submissions.lock().unwrap() = results.clone();

    let items = results
        .into_iter()
        .map(|submission| {
            let naive = NaiveDateTime::from_timestamp_opt(submission.creationTimeSeconds, 0)
                .unwrap_or(NaiveDateTime::default());
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let when = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            let name = format!("{} - {}", submission.problem.index, submission.problem.name);

            let verdict = match submission.verdict {
                Some(SubmissionVerdict::OK) => TextSpan::new("Accepted").fg(Color::Green),
                Some(SubmissionVerdict::TESTING) => TextSpan::new(format!(
                    "Testing on test {}",
                    submission.passedTestCount + 1
                ))
                .fg(Color::Blue),
                Some(verdict) => TextSpan::new(format!(
                    "{} on test {}",
                    verdict,
                    submission.passedTestCount + 1
                ))
                .fg(Color::Red),
                None => TextSpan::new("Failed").fg(Color::Red),
            };

            let time_consumed = format!("{}ms", submission.timeConsumedMillis);
            let memory_consumed = format!(
                "{}",
                ByteSize::b(submission.memoryConsumedBytes).to_string_as(false)
            );

            vec![
                TextSpan::new(when),
                TextSpan::new(name),
                verdict,
                TextSpan::new(time_consumed),
                TextSpan::new(memory_consumed),
            ]
        })
        .collect();

    table.lock().unwrap().set_items(items);
    *updating.lock().unwrap() = false;
    Ok(())
}

impl SubmissionsList {
    pub fn new(contest: Contest) -> Result<Self> {
        let table = Table::new(
            DEFAULT_HEADER.clone(),
            DEFAULT_WIDTHS.clone(),
            contest.name.clone(),
        );
        let mut default = SubmissionsList {
            contest,
            component: Arc::new(Mutex::new(table)),
            updating: Arc::new(Mutex::new(false)),
            submissions: Arc::new(Mutex::new(vec![])),
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
        let submissions = Arc::clone(&self.submissions);
        let popup = Arc::clone(&POPUP);
        tokio::spawn(async move {
            if let Err(err) = update(contest_id, table, updating.clone(), submissions).await {
                if let Ok(mut popup) = popup.lock() {
                    *popup = Some(PopupView::new(
                        TextSpan::new("Error from Submission").fg(Color::Red),
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
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
    }
}
