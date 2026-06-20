use crate::audio::{self, AudioState, DeviceInfo};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Channel {
    Left,
    Right,
    Both,
}

impl Channel {
    fn as_str(self) -> &'static str {
        match self {
            Channel::Left => "left",
            Channel::Right => "right",
            Channel::Both => "both",
        }
    }
}

pub struct HeadphonesPanel {
    devices: Vec<DeviceInfo>,
    selected: Option<usize>,
    device_error: Option<String>,
    play_error: Option<String>,
    playing: Option<Channel>,
}

impl Default for HeadphonesPanel {
    fn default() -> Self {
        let mut panel = Self {
            devices: Vec::new(),
            selected: None,
            device_error: None,
            play_error: None,
            playing: None,
        };
        panel.refresh_devices();
        panel
    }
}

impl HeadphonesPanel {
    fn refresh_devices(&mut self) {
        match audio::list_output_devices() {
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
        ui.heading("Headphones / Speakers Test");
        ui.label(
            "Pick your output device, then test left, right, and both channels to confirm \
             your audio is wired and working correctly.",
        );
        ui.add_space(12.0);

        ui.horizontal(|ui| {
            ui.label("Output device:");
            let selected_label = self
                .selected_device()
                .map(|d| d.label.clone())
                .unwrap_or_else(|| "No outputs found".to_string());

            egui::ComboBox::from_id_salt("output-select")
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for (i, device) in self.devices.iter().enumerate() {
                        let selected = self.selected == Some(i);
                        if ui.selectable_label(selected, &device.label).clicked() {
                            self.selected = Some(i);
                        }
                    }
                });

            if ui.button("Refresh").clicked() {
                self.refresh_devices();
            }
        });

        if let Some(err) = self.device_error.clone().or(self.play_error.clone()) {
            ui.add_space(8.0);
            ui.colored_label(egui::Color32::from_rgb(229, 72, 77), err);
        }

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.playing == Some(Channel::Left), "Test Left")
                .clicked()
            {
                self.play_channel(audio_state, Channel::Left);
            }
            if ui
                .selectable_label(self.playing == Some(Channel::Both), "Test Both")
                .clicked()
            {
                self.play_channel(audio_state, Channel::Both);
            }
            if ui
                .selectable_label(self.playing == Some(Channel::Right), "Test Right")
                .clicked()
            {
                self.play_channel(audio_state, Channel::Right);
            }
        });

        ui.add_space(12.0);

        if ui
            .add_enabled(self.playing.is_some(), egui::Button::new("Stop"))
            .clicked()
        {
            self.stop(audio_state);
        }
    }

    fn play_channel(&mut self, audio_state: &AudioState, channel: Channel) {
        self.play_error = None;
        let device_id = self
            .selected_device()
            .map(|d| d.id.clone())
            .unwrap_or_default();
        match audio::play_test_tone(audio_state, device_id, channel.as_str().to_string()) {
            Ok(()) => self.playing = Some(channel),
            Err(err) => self.play_error = Some(err),
        }
    }

    fn stop(&mut self, audio_state: &AudioState) {
        if let Err(err) = audio::stop_test_tone(audio_state) {
            self.play_error = Some(err);
        }
        self.playing = None;
    }
}
