// --- Аудиозахват для кроссплатформенного приложения ---
// Используется библиотека cpal для работы с аудиоустройствами
// Поддерживаются Windows, macOS, Linux (alsa/pulse/jack)
mod commands;
mod audio;

use std::{
	sync::{Arc, Mutex},
	time::{Instant},
};
use dotenv::dotenv;
use crate::commands::device::{get_default_input_device_name, get_input_device_names};
use crate::commands::audio::{start_recording, stop_recording};
use crate::commands::recognition::recognize_audio;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub struct AudioCapture {
    pub is_recording: Arc<Mutex<bool>>,       // Флаг записи
    pub buffer: Arc<Mutex<Vec<f32>>>,        // Буфер для сэмплов
    pub sample_rate: u32,                    // Частота дискретизации
    pub channels: u16,                       // Количество каналов
    pub start_time: Option<Instant>,         // Время старта записи
    pub volume_level: f32,                   // Последний уровень громкости
    // Перенесённые свойства процессора
    pub gain: f32,
    pub noise_threshold: f32,
    pub buffer_duration_seconds: usize,
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
            gain: 1.0,
            noise_threshold: 0.02,
            buffer_duration_seconds: 10,
        }
    }
}


#[tokio::main]
async fn main() {
    // Загружаем переменные окружения из файла .env
    dotenv().ok();
    
    // Инициализируем логирование
    env_logger::init();
    
    let capture = std::sync::Arc::new(Mutex::new(AudioCapture::default()));

    // Создаём канал для очереди задач обработки
    let (tx, rx) = mpsc::channel::<Vec<f32>>(4);

    // Собираем и запускаем приложение Tauri
    tauri::Builder::default()
        .manage(capture.clone())
        .manage(tx)
        .invoke_handler(tauri::generate_handler![
            get_default_input_device_name,
            get_input_device_names,
            start_recording,
            stop_recording,
            recognize_audio
        ])
        .setup(move |app| {
            // Запускаем воркер обработки в фоне, передаём rx и клон capture
            let handle = app.handle().clone();
            let capture_for_worker = capture.clone();
            tokio::spawn(async move {
                crate::audio::worker::run(rx, capture_for_worker, handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Error while running Tauri application");
}
