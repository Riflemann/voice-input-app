// --- Аудиозахват для кроссплатформенного приложения ---
// Используется библиотека cpal для работы с аудиоустройствами
// Поддерживаются Windows, macOS, Linux (alsa/pulse/jack)
mod commands;
use cpal::{
	SampleFormat, SupportedStreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}
};
use std::{
	sync::{Arc, Mutex},
	time::{Instant},
};
use tauri::{State, Emitter};
use dotenv::dotenv;
use crate::commands::audio::get_input_device_name;

struct AudioCapture {
	is_recording: bool,                  // Флаг записи
	buffer: Arc<Mutex<Vec<f32>>>,        // Буфер для сэмплов
	sample_rate: u32,                    // Частота дискретизации
	channels: u16,                       // Количество каналов
	start_time: Option<Instant>,         // Время старта записи
	volume_level: f32,                   // Последний уровень громкости
}

impl Default for AudioCapture {
	fn default() -> Self {
		Self {
			is_recording: false,
			buffer: Arc::new(Mutex::new(Vec::new())),
			sample_rate: 44100,
			channels: 1,
			start_time: None,
			volume_level: 1.0,
		}
	}
}

#[tauri::command]
async fn start_audio_capture(
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
				let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 32768.0).collect();
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
				let data_f32: Vec<f32> = data.iter().map(|&x| x as f32 / 65535.0 * 2.0 - 1.0).collect();
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

fn process_audio_data(data: &[f32], buffer: &Arc<Mutex<Vec<f32>>>, window: &tauri::Window) {
	let mut buffer_guard = buffer.lock().unwrap();
	for &sample in data {
		buffer_guard.push(sample);
	}
	let max_samples: usize = 44100 * 10;
	let buffer_len = buffer_guard.len();
	if buffer_len > max_samples {
		buffer_guard.drain(0..buffer_len - max_samples);
	}

	// Limit the scope of the mutable borrow
	let should_calculate_rms = buffer_guard.len() % 1024 == 0;
	let buffer_copy = if should_calculate_rms {
		Some(buffer_guard.clone()) // Clone the data for RMS calculation
	} else {
		None
	};
	drop(buffer_guard); // Explicitly drop the mutable borrow

	if let Some(buffer_copy) = buffer_copy {
		let rms = calculate_rms(&buffer_copy);
		let _ = window.emit("audio-level", rms);
	}
}

fn calculate_rms(data: &[f32]) -> f32 {
	if data.is_empty() { return 0.0; }
	let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
	(sum_squares / data.len() as f32).sqrt()
}

#[tauri::command]
async fn stop_and_save_audio(
	state: State<'_, Mutex<AudioCapture>>,
	file_path: String,
) -> Result<(), String> {
	use hound::{WavWriter, WavSpec};

	let mut capture = state.lock().unwrap();
	capture.is_recording = false;
	// Сохраняем буфер
	let buffer = capture.buffer.lock().unwrap();
	let spec = WavSpec {
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

#[tokio::main]
async fn main() {
	dotenv().ok();

	env_logger::init();
	log::info!("Logger initialized");
	// При старте выводим доступные устройства
	match get_input_device_name().await {
		Ok(device) => {
			log::info!("Available input devices:");
			log::info!(" - {}", device.name);
		}
		Err(err) => {
			log::error!("Error retrieving devices: {}", err);
		}
	}

	tauri::Builder::default()
		.manage(Mutex::new(AudioCapture::default()))
		.invoke_handler(tauri::generate_handler![
			get_input_device_name,
			start_audio_capture,
			stop_and_save_audio
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

}
