use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct InputDef {
    pub code: &'static str,
    pub label: &'static str,
}

const fn input(code: &'static str, label: &'static str) -> InputDef {
    InputDef { code, label }
}

// Digital inputs: discrete press/release. Sticks/triggers are tested via deadzone movement instead; L3/R3 are digital clicks here.
fn digital_inputs() -> Vec<InputDef> {
    vec![
        input("FaceNorth", "Face Up"),
        input("FaceEast", "Face Right"),
        input("FaceSouth", "Face Down"),
        input("FaceWest", "Face Left"),
        input("DpadUp", "D-Pad Up"),
        input("DpadDown", "D-Pad Down"),
        input("DpadLeft", "D-Pad Left"),
        input("DpadRight", "D-Pad Right"),
        input("L1", "Left Bumper (L1)"),
        input("R1", "Right Bumper (R1)"),
        input("L3", "Left Stick Click (L3)"),
        input("R3", "Right Stick Click (R3)"),
        input("Start", "Start"),
        input("Select", "Select"),
        input("Guide", "Home"),
    ]
}

// Analog axes (sticks). Each carries an x and y value in the snapshot.
pub const STICK_LEFT: &str = "LeftStick";
pub const STICK_RIGHT: &str = "RightStick";

// Analog triggers (L2/R2): a single 0.0..=1.0 pull amount.
pub const TRIGGER_L2: &str = "L2";
pub const TRIGGER_R2: &str = "R2";

// Movement past this magnitude counts a stick/trigger as "tested".
pub const STICK_DEADZONE: f32 = 0.5;
pub const TRIGGER_DEADZONE: f32 = 0.25;

#[derive(Clone)]
pub struct ControllerLayout {
    pub digital: Vec<InputDef>,
    pub sticks: Vec<InputDef>,
    pub triggers: Vec<InputDef>,
}

pub fn controller_layout() -> ControllerLayout {
    ControllerLayout {
        digital: digital_inputs(),
        sticks: vec![
            input(STICK_LEFT, "Left Stick"),
            input(STICK_RIGHT, "Right Stick"),
        ],
        triggers: vec![
            input(TRIGGER_L2, "Left Trigger (L2)"),
            input(TRIGGER_R2, "Right Trigger (R2)"),
        ],
    }
}

fn all_codes(layout: &ControllerLayout) -> HashSet<&'static str> {
    let mut all: HashSet<&'static str> = layout.digital.iter().map(|d| d.code).collect();
    all.extend(layout.sticks.iter().map(|d| d.code));
    all.extend(layout.triggers.iter().map(|d| d.code));
    all
}

#[derive(Clone)]
pub struct ControllerSnapshot {
    pub connected: bool,
    pub pressed: Vec<String>,
    pub tested: Vec<String>,
    pub tested_count: usize,
    pub total_count: usize,
    pub counts: HashMap<String, u64>,
    pub left_stick: (f32, f32),
    pub right_stick: (f32, f32),
    pub l2: f32,
    pub r2: f32,
}

pub struct ControllerState {
    connected: Mutex<bool>,
    pressed: Mutex<HashSet<String>>,
    tested: Mutex<HashSet<String>>,
    counts: Mutex<HashMap<String, u64>>,
    left_stick: Mutex<(f32, f32)>,
    right_stick: Mutex<(f32, f32)>,
    l2: Mutex<f32>,
    r2: Mutex<f32>,
    all_codes: HashSet<&'static str>,
    // Tracks which analog inputs are currently past the deadzone, so each push counts once instead of once per frame held.
    stick_active: Mutex<HashSet<String>>,
}

impl Default for ControllerState {
    fn default() -> Self {
        let layout = controller_layout();
        Self {
            connected: Mutex::new(false),
            pressed: Mutex::new(HashSet::new()),
            tested: Mutex::new(HashSet::new()),
            counts: Mutex::new(HashMap::new()),
            left_stick: Mutex::new((0.0, 0.0)),
            right_stick: Mutex::new((0.0, 0.0)),
            l2: Mutex::new(0.0),
            r2: Mutex::new(0.0),
            all_codes: all_codes(&layout),
            stick_active: Mutex::new(HashSet::new()),
        }
    }
}

pub fn controller_snapshot(state: &ControllerState) -> ControllerSnapshot {
    let pressed = state.pressed.lock();
    let tested = state.tested.lock();
    let counts = state.counts.lock();
    ControllerSnapshot {
        connected: *state.connected.lock(),
        pressed: pressed.iter().cloned().collect(),
        tested: tested.iter().cloned().collect(),
        tested_count: tested.len(),
        total_count: state.all_codes.len(),
        counts: counts.clone(),
        left_stick: *state.left_stick.lock(),
        right_stick: *state.right_stick.lock(),
        l2: *state.l2.lock(),
        r2: *state.r2.lock(),
    }
}

pub fn set_connected(state: &ControllerState, connected: bool) {
    *state.connected.lock() = connected;
}

pub fn button_down(state: &ControllerState, code: String) {
    if !state.all_codes.contains(code.as_str()) {
        return;
    }
    state.pressed.lock().insert(code.clone());
    state.tested.lock().insert(code.clone());
    *state.counts.lock().entry(code).or_insert(0) += 1;
}

pub fn button_up(state: &ControllerState, code: String) {
    if !state.all_codes.contains(code.as_str()) {
        return;
    }
    state.pressed.lock().remove(&code);
}

pub fn set_stick(state: &ControllerState, code: &str, x: f32, y: f32) {
    if !state.all_codes.contains(code) {
        return;
    }
    match code {
        STICK_LEFT => *state.left_stick.lock() = (x, y),
        STICK_RIGHT => *state.right_stick.lock() = (x, y),
        _ => return,
    }

    let mag = (x * x + y * y).sqrt();
    let mut active = state.stick_active.lock();
    if mag >= STICK_DEADZONE {
        if active.insert(code.to_string()) {
            state.tested.lock().insert(code.to_string());
            *state.counts.lock().entry(code.to_string()).or_insert(0) += 1;
        }
    } else if mag < STICK_DEADZONE * 0.6 {
        active.remove(code);
    }
}

pub fn set_trigger(state: &ControllerState, code: &str, value: f32) {
    if !state.all_codes.contains(code) {
        return;
    }
    match code {
        TRIGGER_L2 => *state.l2.lock() = value,
        TRIGGER_R2 => *state.r2.lock() = value,
        _ => return,
    }

    let mut active = state.stick_active.lock();
    if value >= TRIGGER_DEADZONE {
        if active.insert(code.to_string()) {
            state.tested.lock().insert(code.to_string());
            *state.counts.lock().entry(code.to_string()).or_insert(0) += 1;
        }
    } else if value < TRIGGER_DEADZONE * 0.6 {
        active.remove(code);
    }
}

pub fn reset_tested(state: &ControllerState) {
    state.tested.lock().clear();
    state.counts.lock().clear();
    state.stick_active.lock().clear();
}

pub fn clear_pressed(state: &ControllerState) {
    state.pressed.lock().clear();
    *state.left_stick.lock() = (0.0, 0.0);
    *state.right_stick.lock() = (0.0, 0.0);
    *state.l2.lock() = 0.0;
    *state.r2.lock() = 0.0;
}
