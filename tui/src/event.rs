#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyAction {
    NextPanel,
    PreviousPanel,
    ScrollUp,
    ScrollDown,
    Help,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    Tick,
    Resize { width: u16, height: u16 },
    Key(KeyAction),
}
