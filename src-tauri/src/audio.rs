use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use parking_lot::Mutex;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub label: String,
}

// Wraps cpal::Stream so it can live in managed state; only used to keep the stream alive until dropped
#[allow(dead_code)]
struct SendStream(cpal::Stream);
unsafe impl Send for SendStream {}
unsafe impl Sync for SendStream {}

#[derive(Default)]
pub struct AudioState {
    mic_stream: Mutex<Option<SendStream>>,
    tone_stream: Mutex<Option<SendStream>>,
    mic_running: Arc<AtomicBool>,
}

fn host() -> cpal::Host {
    cpal::default_host()
}

fn find_input_device(id: &str) -> Result<cpal::Device, String> {
    let host = host();
    if id.is_empty() {
        return host
            .default_input_device()
            .ok_or_else(|| "No default input device".to_string());
    }
    host.input_devices()
        .map_err(|e| e.to_string())?
        .find(|d| d.name().map(|n| n == id).unwrap_or(false))
        .ok_or_else(|| format!("Input device not found: {id}"))
}

fn find_output_device(id: &str) -> Result<cpal::Device, String> {
    let host = host();
    if id.is_empty() {
        return host
            .default_output_device()
            .ok_or_else(|| "No default output device".to_string());
    }
    host.output_devices()
        .map_err(|e| e.to_string())?
        .find(|d| d.name().map(|n| n == id).unwrap_or(false))
        .ok_or_else(|| format!("Output device not found: {id}"))
}

#[tauri::command]
pub fn list_input_devices() -> Result<Vec<DeviceInfo>, String> {
    let host = host();
    let devices = host.input_devices().map_err(|e| e.to_string())?;
    Ok(devices
        .filter_map(|d| d.name().ok())
        .map(|name| DeviceInfo {
            id: name.clone(),
            label: name,
        })
        .collect())
}

#[tauri::command]
pub fn list_output_devices() -> Result<Vec<DeviceInfo>, String> {
    let host = host();
    let devices = host.output_devices().map_err(|e| e.to_string())?;
    Ok(devices
        .filter_map(|d| d.name().ok())
        .map(|name| DeviceInfo {
            id: name.clone(),
            label: name,
        })
        .collect())
}

#[derive(Serialize, Clone)]
struct MicLevelPayload {
    level: f32,
}

#[tauri::command]
pub fn start_mic_monitor(
    app: AppHandle,
    state: State<AudioState>,
    device_id: String,
) -> Result<(), String> {
    state.mic_running.store(false, Ordering::SeqCst);
    *state.mic_stream.lock() = None;

    let device = find_input_device(&device_id)?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;

    let running = state.mic_running.clone();
    running.store(true, Ordering::SeqCst);

    let err_fn = |err| eprintln!("audio stream error: {err}");
    let last_emit = Arc::new(Mutex::new(std::time::Instant::now()));
    const EMIT_INTERVAL: std::time::Duration = std::time::Duration::from_millis(33);

    macro_rules! build_stream {
        ($sample_ty:ty) => {{
            let app = app.clone();
            let running = running.clone();
            let last_emit = last_emit.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[$sample_ty], _| {
                    if !running.load(Ordering::SeqCst) {
                        return;
                    }
                    let mut sum_squares = 0f32;
                    let mut count = 0usize;
                    for frame in data.chunks(channels) {
                        if let Some(s) = frame.first() {
                            let v: f32 = s.to_sample();
                            sum_squares += v * v;
                            count += 1;
                        }
                    }
                    if count == 0 {
                        return;
                    }
                    let rms = (sum_squares / count as f32).sqrt();
                    let level = (rms * 180.0).min(100.0);

                    let mut last = last_emit.lock();
                    if last.elapsed() >= EMIT_INTERVAL {
                        *last = std::time::Instant::now();
                        let _ = app.emit("mic-level", MicLevelPayload { level });
                    }
                },
                err_fn,
                None,
            )
        }};
    }

    let stream = match sample_format {
        SampleFormat::F32 => build_stream!(f32),
        SampleFormat::I16 => build_stream!(i16),
        SampleFormat::U16 => build_stream!(u16),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    *state.mic_stream.lock() = Some(SendStream(stream));
    Ok(())
}

#[tauri::command]
pub fn stop_mic_monitor(state: State<AudioState>) -> Result<(), String> {
    state.mic_running.store(false, Ordering::SeqCst);
    *state.mic_stream.lock() = None;
    Ok(())
}

#[tauri::command]
pub fn record_mic_clip(device_id: String, duration_ms: u64) -> Result<String, String> {
    let device = find_input_device(&device_id)?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate.0;

    let samples: Arc<Mutex<Vec<i16>>> = Arc::new(Mutex::new(Vec::new()));
    let err_fn = |err| eprintln!("audio stream error: {err}");

    macro_rules! build_record_stream {
        ($sample_ty:ty) => {{
            let samples = samples.clone();
            device.build_input_stream(
                &stream_config,
                move |data: &[$sample_ty], _| {
                    let mut buf = samples.lock();
                    for frame in data.chunks(channels) {
                        if let Some(s) = frame.first() {
                            let v: i16 = s.to_sample();
                            buf.push(v);
                        }
                    }
                },
                err_fn,
                None,
            )
        }};
    }

    let stream = match sample_format {
        SampleFormat::F32 => build_record_stream!(f32),
        SampleFormat::I16 => build_record_stream!(i16),
        SampleFormat::U16 => build_record_stream!(u16),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(duration_ms));
    drop(stream);

    let collected = samples.lock();
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(&mut cursor, spec).map_err(|e| e.to_string())?;
        for &s in collected.iter() {
            writer.write_sample(s).map_err(|e| e.to_string())?;
        }
        writer.finalize().map_err(|e| e.to_string())?;
    }

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        cursor.into_inner(),
    ))
}

#[tauri::command]
pub fn play_test_tone(
    state: State<AudioState>,
    device_id: String,
    channel: String,
) -> Result<(), String> {
    *state.tone_stream.lock() = None;

    let device = find_output_device(&device_id)?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;
    let sample_rate = stream_config.sample_rate.0 as f32;

    let (left_on, right_on) = match channel.as_str() {
        "left" => (true, false),
        "right" => (false, true),
        _ => (true, true),
    };

    let mut phase = 0f32;
    let freq = 440f32;
    let err_fn = |err| eprintln!("audio stream error: {err}");

    macro_rules! build_tone_stream {
        ($sample_ty:ty) => {{
            device.build_output_stream(
                &stream_config,
                move |data: &mut [$sample_ty], _| {
                    for frame in data.chunks_mut(channels) {
                        let value = (phase * 2.0 * std::f32::consts::PI).sin() * 0.25;
                        phase = (phase + freq / sample_rate) % 1.0;
                        for (i, sample) in frame.iter_mut().enumerate() {
                            let on = if channels == 1 {
                                left_on || right_on
                            } else if i % 2 == 0 {
                                left_on
                            } else {
                                right_on
                            };
                            *sample = Sample::from_sample(if on { value } else { 0.0 });
                        }
                    }
                },
                err_fn,
                None,
            )
        }};
    }

    let stream = match sample_format {
        SampleFormat::F32 => build_tone_stream!(f32),
        SampleFormat::I16 => build_tone_stream!(i16),
        SampleFormat::U16 => build_tone_stream!(u16),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    *state.tone_stream.lock() = Some(SendStream(stream));
    Ok(())
}

#[tauri::command]
pub fn stop_test_tone(state: State<AudioState>) -> Result<(), String> {
    *state.tone_stream.lock() = None;
    Ok(())
}
