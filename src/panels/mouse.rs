use crate::mouse::{self, MouseState};

// Scroll events have no native "up" counterpart, so the pressed flash is cleared by hand
const SCROLL_FLASH: std::time::Duration = std::time::Duration::from_millis(150);

fn button_code(button: egui::PointerButton) -> Option<&'static str> {
    match button {
        egui::PointerButton::Primary => Some("Left"),
        egui::PointerButton::Secondary => Some("Right"),
        egui::PointerButton::Middle => Some("Middle"),
        egui::PointerButton::Extra1 => Some("Back"),
        egui::PointerButton::Extra2 => Some("Forward"),
    }
}

pub struct MousePanel {
    layout: mouse::MouseLayout,
    scroll_up_until: Option<std::time::Instant>,
    scroll_down_until: Option<std::time::Instant>,
}

impl Default for MousePanel {
    fn default() -> Self {
        Self {
            layout: mouse::mouse_layout(),
            scroll_up_until: None,
            scroll_down_until: None,
        }
    }
}

impl MousePanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &MouseState) {
        ui.heading("Mouse Test");
        ui.label(
            "Click into this window, then click each mouse button, press the side buttons, \
             and scroll the wheel up and down. Each input lights up while active, and turns \
             green once it's been tested at least once.",
        );
        ui.add_space(12.0);

        self.poll_input(ui, state);

        let snapshot = mouse::mouse_snapshot(state);

        ui.horizontal(|ui| {
            ui.label(format!(
                "{} / {} inputs tested",
                snapshot.tested_count, snapshot.total_count
            ));
            if ui.button("Reset").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                mouse::mouse_reset_tested(state);
            }
        });
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            self.draw_diagram(ui, &snapshot);
            ui.add_space(40.0);
            self.draw_counters(ui, &snapshot);
        });
    }

    fn poll_input(&mut self, ui: &egui::Ui, state: &MouseState) {
        let now = std::time::Instant::now();

        ui.input(|input| {
            for &button in &[
                egui::PointerButton::Primary,
                egui::PointerButton::Secondary,
                egui::PointerButton::Middle,
                egui::PointerButton::Extra1,
                egui::PointerButton::Extra2,
            ] {
                if let Some(code) = button_code(button) {
                    if input.pointer.button_pressed(button) {
                        mouse::button_down(state, code.to_string());
                    }
                    if input.pointer.button_released(button) {
                        mouse::button_up(state, code.to_string());
                    }
                }
            }

            let scroll = input.raw_scroll_delta.y;
            if scroll > 0.0 {
                mouse::scroll(state, "up");
                self.scroll_up_until = Some(now + SCROLL_FLASH);
            } else if scroll < 0.0 {
                mouse::scroll(state, "down");
                self.scroll_down_until = Some(now + SCROLL_FLASH);
            }

            if !input.focused {
                mouse::mouse_clear_pressed(state);
            }
        });

        if self.scroll_up_until.is_some_and(|t| now >= t) {
            mouse::button_up(state, "ScrollUp".to_string());
            self.scroll_up_until = None;
        }
        if self.scroll_down_until.is_some_and(|t| now >= t) {
            mouse::button_up(state, "ScrollDown".to_string());
            self.scroll_down_until = None;
        }
    }

    fn zone_color(snapshot: &crate::mouse::MouseSnapshot, code: &str) -> egui::Color32 {
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

    // Recreates the v0.4.0 (pre-egui-migration) CSS mouse diagram 1:1; literal pixel values below are taken straight from the original App.css.
    fn draw_diagram(&self, ui: &mut egui::Ui, snapshot: &crate::mouse::MouseSnapshot) {
        let scale = 0.85;
        let body_w = 240.0 * scale;
        let body_h = 410.0 * scale;
        let outline_w = 268.0 * scale;
        let outline_h = 410.0 * scale;

        let (outline_rect, _response) =
            ui.allocate_exact_size(egui::vec2(outline_w, outline_h), egui::Sense::hover());
        let painter = ui.painter();

        // Body is inset 28px from the outline's left edge; the side buttons occupy that margin.
        let body_rect = egui::Rect::from_min_size(
            egui::pos2(outline_rect.min.x + 28.0 * scale, outline_rect.min.y),
            egui::vec2(body_w, body_h),
        );

        let top_h = body_h * 0.52;
        let gap = 3.0 * scale;

        let top_row_rect = egui::Rect::from_min_size(
            body_rect.min,
            egui::vec2(body_rect.width(), top_h),
        );
        let bottom_rect = egui::Rect::from_min_max(
            egui::pos2(body_rect.min.x + gap, top_row_rect.max.y),
            egui::pos2(body_rect.max.x - gap, body_rect.max.y - gap),
        );

        let center_w = 40.0 * scale;
        let center_x = body_rect.center().x;
        let left_zone_rect = egui::Rect::from_min_max(
            egui::pos2(top_row_rect.min.x + gap, top_row_rect.min.y + gap),
            egui::pos2(center_x - center_w / 2.0, top_row_rect.max.y),
        );
        let right_zone_rect = egui::Rect::from_min_max(
            egui::pos2(center_x + center_w / 2.0, top_row_rect.min.y + gap),
            egui::pos2(top_row_rect.max.x - gap, top_row_rect.max.y),
        );

        let outer_radius = 140.0 * scale;
        let inner_radius = 90.0 * scale;

        // body background (shows through as the thin gaps between the top row and the edges)
        painter.add(egui::Shape::Rect(egui::epaint::RectShape::new(
            body_rect,
            egui::Rounding {
                nw: outer_radius,
                ne: outer_radius,
                sw: inner_radius,
                se: inner_radius,
            },
            egui::Color32::from_gray(22),
            egui::Stroke::new(1.0, egui::Color32::from_gray(39)),
        )));

        // Left/Right zones, rounded only on their outer top corner (matches the original `border-radius: 110px 0 0 0` / `0 110px 0 0`).
        let zone_radius = 130.0 * scale;
        painter.add(egui::Shape::Rect(egui::epaint::RectShape::new(
            left_zone_rect,
            egui::Rounding { nw: zone_radius, ne: 0.0, sw: 0.0, se: 0.0 },
            Self::zone_color(snapshot, "Left"),
            egui::Stroke::new(1.0, egui::Color32::from_gray(39)),
        )));
        painter.add(egui::Shape::Rect(egui::epaint::RectShape::new(
            right_zone_rect,
            egui::Rounding { nw: 0.0, ne: zone_radius, sw: 0.0, se: 0.0 },
            Self::zone_color(snapshot, "Right"),
            egui::Stroke::new(1.0, egui::Color32::from_gray(39)),
        )));

        // Bottom corners reuse the body's own rounding so this rect's curve lines up with the body outline.
        painter.add(egui::Shape::Rect(egui::epaint::RectShape::new(
            bottom_rect,
            egui::Rounding {
                nw: 0.0,
                ne: 0.0,
                sw: (inner_radius - gap).max(0.0),
                se: (inner_radius - gap).max(0.0),
            },
            egui::Color32::from_gray(31),
            egui::Stroke::NONE,
        )));

        // center column: scroll-up indicator, wheel, scroll-down indicator, stacked and centered
        let wheel_w = 20.0 * scale;
        let wheel_h = 54.0 * scale;
        let indicator_h = 20.0 * scale;
        let col_gap = 8.0 * scale;
        let col_total_h = indicator_h * 2.0 + wheel_h + col_gap * 2.0;
        let col_top = top_row_rect.center().y - col_total_h / 2.0;

        let scroll_up_rect = egui::Rect::from_min_size(
            egui::pos2(center_x - center_w / 2.0, col_top),
            egui::vec2(center_w, indicator_h),
        );
        let wheel_rect = egui::Rect::from_min_size(
            egui::pos2(center_x - wheel_w / 2.0, scroll_up_rect.max.y + col_gap),
            egui::vec2(wheel_w, wheel_h),
        );
        let scroll_down_rect = egui::Rect::from_min_size(
            egui::pos2(center_x - center_w / 2.0, wheel_rect.max.y + col_gap),
            egui::vec2(center_w, indicator_h),
        );

        painter.rect_filled(wheel_rect, 10.0 * scale, Self::zone_color(snapshot, "Middle"));

        // Filled triangles instead of a glyph-on-square, so only the arrow itself lights up.
        let tri_w = 9.0 * scale;
        let tri_h = 7.0 * scale;
        let up_center = scroll_up_rect.center();
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(up_center.x, up_center.y - tri_h / 2.0),
                egui::pos2(up_center.x - tri_w / 2.0, up_center.y + tri_h / 2.0),
                egui::pos2(up_center.x + tri_w / 2.0, up_center.y + tri_h / 2.0),
            ],
            Self::zone_color(snapshot, "ScrollUp"),
            egui::Stroke::NONE,
        ));
        let down_center = scroll_down_rect.center();
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(down_center.x, down_center.y + tri_h / 2.0),
                egui::pos2(down_center.x - tri_w / 2.0, down_center.y - tri_h / 2.0),
                egui::pos2(down_center.x + tri_w / 2.0, down_center.y - tri_h / 2.0),
            ],
            Self::zone_color(snapshot, "ScrollDown"),
            egui::Stroke::NONE,
        ));

        painter.text(
            left_zone_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Left",
            egui::FontId::proportional(14.0 * scale),
            egui::Color32::from_gray(244),
        );
        painter.text(
            right_zone_rect.center(),
            egui::Align2::CENTER_CENTER,
            "Right",
            egui::FontId::proportional(14.0 * scale),
            egui::Color32::from_gray(244),
        );

        // Side (thumb) buttons: shallow pill bulges overlapping the body's left edge, matching the original's `left: 22px` against the body's `left: 28px`.
        let side_w = 6.0 * scale;
        let side_h = 32.0 * scale;
        let side_gap = 8.0 * scale;
        let side_x = outline_rect.min.x + 22.0 * scale;
        let side_top = body_rect.min.y + body_rect.height() * 0.32;

        let forward_rect = egui::Rect::from_min_size(
            egui::pos2(side_x, side_top),
            egui::vec2(side_w, side_h),
        );
        let back_rect = egui::Rect::from_min_size(
            egui::pos2(side_x, side_top + side_h + side_gap),
            egui::vec2(side_w, side_h),
        );
        let side_radius = side_h / 2.0;
        painter.rect_filled(forward_rect, side_radius, Self::zone_color(snapshot, "Forward"));
        painter.rect_filled(back_rect, side_radius, Self::zone_color(snapshot, "Back"));
    }

    fn draw_counters(&self, ui: &mut egui::Ui, snapshot: &crate::mouse::MouseSnapshot) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Click Counter").strong());
            ui.add_space(8.0);
            egui::Grid::new("mouse-counters").striped(true).show(ui, |ui| {
                for button in &self.layout.buttons {
                    ui.label(button.label);
                    let count = snapshot.counts.get(button.code).copied().unwrap_or(0);
                    ui.label(count.to_string());
                    ui.end_row();
                }
            });
        });
    }
}
