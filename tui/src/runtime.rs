use std::fmt;
use std::io::{stdout, IsTerminal};
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EventStream};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use mc_communication::{
    ApprovalRequest, ApprovalResponse, BroadcastEvent, ControlMessage, StateMessage,
};
use mc_core::RunEventEnvelope;
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
    update_rx: mpsc::Receiver<TuiUpdate>,
    tick_rate: Duration,
}

/// Sender used by the rest of the system to push updates into the TUI.
#[derive(Debug, Clone)]
pub struct TuiHandle {
    update_tx: mpsc::Sender<TuiUpdate>,
}

impl Tui {
    pub fn new(title: impl Into<String>) -> (Self, TuiHandle) {
        Self::from_app(App::with_title(title))
    }

    pub fn from_app(app: App) -> (Self, TuiHandle) {
        let (update_tx, update_rx) = mpsc::channel(1024);
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

    /// Returns `true` when stdout is attached to an interactive terminal that
    /// can support raw mode, alternate screen, and mouse capture.
    pub fn is_terminal_available() -> bool {
        stdout().is_terminal()
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
        self.app.state_mut().terminal_size = (terminal.size()?.width, terminal.size()?.height);
        set_mouse_capture_enabled(self.app.state().mouse_support())?;

        let result = self.run_loop(&mut terminal).await;
        terminal.show_cursor()?;
        result
    }

    async fn run_loop<B>(&mut self, terminal: &mut Terminal<B>) -> Result<AppExit, TuiError>
    where
        B: Backend,
    {
        let mut events = EventStream::new();
        let mut tick_rate_ms = self.app.state().tick_rate_ms();
        let mut mouse_support = self.app.state().mouse_support();
        self.tick_rate = Duration::from_millis(tick_rate_ms.max(16));
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

            let desired = self.app.state().tick_rate_ms();
            if desired != tick_rate_ms {
                tick_rate_ms = desired;
                self.tick_rate = Duration::from_millis(tick_rate_ms.max(16));
                ticks = time::interval(self.tick_rate);
                ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
            }

            let desired_mouse_support = self.app.state().mouse_support();
            if desired_mouse_support != mouse_support {
                mouse_support = desired_mouse_support;
                set_mouse_capture_enabled(mouse_support)?;
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
            .try_send(update)
            .map_err(|error| match error {
                tokio::sync::mpsc::error::TrySendError::Closed(_) => TuiError::UpdateChannelClosed,
                tokio::sync::mpsc::error::TrySendError::Full(_) => TuiError::UpdateChannelClosed,
            })
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

    pub fn run_event(&self, event: RunEventEnvelope) -> Result<(), TuiError> {
        self.send(TuiUpdate::RunEvent(event))
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
        let _ = execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen);
    }
}

fn set_mouse_capture_enabled(enabled: bool) -> Result<(), TuiError> {
    if enabled {
        execute!(stdout(), EnableMouseCapture)?;
    } else {
        execute!(stdout(), DisableMouseCapture)?;
    }
    Ok(())
}
