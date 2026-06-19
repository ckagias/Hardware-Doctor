mod audio;
mod keyboard;

use audio::AudioState;
use keyboard::KeyboardState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AudioState::default())
        .manage(KeyboardState::default())
        .invoke_handler(tauri::generate_handler![
            audio::list_input_devices,
            audio::list_output_devices,
            audio::start_mic_monitor,
            audio::stop_mic_monitor,
            audio::record_mic_clip,
            audio::play_test_tone,
            audio::stop_test_tone,
            keyboard::keyboard_layout,
            keyboard::keyboard_snapshot,
            keyboard::key_down,
            keyboard::key_up,
            keyboard::reset_tested,
            keyboard::clear_pressed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
