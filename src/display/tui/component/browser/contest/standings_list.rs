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
    api::{
        methods::contest_standings,
        objects::{Contest, ProblemResult},
    },
    display::tui::{
        app::POPUP,
        base_component::Table,
        event::AppEvent,
        msg::ComponentMsg,
        types::TextSpans,
        utils::{is_down_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key},
        view::PopupView,
        BaseComponent, Component,
    },
};

pub struct StandingsList {
    contest: Contest,
    component: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
}

impl Component for StandingsList {
    fn on(&mut self, event: &AppEvent) -> Result<ComponentMsg> {
        let mut component = match self.component.try_lock() {
            Ok(component) => component,
            Err(_err) => return Ok(ComponentMsg::Locked),
        };
        match *event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                component.next();
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
    static ref DEFAULT_HEADER: Vec<String> = vec![String::from("#"), String::from("Name")];
    static ref DEFAULT_WIDTHS: Vec<Constraint> =
        vec![Constraint::Length(5), Constraint::Percentage(50),];
}

async fn update(
    contest_id: i32,
    table: Arc<Mutex<Table>>,
    updating: Arc<Mutex<bool>>,
) -> Result<()> {
    let standings = contest_standings(contest_id, None, None, None, None, Some(true)).await?;

    let mut indexes: Vec<String> = standings
        .problems
        .into_iter()
        .map(|problem| problem.index.to_string())
        .collect();
    let mut header = DEFAULT_HEADER.clone();
    let mut widths = DEFAULT_WIDTHS.clone();
    for _ in 0..indexes.iter().count() {
        widths.push(Constraint::Min(4));
    }
    header.append(&mut indexes);

    let rows = standings.rows;
    let items: Vec<Vec<TextSpans>> = rows
        .into_iter()
        .map(|row| {
            let rank = row.rank;
            let handles = row
                .party
                .members
                .into_iter()
                .map(|member| member.handle)
                .collect::<Vec<String>>()
                .join(",");
            let mut ret = vec![
                TextSpan::new(rank.to_string()).fg(Color::White),
                TextSpan::new(handles).fg(Color::White),
            ];

            let results = row.problemResults;
            for ProblemResult { points, .. } in results {
                ret.push(TextSpan::new(format!("{points:.0}")).fg(Color::White));
            }

            ret.into_iter().map(|text| text.into()).collect()
        })
        .collect();
    table
        .lock()
        .unwrap()
        .set_items(items)
        .set_header(header)
        .set_widths(widths);
    *updating.lock().unwrap() = false;
    Ok(())
}

impl StandingsList {
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
        let popup = Arc::clone(&POPUP);
        tokio::spawn(async move {
            if let Err(err) = update(contest_id, table, updating.clone()).await {
                if let Ok(mut popup) = popup.lock() {
                    *popup = Some(PopupView::new(
                        TextSpan::new("Error from Standings").fg(Color::Red),
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
