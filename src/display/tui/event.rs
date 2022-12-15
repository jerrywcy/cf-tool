use color_eyre::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

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
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                loop {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    if event::poll(timeout).expect("No event available") {
                        match event::read().expect("Unable to read event") {
                            CrosstermEvent::FocusGained => sender.send(AppEvent::FocusGained),
                            CrosstermEvent::FocusLost => sender.send(AppEvent::FocusLost),
                            CrosstermEvent::Key(evt) => sender.send(AppEvent::Key(evt)),
                            CrosstermEvent::Mouse(evt) => sender.send(AppEvent::Mouse(evt)),
                            CrosstermEvent::Paste(string) => sender.send(AppEvent::Paste(string)),
                            CrosstermEvent::Resize(width, height) => {
                                sender.send(AppEvent::WindowResize(width, height))
                            }
                        }
                        .expect("Failed to send terminal event")
                    }

                    if last_tick.elapsed() >= tick_rate {
                        sender
                            .send(AppEvent::Tick)
                            .expect("failed to send tick event");
                        last_tick = Instant::now();
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

    pub fn next(&self) -> Result<AppEvent> {
        Ok(self.receiver.recv()?)
    }
}
