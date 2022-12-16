use std::{
    io::{self, Stdout},
    sync::{Arc, Mutex},
};

use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use lazy_static::lazy_static;
use tuirealm::tui::{backend::CrosstermBackend, Terminal};

use super::{
    event::EventHandler,
    msg::ViewMsg,
    view::{PopupView, View},
};

lazy_static! {
    pub static ref POPUP: Arc<Mutex<Option<PopupView>>> = Arc::new(Mutex::new(None));
}

pub struct App {
    running: bool,
    views: Vec<Box<dyn View>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    events: EventHandler,
}

impl Drop for App {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

impl App {
    pub async fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(250).await;
        Ok(Self {
            running: true,
            views: Vec::default(),
            terminal,
            events,
        })
    }

    pub fn close(&mut self) {
        self.running = false;
        while !self.views.is_empty() {
            self.views.pop();
        }
    }

    pub fn exit(&mut self) -> Result<()> {
        terminal::disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        while self.running && !self.views.is_empty() {
            let view = match self.views.last_mut() {
                Some(view) => view,
                None => {
                    self.close();
                    break;
                }
            };
            self.terminal.draw(|frame| view.render(frame))?;

            let event = self.events.next().await?;

            match view.handle_event(&event)? {
                ViewMsg::AppClose => {
                    self.close();
                    break;
                }
                ViewMsg::EnterNewView(view) => {
                    self.enter_new_view(view);
                }
                ViewMsg::ExitCurrentView => {
                    self.exit_current_view();
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub fn enter_new_view(&mut self, view: Box<dyn View>) {
        self.views.push(view);
    }

    pub fn exit_current_view(&mut self) {
        let popup = Arc::clone(&POPUP);
        if let Ok(mut result) = popup.try_lock() {
            if result.is_some() {
                *result = None;
                return;
            }
        };
        self.views.pop();
    }
}
