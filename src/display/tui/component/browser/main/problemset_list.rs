use std::sync::{Arc, Mutex};

use color_eyre::Result;


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
    api::{methods::problemset_problems, objects::Problem},
    display::tui::{
        app::POPUP,
        base_component::Table,
        event::AppEvent,
        msg::ComponentMsg,
        utils::{is_down_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key},
        view::PopupView,
        BaseComponent, Component,
    },
};

pub struct ProblemsetList {
    problems: Arc<Mutex<Vec<Problem>>>,
    component: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
}

impl Component for ProblemsetList {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        let mut component = match self.component.try_lock() {
            Ok(component) => component,
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
                self.update()?;
                Ok(ComponentMsg::Update)
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
        String::from("Tags"),
    ];
    static ref DEFAULT_WIDTHS: Vec<Constraint> = vec![
        Constraint::Length(6),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ];
}

impl Default for ProblemsetList {
    fn default() -> Self {
        let table = Table::new(DEFAULT_HEADER.clone(), DEFAULT_WIDTHS.clone(), "");
        Self {
            problems: Arc::new(Mutex::new(vec![])),
            component: Arc::new(Mutex::new(table)),
            updating: Arc::new(Mutex::new(false)),
        }
    }
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

async fn update(
    table: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
    problems: Arc<Mutex<Vec<Problem>>>,
) -> Result<()> {
    let results = get_result().await?;
    *problems.lock().unwrap() = results.clone();
    let items: Vec<Vec<String>> = results
        .into_iter()
        .map(|problem| format_item(problem))
        .collect();
    table.lock().unwrap().set_items(items);
    *updating.lock().unwrap() = false;
    Ok(())
}

impl ProblemsetList {
    pub fn new() -> Result<Self> {
        let mut default = ProblemsetList::default();
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
        let problems = Arc::clone(&self.problems);
        let popup = Arc::clone(&POPUP);
        tokio::spawn(async move {
            if let Err(err) = update(table, updating.clone(), problems).await {
                if let Ok(mut popup) = popup.lock() {
                    *popup = Some(PopupView::new(
                        TextSpan::new("Error from ProblemSet").fg(Color::Red),
                        format!("{err:#}"),
                    ))
                }
                if let Ok(mut updating) = updating.try_lock() {
                    *updating = false;
                }
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
