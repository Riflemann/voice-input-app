// --- Аудиозахват для кроссплатформенного приложения ---
// Используется библиотека cpal для работы с аудиоустройствами
// Поддерживаются Windows, macOS, Linux (alsa/pulse/jack)
mod commands;
mod audio;
use std::{
	sync::{Arc, Mutex},
	time::{Instant},
};
use tauri::{State, Emitter};
use dotenv::dotenv;
use crate::audio::capture::start_audio_capture;
use crate::commands::device::{get_input_device_name, get_input_device_names};

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
			get_input_device_names,
			stop_and_save_audio
		])
		.run(tauri::generate_context!())
		.expect("error while running tauri application");

}
