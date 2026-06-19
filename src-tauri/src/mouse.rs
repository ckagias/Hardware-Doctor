use parking_lot::Mutex;
use serde::Serialize;
use std::collections::HashSet;
use tauri::{AppHandle, Emitter, State};

// One mouse input (button or scroll direction), identified by a stable code
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ButtonDef {
    pub code: &'static str,
    pub label: &'static str,
}

const fn button(code: &'static str, label: &'static str) -> ButtonDef {
    ButtonDef { code, label }
}

// All testable mouse inputs, codes match what the frontend sends
fn mouse_buttons() -> Vec<ButtonDef> {
    vec![
        button("Left", "Left Click"),
        button("Right", "Right Click"),
        button("Middle", "Wheel Click"),
        button("Back", "Back"),
        button("Forward", "Forward"),
        button("ScrollUp", "Scroll Up"),
        button("ScrollDown", "Scroll Down"),
    ]
}

#[derive(Serialize, Clone)]
pub struct MouseLayout {
    pub buttons: Vec<ButtonDef>,
}

fn all_buttons() -> HashSet<&'static str> {
    mouse_buttons().into_iter().map(|b| b.code).collect()
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MouseSnapshot {
    pressed: Vec<String>,
    tested: Vec<String>,
    tested_count: usize,
    total_count: usize,
}

pub struct MouseState {
    pressed: Mutex<HashSet<String>>,
    tested: Mutex<HashSet<String>>,
    all_buttons: HashSet<&'static str>,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            pressed: Mutex::new(HashSet::new()),
            tested: Mutex::new(HashSet::new()),
            all_buttons: all_buttons(),
        }
    }
}

fn emit_snapshot(app: &AppHandle, state: &State<MouseState>) -> Result<(), String> {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    let snapshot = MouseSnapshot {
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_buttons.len(),
    };
    app.emit("mouse-state", snapshot).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn mouse_layout() -> MouseLayout {
    MouseLayout {
        buttons: mouse_buttons(),
    }
}

#[tauri::command]
pub fn mouse_snapshot(state: State<MouseState>) -> MouseSnapshot {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    MouseSnapshot {
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_buttons.len(),
    }
}

#[tauri::command]
pub fn button_down(app: AppHandle, state: State<MouseState>, code: String) -> Result<(), String> {
    if !state.all_buttons.contains(code.as_str()) {
        return Ok(());
    }
    state.pressed.lock().insert(code.clone());
    state.tested.lock().insert(code);
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn button_up(app: AppHandle, state: State<MouseState>, code: String) -> Result<(), String> {
    if !state.all_buttons.contains(code.as_str()) {
        return Ok(());
    }
    state.pressed.lock().remove(&code);
    emit_snapshot(&app, &state)
}

// Scroll events have no native "up" counterpart, so this just flashes the direction as
// pressed and marks it tested; the frontend clears the flash via button_up shortly after
#[tauri::command]
pub fn scroll(app: AppHandle, state: State<MouseState>, direction: String) -> Result<(), String> {
    let code = match direction.as_str() {
        "up" => "ScrollUp",
        "down" => "ScrollDown",
        _ => return Ok(()),
    };
    state.pressed.lock().insert(code.to_string());
    state.tested.lock().insert(code.to_string());
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn mouse_reset_tested(app: AppHandle, state: State<MouseState>) -> Result<(), String> {
    state.tested.lock().clear();
    emit_snapshot(&app, &state)
}

#[tauri::command]
pub fn mouse_clear_pressed(app: AppHandle, state: State<MouseState>) -> Result<(), String> {
    state.pressed.lock().clear();
    emit_snapshot(&app, &state)
}
