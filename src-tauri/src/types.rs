use std::{sync::{Arc, Mutex}, time::Instant};

/// Структура для управления аудиозахватом
/// 
/// Соответствует лучшим практикам для распознавания речи (Whisper):
/// - Частота дискретизации: 48000 Hz (выше 44100, улучшает детализацию)
/// - Каналы: 1 (монo - стандарт для речи, предотвращает синхронизацию)
/// - Формат: WAV без потерь (F32/I16)
/// - Предотвращение клиппинга (target_rms = 0.12)
/// - Адаптивная подстройка шума (noise_gate = 15% RMS)
#[allow(dead_code)]
pub struct AudioCapture {
    pub is_recording: Arc<Mutex<bool>>,       // Флаг записи
    pub buffer: Arc<Mutex<Vec<f32>>>,        // Буфер для сэмплов
    pub sample_rate: u32,                    // Частота дискретизации (рекомендуется 48000 Hz для качества)
    pub channels: u16,                       // Количество каналов (1 = mono, оптимально для речи)
    pub start_time: Option<Instant>,         // Время старта записи
    pub volume_level: f32,                   // Последний уровень громкости
    // Перенесённые свойства процессора
    pub gain: f32,              // Адаптивный коэффициент усиления (целевой RMS: 0.12)
    pub noise_threshold: f32,   // Адаптивный порог шума (15% от входного RMS)
    pub rms_input: f32,                      // RMS входного сигнала
    pub rms_output: f32,                     // RMS выходного сигнала после обработки
    pub buffer_duration_seconds: usize,
    // Рекомендуемые параметры обработки:
    pub peak_prevention_threshold: f32,      // Предотвращение клиппинга (обычно 0.95)
    pub soft_noise_gate_factor: f32,         // Мягкий noise gate (оставляет % от тихих сэмплов)
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self {
            is_recording: Arc::new(Mutex::new(false)),
            buffer: Arc::new(Mutex::new(Vec::new())),
            // Использование 48000 Hz вместо 44100 для лучшей детализации речи
            // Whisper будет ресэмплировать до 16000 Hz без потери качества
            sample_rate: 48000,
            // Mono (1 канал) - стандарт для речи, предотвращает проблемы синхронизации каналов
            channels: 1,
            start_time: None,
            volume_level: 1.0,
            // Адаптивный gain (целевой RMS: 0.12 для оптимального распознавания)
            gain: 1.0,
            // Адаптивный noise threshold (вычисляется как 15% от входного RMS)
            noise_threshold: 0.0,
            buffer_duration_seconds: 10,
            // Предотвращение клиппинга: если амплитуда > 0.95, снижаем gain
            peak_prevention_threshold: 0.95,
            // Мягкий noise gate: оставляем 20% от очень тихих сэмплов для сохранения деталей
            soft_noise_gate_factor: 0.2,
            rms_input: 0.0,
            rms_output: 0.0,
        }
    }
}
