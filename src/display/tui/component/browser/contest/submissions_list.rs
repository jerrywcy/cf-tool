use bytesize::ByteSize;
use chrono::prelude::*;
use color_eyre::{
    eyre::{bail, eyre},
    Result,
};

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
        methods::contest_status,
        objects::{Contest, Submission, SubmissionVerdict},
        utils::BASEURL,
    },
    display::tui::{
        base_component::Table,
        component::ComponentSender,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewConstructor},
        types::Text,
        utils::{
            is_down_key, is_enter_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key,
        },
        BaseComponent, Component,
    },
    settings::SETTINGS,
};

#[derive(Debug, Default)]
struct UpdateResult {
    items: Vec<Vec<Text>>,
    submissions: Vec<Submission>,
}

pub struct SubmissionsList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    contest: Contest,
    component: Table,
    updating: u32,
    submissions: Vec<Submission>,
}

impl Component for SubmissionsList {
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
                let id = self
                    .submissions
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}\nCommonly this is a problem of the application."
                    ))?
                    .id
                    .clone();
                let contest_id = self.contest.id;
                let url = format!("{BASEURL}contest/{contest_id}/submission/{id}");
                webbrowser::open(url.as_str())?;
                self.send(ComponentMsg::OpenedWebsite(url))?;
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

async fn update(sender: mpsc::Sender<UpdateResult>, contest_id: i32) -> Result<()> {
    if let None = SETTINGS.username {
        bail!(
            "No username configured.\
               Please configure your usename."
        );
    }
    let submissions = contest_status(contest_id, SETTINGS.username.clone(), None, None).await?;

    let items: Vec<Vec<Text>> = submissions
        .iter()
        .map(|submission| {
            let naive = NaiveDateTime::from_timestamp_opt(submission.creationTimeSeconds, 0)
                .unwrap_or(NaiveDateTime::default());
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let when = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

            let name = format!(
                "{} - {}",
                submission.problem.index.clone(),
                submission.problem.name.clone()
            );

            let verdict = match submission.verdict.clone() {
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
            .into_iter()
            .map(|text| text.into())
            .collect::<Vec<Text>>()
        })
        .collect();

    sender.send(UpdateResult { items, submissions }).unwrap();
    Ok(())
}

impl SubmissionsList {
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
            submissions: vec![],
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { items, submissions } = result;
            self.component.set_items(items);
            self.submissions = submissions;
            self.updating -= 1;
        }
    }

    fn send(&mut self, msg: ComponentMsg) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn update(&mut self) -> &mut Self {
        let update_sender = self.handler.sender.clone();
        let popup_sender = self.sender.clone();
        let error_sender = self.handler.sender.clone();
        let contest_id = self.contest.id;
        self.updating += 1;

        tokio::spawn(async move {
            if let Err(err) = update(update_sender, contest_id).await {
                error_sender.send(UpdateResult::default()).unwrap();
                popup_sender
                    .send(ComponentMsg::EnterNewView(ViewConstructor::ErrorPopup(
                        String::from("Error from Submission"),
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
