use crate::audio::{self, AudioState, DeviceInfo, RecordedClip};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

struct ActiveRecording {
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
    stop_flag: Arc<AtomicBool>,
    started_at: Instant,
    stream: cpal::Stream,
}

pub struct MicrophonePanel {
    devices: Vec<DeviceInfo>,
    selected: Option<usize>,
    device_error: Option<String>,
    monitor_error: Option<String>,
    peak_level: f32,
    active_recording: Option<ActiveRecording>,
    clip: Option<RecordedClip>,
    clip_duration_secs: f32,
    playing: bool,
    playback_volume: f32,
}

impl Default for MicrophonePanel {
    fn default() -> Self {
        let mut panel = Self {
            devices: Vec::new(),
            selected: None,
            device_error: None,
            monitor_error: None,
            peak_level: 0.0,
            active_recording: None,
            clip: None,
            clip_duration_secs: 0.0,
            playing: false,
            playback_volume: 1.0,
        };
        panel.refresh_devices();
        panel
    }
}

impl MicrophonePanel {
    fn refresh_devices(&mut self) {
        match audio::list_input_devices() {
            Ok(devices) => {
                self.devices = devices;
                self.device_error = None;
                if self.selected.is_none() && !self.devices.is_empty() {
                    self.selected = Some(0);
                }
            }
            Err(err) => self.device_error = Some(err),
        }
    }

    fn selected_device(&self) -> Option<&DeviceInfo> {
        self.selected.and_then(|i| self.devices.get(i))
    }

    pub fn show(&mut self, ui: &mut egui::Ui, audio_state: &AudioState) {
        ui.heading("Microphone Test");
        ui.label(
            "Pick your mic below, speak, and watch the level meter. Use \"Record & Play Back\" \
             to hear exactly what your mic captures.",
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("Input device:");
            let selected_label = self
                .selected_device()
                .map(|d| d.label.clone())
                .unwrap_or_else(|| "No microphones found".to_string());

            let mut newly_selected = None;
            egui::ComboBox::from_id_salt("mic-select")
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (i, device) in self.devices.iter().enumerate() {
                        let selected = self.selected == Some(i);
                        if ui.selectable_label(selected, &device.label).clicked()
                            && self.selected != Some(i)
                        {
                            newly_selected = Some(i);
                        }
                    }
                });
            if let Some(i) = newly_selected {
                self.selected = Some(i);
                self.start_monitor(audio_state);
            }

            if ui.button("Refresh").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                self.refresh_devices();
            }
        });

        if let Some(err) = self.device_error.clone().or(self.monitor_error.clone()) {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::from_rgb(229, 72, 77), err);
        }

        ui.add_space(16.0);

        let level = audio_state.mic_level();
        let pct = level.round() as i32;
        if pct as f32 > self.peak_level {
            self.peak_level = pct as f32;
        } else {
            self.peak_level = (self.peak_level - 1.0).max(0.0);
        }

        ui.horizontal(|ui| {
            ui.label("Input Level");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{pct}%"));
            });
        });

        let bar_color = if level > 85.0 {
            egui::Color32::from_rgb(229, 72, 77)
        } else if level > 60.0 {
            egui::Color32::from_rgb(245, 184, 0)
        } else {
            egui::Color32::from_rgb(48, 164, 108)
        };

        let desired_size = egui::vec2(ui.available_width(), 16.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        ui.painter().rect_filled(rect, 4.0, egui::Color32::from_gray(40));
        let fill_width = rect.width() * (level / 100.0).clamp(0.0, 1.0);
        let fill_rect = egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
        ui.painter().rect_filled(fill_rect, 4.0, bar_color);
        let peak_x = rect.min.x + rect.width() * (self.peak_level / 100.0).clamp(0.0, 1.0);
        ui.painter().vline(
            peak_x,
            rect.min.y..=rect.max.y,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );

        ui.add_space(20.0);

        let elapsed_secs = self
            .active_recording
            .as_ref()
            .map(|r| r.started_at.elapsed().as_secs_f32());
        let is_recording = elapsed_secs.is_some();

        ui.horizontal(|ui| {
            let (label, enabled) = if is_recording {
                ("Stop Recording", true)
            } else {
                ("Record & Play Back", self.selected_device().is_some())
            };

            if ui.add_enabled(enabled, egui::Button::new(label)).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                if is_recording {
                    self.stop_recording(audio_state);
                } else {
                    self.start_recording();
                }
            }

            if let Some(secs) = elapsed_secs {
                ui.label(format!("Recording... {:.1}s", secs));
            }
        });

        if is_recording {
            ui.ctx().request_repaint();
        }

        if self.clip.is_some() {
            ui.add_space(16.0);
            self.show_player(ui, audio_state);
        }
    }

    fn show_player(&mut self, ui: &mut egui::Ui, audio_state: &AudioState) {
        if audio_state.playback_finished() && self.playing {
            self.playing = false;
        }

        let pos = audio_state.playback_position().clamp(0.0, 1.0);
        let dur = self.clip_duration_secs;
        let elapsed = pos * dur;

        let pill_height = 36.0;
        let pill_width = ui.available_width().min(480.0);
        let pill_color = egui::Color32::from_gray(230);
        let track_color = egui::Color32::from_gray(150);
        let fill_color = egui::Color32::from_gray(80);
        let text_color = egui::Color32::from_gray(30);

        let (pill_rect, _) = ui.allocate_exact_size(
            egui::vec2(pill_width, pill_height),
            egui::Sense::hover(),
        );
        ui.painter().rect_filled(pill_rect, pill_height / 2.0, pill_color);

        let pad = 12.0;
        let play_btn_w = 20.0;
        let time_w = 72.0; // "0:00 / 1:44"
        let speaker_w = 20.0;
        let scrubber_x = pill_rect.min.x + pad + play_btn_w + 8.0 + time_w + 8.0;
        let scrubber_right = pill_rect.max.x - pad - speaker_w - 8.0;
        let scrubber_w = (scrubber_right - scrubber_x).max(0.0);
        let cy = pill_rect.center().y;

        let painter = ui.painter();

        let play_icon = if self.playing { "⏸" } else { "▶" };
        painter.text(
            egui::pos2(pill_rect.min.x + pad + play_btn_w / 2.0, cy),
            egui::Align2::CENTER_CENTER,
            play_icon,
            egui::FontId::proportional(14.0),
            text_color,
        );

        let time_label = format!("{} / {}", fmt_time(elapsed), fmt_time(dur));
        painter.text(
            egui::pos2(pill_rect.min.x + pad + play_btn_w + 8.0, cy),
            egui::Align2::LEFT_CENTER,
            time_label,
            egui::FontId::proportional(11.0),
            text_color,
        );

        let track_h = 3.0;
        let track_rect = egui::Rect::from_center_size(
            egui::pos2(scrubber_x + scrubber_w / 2.0, cy),
            egui::vec2(scrubber_w, track_h),
        );
        painter.rect_filled(track_rect, track_h / 2.0, track_color);
        let fill_w = scrubber_w * pos;
        let fill_rect = egui::Rect::from_min_size(track_rect.min, egui::vec2(fill_w, track_h));
        painter.rect_filled(fill_rect, track_h / 2.0, fill_color);

        let vol_icon = if self.playback_volume == 0.0 {
            "🔇"
        } else if self.playback_volume < 0.4 {
            "🔈"
        } else if self.playback_volume < 0.75 {
            "🔉"
        } else {
            "🔊"
        };
        painter.text(
            egui::pos2(pill_rect.max.x - pad - speaker_w / 2.0, cy),
            egui::Align2::CENTER_CENTER,
            vol_icon,
            egui::FontId::proportional(14.0),
            text_color,
        );

        let play_rect = egui::Rect::from_center_size(
            egui::pos2(pill_rect.min.x + pad + play_btn_w / 2.0, cy),
            egui::vec2(play_btn_w + 8.0, pill_height),
        );
        if ui.put(play_rect, egui::Button::new("").fill(egui::Color32::TRANSPARENT).stroke(egui::Stroke::NONE)).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            if self.playing {
                audio_state.stop_playback();
                self.playing = false;
            } else {
                if audio_state.playback_finished() {
                    audio_state.seek_playback(0.0);
                }
                self.play_clip(audio_state);
            }
        }

        let vol_id = egui::Id::new("player_volume");
        let vol_popup_id = vol_id.with("popup");
        let vol_rect = egui::Rect::from_center_size(
            egui::pos2(pill_rect.max.x - pad - speaker_w / 2.0, cy),
            egui::vec2(speaker_w + 8.0, pill_height),
        );
        let vol_resp = ui.interact(vol_rect, vol_id, egui::Sense::hover())
            .on_hover_cursor(egui::CursorIcon::PointingHand);

        if vol_resp.hovered() {
            ui.memory_mut(|m| m.open_popup(vol_popup_id));
        }

        egui::popup::popup_above_or_below_widget(
            ui,
            vol_popup_id,
            &vol_resp,
            egui::AboveOrBelow::Above,
            egui::popup::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                ui.set_min_width(16.0);
                ui.set_max_width(24.0);
                ui.vertical_centered(|ui| {
                    let slider = egui::Slider::new(&mut self.playback_volume, 0.0..=1.0)
                        .vertical()
                        .show_value(false);
                    if ui.add(slider).changed() {
                        audio_state.set_playback_volume(self.playback_volume);
                    }
                });
            },
        );

        // interact() rather than put() so the response exposes drag events
        let scrubber_sense_rect = egui::Rect::from_center_size(
            track_rect.center(),
            egui::vec2(scrubber_w, pill_height),
        );
        let scrubber_resp = ui.interact(scrubber_sense_rect, egui::Id::new("player_scrubber"), egui::Sense::click_and_drag())
            .on_hover_and_drag_cursor(egui::CursorIcon::PointingHand);
        if scrubber_resp.clicked() || scrubber_resp.dragged() {
            if let Some(ptr) = scrubber_resp.interact_pointer_pos() {
                let t = ((ptr.x - track_rect.min.x) / scrubber_w).clamp(0.0, 1.0);
                audio_state.seek_playback(t);
                if !self.playing {
                    self.play_clip(audio_state);
                }
            }
        }

        ui.add_space(8.0);
        if ui.button("⬇ Download recording").on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            self.save_wav();
        }

        if self.playing {
            ui.ctx().request_repaint();
        }
    }

    fn save_wav(&self) {
        let Some(clip) = &self.clip else { return };
        let wav = audio::encode_wav(&clip.samples, clip.sample_rate);

        let default_name = {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            format!("recording_{}.wav", secs)
        };

        let path = rfd::FileDialog::new()
            .set_title("Save recording")
            .set_file_name(&default_name)
            .add_filter("WAV audio", &["wav"])
            .save_file();

        if let Some(path) = path {
            if let Err(e) = std::fs::write(&path, &wav) {
                eprintln!("Failed to save WAV: {e}");
            }
        }
    }

    fn start_monitor(&mut self, audio_state: &AudioState) {
        self.monitor_error = None;
        let Some(device) = self.selected_device() else { return };
        if let Err(err) = audio::start_mic_monitor(audio_state, device.id.clone()) {
            self.monitor_error = Some(err);
        }
    }

    fn start_recording(&mut self) {
        let Some(device_id) = self.selected_device().map(|d| d.id.clone()) else { return };
        self.clip = None;
        self.clip_duration_secs = 0.0;
        self.monitor_error = None;

        match audio::start_mic_recording(&device_id) {
            Ok((samples, sample_rate, stop_flag, stream)) => {
                self.active_recording = Some(ActiveRecording {
                    samples,
                    sample_rate,
                    stop_flag,
                    started_at: Instant::now(),
                    stream,
                });
            }
            Err(err) => self.monitor_error = Some(err),
        }
    }

    fn stop_recording(&mut self, audio_state: &AudioState) {
        let Some(ActiveRecording { samples, sample_rate, stop_flag, stream, .. }) =
            self.active_recording.take()
        else {
            return;
        };
        stop_flag.store(true, Ordering::Relaxed);
        drop(stream);
        let collected = samples.lock().clone();
        self.clip_duration_secs = collected.len() as f32 / sample_rate as f32;
        self.clip = Some(RecordedClip { samples: collected, sample_rate });
        self.play_clip(audio_state);
    }

    fn play_clip(&mut self, audio_state: &AudioState) {
        let Some(clip) = &self.clip else { return };
        match audio::play_clip(audio_state, clip) {
            Ok(()) => self.playing = true,
            Err(err) => self.monitor_error = Some(err),
        }
    }
}

fn fmt_time(secs: f32) -> String {
    let s = secs as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

