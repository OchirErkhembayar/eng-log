use anyhow::Result;
use chrono::TimeZone;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event as CrossTermEvent, KeyEvent, KeyEventKind,
    },
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::FutureExt;
use futures::StreamExt;
use ratatui::backend::CrosstermBackend;
use std::io::{self, Stdout};
use std::time::Duration;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

use crate::app::App;

type CrosstermTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Saving(bool),
}

pub struct Tui {
    terminal: CrosstermTerminal,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_tx: UnboundedSender<Event>,
    cancellation_token: CancellationToken,
    task: JoinHandle<()>,
}

impl Tui {
    pub fn new(terminal: CrosstermTerminal) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            terminal,
            cancellation_token: CancellationToken::new(),
            event_rx,
            event_tx,
            task: tokio::spawn(async {}),
        }
    }

    pub fn start(&mut self) {
        let tick_delay = std::time::Duration::from_secs_f64(1.0 / 4.0);
        self.cancel();
        self.cancellation_token = CancellationToken::new();
        let _cancellation_token = self.cancellation_token.clone();
        let _event_tx = self.event_tx.clone();
        self.task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_delay);
            loop {
                let tick_delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = _cancellation_token.cancelled() => {
                        break;
                    }
                    maybe_event = crossterm_event => {
                        if let Some(Ok(CrossTermEvent::Key(key))) = maybe_event {
                            if key.kind == KeyEventKind::Press {
                                _event_tx.send(Event::Key(key)).unwrap();
                            }
                        }
                    },
                    _ = tick_delay => {
                        _event_tx.send(Event::Tick).unwrap();
                    },
                }
            }
        });
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        self.start();
        Ok(())
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }

    pub fn stop(&self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                break;
            }
        }
        Ok(())
    }

    pub fn cancel(&self) {
        self.cancellation_token.cancel();
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        self.stop()?;
        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Tui {
    pub fn draw<T: TimeZone>(&mut self, app: &mut App<T>) -> Result<()> {
        self.terminal.draw(|f| crate::ui::ui(f, app))?;
        Ok(())
    }
}
