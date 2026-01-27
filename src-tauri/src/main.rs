// --- Аудиозахват для кроссплатформенного приложения ---
// Используется библиотека cpal для работы с аудиоустройствами
// Поддерживаются Windows, macOS, Linux (alsa/pulse/jack)
mod commands;
mod audio;

use std::{
	sync::{Arc, Mutex},
	time::{Instant},
};
use tauri::{Emitter, State};
use dotenv::dotenv;
use crate::commands::device::{get_default_input_device_name, get_input_device_names};
use config::Config; 

#[allow(dead_code)]
struct AudioCapture {
	is_recording: Arc<Mutex<bool>>,       // Флаг записи
	buffer: Arc<Mutex<Vec<f32>>>,        // Буфер для сэмплов
	sample_rate: u32,                    // Частота дискретизации
	channels: u16,                       // Количество каналов
	start_time: Option<Instant>,         // Время старта записи
	volume_level: f32,                   // Последний уровень громкости
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 44100,
            channels: 1,
            start_time: None,
            volume_level: 1.0,
        }
    }
}


#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();

    // Initialize the audio capture state
    let capture = Mutex::new(AudioCapture::default());

    // Retrieve the default input device name
    let device = match get_default_input_device_name().await {
        Ok(device) => device.name,
        Err(err) => {
            log::error!("Failed to get default input device: {}", err);
            return;
        }
    };

    // Start audio capture with the selected device
    if let Err(err) = start_audio_capture_with_stream(capture.clone(), device).await {
        log::error!("Failed to start audio capture: {}", err);
        return;
    }

    // Build and run the Tauri application
    tauri::Builder::default()
        .manage(capture) // Share the audio capture state with the Tauri app
        .invoke_handler(tauri::generate_handler![
            get_input_device_names
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
