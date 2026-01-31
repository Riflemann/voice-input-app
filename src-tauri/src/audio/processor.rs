use std::sync::{Arc, RwLock, Mutex};
use crate::types::AudioCapture;

/// Обрабатывает сырые аудиосэмплы: применяет noise gate и усиление.
/// 
/// Автоматически адаптирует параметры на основе входного RMS:
/// - Если RMS < 0.01: увеличивает gain до нормального уровня
/// - Если RMS > 0.15: уменьшает gain чтобы избежать обрезания
/// - Noise threshold вычисляется автоматически на основе уровня шума
/// 
/// Параметры:
/// * `input_data` - срез входных аудиосэмплов (f32)
/// * `buffer` - Arc на RwLock буфер для записи обработанных данных
/// * `state` - Arc на Mutex состояния AudioCapture с параметрами обработки
pub fn process_audio(
    input_data: &[f32],
    buffer: &Arc<RwLock<Vec<f32>>>,
    state: std::sync::Arc<Mutex<AudioCapture>>,
) {
    log::info!("Processing started");

    let rms_input = calculate_rms(input_data);
    
    // Автоматическая адаптивная подстройка параметров на основе RMS
    let (mut gain, mut noise_threshold, sample_rate, buffer_duration_seconds) = {
        let capture = state.lock().unwrap();
        (
            capture.gain,
            capture.noise_threshold,
            capture.sample_rate as usize,
            capture.buffer_duration_seconds,
        )
    };

    // Целевой RMS для речи: 0.12 (оптимизировано для русского языка)
    let target_rms = 0.12;
    
    // Автоматически подстраиваем gain на основе входного RMS
    if rms_input > 0.001 {
        let adaptive_gain = target_rms / rms_input;
        
        // Ограничиваем диапазон: от 0.5 до 10.0 (увеличено для лучшего усиления)
        gain = adaptive_gain.clamp(0.5, 10.0);
        
        log::info!(
            "ADAPTIVE: Input RMS {:.6} → Calculated gain: {:.2} (target RMS: {:.2})",
            rms_input, gain, target_rms
        );
    }
    
    // Noise threshold = 15% от входного RMS (смягченный noise gate для сохранения деталей речи)
    noise_threshold = (rms_input * 0.15).min(0.01);
    
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

/// Применяет noise gate и gain к каждому сэмплу.
/// 
/// Сэмплы с амплитудой меньше noise_threshold обнуляются,
/// остальные умножаются на gain и ограничиваются диапазоном [-1.0, 1.0].
/// 
/// Параметры:
/// * `input_data` - входные сэмплы
/// * `noise_threshold` - порог шума (абсолютное значение)
/// * `gain` - коэффициент усиления
fn process_and_filter(input_data: &[f32], noise_threshold: f32, gain: f32) -> Vec<f32> {
    let rms_input = calculate_rms(input_data);
    let peak_input = input_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    log::info!("Input RMS: {:.6}, Peak: {:.6}", rms_input, peak_input);

    let processed: Vec<f32> = input_data
        .iter()
        .map(|&sample| {
            // Мягкий noise gate: не полностью обнуляем, а снижаем амплитуду плавно
            if sample.abs() < noise_threshold {
                sample * 0.2 // Оставляем 20% от тихих сэмплов для лучшего сохранения деталей
            } else {
                sample
            }
        })
        .map(|sample| {
            let amplified = sample * gain;
            amplified.clamp(-1.0, 1.0)
        })
        .collect();

    let rms_output = calculate_rms(&processed);
    let peak_output = processed.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    log::info!("Output RMS: {:.6}, Peak: {:.6} (gain={}, threshold={})", rms_output, peak_output, gain, noise_threshold);
    
    processed
}

/// Тримит буфер до максимального размера и логирует RMS каждые 1024 сэмплов.
/// 
/// Если буфер превышает max_samples (sample_rate * buffer_duration_seconds),
/// удаляет старейшие данные через drain.
/// 
/// Параметры:
/// * `buffer` - мутабельная ссылка на буфер сэмплов
/// * `sample_rate` - частота дискретизации (Гц)
/// * `buffer_duration_seconds` - максимальная длительность буфера (сек)
fn manage_buffer(buffer: &mut Vec<f32>, sample_rate: usize, buffer_duration_seconds: usize) {
    let max_samples = sample_rate * buffer_duration_seconds;

    if buffer.len() > max_samples {
        buffer.drain(0..buffer.len() - max_samples);
    }

    let should_log = buffer.len() % 4096 == 0 && buffer.len() > 0;
    if should_log {
        let rms = calculate_rms(buffer);
        log::debug!("Buffer RMS: {:.6}, size: {} samples", rms, buffer.len());
    }
}

/// Вычисляет RMS (Root Mean Square) для массива сэмплов.
/// 
/// Возвращает 0.0 для пустого массива.
/// 
/// Параметры:
/// * `data` - срез аудиосэмплов
fn calculate_rms(data: &[f32]) -> f32 {
    if data.is_empty() { return 0.0; }
    let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
    (sum_squares / data.len() as f32).sqrt()
}