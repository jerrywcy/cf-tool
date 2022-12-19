#![allow(unused_must_use)]
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, MouseEvent};
use std::{
    sync::mpsc::{self, RecvError},
    time::Duration,
};
use tokio::{select, task, time::sleep};
use tokio_stream::StreamExt;

#[derive(Debug)]
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
pub struct EventListener {
    sender: mpsc::Sender<AppEvent>,
    receiver: mpsc::Receiver<AppEvent>,
    handler: task::JoinHandle<()>,
}

impl EventListener {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            tokio::spawn(async move {
                let mut reader = EventStream::new();
                loop {
                    let delay = sleep(tick_rate);
                    let event = reader.next();
                    select! {
                        _ = delay => {sender.send(AppEvent::Tick);},
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

                                    };
                                },
                                Some(Err(_err)) => {},
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

    pub fn next(&mut self) -> Result<AppEvent, RecvError> {
        self.receiver.recv()
    }
}
