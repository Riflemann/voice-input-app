// Обработка аудио
use std::sync::{Arc, RwLock, Mutex};
use crate::AudioCapture;

/// Обрабатывает аудиоданные: фильтрует шум и усиливает сигнал.
///
/// Функция применяет фильтрацию шума и усиление к входным данным,
/// затем добавляет обработанные сэмплы в общий буфер и управляет его размером.
/// При ошибках логирует предупреждения или ошибки.
pub fn process_audio(
    input_data: &[f32],
    buffer: &Arc<RwLock<Vec<f32>>>,
    state: std::sync::Arc<Mutex<AudioCapture>>,
) {
    log::info!("Processing started");

    // Читаем параметры из состояния захвата
    let (gain, noise_threshold, sample_rate, buffer_duration_seconds) = {
        let capture = state.lock().unwrap();
        (
            capture.gain,
            capture.noise_threshold,
            capture.sample_rate as usize,
            capture.buffer_duration_seconds,
        )
    };

    let processed = process_and_filter(input_data, noise_threshold, gain);

    match buffer.write() {
        Ok(mut buf) => {
            buf.extend_from_slice(&processed);
            manage_buffer(&mut buf, sample_rate, buffer_duration_seconds);
        }
        Err(e) => {
            log::error!("Failed to acquire write lock on buffer: {}", e);
        }
    }
}

/// Фильтрует и усиливает аудиосэмплы.
///
/// # Возвращает
/// Вектор обработанных аудиосэмплов.
fn process_and_filter(input_data: &[f32], noise_threshold: f32, gain: f32) -> Vec<f32> {
    input_data
        .iter()
        .map(|&sample| if sample.abs() < noise_threshold { 0.0 } else { sample })
        .map(|sample| {
            let amplified = sample * gain;
            amplified.clamp(-1.0, 1.0)
        })
        .collect()
}

/// Управляет размером аудиобуфера и периодически рассчитывает RMS.
///
/// Обеспечивает, что буфер не превышает максимальный размер, и периодически
/// вычисляет среднеквадратичное значение (RMS) аудиоданных.
fn manage_buffer(buffer: &mut Vec<f32>, sample_rate: usize, buffer_duration_seconds: usize) {
    let max_samples = sample_rate * buffer_duration_seconds;

    if buffer.len() > max_samples {
        buffer.drain(0..buffer.len() - max_samples);
    }

    let should_calculate_rms = buffer.len() % 1024 == 0;
    if should_calculate_rms {
        let _rms = calculate_rms(buffer);
        log::debug!("Current RMS: {}", _rms);
    }
}

/// Вычисляет среднеквадратичное значение (RMS) аудиоданных.
///
/// # Аргументы
/// * `data` - Срез аудиосэмплов.
///
/// # Возвращает
/// Значение RMS для переданных данных.
fn calculate_rms(data: &[f32]) -> f32 {
    if data.is_empty() { return 0.0; }
    let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
    (sum_squares / data.len() as f32).sqrt()
}