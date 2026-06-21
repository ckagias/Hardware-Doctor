use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};

// One mouse input (button or scroll direction), identified by a stable code
#[derive(Clone)]
pub struct ButtonDef {
    pub code: &'static str,
    pub label: &'static str,
}

const fn button(code: &'static str, label: &'static str) -> ButtonDef {
    ButtonDef { code, label }
}

// All testable mouse inputs
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

#[derive(Clone)]
pub struct MouseLayout {
    pub buttons: Vec<ButtonDef>,
}

fn all_buttons() -> HashSet<&'static str> {
    mouse_buttons().into_iter().map(|b| b.code).collect()
}

#[derive(Clone)]
pub struct MouseSnapshot {
    pub pressed: Vec<String>,
    pub tested: Vec<String>,
    pub tested_count: usize,
    pub total_count: usize,
    pub counts: HashMap<String, u64>,
}

pub struct MouseState {
    pressed: Mutex<HashSet<String>>,
    tested: Mutex<HashSet<String>>,
    counts: Mutex<HashMap<String, u64>>,
    all_buttons: HashSet<&'static str>,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            pressed: Mutex::new(HashSet::new()),
            tested: Mutex::new(HashSet::new()),
            counts: Mutex::new(HashMap::new()),
            all_buttons: all_buttons(),
        }
    }
}

pub fn mouse_layout() -> MouseLayout {
    MouseLayout {
        buttons: mouse_buttons(),
    }
}

pub fn mouse_snapshot(state: &MouseState) -> MouseSnapshot {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    let counts = state.counts.lock();
    MouseSnapshot {
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_buttons.len(),
        counts: counts.clone(),
    }
}

pub fn button_down(state: &MouseState, code: String) {
    if !state.all_buttons.contains(code.as_str()) {
        return;
    }
    state.pressed.lock().insert(code.clone());
    state.tested.lock().insert(code.clone());
    *state.counts.lock().entry(code).or_insert(0) += 1;
}

pub fn button_up(state: &MouseState, code: String) {
    if !state.all_buttons.contains(code.as_str()) {
        return;
    }
    state.pressed.lock().remove(&code);
}

// Scroll has no native "up" event, so this flashes the direction as pressed; the UI clears it via button_up shortly after.
pub fn scroll(state: &MouseState, direction: &str) {
    let code = match direction {
        "up" => "ScrollUp",
        "down" => "ScrollDown",
        _ => return,
    };
    state.pressed.lock().insert(code.to_string());
    state.tested.lock().insert(code.to_string());
    *state.counts.lock().entry(code.to_string()).or_insert(0) += 1;
}

pub fn mouse_reset_tested(state: &MouseState) {
    state.tested.lock().clear();
    state.counts.lock().clear();
}

pub fn mouse_clear_pressed(state: &MouseState) {
    state.pressed.lock().clear();
}
