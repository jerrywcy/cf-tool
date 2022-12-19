#![allow(unused_must_use)]
use color_eyre::{eyre::eyre, Result};

use lazy_static::lazy_static;
use std::sync::mpsc;
use tuirealm::{
    props::{Alignment, BorderType},
    tui::{
        layout::{Constraint, Rect},
        widgets::{Block, Borders, Paragraph},
    },
    Frame,
};

use crate::{
    api::{methods::problemset_problems, objects::Problem, utils::BASEURL},
    display::tui::{
        base_component::Table,
        component::ComponentSender,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewConstructor},
        utils::{
            is_down_key, is_enter_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key,
        },
        BaseComponent, Component,
    },
};

#[derive(Debug, Default)]
struct UpdateResult {
    problems: Vec<Problem>,
    items: Vec<Vec<String>>,
}

pub struct ProblemsetList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    problems: Vec<Problem>,
    component: Table,
    updating: u32,
}

impl Component for ProblemsetList {
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
                let problem = self.problems.get(index).ok_or(eyre!(
                    "No such index: {index}\nCommonly this is a problem of the application."
                ))?;
                let index = problem.index.clone();
                let url = match problem.contestId {
                    Some(contest_id) => format!("{BASEURL}problemset/problem/{contest_id}/{index}"),
                    None => format!("{BASEURL}acmsguru/problem/99999/{index}"),
                };
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
    static ref DEFAULT_HEADER: Vec<String> = vec![
        String::from("#"),
        String::from("Name"),
        String::from("Tags"),
    ];
    static ref DEFAULT_WIDTHS: Vec<Constraint> = vec![
        Constraint::Length(6),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ];
}

async fn get_result() -> Result<Vec<Problem>> {
    let problems = problemset_problems(None, None).await?.problems;
    Ok(problems)
}

fn format_item(problem: Problem) -> Vec<String> {
    let index = match problem.contestId {
        Some(contest_id) => format!("{}{}", contest_id, problem.index),
        None => format!("{}", problem.index),
    };
    let name = problem.name;
    let tags = problem.tags.join(",");
    vec![index.to_string(), name, tags]
}

async fn update(sender: mpsc::Sender<UpdateResult>) -> Result<()> {
    let results = get_result().await?;
    let problems = results.clone();
    let items: Vec<Vec<String>> = results
        .into_iter()
        .map(|problem| format_item(problem))
        .collect();
    sender.send(UpdateResult { problems, items });
    Ok(())
}

impl ProblemsetList {
    pub fn new(sender: ComponentSender) -> Self {
        let table = Table::new(DEFAULT_HEADER.clone(), DEFAULT_WIDTHS.clone(), "");
        let handler = ChannelHandler::new();
        Self {
            sender,
            handler,
            problems: vec![],
            component: table,
            updating: 0,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { items, problems } = result;
            self.component.set_items(items);
            self.problems = problems;
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
        self.updating += 1;
        tokio::spawn(async move {
            if let Err(err) = update(update_sender).await {
                error_sender.send(UpdateResult::default());
                popup_sender.send(ComponentMsg::EnterNewView(ViewConstructor::ErrorPopup(
                    String::from("Error from ProblemSet"),
                    format!("{err:#}"),
                )));
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
