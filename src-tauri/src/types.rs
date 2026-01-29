use std::{sync::{Arc, Mutex}, time::Instant};

/// Структура для управления аудиозахватом
#[allow(dead_code)]
pub struct AudioCapture {
    pub is_recording: Arc<Mutex<bool>>,       // Флаг записи
    pub buffer: Arc<Mutex<Vec<f32>>>,        // Буфер для сэмплов
    pub sample_rate: u32,                    // Частота дискретизации
    pub channels: u16,                       // Количество каналов
    pub start_time: Option<Instant>,         // Время старта записи
    pub volume_level: f32,                   // Последний уровень громкости
    // Перенесённые свойства процессора
    pub gain: f32,              // Set to 1.0 to disable gain
    pub noise_threshold: f32,  // Set to 0.0 to disable noise gate
    pub rms_input: f32,                      // RMS входного сигнала
    pub rms_output: f32,                     // RMS выходного сигнала после обработки
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
            gain: 1.0,  // Базовое значение (переопределится адаптивной подстройкой)
            noise_threshold: 0.0,  // Базовое значение (переопределится адаптивной подстройкой)
            buffer_duration_seconds: 10,
            rms_input: 0.0,
            rms_output: 0.0,
        }
    }
}
