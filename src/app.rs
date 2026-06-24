#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Module {
    Microphone,
    Headphones,
    Keyboard,
    Mouse,
    Controller,
}

impl Module {
    const ALL: [Module; 5] = [
        Module::Microphone,
        Module::Headphones,
        Module::Keyboard,
        Module::Mouse,
        Module::Controller,
    ];

    fn label(self) -> &'static str {
        match self {
            Module::Microphone => "Microphone",
            Module::Headphones => "Headphones / Speakers",
            Module::Keyboard => "Keyboard",
            Module::Mouse => "Mouse",
            Module::Controller => "Controller",
        }
    }
}

pub struct TroubleApp {
    active: Module,
    audio: crate::audio::AudioState,
    keyboard: crate::keyboard::KeyboardState,
    mouse: crate::mouse::MouseState,
    controller: crate::controller::ControllerState,
    microphone_panel: crate::panels::microphone::MicrophonePanel,
    headphones_panel: crate::panels::headphones::HeadphonesPanel,
    keyboard_panel: crate::panels::keyboard::KeyboardPanel,
    mouse_panel: crate::panels::mouse::MousePanel,
    controller_panel: crate::panels::controller::ControllerPanel,
}

impl TroubleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_zoom_factor(1.15);
        Self {
            active: Module::Microphone,
            audio: crate::audio::AudioState::default(),
            keyboard: crate::keyboard::KeyboardState::default(),
            mouse: crate::mouse::MouseState::default(),
            controller: crate::controller::ControllerState::default(),
            microphone_panel: crate::panels::microphone::MicrophonePanel::default(),
            headphones_panel: crate::panels::headphones::HeadphonesPanel::default(),
            keyboard_panel: crate::panels::keyboard::KeyboardPanel::default(),
            mouse_panel: crate::panels::mouse::MousePanel::default(),
            controller_panel: crate::panels::controller::ControllerPanel::default(),
        }
    }
}

impl eframe::App for TroubleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Keeps the mic level meter / recording-done polling live every frame, not just on input events.
        ctx.request_repaint_after(std::time::Duration::from_millis(33));

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            for module in Module::ALL {
                let selected = self.active == module;
                if ui.selectable_label(selected, module.label()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() && !selected {
                    // Leaving the mic tab stops the live monitor stream instead of letting it run in the background.
                    if self.active == Module::Microphone {
                        let _ = crate::audio::stop_mic_monitor(&self.audio);
                    }
                    self.active = module;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.active {
            Module::Microphone => self.microphone_panel.show(ui, &self.audio),
            Module::Headphones => self.headphones_panel.show(ui, &self.audio),
            Module::Keyboard => self.keyboard_panel.show(ui, &self.keyboard),
            Module::Mouse => self.mouse_panel.show(ui, &self.mouse),
            Module::Controller => self.controller_panel.show(ui, &self.controller),
        });
    }
}
