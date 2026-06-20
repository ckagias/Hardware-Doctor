use crate::audio::{self, AudioState, DeviceInfo, RecordedClip};
use std::sync::mpsc::{Receiver, Sender, channel};

enum RecordMsg {
    Done(Result<RecordedClip, String>),
}

pub struct MicrophonePanel {
    devices: Vec<DeviceInfo>,
    selected: Option<usize>,
    device_error: Option<String>,
    monitor_error: Option<String>,
    peak_level: f32,
    recording: bool,
    record_rx: Option<Receiver<RecordMsg>>,
    clip: Option<RecordedClip>,
    playing: bool,
}

impl Default for MicrophonePanel {
    fn default() -> Self {
        let mut panel = Self {
            devices: Vec::new(),
            selected: None,
            device_error: None,
            monitor_error: None,
            peak_level: 0.0,
            recording: false,
            record_rx: None,
            clip: None,
            playing: false,
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

            if ui.button("Refresh").clicked() {
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

        let color = if level > 85.0 {
            egui::Color32::from_rgb(229, 72, 77)
        } else if level > 60.0 {
            egui::Color32::from_rgb(245, 184, 0)
        } else {
            egui::Color32::from_rgb(48, 164, 108)
        };

        let desired_size = egui::vec2(ui.available_width(), 16.0);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
        ui.painter()
            .rect_filled(rect, 4.0, egui::Color32::from_gray(40));
        let fill_width = rect.width() * (level / 100.0).clamp(0.0, 1.0);
        let fill_rect =
            egui::Rect::from_min_size(rect.min, egui::vec2(fill_width, rect.height()));
        ui.painter().rect_filled(fill_rect, 4.0, color);
        let peak_x = rect.min.x + rect.width() * (self.peak_level / 100.0).clamp(0.0, 1.0);
        ui.painter().vline(
            peak_x,
            rect.min.y..=rect.max.y,
            egui::Stroke::new(2.0, egui::Color32::WHITE),
        );

        ui.add_space(20.0);

        if let Some(rx) = &self.record_rx {
            if let Ok(RecordMsg::Done(result)) = rx.try_recv() {
                self.recording = false;
                self.record_rx = None;
                match result {
                    Ok(clip) => {
                        self.clip = Some(clip);
                        self.play_clip(audio_state);
                    }
                    Err(err) => self.monitor_error = Some(err),
                }
            }
        }

        ui.horizontal(|ui| {
            let label = if self.recording {
                "Recording... (4s)"
            } else {
                "Record & Play Back"
            };
            let enabled = self.selected_device().is_some() && !self.recording;
            if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
                self.start_recording();
            }

            if self.playing {
                ui.label("Playing back...");
                if audio_state.playback_finished() {
                    self.playing = false;
                }
            }
        });
    }

    fn start_monitor(&mut self, audio_state: &AudioState) {
        self.monitor_error = None;
        let Some(device) = self.selected_device() else {
            return;
        };
        if let Err(err) = audio::start_mic_monitor(audio_state, device.id.clone()) {
            self.monitor_error = Some(err);
        }
    }

    fn start_recording(&mut self) {
        let Some(device) = self.selected_device() else {
            return;
        };
        let device_id = device.id.clone();
        self.recording = true;
        self.clip = None;

        let (tx, rx): (Sender<RecordMsg>, Receiver<RecordMsg>) = channel();
        self.record_rx = Some(rx);

        std::thread::spawn(move || {
            let result = audio::record_mic_clip(&device_id, 4000);
            let _ = tx.send(RecordMsg::Done(result));
        });
    }

    fn play_clip(&mut self, audio_state: &AudioState) {
        let Some(clip) = &self.clip else { return };
        match audio::play_clip(audio_state, clip) {
            Ok(()) => self.playing = true,
            Err(err) => self.monitor_error = Some(err),
        }
    }
}
