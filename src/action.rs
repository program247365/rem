use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,
    // Navigation actions
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    // EventKit actions
    CheckPermissions,
    RequestPermissions,
    LoadLists,
    LoadReminders(String),
    SelectList(String),
}
