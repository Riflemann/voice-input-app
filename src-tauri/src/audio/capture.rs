// Захват с микрофона
use std::{sync::{Arc, Mutex}, time::Instant};

use cpal::{SampleFormat, SupportedStreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use tauri::State;

use crate::AudioCapture;

pub fn start_audio_capture_with_stream(
    state: State<'_, Mutex<AudioCapture>>,
    device_name: String,
) -> Result<cpal::Stream, String> {
    log::debug!("Starting audio capture on device: {}", device_name);
    let mut capture = state.lock().unwrap();
    if *capture.is_recording.lock().unwrap() {
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

    let buffer = capture.buffer.clone();
    *capture.is_recording.lock().unwrap() = true;
    capture.start_time = Some(Instant::now());

    let is_recording = Arc::clone(&capture.is_recording);

    let stream = match sample_format {
        SampleFormat::F32 => {
            let is_recording = Arc::clone(&is_recording);
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut buf = buffer.lock().unwrap();
                    if !*is_recording.lock().unwrap() {
                        return; // Stop processing if recording is stopped
                    }
                    buf.extend_from_slice(data);
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        SampleFormat::I16 => {
            let is_recording = Arc::clone(&is_recording);
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let mut buf = buffer.lock().unwrap();
                    if !*is_recording.lock().unwrap() {
                        return; // Stop processing if recording is stopped
                    }
                    buf.extend(data.iter().map(|&sample| sample as f32));
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        SampleFormat::U16 => {
            let is_recording = Arc::clone(&is_recording);
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    let mut buf = buffer.lock().unwrap();
                    if !*is_recording.lock().unwrap() {
                        return; // Stop processing if recording is stopped
                    }
                    buf.extend(data.iter().map(|&sample| sample as f32));
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        _ => {
            return Err(format!("Unsupported sample format: {:?}", sample_format));
        }
    };

    stream.play().map_err(|e| format!("Stream play error: {}", e))?;
    Ok(stream)
}

pub async fn stop_and_save_audio(
	state: State<'_, Mutex<AudioCapture>>,
	file_path: String,
) -> Result<(), String> {
	use hound::{WavWriter, WavSpec};

	let mut capture: std::sync::MutexGuard<'_, AudioCapture> = state.lock().unwrap();
	*capture.is_recording.lock().unwrap() = false;
	// Сохраняем буфер
	let buffer: std::sync::MutexGuard<'_, Vec<f32>> = capture.buffer.lock().unwrap();
	let spec: WavSpec = WavSpec {
		channels: capture.channels,
		sample_rate: capture.sample_rate,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int,
	};
	let mut writer = WavWriter::create(file_path, spec)
		.map_err(|e| format!("Failed to create WAV: {}", e))?;
	for &sample in buffer.iter() {
		let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
		writer.write_sample(sample_i16)
			.map_err(|e| format!("Failed to write sample: {}", e))?;
	}
	writer.finalize().map_err(|e| format!("Finalize error: {}", e))?;
	Ok(())
}
