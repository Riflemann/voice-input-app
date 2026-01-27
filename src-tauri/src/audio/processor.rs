// Обработка аудио
use std::sync::{Arc, RwLock};
use lazy_static::lazy_static;
use tauri::{Emitter, Window};
use config::Config;

const DEFAULT_GAIN: f32 = 1.0;

struct Properties {
    gain: f32, 
    noise_threshold: f32, 
    sample_rate: usize, 
    buffer_duration_seconds: usize,
}

impl Properties {
    pub fn from_env() -> Self {
        let config = Config::builder()
            .add_source(config::File::with_name("config/audio"))
            .add_source(config::Environment::with_prefix("AUDIO"))
            .build()
            .unwrap();

        Self {
            gain: config.get::<f32>("GAIN").unwrap_or(1.0),
            noise_threshold: config.get::<f32>("NOISE_THRESHOLD").unwrap_or(0.02),
            sample_rate: config.get::<usize>("SAMPLE_RATE").unwrap_or(44100),
            buffer_duration_seconds: config.get::<usize>("BUFFER_DURATION_SECONDS").unwrap_or(10),
        }
    }
}

lazy_static! {
    static ref PROPERTIES: Properties = Properties::from_env();
}

/// Processes audio data by filtering noise and amplifying the signal.
/// 
/// # Arguments
/// * `input_data` - A slice of audio samples to process.
/// * `buffer` - A shared buffer to store processed audio data.
/// * `window` - The Tauri window for emitting events.
///
/// This function applies noise filtering and amplification to the input audio data,
/// then appends the processed data to the shared buffer. It also emits a start-animation
/// event to the frontend.
/// Logs warnings or errors if operations fail.
pub fn process_audio(
    input_data: &[f32],
    buffer: &Arc<RwLock<Vec<f32>>>,
) {
    if let Err(e) = window.emit("start-animation", "Processing started") {
        log::warn!("Failed to send start-animation signal: {}", e);
    }

    let processed = process_and_filter(input_data);

    match buffer.write() {
        Ok(mut buf) => {
            buf.extend_from_slice(&processed);
            manage_buffer(&mut buf);
        }
        Err(e) => {
            log::error!("Failed to acquire write lock on buffer: {}", e);
        }
    }
}

/// Filters and amplifies audio samples.
/// 
/// # Arguments
/// * `input_data` - A slice of audio samples to process.
/// 
/// # Returns
/// A vector of processed audio samples.
fn process_and_filter(input_data: &[f32]) -> Vec<f32> {
    input_data
        .iter()
        .map(|&sample| if sample.abs() < PROPERTIES.noise_threshold { 0.0 } else { sample })
        .map(|sample| {
            let amplified = sample * PROPERTIES.gain;
            amplified.clamp(-1.0, 1.0)
        })
        .collect()
}

/// Manages the size of the audio buffer and calculates RMS periodically.
/// 
/// # Arguments
/// * `buffer` - A mutable reference to the audio buffer.
///
/// Ensures the buffer does not exceed the maximum size and calculates the
/// root mean square (RMS) of the audio data at regular intervals.
fn manage_buffer(buffer: &mut Vec<f32>) {
    let max_samples = PROPERTIES.sample_rate * PROPERTIES.buffer_duration_seconds;

    if buffer.len() > max_samples {
        buffer.drain(0..buffer.len() - max_samples);
    }

    let should_calculate_rms = buffer.len() % 1024 == 0;
    if should_calculate_rms {
        let _rms = calculate_rms(buffer);
        log::debug!("Current RMS: {}", _rms);
    }
}

/// Calculates the root mean square (RMS) of audio data.
/// 
/// # Arguments
/// * `data` - A slice of audio samples.
/// 
/// # Returns
/// The RMS value of the audio data.
fn calculate_rms(data: &[f32]) -> f32 {
    if data.is_empty() { return 0.0; }
    let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
    (sum_squares / data.len() as f32).sqrt()
}