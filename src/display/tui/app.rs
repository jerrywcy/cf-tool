use std::io::{self, Stdout};

use color_eyre::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{self, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tuirealm::tui::{backend::CrosstermBackend, Terminal};

use super::{
    event::{AppEvent, EventListener},
    msg::{ChannelHandler, ViewConstructor, ViewMsg},
    utils::is_terminate_key,
    view::View,
};

pub struct App {
    running: bool,
    views: Vec<Box<dyn View>>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    events: EventListener,
    msgs: ChannelHandler<ViewMsg>,
}

impl Drop for App {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}

impl App {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let events = EventListener::new(250);
        let msgs = ChannelHandler::new();
        Ok(Self {
            running: true,
            views: Vec::default(),
            terminal,
            events,
            msgs,
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

    pub fn run(&mut self) -> Result<()> {
        while self.running && !self.views.is_empty() {
            self.terminal.draw(|frame| {
                let mut last_fullscreen_view_index = 0;
                for (i, view) in self.views.iter().enumerate() {
                    if view.is_fullscreen() {
                        last_fullscreen_view_index = i;
                    }
                }
                for (i, view) in self.views.iter_mut().enumerate() {
                    if i >= last_fullscreen_view_index {
                        view.render(frame);
                    }
                }
            })?;
            let view = match self.views.last_mut() {
                Some(view) => view,
                None => {
                    self.close();
                    break;
                }
            };

            let event = self.events.next()?;
            match &event {
                AppEvent::Key(evt) if is_terminate_key(evt) => {
                    self.close();
                    break;
                }
                event => {
                    if let Err(err) = view.handle_event(event) {
                        self.enter_new_view(ViewConstructor::ErrorPopup(
                            String::from("Error from View"),
                            format!("{err:#}"),
                        ))
                    }
                }
            }

            self.handle_msg();
        }
        Ok(())
    }

    fn handle_msg(&mut self) {
        while let Ok(msg) = self.msgs.try_next() {
            match msg {
                ViewMsg::AppClose => {
                    self.close();
                    break;
                }
                ViewMsg::EnterNewView(constructor) => self.enter_new_view(constructor),
                ViewMsg::ExitCurrentView => self.exit_current_view(),
                ViewMsg::None => (),
            }
        }
    }

    pub fn enter_new_view(&mut self, constructor: ViewConstructor) {
        let view = constructor.construct(&self.msgs.sender);
        self.views.push(view);
    }

    pub fn exit_current_view(&mut self) {
        self.views.pop();
    }
}
