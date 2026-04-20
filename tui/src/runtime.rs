use std::fmt;
use std::io::stdout;
use std::time::Duration;

use crossterm::event::EventStream;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use mc_communication::{
    ApprovalRequest, ApprovalResponse, BroadcastEvent, ControlMessage, StateMessage,
};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};

use crate::app::App;
use crate::error::TuiError;
use crate::event::{AppEvent, LogLevel, TuiUpdate};

/// Exit reasons for a terminal UI session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppExit {
    QuitRequested,
}

impl fmt::Display for AppExit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuitRequested => f.write_str("quit requested"),
        }
    }
}

/// Async Ratatui runtime that owns the application state and event loop.
#[derive(Debug)]
pub struct Tui {
    app: App,
    update_rx: mpsc::UnboundedReceiver<TuiUpdate>,
    tick_rate: Duration,
}

/// Sender used by the rest of the system to push updates into the TUI.
#[derive(Debug, Clone)]
pub struct TuiHandle {
    update_tx: mpsc::UnboundedSender<TuiUpdate>,
}

impl Tui {
    pub fn new(title: impl Into<String>) -> (Self, TuiHandle) {
        Self::from_app(App::with_title(title))
    }

    pub fn from_app(app: App) -> (Self, TuiHandle) {
        let (update_tx, update_rx) = mpsc::unbounded_channel();
        (
            Self {
                app,
                update_rx,
                tick_rate: Duration::from_millis(250),
            },
            TuiHandle { update_tx },
        )
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn set_tick_rate(&mut self, tick_rate: Duration) {
        self.tick_rate = tick_rate;
    }

    pub fn draw<B>(&self, terminal: &mut Terminal<B>) -> Result<(), TuiError>
    where
        B: Backend,
    {
        terminal.draw(|frame| self.app.draw(frame))?;
        Ok(())
    }

    pub async fn run(mut self) -> Result<AppExit, TuiError> {
        let _guard = TerminalGuard::enter()?;
        let backend = CrosstermBackend::new(stdout());
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        let result = self.run_loop(&mut terminal).await;
        terminal.show_cursor()?;
        result
    }

    async fn run_loop<B>(&mut self, terminal: &mut Terminal<B>) -> Result<AppExit, TuiError>
    where
        B: Backend,
    {
        let mut events = EventStream::new();
        let mut ticks = time::interval(self.tick_rate);
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);

        self.draw(terminal)?;

        loop {
            tokio::select! {
                _ = ticks.tick() => {
                    self.app.handle_event(AppEvent::Tick)?;
                    self.draw(terminal)?;
                }
                maybe_update = self.update_rx.recv() => {
                    if let Some(update) = maybe_update {
                        self.app.handle_event(AppEvent::Update(Box::new(update)))?;
                        self.draw(terminal)?;
                    }
                }
                maybe_event = events.next() => {
                    match maybe_event {
                        Some(Ok(event)) => {
                            if let Some(app_event) = AppEvent::from_crossterm(event) {
                                self.app.handle_event(app_event)?;
                                self.draw(terminal)?;
                            }
                        }
                        Some(Err(error)) => return Err(TuiError::Io(error)),
                        None => break,
                    }
                }
            }

            if self.app.state().should_quit() {
                break;
            }
        }

        Ok(AppExit::QuitRequested)
    }
}

impl TuiHandle {
    pub fn send(&self, update: TuiUpdate) -> Result<(), TuiError> {
        self.update_tx
            .send(update)
            .map_err(|_| TuiError::UpdateChannelClosed)
    }

    pub fn control(&self, message: ControlMessage) -> Result<(), TuiError> {
        self.send(TuiUpdate::Control(message))
    }

    pub fn state(&self, message: StateMessage) -> Result<(), TuiError> {
        self.send(TuiUpdate::State(message))
    }

    pub fn broadcast(&self, event: BroadcastEvent) -> Result<(), TuiError> {
        self.send(TuiUpdate::Broadcast(event))
    }

    pub fn approval_request(&self, request: ApprovalRequest) -> Result<(), TuiError> {
        self.send(TuiUpdate::ApprovalRequest(request))
    }

    pub fn approval_response(&self, response: ApprovalResponse) -> Result<(), TuiError> {
        self.send(TuiUpdate::ApprovalResponse(response))
    }

    pub fn log(&self, level: LogLevel, message: impl Into<String>) -> Result<(), TuiError> {
        self.send(TuiUpdate::Log {
            level,
            message: message.into(),
        })
    }
}

#[derive(Debug)]
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> Result<Self, TuiError> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}
