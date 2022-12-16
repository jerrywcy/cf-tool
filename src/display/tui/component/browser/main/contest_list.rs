use std::time::Duration;

use chrono::prelude::*;
use color_eyre::{eyre::eyre, Result};

use duration_human::DurationHuman;

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
    api::{methods::contest_list, objects::Contest},
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
    contests: Vec<Contest>,
    items: Vec<Vec<String>>,
}

pub struct ContestList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    contests: Vec<Contest>,
    component: Table,
    updating: u32,
}

impl Component for ContestList {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
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
                let contest = self
                    .contests
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}.\nCommonly this is a problem of the application."
                    ))?
                    .clone();
                self.send(ComponentMsg::EnterNewView(ViewConstructor::ContestBrowser(
                    contest,
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
    pub static ref DEFAULT_HEADER: Vec<String> = vec![
        String::from("Name"),
        String::from("Start"),
        String::from("Length"),
    ];
    pub static ref DEFAULT_WIDTHS: Vec<Constraint> = vec![
        Constraint::Percentage(60),
        Constraint::Min(20),
        Constraint::Percentage(20),
    ];
}

async fn get_result() -> Result<Vec<Contest>> {
    let mut contests = contest_list(None).await?;
    contests.sort_by_key(|contest| contest.id);
    contests.reverse();
    Ok(contests)
}

fn format_item(contest: Contest) -> Vec<String> {
    let name = contest.name;
    let start_time = match contest.startTimeSeconds {
        Some(start_time_seconds) => {
            let naive = NaiveDateTime::from_timestamp_opt(start_time_seconds, 0)
                .unwrap_or(NaiveDateTime::default());
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            datetime.format("%Y-%m-%d %H:%M:%S").to_string()
        }
        None => String::from(""),
    };
    let length = format!(
        "{:#}",
        DurationHuman::from(Duration::from_secs(contest.durationSeconds))
    );
    vec![name, start_time, length]
}

async fn update(sender: mpsc::Sender<UpdateResult>) -> Result<()> {
    let results = get_result().await?;
    let contests = results.clone();
    let items: Vec<Vec<String>> = results
        .into_iter()
        .map(|contest| format_item(contest))
        .collect();
    sender.send(UpdateResult { contests, items }).unwrap();
    Ok(())
}

impl ContestList {
    pub fn new(sender: ComponentSender) -> Self {
        let table = Table::new(DEFAULT_HEADER.clone(), DEFAULT_WIDTHS.clone(), "");
        let handler = ChannelHandler::new();
        Self {
            sender,
            handler,
            contests: vec![],
            component: table,
            updating: 0,
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult { items, contests } = result;
            self.component.set_items(items);
            self.contests = contests;
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
                error_sender.send(UpdateResult::default()).unwrap();
                popup_sender
                    .send(ComponentMsg::EnterNewView(ViewConstructor::ErrorPopup(
                        String::from("Error from Contest"),
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
