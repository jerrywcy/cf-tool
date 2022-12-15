use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::prelude::*;
use color_eyre::{eyre::eyre, Result};

use duration_human::DurationHuman;

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
    api::{methods::contest_list, objects::Contest},
    display::tui::{
        app::POPUP,
        base_component::Table,
        event::AppEvent,
        msg::ComponentMsg,
        utils::{
            is_down_key, is_enter_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key,
        },
        view::{ContestBrowser, PopupView},
        BaseComponent, Component,
    },
};

pub struct ContestList {
    contests: Arc<Mutex<Vec<Contest>>>,
    component: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
}

impl Component for ContestList {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        let mut component = match self.component.try_lock() {
            Ok(component) => component,
            Err(_err) => return Ok(ComponentMsg::Locked),
        };
        let contests = match self.contests.try_lock() {
            Ok(contest_ids) => contest_ids,
            Err(_) => return Ok(ComponentMsg::Locked),
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
                drop(contests);
                self.update()?;
                Ok(ComponentMsg::Update)
            }
            AppEvent::Key(evt) if is_enter_key(evt) => {
                let index = component.selected();
                let contest = contests
                    .get(index)
                    .ok_or(eyre!(
                        "No such index: {index}.\nCommonly this is a problem of the application."
                    ))?
                    .clone();
                Ok(ComponentMsg::EnterNewView(Box::new(ContestBrowser::new(
                    contest,
                )?)))
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

impl Default for ContestList {
    fn default() -> Self {
        let table = Table::new(DEFAULT_HEADER.clone(), DEFAULT_WIDTHS.clone(), "");
        Self {
            contests: Arc::new(Mutex::new(vec![])),
            component: Arc::new(Mutex::new(table)),
            updating: Arc::new(Mutex::new(false)),
        }
    }
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

async fn update(
    table: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    contests: Arc<Mutex<Vec<Contest>>>,
) -> Result<()> {
    let results = get_result().await?;
    *contests.lock().unwrap() = results.clone();
    let items: Vec<Vec<String>> = results
        .into_iter()
        .map(|contest| format_item(contest))
        .collect();
    table.lock().unwrap().set_items(items);
    *updating.lock().unwrap() = false;
    Ok(())
}

impl ContestList {
    pub fn new() -> Result<Self> {
        let mut default = ContestList::default();
        default.update()?;
        Ok(default)
    }

    pub fn update(&mut self) -> Result<()> {
        match self.updating.try_lock() {
            Ok(mut updating) => *updating = true,
            Err(_) => return Ok(()),
        }
        if let Err(_) = self.contests.try_lock() {
            return Ok(());
        }
        let table = Arc::clone(&self.component);
        let updating = Arc::clone(&self.updating);
        let contests = Arc::clone(&self.contests);
        let popup = Arc::clone(&POPUP);
        tokio::spawn(async move {
            if let Err(err) = update(table, updating.clone(), contests).await {
                if let Ok(mut popup) = popup.lock() {
                    *popup = Some(PopupView::new(
                        TextSpan::new("Error from Contest").fg(Color::Red),
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
