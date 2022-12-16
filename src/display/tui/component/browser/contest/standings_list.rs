use color_eyre::Result;

use lazy_static::lazy_static;
use std::sync::mpsc;
use tuirealm::{
    props::{Alignment, BorderType, TextSpan},
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
        base_component::Table,
        component::ComponentSender,
        event::AppEvent,
        msg::{ChannelHandler, ComponentMsg, ViewConstructor},
        types::TextSpans,
        utils::{is_down_key, is_refresh_key, is_scroll_down, is_scroll_up, is_up_key},
        BaseComponent, Component,
    },
};

#[derive(Debug, Default)]
struct UpdateResult {
    items: Vec<Vec<TextSpans>>,
    header: Vec<String>,
    widths: Vec<Constraint>,
}

pub struct StandingsList {
    sender: ComponentSender,
    handler: ChannelHandler<UpdateResult>,
    contest: Contest,
    component: Table,
    updating: u32,
}

impl Component for StandingsList {
    fn on(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
            AppEvent::Key(evt) if is_up_key(evt) => {
                self.component.next();
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
    static ref DEFAULT_HEADER: Vec<String> = vec![String::from("#"), String::from("Name")];
    static ref DEFAULT_WIDTHS: Vec<Constraint> =
        vec![Constraint::Length(5), Constraint::Percentage(50),];
}

async fn update(sender: mpsc::Sender<UpdateResult>, contest_id: i32) -> Result<()> {
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
            let mut ret = vec![TextSpan::new(rank.to_string()), TextSpan::new(handles)];

            let results = row.problemResults;
            for ProblemResult { points, .. } in results {
                ret.push(TextSpan::new(format!("{points:.0}")));
            }

            ret.into_iter().map(|text| text.into()).collect()
        })
        .collect();
    sender
        .send(UpdateResult {
            items,
            header,
            widths,
        })
        .unwrap();
    Ok(())
}

impl StandingsList {
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
        }
    }

    pub fn tick(&mut self) {
        while let Ok(result) = self.handler.try_next() {
            let UpdateResult {
                items,
                header,
                widths,
            } = result;
            self.component
                .set_items(items)
                .set_header(header)
                .set_widths(widths);
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
                        String::from("Error from Standings"),
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
