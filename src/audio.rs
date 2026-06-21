use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

#[derive(Clone)]
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
    playback_stream: Mutex<Option<SendStream>>,
    mic_running: Arc<AtomicBool>,
    // mic level (0-100), stored as u32 bits via to_bits, read each egui frame
    mic_level_bits: Arc<AtomicU32>,
    playback_done: Arc<AtomicBool>,
}

impl AudioState {
    pub fn mic_level(&self) -> f32 {
        f32::from_bits(self.mic_level_bits.load(Ordering::Relaxed))
    }

    // True once a started playback has finished, so the egui panel can reset its button state
    pub fn playback_finished(&self) -> bool {
        self.playback_done.load(Ordering::Relaxed)
    }
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

pub fn start_mic_monitor(state: &AudioState, device_id: String) -> Result<(), String> {
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
    let mic_level_bits = state.mic_level_bits.clone();

    macro_rules! build_stream {
        ($sample_ty:ty) => {{
            let running = running.clone();
            let mic_level_bits = mic_level_bits.clone();
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
                    mic_level_bits.store(level.to_bits(), Ordering::Relaxed);
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

pub fn stop_mic_monitor(state: &AudioState) -> Result<(), String> {
    state.mic_running.store(false, Ordering::SeqCst);
    *state.mic_stream.lock() = None;
    Ok(())
}

pub struct RecordedClip {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

pub fn record_mic_clip(device_id: &str, duration_ms: u64) -> Result<RecordedClip, String> {
    let device = find_input_device(device_id)?;
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

    let collected = samples.lock().clone();
    Ok(RecordedClip {
        samples: collected,
        sample_rate,
    })
}

// Plays back a recorded clip (mono i16 @ clip_rate), resampling via linear interpolation if rates differ.
pub fn play_clip(state: &AudioState, clip: &RecordedClip) -> Result<(), String> {
    *state.playback_stream.lock() = None;
    state.playback_done.store(false, Ordering::Relaxed);

    let device = host()
        .default_output_device()
        .ok_or_else(|| "No default output device".to_string())?;
    let config = device.default_output_config().map_err(|e| e.to_string())?;
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.into();
    let channels = stream_config.channels as usize;
    let out_rate = stream_config.sample_rate.0 as f32;

    let in_rate = clip.sample_rate as f32;
    let src = clip.samples.clone();
    let mut pos = 0f32;
    let step = in_rate / out_rate;

    let done = Arc::new(AtomicBool::new(false));
    let playback_done = state.playback_done.clone();
    let err_fn = |err| eprintln!("audio stream error: {err}");

    macro_rules! build_playback_stream {
        ($sample_ty:ty) => {{
            let done = done.clone();
            let playback_done = playback_done.clone();
            device.build_output_stream(
                &stream_config,
                move |data: &mut [$sample_ty], _| {
                    for frame in data.chunks_mut(channels) {
                        let value = if (pos as usize) < src.len() {
                            let s = src[pos as usize];
                            (s as f32 / i16::MAX as f32).clamp(-1.0, 1.0)
                        } else {
                            if !done.swap(true, Ordering::Relaxed) {
                                playback_done.store(true, Ordering::Relaxed);
                            }
                            0.0
                        };
                        pos += step;
                        for sample in frame.iter_mut() {
                            *sample = Sample::from_sample(value);
                        }
                    }
                },
                err_fn,
                None,
            )
        }};
    }

    let stream = match sample_format {
        SampleFormat::F32 => build_playback_stream!(f32),
        SampleFormat::I16 => build_playback_stream!(i16),
        SampleFormat::U16 => build_playback_stream!(u16),
        _ => return Err("Unsupported sample format".to_string()),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    *state.playback_stream.lock() = Some(SendStream(stream));
    Ok(())
}

pub fn play_test_tone(
    state: &AudioState,
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

pub fn stop_test_tone(state: &AudioState) -> Result<(), String> {
    *state.tone_stream.lock() = None;
    Ok(())
}
