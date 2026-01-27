// Захват с микрофона

use std::{sync::{Arc, Mutex}, time::Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SupportedStreamConfig};

use tauri::{State};

use crate::AudioCapture;

// Define constants for normalization factors
const I16_NORMALIZATION_FACTOR: f32 = 32768.0;
const U16_NORMALIZATION_FACTOR: f32 = 65535.0;
const U16_OFFSET: f32 = 2.0;
const U16_SHIFT: f32 = 1.0;

#[tauri::command]
pub async fn start_audio_capture(
	state: State<'_, Mutex<AudioCapture>>,
	device_name: String,
	window: tauri::Window,
) -> Result<(), String> {
	log::debug!("Starting audio capture on device: {}", device_name);
	let mut capture = state.lock().unwrap();
	if capture.is_recording {
		return Err("Already recording".to_string());
	}

	let host = cpal::default_host();
	let device = host.input_devices()
		.map_err(|e| format!("Device error: {}", e))?
		.find(|d| d.name().unwrap_or_default() == device_name)
		.ok_or_else(|| "Device not found".to_string())?;

	let config: SupportedStreamConfig = device.default_input_config()
		.map_err(|e| format!("Config error: {}", e))?;
	let sample_format = config.sample_format();
	let sample_rate = config.sample_rate().0;
	let channels = config.channels();

	capture.sample_rate = sample_rate;
	capture.channels = channels;

	let buffer = Arc::clone(&capture.buffer);
	let window_clone = window.clone();

	let stream = match sample_format {
		SampleFormat::F32 => device.build_input_stream(
			&config.into(),
			move |data: &[f32], _| {
				process_audio_data(data, &buffer, &window_clone);
			},
			move |err| {
				log::error!("Stream error: {}", err);
			},
			None,
		),
		SampleFormat::I16 => device.build_input_stream(
			&config.into(),
			move |data: &[i16], _| {
				let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / I16_NORMALIZATION_FACTOR).collect();
				process_audio_data(&data_f32, &buffer, &window_clone);
			},
			move |err| {
				log::error!("Stream error: {}", err);
			},
			None,
		),
		SampleFormat::U16 => device.build_input_stream(
			&config.into(),
			move |data: &[u16], _| {
				let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / U16_NORMALIZATION_FACTOR * U16_OFFSET - U16_SHIFT).collect();
				process_audio_data(&data_f32, &buffer, &window_clone);
			},
			move |err| {
				log::error!("Stream error: {}", err);
			},
			None,
		),
		_ => {
			return Err(format!("Unsupported sample format: {:?}", sample_format));
		}
	};

	let stream = stream.map_err(|e| format!("Stream creation failed: {}", e))?;
	stream.play().map_err(|e| format!("Stream play error: {}", e))?;

	capture.is_recording = true;
	capture.start_time = Some(Instant::now());
	Ok(())
}

#[tauri::command]
pub async fn start_audio_capture_with_stream(
    state: State<'_, Mutex<AudioCapture>>,
    device_name: String,
    _window: tauri::Window,
) -> Result<cpal::Stream, String> {
    log::debug!("Starting audio capture on device: {}", device_name);
    let mut capture = state.lock().unwrap();
    if capture.is_recording {
        return Err("Already recording".to_string());
    }

    let host = cpal::default_host();
    let device = host.input_devices()
        .map_err(|e| format!("Device error: {}", e))?
        .find(|d| d.name().unwrap_or_default() == device_name)
        .ok_or_else(|| "Device not found".to_string())?;

    let config: SupportedStreamConfig = device.default_input_config()
        .map_err(|e| format!("Config error: {}", e))?
        .into();
    let sample_format = config.sample_format();

    let stream = match sample_format {
        SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |_data: &[f32], _| {},
            move |err| {
                log::error!("Stream error: {}", err);
            },
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |_data: &[i16], _| {},
            move |err| {
                log::error!("Stream error: {}", err);
            },
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config.into(),
            move |_data: &[u16], _| {},
            move |err| {
                log::error!("Stream error: {}", err);
            },
            None,
        ),
        _ => {
            return Err(format!("Unsupported sample format: {:?}", sample_format));
        }
    };

    let stream = stream.map_err(|e| format!("Stream creation failed: {}", e))?;
    stream.play().map_err(|e| format!("Stream play error: {}", e))?;

    capture.is_recording = true;
    capture.start_time = Some(Instant::now());
    Ok(stream)
}