use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashSet;
use tauri::{AppHandle, Emitter, State};

// Position of a key on the keyboard grid (column, row, and optional span)
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyDef {
    pub code: &'static str,
    pub label: &'static str,
    pub col: f32,
    pub row: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub col_span: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_span: Option<u8>,
}

const fn key(code: &'static str, label: &'static str, col: f32, row: u8) -> KeyDef {
    KeyDef { code, label, col, row, col_span: None, row_span: None }
}

const fn key_w(code: &'static str, label: &'static str, col: f32, row: u8, col_span: f32) -> KeyDef {
    KeyDef { code, label, col, row, col_span: Some(col_span), row_span: None }
}

const fn key_full(
    code: &'static str,
    label: &'static str,
    col: f32,
    row: u8,
    col_span: f32,
    row_span: u8,
) -> KeyDef {
    KeyDef {
        code,
        label,
        col,
        row,
        col_span: Some(col_span),
        row_span: Some(row_span),
    }
}

// Main 100% keyboard layout (function row, letters, nav cluster), positions match KeyboardEvent.code
fn keyboard_keys() -> Vec<KeyDef> {
    vec![
        // function row
        key("Escape", "Esc", 1.0, 0),
        key("F1", "F1", 3.0, 0),
        key("F2", "F2", 4.0, 0),
        key("F3", "F3", 5.0, 0),
        key("F4", "F4", 6.0, 0),
        key("F5", "F5", 7.5, 0),
        key("F6", "F6", 8.5, 0),
        key("F7", "F7", 9.5, 0),
        key("F8", "F8", 10.5, 0),
        key("F9", "F9", 12.0, 0),
        key("F10", "F10", 13.0, 0),
        key("F11", "F11", 14.0, 0),
        key("F12", "F12", 15.0, 0),
        key("PrintScreen", "PrtSc", 16.5, 0),
        key("ScrollLock", "ScrLk", 17.5, 0),
        key("Pause", "Pause", 18.5, 0),
        // number row
        key("Backquote", "`", 1.0, 1),
        key("Digit1", "1", 2.0, 1),
        key("Digit2", "2", 3.0, 1),
        key("Digit3", "3", 4.0, 1),
        key("Digit4", "4", 5.0, 1),
        key("Digit5", "5", 6.0, 1),
        key("Digit6", "6", 7.0, 1),
        key("Digit7", "7", 8.0, 1),
        key("Digit8", "8", 9.0, 1),
        key("Digit9", "9", 10.0, 1),
        key("Digit0", "0", 11.0, 1),
        key("Minus", "-", 12.0, 1),
        key("Equal", "=", 13.0, 1),
        key_w("Backspace", "Backspace", 14.0, 1, 2.0),
        key("Insert", "Ins", 16.5, 1),
        key("Home", "Home", 17.5, 1),
        key("PageUp", "PgUp", 18.5, 1),
        // tab row
        key_w("Tab", "Tab", 1.0, 2, 1.5),
        key("KeyQ", "Q", 2.5, 2),
        key("KeyW", "W", 3.5, 2),
        key("KeyE", "E", 4.5, 2),
        key("KeyR", "R", 5.5, 2),
        key("KeyT", "T", 6.5, 2),
        key("KeyY", "Y", 7.5, 2),
        key("KeyU", "U", 8.5, 2),
        key("KeyI", "I", 9.5, 2),
        key("KeyO", "O", 10.5, 2),
        key("KeyP", "P", 11.5, 2),
        key("BracketLeft", "[", 12.5, 2),
        key("BracketRight", "]", 13.5, 2),
        key_w("Backslash", "\\", 14.5, 2, 1.5),
        key("Delete", "Del", 16.5, 2),
        key("End", "End", 17.5, 2),
        key("PageDown", "PgDn", 18.5, 2),
        // caps row
        key_w("CapsLock", "Caps", 1.0, 3, 1.75),
        key("KeyA", "A", 2.75, 3),
        key("KeyS", "S", 3.75, 3),
        key("KeyD", "D", 4.75, 3),
        key("KeyF", "F", 5.75, 3),
        key("KeyG", "G", 6.75, 3),
        key("KeyH", "H", 7.75, 3),
        key("KeyJ", "J", 8.75, 3),
        key("KeyK", "K", 9.75, 3),
        key("KeyL", "L", 10.75, 3),
        key("Semicolon", ";", 11.75, 3),
        key("Quote", "'", 12.75, 3),
        key_w("Enter", "Enter", 13.75, 3, 2.25),
        // shift row
        key_w("ShiftLeft", "Shift", 1.0, 4, 2.25),
        key("KeyZ", "Z", 3.25, 4),
        key("KeyX", "X", 4.25, 4),
        key("KeyC", "C", 5.25, 4),
        key("KeyV", "V", 6.25, 4),
        key("KeyB", "B", 7.25, 4),
        key("KeyN", "N", 8.25, 4),
        key("KeyM", "M", 9.25, 4),
        key("Comma", ",", 10.25, 4),
        key("Period", ".", 11.25, 4),
        key("Slash", "/", 12.25, 4),
        key_w("ShiftRight", "Shift", 13.25, 4, 2.75),
        key("ArrowUp", "\u{2191}", 17.5, 4),
        // bottom row
        key_w("ControlLeft", "Ctrl", 1.0, 5, 1.5),
        key_w("MetaLeft", "Win", 2.5, 5, 1.25),
        key_w("AltLeft", "Alt", 3.75, 5, 1.25),
        key_w("Space", "", 5.0, 5, 6.25),
        key_w("AltRight", "Alt", 11.25, 5, 1.25),
        key_w("MetaRight", "Win", 12.5, 5, 1.25),
        key_w("ContextMenu", "Menu", 13.75, 5, 1.25),
        key_w("ControlRight", "Ctrl", 15.0, 5, 1.5),
        key("ArrowLeft", "\u{2190}", 16.5, 5),
        key("ArrowDown", "\u{2193}", 17.5, 5),
        key("ArrowRight", "\u{2192}", 18.5, 5),
    ]
}

// Numpad keys, positioned on the same grid as keyboard_keys() so they render in one frame
fn numpad_keys() -> Vec<KeyDef> {
    vec![
        key("NumLock", "Num", 20.0, 1),
        key("NumpadDivide", "/", 21.0, 1),
        key("NumpadMultiply", "*", 22.0, 1),
        key("NumpadSubtract", "-", 23.0, 1),
        key("Numpad7", "7", 20.0, 2),
        key("Numpad8", "8", 21.0, 2),
        key("Numpad9", "9", 22.0, 2),
        key_full("NumpadAdd", "+", 23.0, 2, 1.0, 2),
        key("Numpad4", "4", 20.0, 3),
        key("Numpad5", "5", 21.0, 3),
        key("Numpad6", "6", 22.0, 3),
        key("Numpad1", "1", 20.0, 4),
        key("Numpad2", "2", 21.0, 4),
        key("Numpad3", "3", 22.0, 4),
        key_full("NumpadEnter", "Enter", 23.0, 4, 1.0, 2),
        key_w("Numpad0", "0", 20.0, 5, 2.0),
        key("NumpadDecimal", ".", 22.0, 5),
    ]
}

#[derive(Serialize, Clone)]
pub struct KeyboardLayout {
    pub main: Vec<KeyDef>,
    pub numpad: Vec<KeyDef>,
}

fn all_keys() -> HashSet<&'static str> {
    keyboard_keys()
        .into_iter()
        .map(|k| k.code)
        .chain(numpad_keys().into_iter().map(|k| k.code))
        .collect()
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardSnapshot {
    pressed: Vec<String>,
    tested: Vec<String>,
    tested_count: usize,
    total_count: usize,
}

pub struct KeyboardState {
    pressed: Mutex<HashSet<String>>,
    tested: Mutex<HashSet<String>>,
    all_keys: HashSet<&'static str>,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            pressed: Mutex::new(HashSet::new()),
            tested: Mutex::new(HashSet::new()),
            all_keys: all_keys(),
        }
    }
}

fn emit_snapshot(app: &AppHandle, state: &State<KeyboardState>) -> Result<(), String> {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    let snapshot = KeyboardSnapshot {
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_keys.len(),
    };
    app.emit("keyboard-state", snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn keyboard_layout() -> KeyboardLayout {
    KeyboardLayout {
        main: keyboard_keys(),
        numpad: numpad_keys(),
    }
}

#[tauri::command]
pub fn keyboard_snapshot(state: State<KeyboardState>) -> KeyboardSnapshot {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    KeyboardSnapshot {
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_keys.len(),
    }
}

#[tauri::command]
pub fn key_down(app: AppHandle, state: State<KeyboardState>, code: String) -> Result<(), String> {
    if !state.all_keys.contains(code.as_str()) {
        return Ok(());
    }
    state.pressed.lock().insert(code.clone());
    state.tested.lock().insert(code);
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn key_up(app: AppHandle, state: State<KeyboardState>, code: String) -> Result<(), String> {
    if !state.all_keys.contains(code.as_str()) {
        return Ok(());
    }
    state.pressed.lock().remove(&code);
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn reset_tested(app: AppHandle, state: State<KeyboardState>) -> Result<(), String> {
    state.tested.lock().clear();
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn clear_pressed(app: AppHandle, state: State<KeyboardState>) -> Result<(), String> {
    state.pressed.lock().clear();
    emit_snapshot(&app, &state)
}
