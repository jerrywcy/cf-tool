use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use std::time::{Duration, Instant};
use tokio::{select, sync::mpsc, task, time::sleep};
use tokio_stream::StreamExt;

pub enum AppEvent {
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    WindowResize(u16, u16),
    Tick,
}

#[allow(dead_code)]
pub struct EventHandler {
    sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    handler: task::JoinHandle<()>,
}

impl EventHandler {
    pub async fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel(100);
        let handler = {
            let sender = sender.clone();
            tokio::spawn(async move {
                let mut reader = EventStream::new();
                loop {
                    let mut delay = sleep(tick_rate);
                    let mut event = reader.next();
                    select! {
                        _ = delay => {sender.send(AppEvent::Tick).await;},
                        maybe_event = event => {
                            match maybe_event {
                                Some(Ok(event)) => {
                                    match event {
                                        CrosstermEvent::FocusGained => sender.send(AppEvent::FocusGained),
                                        CrosstermEvent::FocusLost => sender.send(AppEvent::FocusLost),
                                        CrosstermEvent::Key(evt) => sender.send(AppEvent::Key(evt)),
                                        CrosstermEvent::Mouse(evt) => sender.send(AppEvent::Mouse(evt)),
                                        CrosstermEvent::Paste(string) => sender.send(AppEvent::Paste(string)),
                                        CrosstermEvent::Resize(width, height) =>
                                            sender.send(AppEvent::WindowResize(width, height))

                                    }.await;
                                },
                                Some(Err(err)) => {},
                                None => {break;},
                            };
                        }
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub async fn next(&mut self) -> Result<AppEvent> {
        Ok(self.receiver.recv().await.unwrap())
    }
}
