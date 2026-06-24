use crate::keyboard::{self, KeyDef, KeyboardLayout, KeyboardState};
use rdev::Key as RdKey;
use std::sync::mpsc::{Receiver, Sender, channel};

// Maps rdev physical keys to W3C code strings; rdev can't distinguish AltLeft/AltRight and omits ContextMenu and numpad-distinct keys.
fn rdev_key_to_code(key: RdKey) -> Option<&'static str> {
    Some(match key {
        RdKey::Escape => "Escape",
        RdKey::F1 => "F1",
        RdKey::F2 => "F2",
        RdKey::F3 => "F3",
        RdKey::F4 => "F4",
        RdKey::F5 => "F5",
        RdKey::F6 => "F6",
        RdKey::F7 => "F7",
        RdKey::F8 => "F8",
        RdKey::F9 => "F9",
        RdKey::F10 => "F10",
        RdKey::F11 => "F11",
        RdKey::F12 => "F12",
        RdKey::PrintScreen => "PrintScreen",
        RdKey::ScrollLock => "ScrollLock",
        RdKey::Pause => "Pause",
        RdKey::BackQuote => "Backquote",
        RdKey::Num1 => "Digit1",
        RdKey::Num2 => "Digit2",
        RdKey::Num3 => "Digit3",
        RdKey::Num4 => "Digit4",
        RdKey::Num5 => "Digit5",
        RdKey::Num6 => "Digit6",
        RdKey::Num7 => "Digit7",
        RdKey::Num8 => "Digit8",
        RdKey::Num9 => "Digit9",
        RdKey::Num0 => "Digit0",
        RdKey::Minus => "Minus",
        RdKey::Equal => "Equal",
        RdKey::Backspace => "Backspace",
        RdKey::Insert => "Insert",
        RdKey::Home => "Home",
        RdKey::PageUp => "PageUp",
        RdKey::Tab => "Tab",
        RdKey::KeyQ => "KeyQ",
        RdKey::KeyW => "KeyW",
        RdKey::KeyE => "KeyE",
        RdKey::KeyR => "KeyR",
        RdKey::KeyT => "KeyT",
        RdKey::KeyY => "KeyY",
        RdKey::KeyU => "KeyU",
        RdKey::KeyI => "KeyI",
        RdKey::KeyO => "KeyO",
        RdKey::KeyP => "KeyP",
        RdKey::LeftBracket => "BracketLeft",
        RdKey::RightBracket => "BracketRight",
        RdKey::BackSlash => "Backslash",
        RdKey::Delete => "Delete",
        RdKey::End => "End",
        RdKey::PageDown => "PageDown",
        RdKey::CapsLock => "CapsLock",
        RdKey::KeyA => "KeyA",
        RdKey::KeyS => "KeyS",
        RdKey::KeyD => "KeyD",
        RdKey::KeyF => "KeyF",
        RdKey::KeyG => "KeyG",
        RdKey::KeyH => "KeyH",
        RdKey::KeyJ => "KeyJ",
        RdKey::KeyK => "KeyK",
        RdKey::KeyL => "KeyL",
        RdKey::SemiColon => "Semicolon",
        RdKey::Quote => "Quote",
        RdKey::Return => "Enter",
        RdKey::ShiftLeft => "ShiftLeft",
        RdKey::KeyZ => "KeyZ",
        RdKey::KeyX => "KeyX",
        RdKey::KeyC => "KeyC",
        RdKey::KeyV => "KeyV",
        RdKey::KeyB => "KeyB",
        RdKey::KeyN => "KeyN",
        RdKey::KeyM => "KeyM",
        RdKey::Comma => "Comma",
        RdKey::Dot => "Period",
        RdKey::Slash => "Slash",
        RdKey::ShiftRight => "ShiftRight",
        RdKey::UpArrow => "ArrowUp",
        RdKey::ControlLeft => "ControlLeft",
        RdKey::MetaLeft => "MetaLeft",
        RdKey::Alt => "AltLeft",
        RdKey::Space => "Space",
        RdKey::AltGr => "AltRight",
        RdKey::MetaRight => "MetaRight",
        RdKey::ControlRight => "ControlRight",
        RdKey::LeftArrow => "ArrowLeft",
        RdKey::DownArrow => "ArrowDown",
        RdKey::RightArrow => "ArrowRight",
        RdKey::NumLock => "NumLock",
        RdKey::KpDivide => "NumpadDivide",
        RdKey::KpMultiply => "NumpadMultiply",
        RdKey::KpMinus => "NumpadSubtract",
        RdKey::Kp7 => "Numpad7",
        RdKey::Kp8 => "Numpad8",
        RdKey::Kp9 => "Numpad9",
        RdKey::KpPlus => "NumpadAdd",
        RdKey::Kp4 => "Numpad4",
        RdKey::Kp5 => "Numpad5",
        RdKey::Kp6 => "Numpad6",
        RdKey::Kp1 => "Numpad1",
        RdKey::Kp2 => "Numpad2",
        RdKey::Kp3 => "Numpad3",
        RdKey::KpReturn => "NumpadEnter",
        RdKey::Kp0 => "Numpad0",
        RdKey::KpDelete => "NumpadDecimal",
        _ => return None,
    })
}

#[derive(Clone, Copy)]
enum ArrowDirection {
    Up,
    Down,
    Left,
    Right,
}

fn arrow_direction(code: &str) -> Option<ArrowDirection> {
    match code {
        "ArrowUp" => Some(ArrowDirection::Up),
        "ArrowDown" => Some(ArrowDirection::Down),
        "ArrowLeft" => Some(ArrowDirection::Left),
        "ArrowRight" => Some(ArrowDirection::Right),
        _ => None,
    }
}

fn draw_arrow_triangle(painter: &egui::Painter, center: egui::Pos2, direction: ArrowDirection) {
    let w = 9.0;
    let h = 8.0;
    let points = match direction {
        ArrowDirection::Up => vec![
            egui::pos2(center.x, center.y - h / 2.0),
            egui::pos2(center.x - w / 2.0, center.y + h / 2.0),
            egui::pos2(center.x + w / 2.0, center.y + h / 2.0),
        ],
        ArrowDirection::Down => vec![
            egui::pos2(center.x, center.y + h / 2.0),
            egui::pos2(center.x - w / 2.0, center.y - h / 2.0),
            egui::pos2(center.x + w / 2.0, center.y - h / 2.0),
        ],
        ArrowDirection::Left => vec![
            egui::pos2(center.x - w / 2.0, center.y),
            egui::pos2(center.x + w / 2.0, center.y - h / 2.0),
            egui::pos2(center.x + w / 2.0, center.y + h / 2.0),
        ],
        ArrowDirection::Right => vec![
            egui::pos2(center.x + w / 2.0, center.y),
            egui::pos2(center.x - w / 2.0, center.y - h / 2.0),
            egui::pos2(center.x - w / 2.0, center.y + h / 2.0),
        ],
    };
    painter.add(egui::Shape::convex_polygon(
        points,
        egui::Color32::WHITE,
        egui::Stroke::NONE,
    ));
}

// Maps egui Key to W3C code strings for the focused-window input path (rdev hook goes silent when the window has focus).
fn egui_key_to_code(key: egui::Key) -> Option<&'static str> {
    use egui::Key;
    Some(match key {
        Key::Escape => "Escape",
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",
        Key::Backtick => "Backquote",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",
        Key::Num0 => "Digit0",
        Key::Minus => "Minus",
        Key::Equals => "Equal",
        Key::Backspace => "Backspace",
        Key::Insert => "Insert",
        Key::Home => "Home",
        Key::PageUp => "PageUp",
        Key::Tab => "Tab",
        Key::Q => "KeyQ",
        Key::W => "KeyW",
        Key::E => "KeyE",
        Key::R => "KeyR",
        Key::T => "KeyT",
        Key::Y => "KeyY",
        Key::U => "KeyU",
        Key::I => "KeyI",
        Key::O => "KeyO",
        Key::P => "KeyP",
        Key::OpenBracket => "BracketLeft",
        Key::CloseBracket => "BracketRight",
        Key::Backslash => "Backslash",
        Key::Delete => "Delete",
        Key::End => "End",
        Key::PageDown => "PageDown",
        Key::A => "KeyA",
        Key::S => "KeyS",
        Key::D => "KeyD",
        Key::F => "KeyF",
        Key::G => "KeyG",
        Key::H => "KeyH",
        Key::J => "KeyJ",
        Key::K => "KeyK",
        Key::L => "KeyL",
        Key::Semicolon => "Semicolon",
        Key::Quote => "Quote",
        Key::Enter => "Enter",
        Key::Z => "KeyZ",
        Key::X => "KeyX",
        Key::C => "KeyC",
        Key::V => "KeyV",
        Key::B => "KeyB",
        Key::N => "KeyN",
        Key::M => "KeyM",
        Key::Comma => "Comma",
        Key::Period => "Period",
        Key::Slash => "Slash",
        Key::ArrowUp => "ArrowUp",
        Key::Space => "Space",
        Key::ArrowLeft => "ArrowLeft",
        Key::ArrowDown => "ArrowDown",
        Key::ArrowRight => "ArrowRight",
        _ => return None,
    })
}

fn update_modifier(state: &KeyboardState, code: &'static str, pressed: bool) {
    if pressed {
        keyboard::key_down(state, code.to_string());
    } else {
        keyboard::key_up(state, code.to_string());
    }
}

enum KeyMsg {
    Down(&'static str),
    Up(&'static str),
}

pub struct KeyboardPanel {
    layout: KeyboardLayout,
    rx: Receiver<KeyMsg>,
    _tx: Sender<KeyMsg>,
    listener_started: bool,
    prev_modifiers: egui::Modifiers,
    prev_caps: bool,
}

impl Default for KeyboardPanel {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            layout: keyboard::keyboard_layout(),
            rx,
            _tx: tx,
            listener_started: false,
            prev_modifiers: egui::Modifiers::default(),
            prev_caps: false,
        }
    }
}

impl KeyboardPanel {
    fn ensure_listener(&mut self) {
        if self.listener_started {
            return;
        }
        self.listener_started = true;
        let tx = self._tx.clone();
        std::thread::spawn(move || {
            let _ = rdev::listen(move |event| {
                let msg = match event.event_type {
                    rdev::EventType::KeyPress(key) => rdev_key_to_code(key).map(KeyMsg::Down),
                    rdev::EventType::KeyRelease(key) => rdev_key_to_code(key).map(KeyMsg::Up),
                    _ => None,
                };
                if let Some(msg) = msg {
                    let _ = tx.send(msg);
                }
            });
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &KeyboardState) {
        self.ensure_listener();

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                KeyMsg::Down(code) => keyboard::key_down(state, code.to_string()),
                KeyMsg::Up(code) => keyboard::key_up(state, code.to_string()),
            }
        }

        // egui input path (focused window); modifiers aren't in Event::Key so we diff input.modifiers per frame.
        let cur_mods = ui.input(|input| {
            for event in &input.events {
                if let egui::Event::Key { key, pressed, .. } = event {
                    if let Some(code) = egui_key_to_code(*key) {
                        if *pressed {
                            keyboard::key_down(state, code.to_string());
                        } else {
                            keyboard::key_up(state, code.to_string());
                        }
                    }
                }
            }
            input.modifiers
        });
        let prev = self.prev_modifiers;
        if cur_mods.shift != prev.shift {
            update_modifier(state, "ShiftLeft", cur_mods.shift);
            update_modifier(state, "ShiftRight", cur_mods.shift);
        }
        if cur_mods.ctrl != prev.ctrl {
            update_modifier(state, "ControlLeft", cur_mods.ctrl);
            update_modifier(state, "ControlRight", cur_mods.ctrl);
        }
        if cur_mods.alt != prev.alt {
            update_modifier(state, "AltLeft", cur_mods.alt);
        }
        if cur_mods.command != prev.command {
            update_modifier(state, "MetaLeft", cur_mods.command);
            update_modifier(state, "MetaRight", cur_mods.command);
        }
        self.prev_modifiers = cur_mods;

        // CapsLock has no egui Key or Modifiers entry; poll GetKeyState(VK_CAPITAL) each frame instead.
        #[cfg(target_os = "windows")]
        {
            extern "system" {
                fn GetKeyState(n_virt_key: i32) -> i16;
            }
            let caps_on = (unsafe { GetKeyState(0x14) } & 0x0001) != 0;
            if caps_on != self.prev_caps {
                if caps_on {
                    keyboard::key_down(state, "CapsLock".to_string());
                } else {
                    keyboard::key_up(state, "CapsLock".to_string());
                }
                self.prev_caps = caps_on;
            }
        }

        let snapshot = keyboard::keyboard_snapshot(state);

        ui.heading("Keyboard Test");
        ui.label(
            "Click into this window, then press keys on your physical keyboard. Each key \
             lights up while held, and turns green once it's been tested at least once.",
        );
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(format!(
                "Tested {} / {} keys",
                snapshot.tested_count, snapshot.total_count
            ));
            if ui.button("Reset").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                keyboard::reset_tested(state);
            }
        });
        ui.add_space(12.0);

        let unit = 32.0;
        let gap = 4.0;
        let origin = ui.cursor().min;

        let all_defs: Vec<&KeyDef> = self.layout.main.iter().chain(self.layout.numpad.iter()).collect();

        for def in &all_defs {
            let w = def.col_span.unwrap_or(1.0) * unit - gap;
            let h = def.row_span.unwrap_or(1) as f32 * unit - gap;
            let x = origin.x + def.col * unit;
            let y = origin.y + def.row as f32 * unit;
            let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h));

            let pressed = snapshot.pressed.iter().any(|c| c == def.code);
            let tested = snapshot.tested.iter().any(|c| c == def.code);

            let fill = if pressed {
                egui::Color32::from_rgb(56, 189, 248)
            } else if tested {
                egui::Color32::from_rgb(48, 164, 108)
            } else {
                egui::Color32::from_gray(45)
            };

            ui.painter().rect_filled(rect, 3.0, fill);
            ui.painter().rect_stroke(
                rect,
                3.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(20)),
            );

            // Triangles instead of unicode arrow glyphs, since egui's embedded font renders those as tofu boxes.
            if let Some(direction) = arrow_direction(def.code) {
                draw_arrow_triangle(ui.painter(), rect.center(), direction);
            } else {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    def.label,
                    egui::FontId::proportional(11.0),
                    egui::Color32::WHITE,
                );
            }
        }

        let max_col = all_defs
            .iter()
            .map(|d| d.col + d.col_span.unwrap_or(1.0))
            .fold(0.0f32, f32::max);
        let max_row = all_defs
            .iter()
            .map(|d| d.row as f32 + d.row_span.unwrap_or(1) as f32)
            .fold(0.0f32, f32::max);
        ui.allocate_space(egui::vec2(max_col * unit, max_row * unit));
    }
}
