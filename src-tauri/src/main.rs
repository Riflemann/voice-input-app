// --- Аудиозахват для кроссплатформенного приложения ---
// Используется библиотека cpal для работы с аудиоустройствами
// Поддерживаются Windows, macOS, Linux (alsa/pulse/jack)
mod commands;
mod audio;
mod recognition;
mod types;
mod utils;

use std::sync::{Arc, Mutex};
use dotenv::dotenv;
use crate::types::AudioCapture;
use crate::commands::device::{get_default_input_device_name, get_input_device_names};
use crate::commands::audio::{start_recording, stop_recording, get_recording_status};
use crate::commands::recognition::{recognize_audio, init_whisper};
use crate::recognition::models::ModelSize;
use crate::recognition::whisper;
use crate::utils::cache::AudioCache;
use tokio::sync::mpsc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    // Загружаем переменные окружения из файла .env
    dotenv().ok();
    
    // Инициализируем логирование
    env_logger::init();
    
    let capture = Arc::new(Mutex::new(AudioCapture::default()));
    
    // Создаём кэш для временных WAV файлов
    let cache = Arc::new(AudioCache::new().expect("Failed to create audio cache"));

    // Создаём канал для очереди задач обработки
    let (tx, rx) = mpsc::channel::<Vec<f32>>(4);

    // Собираем и запускаем приложение Tauri
    tauri::Builder::default()
        .manage(capture.clone())
        .manage(cache.clone())
        .manage(tx)
        .invoke_handler(tauri::generate_handler![
            get_default_input_device_name,
            get_input_device_names,
            start_recording,
            stop_recording,
            get_recording_status,
            recognize_audio,
            init_whisper
        ])
        .setup(move |app| {
            // Запускаем воркер обработки в фоне, передаём rx, capture и cache
            let handle = app.handle().clone();
            let capture_for_worker = capture.clone();
            let cache_for_worker = cache.clone();
            tokio::spawn(async move {
                crate::audio::worker::run(rx, capture_for_worker, cache_for_worker, handle).await;
            });

            // Инициализируем Whisper модель на старте приложения (в фоне)
            tokio::spawn(async move {
                let start = Instant::now();
                log::info!("[startup] Initializing Whisper model: base");
                let result = tokio::task::spawn_blocking(|| whisper::init(ModelSize::Base))
                    .await
                    .map_err(|e| format!("Init task failed: {}", e))
                    .and_then(|res| res);

                match result {
                    Ok(()) => {
                        log::info!("[startup] Whisper model initialized in {:?}", start.elapsed());
                    }
                    Err(err) => {
                        log::error!("[startup] Whisper initialization failed: {}", err);
                    }
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
