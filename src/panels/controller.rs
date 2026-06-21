use crate::controller::{self, ControllerLayout, ControllerState};
use std::sync::mpsc::{Receiver, Sender, channel};

use egui::{Color32, Painter, Pos2, Rect, Rounding, Stroke, pos2};

// Two pill-shaped tabs above the shoulder bumps, with an axis fill bar for trigger depth.
fn draw_l2_r2(
    painter: &Painter,
    rect: Rect,
    l2_color: Color32,
    r2_color: Color32,
    l2_axis: f32,   // 0.0 = not pressed, 1.0 = fully pressed
    r2_axis: f32,
    outline: Stroke,
) {
    let w = rect.width();
    let h = rect.height();
    let ox = rect.min.x;
    let oy = rect.min.y;
    let n = |x: f32, y: f32| -> Pos2 { pos2(ox + x * w, oy + y * h) };

    let bg   = Color32::from_gray(30);
    let rnd  = Rounding::same(w * 0.025);

    // L2 (shifted up by h * 0.19 so it floats clear above the body outline with no overlap)
    let l2 = Rect::from_min_max(n(0.08, 0.01 - 0.19), n(0.30, 0.11 - 0.19));
    painter.rect_filled(l2, rnd, bg);
    // axis fill bar (left-to-right inside the rect)
    if l2_axis > 0.0 {
        let bar = Rect::from_min_max(
            l2.min,
            pos2(l2.min.x + l2.width() * l2_axis, l2.max.y),
        );
        painter.rect_filled(bar, rnd, l2_color);
    }
    painter.rect_stroke(l2, rnd, outline);

    // R2 (shifted up by h * 0.19, same as L2)
    let r2 = Rect::from_min_max(n(0.70, 0.01 - 0.19), n(0.92, 0.11 - 0.19));
    painter.rect_filled(r2, rnd, bg);
    if r2_axis > 0.0 {
        let bar = Rect::from_min_max(
            r2.min,
            pos2(r2.min.x + r2.width() * r2_axis, r2.max.y),
        );
        painter.rect_filled(bar, rnd, r2_color);
    }
    painter.rect_stroke(r2, rnd, outline);

    // Labels (below their boxes)
    painter.text(
        pos2(l2.center().x, l2.max.y + h * 0.02),
        egui::Align2::CENTER_TOP,
        format!("L2 {:.0}%", l2_axis * 100.0),
        egui::FontId::proportional(10.0),
        Color32::from_gray(160),
    );
    painter.text(
        pos2(r2.center().x, r2.max.y + h * 0.02),
        egui::Align2::CENTER_TOP,
        format!("R2 {:.0}%", r2_axis * 100.0),
        egui::FontId::proportional(10.0),
        Color32::from_gray(160),
    );
}

// Same floating-pill style as L2/R2, smaller and stacked above; solid fill since they're digital.
fn draw_l1_r1(
    painter: &Painter,
    rect: Rect,
    l1_color: Color32,
    r1_color: Color32,
    outline: Stroke,
) {
    let w = rect.width();
    let h = rect.height();
    let ox = rect.min.x;
    let oy = rect.min.y;
    let n = |x: f32, y: f32| -> Pos2 { pos2(ox + x * w, oy + y * h) };

    let rnd = Rounding::same(w * 0.02);

    // L1 (smaller than L2/R2, stacked above it)
    let l1 = Rect::from_min_max(n(0.11, 0.03 - 0.30), n(0.27, 0.09 - 0.30));
    painter.rect_filled(l1, rnd, l1_color);
    painter.rect_stroke(l1, rnd, outline);
    painter.text(
        pos2(l1.center().x, l1.min.y - h * 0.02),
        egui::Align2::CENTER_BOTTOM,
        "L1",
        egui::FontId::proportional(10.0),
        Color32::from_gray(160),
    );

    // R1 (mirror of L1 -- x flipped around 0.5)
    let r1 = Rect::from_min_max(n(0.73, 0.03 - 0.30), n(0.89, 0.09 - 0.30));
    painter.rect_filled(r1, rnd, r1_color);
    painter.rect_stroke(r1, rnd, outline);
    painter.text(
        pos2(r1.center().x, r1.min.y - h * 0.02),
        egui::Align2::CENTER_BOTTOM,
        "R1",
        egui::FontId::proportional(10.0),
        Color32::from_gray(160),
    );
}

enum PadMsg {
    Down(&'static str),
    Up(&'static str),
    Stick { code: &'static str, x: f32, y: f32 },
    Trigger { code: &'static str, value: f32 },
    Connected,
    Disconnected,
}

fn button_code(button: gilrs::Button) -> Option<&'static str> {
    use gilrs::Button;
    Some(match button {
        Button::North => "FaceNorth",
        Button::East => "FaceEast",
        Button::South => "FaceSouth",
        Button::West => "FaceWest",
        Button::DPadUp => "DpadUp",
        Button::DPadDown => "DpadDown",
        Button::DPadLeft => "DpadLeft",
        Button::DPadRight => "DpadRight",
        Button::LeftTrigger => "L1",
        Button::RightTrigger => "R1",
        Button::LeftThumb => "L3",
        Button::RightThumb => "R3",
        Button::Start => "Start",
        Button::Select => "Select",
        Button::Mode => "Guide",
        // LeftTrigger2/RightTrigger2 are analog and handled separately via ButtonChanged
        _ => return None,
    })
}

// gilrs reports stick X/Y as separate events, so this keeps a running (x, y) per stick and sends the combined value on each change.
fn handle_axis(tx: &Sender<PadMsg>, axis: gilrs::Axis, value: f32, left: &mut (f32, f32), right: &mut (f32, f32)) {
    use gilrs::Axis;
    match axis {
        Axis::LeftStickX => left.0 = value,
        Axis::LeftStickY => left.1 = value,
        Axis::RightStickX => right.0 = value,
        Axis::RightStickY => right.1 = value,
        // Some backends report analog triggers as Z axes instead of ButtonChanged.
        Axis::LeftZ => {
            let _ = tx.send(PadMsg::Trigger { code: controller::TRIGGER_L2, value });
            return;
        }
        Axis::RightZ => {
            let _ = tx.send(PadMsg::Trigger { code: controller::TRIGGER_R2, value });
            return;
        }
        _ => return,
    }
    match axis {
        Axis::LeftStickX | Axis::LeftStickY => {
            let _ = tx.send(PadMsg::Stick { code: controller::STICK_LEFT, x: left.0, y: left.1 });
        }
        Axis::RightStickX | Axis::RightStickY => {
            let _ = tx.send(PadMsg::Stick { code: controller::STICK_RIGHT, x: right.0, y: right.1 });
        }
        _ => {}
    }
}

pub struct ControllerPanel {
    layout: ControllerLayout,
    rx: Receiver<PadMsg>,
    _tx: Sender<PadMsg>,
    listener_started: bool,
}

impl Default for ControllerPanel {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            layout: controller::controller_layout(),
            rx,
            _tx: tx,
            listener_started: false,
        }
    }
}

impl ControllerPanel {
    fn ensure_listener(&mut self) {
        if self.listener_started {
            return;
        }
        self.listener_started = true;
        let tx = self._tx.clone();
        std::thread::spawn(move || {
            let mut gilrs = match gilrs::Gilrs::new() {
                Ok(g) => g,
                // No gamepad backend available on this machine; panel just stays disconnected.
                Err(_) => return,
            };

            if gilrs.gamepads().next().is_some() {
                let _ = tx.send(PadMsg::Connected);
            }

            let mut left = (0.0f32, 0.0f32);
            let mut right = (0.0f32, 0.0f32);

            loop {
                while let Some(event) = gilrs.next_event() {
                    use gilrs::EventType;
                    match event.event {
                        EventType::ButtonPressed(button, _) => {
                            if let Some(code) = button_code(button) {
                                let _ = tx.send(PadMsg::Down(code));
                            }
                        }
                        EventType::ButtonReleased(button, _) => {
                            if let Some(code) = button_code(button) {
                                let _ = tx.send(PadMsg::Up(code));
                            }
                        }
                        EventType::ButtonChanged(gilrs::Button::LeftTrigger2, value, _) => {
                            let _ = tx.send(PadMsg::Trigger { code: controller::TRIGGER_L2, value });
                        }
                        EventType::ButtonChanged(gilrs::Button::RightTrigger2, value, _) => {
                            let _ = tx.send(PadMsg::Trigger { code: controller::TRIGGER_R2, value });
                        }
                        EventType::AxisChanged(axis, value, _) => {
                            handle_axis(&tx, axis, value, &mut left, &mut right);
                        }
                        EventType::Connected => {
                            let _ = tx.send(PadMsg::Connected);
                        }
                        EventType::Disconnected => {
                            let _ = tx.send(PadMsg::Disconnected);
                        }
                        _ => {}
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(8));
            }
        });
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &ControllerState) {
        self.ensure_listener();

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                PadMsg::Down(code) => controller::button_down(state, code.to_string()),
                PadMsg::Up(code) => controller::button_up(state, code.to_string()),
                PadMsg::Stick { code, x, y } => controller::set_stick(state, code, x, y),
                PadMsg::Trigger { code, value } => controller::set_trigger(state, code, value),
                PadMsg::Connected => controller::set_connected(state, true),
                PadMsg::Disconnected => {
                    controller::set_connected(state, false);
                    controller::clear_pressed(state);
                }
            }
        }

        let snapshot = controller::controller_snapshot(state);

        ui.heading("Controller Test");
        ui.label(
            "Plug in a gamepad and exercise every button, both sticks, and both triggers. \
             Inputs light up while active, and turn green once they've been tested at least once.",
        );
        ui.add_space(12.0);

        if !snapshot.connected {
            ui.colored_label(
                egui::Color32::from_gray(160),
                "No controller detected. Connect one to begin.",
            );
        }

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} / {} inputs tested",
                snapshot.tested_count, snapshot.total_count
            ));
            if ui.button("Reset").clicked() {
                controller::reset_tested(state);
            }
        });
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            self.draw_diagram(ui, &snapshot);
            ui.add_space(40.0);
            self.draw_counters(ui, &snapshot);
        });
    }

    fn zone_color(snapshot: &controller::ControllerSnapshot, code: &str) -> egui::Color32 {
        let pressed = snapshot.pressed.iter().any(|c| c == code);
        let tested = snapshot.tested.iter().any(|c| c == code);
        if pressed {
            egui::Color32::from_rgb(56, 189, 248)
        } else if tested {
            egui::Color32::from_rgb(48, 164, 108)
        } else {
            egui::Color32::from_gray(45)
        }
    }

    fn trigger_color(snapshot: &controller::ControllerSnapshot, value: f32, code: &str) -> egui::Color32 {
        if value >= controller::TRIGGER_DEADZONE {
            egui::Color32::from_rgb(56, 189, 248)
        } else if snapshot.tested.iter().any(|c| c == code) {
            egui::Color32::from_rgb(48, 164, 108)
        } else {
            egui::Color32::from_gray(45)
        }
    }

    // Ear-clipping triangulation; returns vertex index triples into `pts`. O(n^2), fine for the 44-point silhouette.
    fn triangulate(pts: &[egui::Pos2]) -> Vec<[u32; 3]> {
        fn cross(o: egui::Pos2, a: egui::Pos2, b: egui::Pos2) -> f32 {
            (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
        }
        fn point_in_triangle(p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> bool {
            let d1 = cross(a, b, p);
            let d2 = cross(b, c, p);
            let d3 = cross(c, a, p);
            let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
            let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
            !(has_neg && has_pos)
        }

        let signed_area: f32 = (0..pts.len())
            .map(|i| {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                a.x * b.y - b.x * a.y
            })
            .sum();
        let mut order: Vec<u32> = (0..pts.len() as u32).collect();
        if signed_area < 0.0 {
            order.reverse(); // ear clipping below assumes counter-clockwise winding
        }

        let mut triangles = Vec::new();
        let mut remaining = order.clone();
        let mut guard = 0;
        while remaining.len() > 3 && guard < pts.len() * pts.len() {
            guard += 1;
            let n = remaining.len();
            let mut clipped = false;
            for i in 0..n {
                let prev = remaining[(i + n - 1) % n];
                let cur = remaining[i];
                let next = remaining[(i + 1) % n];
                let (a, b, c) = (pts[prev as usize], pts[cur as usize], pts[next as usize]);
                if cross(a, b, c) <= 0.0 {
                    continue; // reflex vertex, can't be an ear
                }
                let is_ear = remaining
                    .iter()
                    .filter(|&&idx| idx != prev && idx != cur && idx != next)
                    .all(|&idx| !point_in_triangle(pts[idx as usize], a, b, c));
                if is_ear {
                    triangles.push([prev, cur, next]);
                    remaining.remove(i);
                    clipped = true;
                    break;
                }
            }
            if !clipped {
                break; // degenerate input; stop rather than loop forever
            }
        }
        if remaining.len() == 3 {
            triangles.push([remaining[0], remaining[1], remaining[2]]);
        }
        triangles
    }

    // Draws the body silhouette (ear-clipped fill, then stroke outline on top) and the L1/L2/R1/R2 boxes anchored to it.
    fn draw_controller_body(
        painter: &egui::Painter,
        rect: egui::Rect,
        snapshot: &controller::ControllerSnapshot,
        body_fill: egui::Color32,
        outline_color: egui::Color32,
        outline_width: f32,
    ) {
        let w = rect.width();
        let h = rect.height();
        let ox = rect.min.x;
        let oy = rect.min.y;
        let n = |x: f32, y: f32| -> egui::Pos2 { egui::pos2(ox + x * w, oy + y * h) };

        // 44 points sampled clockwise from the controllerdoctor.com SVG silhouette (viewBox 0 0 1480 980), shared by the fill and outline below.
        let pts: Vec<egui::Pos2> = vec![
            // Left grip: bottom
            n(0.108, 0.999),
            n(0.062, 0.967),
            n(0.032, 0.900),
            n(0.012, 0.824),
            n(0.003, 0.745),
            n(0.002, 0.662), // left side, widest (min x)
            // Left side: up toward shoulder (mirrored from the right side, x -> 1-x)
            n(0.014, 0.500),
            n(0.044, 0.291),
            n(0.082, 0.150),
            // Left shoulder bump (protrudes upward, mirrored from the right shoulder bump)
            n(0.122, 0.097),
            n(0.154, 0.033),
            n(0.203, 0.008),
            n(0.261, 0.006), // left shoulder peak (top)
            n(0.298, 0.036),
            // Top edge: left inner -> center (mirrored from the right side)
            n(0.400, 0.022),
            n(0.487, 0.018), // top center
            // Top edge: center -> right inner
            n(0.600, 0.022),
            n(0.702, 0.036),
            // Right shoulder bump (protrudes upward)
            n(0.739, 0.006), // right shoulder peak (top)
            n(0.797, 0.008),
            n(0.846, 0.033),
            n(0.878, 0.097),
            n(0.918, 0.150),
            // Right side: down toward grip
            n(0.956, 0.291),
            n(0.983, 0.459),
            n(0.999, 0.624),
            n(0.998, 0.662), // right side, widest (max x) -- mirrored from left's n(0.002, 0.662)
            n(0.997, 0.745), // mirrored from left's n(0.003, 0.745)
            n(0.988, 0.824), // mirrored from left's n(0.012, 0.824)
            n(0.968, 0.900), // mirrored from left's n(0.032, 0.900)
            n(0.938, 0.967), // mirrored from left's n(0.062, 0.967)
            // Right grip: bottom (mirrored from left's n(0.108, 0.999))
            n(0.892, 0.999),
            // Right grip: inner edge going up toward shelf (mirrored from left's inner edge)
            n(0.842, 0.986), // mirrored from left's n(0.158, 0.986)
            n(0.819, 0.927), // mirrored from left's n(0.181, 0.927)
            n(0.788, 0.805), // mirrored from left's n(0.212, 0.805)
            n(0.763, 0.729), // mirrored from left's n(0.237, 0.729)
            n(0.746, 0.695), // mirrored from left's n(0.254, 0.695)
            // Shelf between grips: right -> left
            n(0.676, 0.670),
            n(0.516, 0.674), // center of shelf
            n(0.354, 0.674),
            n(0.324, 0.670), // mirrored from right's n(0.676, 0.670)
            // Left grip: inner edge going down from shelf
            n(0.254, 0.695),
            n(0.237, 0.729),
            n(0.212, 0.805),
            n(0.181, 0.927),
            n(0.158, 0.986),
        ];

        // Ear-clip into a Mesh instead of a fan-filled PathShape, since the concave grip necks break naive fan filling.
        let mut body_mesh = egui::epaint::Mesh::default();
        for p in &pts {
            body_mesh.colored_vertex(*p, body_fill);
        }
        for [a, b, c] in Self::triangulate(&pts) {
            body_mesh.add_triangle(a, b, c);
        }
        painter.add(egui::Shape::mesh(body_mesh));

        let outline_stroke = egui::Stroke::new(outline_width, outline_color);
        draw_l2_r2(
            painter,
            rect,
            Self::trigger_color(snapshot, snapshot.l2, "L2"),
            Self::trigger_color(snapshot, snapshot.r2, "R2"),
            snapshot.l2.clamp(0.0, 1.0),
            snapshot.r2.clamp(0.0, 1.0),
            outline_stroke,
        );
        draw_l1_r1(painter, rect, Self::zone_color(snapshot, "L1"), Self::zone_color(snapshot, "R1"), outline_stroke);

        // Same 44-point silhouette, traced as a closed stroke-only outline on top of the fill.
        painter.add(egui::Shape::Path(egui::epaint::PathShape {
            points: pts,
            closed: true,
            fill: egui::Color32::TRANSPARENT,
            stroke: egui::epaint::PathStroke::new(outline_width, outline_color),
        }));
    }

    // D-pad: a plus sign made of four rounded arms plus a center square, no triangles.
    fn draw_dpad(painter: &egui::Painter, snapshot: &controller::ControllerSnapshot, center: egui::Pos2) {
        let arm_len = 14.5;
        let arm_w = 17.4; // same ratio to arm_len as before
        let center_sq = 12.4;
        let up = egui::Rect::from_center_size(center + egui::vec2(0.0, -arm_len), egui::vec2(arm_w, arm_len * 2.0));
        let down = egui::Rect::from_center_size(center + egui::vec2(0.0, arm_len), egui::vec2(arm_w, arm_len * 2.0));
        let left = egui::Rect::from_center_size(center + egui::vec2(-arm_len, 0.0), egui::vec2(arm_len * 2.0, arm_w));
        let right = egui::Rect::from_center_size(center + egui::vec2(arm_len, 0.0), egui::vec2(arm_len * 2.0, arm_w));
        let mid = egui::Rect::from_center_size(center, egui::vec2(center_sq, center_sq));
        for (r, code) in [(up, "DpadUp"), (down, "DpadDown"), (left, "DpadLeft"), (right, "DpadRight")] {
            painter.rect_filled(r, 3.0, Self::zone_color(snapshot, code));
        }
        painter.rect_filled(mid, 2.0, egui::Color32::from_gray(45));
    }

    // Draws the PS5-DualSense-style gamepad: body, then center buttons, D-pad, face buttons, and both sticks.
    fn draw_diagram(&self, ui: &mut egui::Ui, snapshot: &controller::ControllerSnapshot) {
        let size = egui::vec2(460.0, 414.0);
        let (canvas, _response) = ui.allocate_exact_size(size, egui::Sense::hover());
        let painter = ui.painter();
        // Top margin reserved for the L1/L2/R1/R2 boxes floating above the body outline.
        let rect = egui::Rect::from_min_size(
            egui::pos2(canvas.left() + 30.0, canvas.top() + 105.0),
            egui::vec2(canvas.width() - 60.0, canvas.height() - 125.0),
        );
        let x = |f: f32| rect.left() + rect.width() * f;
        let y = |f: f32| rect.top() + rect.height() * f;

        Self::draw_controller_body(
            painter,
            rect,
            snapshot,
            egui::Color32::from_gray(20), // hardcoded for now to verify the fill renders correctly
            egui::Color32::from_gray(55),
            1.5,
        );

        // D-pad: upper-left (same row as the face buttons)
        Self::draw_dpad(painter, snapshot, egui::pos2(x(0.18), y(0.36 - 0.08)));

        // Face buttons: upper-right, 4 circles at compass positions (left blank, no labels)
        let face_center = egui::pos2(x(0.82), y(0.36 - 0.08));
        let face_r = 13.0;
        let face_offset = 22.0;
        for (pos, code) in [
            (face_center + egui::vec2(0.0, -face_offset), "FaceNorth"),
            (face_center + egui::vec2(0.0, face_offset), "FaceSouth"),
            (face_center + egui::vec2(-face_offset, 0.0), "FaceWest"),
            (face_center + egui::vec2(face_offset, 0.0), "FaceEast"),
        ] {
            painter.circle_filled(pos, face_r, Self::zone_color(snapshot, code));
            painter.circle_stroke(pos, face_r, egui::Stroke::new(1.0, egui::Color32::from_gray(20)));
        }

        // Select/Start: small circles pushed diagonally up and out from center.
        let center_r = 8.0;
        for (fx, dy, code, label) in [
            (0.30, -rect.height() * 0.22, "Select", "Sel"),
            (0.70, -rect.height() * 0.22, "Start", "St"),
        ] {
            let pos = egui::pos2(x(fx), y(0.42 - 0.08) + rect.height() * 0.04 + dy);
            painter.circle_filled(pos, center_r, Self::zone_color(snapshot, code));
            painter.text(egui::pos2(pos.x, pos.y + center_r + 9.0), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(8.0), egui::Color32::from_gray(200));
        }

        // Sticks: lower-left/right, each an outer circle with a live position dot (gilrs is Y-up, screen is Y-down, so the dot's y is negated below).
        let stick_r = 24.0;
        let dot_r = 7.0;
        let left_stick_center = egui::pos2(x(0.30), y(0.62 - 0.08 - 0.03));
        let right_stick_center = egui::pos2(x(0.70), y(0.62 - 0.08 - 0.03));

        // Home: centered between the sticks, aligned with the L3/R3 dots.
        let home_pos = egui::pos2(x(0.5), left_stick_center.y);
        painter.circle_filled(home_pos, center_r, Self::zone_color(snapshot, "Guide"));
        painter.text(egui::pos2(home_pos.x, home_pos.y + center_r + 9.0), egui::Align2::CENTER_CENTER, "Home", egui::FontId::proportional(8.0), egui::Color32::from_gray(200));

        for (center, (sx, sy), click_code, stick_code, label) in [
            (left_stick_center, snapshot.left_stick, "L3", controller::STICK_LEFT, "L"),
            (right_stick_center, snapshot.right_stick, "R3", controller::STICK_RIGHT, "R"),
        ] {
            let ring_color = if snapshot.pressed.iter().any(|c| c == click_code) {
                egui::Color32::from_rgb(56, 189, 248)
            } else if snapshot.tested.iter().any(|c| c == click_code) {
                egui::Color32::from_rgb(48, 164, 108)
            } else {
                egui::Color32::from_gray(39)
            };
            painter.circle_filled(center, stick_r, egui::Color32::from_gray(31));
            painter.circle_stroke(center, stick_r, egui::Stroke::new(2.0, ring_color));
            let dot_pos = egui::pos2(center.x + sx * (stick_r - dot_r), center.y - sy * (stick_r - dot_r));
            let mag = (sx * sx + sy * sy).sqrt();
            let dot_color = if mag >= controller::STICK_DEADZONE {
                egui::Color32::from_rgb(56, 189, 248)
            } else if snapshot.tested.iter().any(|c| c == stick_code) {
                egui::Color32::from_rgb(48, 164, 108)
            } else {
                egui::Color32::from_gray(120)
            };
            painter.circle_filled(dot_pos, dot_r, dot_color);
            painter.text(egui::pos2(center.x, center.y + stick_r + 12.0), egui::Align2::CENTER_CENTER, format!("{} Stick", label), egui::FontId::proportional(10.0), egui::Color32::from_gray(200));
        }
    }

    fn draw_counters(&self, ui: &mut egui::Ui, snapshot: &controller::ControllerSnapshot) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Input Counter").strong());
            ui.add_space(8.0);
            egui::Grid::new("controller-counters").striped(true).show(ui, |ui| {
                for input in self.layout.digital.iter().chain(&self.layout.sticks).chain(&self.layout.triggers) {
                    ui.label(input.label);
                    let count = snapshot.counts.get(input.code).copied().unwrap_or(0);
                    ui.label(count.to_string());
                    ui.end_row();
                }
            });
        });
    }
}
