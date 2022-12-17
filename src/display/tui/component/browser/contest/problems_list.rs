use std::collections::HashMap;

use color_eyre::{eyre::eyre, Result};


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
        utils::BASEURL,
    },
    display::tui::{
        base_component::Table,
        component::ComponentSender,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewConstructor},
        types::TextSpans,
        utils::{
            is_down_key, is_enter_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key,
        },
        BaseComponent, Component,
    },
    settings::SETTINGS,
};

#[derive(Debug, Default)]
struct UpdateResult {
    problems: Vec<Problem>,
    items: Vec<Vec<TextSpans>>,
}

pub struct ProblemsList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    contest: Contest,
    component: Table,
    updating: u32,
    problems: Vec<Problem>,
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
    let items: Vec<Vec<TextSpans>> = index_to_problem
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
            .collect::<Vec<TextSpans>>()
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
