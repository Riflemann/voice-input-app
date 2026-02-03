use std::sync::{Arc, RwLock, Mutex};
use crate::types::AudioCapture;

/// Обрабатывает сырые аудиосэмплы: применяет noise gate и усиление.
/// 
/// Реализует лучшие практики для распознавания речи (Whisper):
/// - Целевой RMS: 0.12 (оптимально для речи, особенно русского языка)
/// - Адаптивный gain: автоматически подстраивается на основе входного RMS
/// - Мягкий noise gate: 15% от входного RMS (сохраняет детали речи)
/// - Предотвращение клиппинга: если peak > 0.95, снижаем gain
/// - Используется F32 формат без потерь (WAV)
/// 
/// Параметры:
/// * `input_data` - срез входных аудиосэмплов (f32, без потерь)
/// * `buffer` - Arc на RwLock буфер для записи обработанных данных
/// * `state` - Arc на Mutex состояния AudioCapture с параметрами обработки
pub fn process_audio(
    input_data: &[f32],
    buffer: &Arc<RwLock<Vec<f32>>>,
    state: std::sync::Arc<Mutex<AudioCapture>>,
) {
    let rms_input = calculate_rms(input_data);
    let peak_input = input_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    
    log::debug!(
        "Processing audio: input RMS={:.6}, peak={:.6}",
        rms_input, peak_input
    );

    // Автоматическая адаптивная подстройка параметров на основе RMS
    let (mut gain, 
        sample_rate, 
        buffer_duration_seconds,
         peak_threshold,
          soft_gate_factor
        ) = {
        let capture = state.lock().unwrap();
        (
            capture.gain,
            capture.sample_rate as usize,
            capture.buffer_duration_seconds,
            capture.peak_prevention_threshold,
            capture.soft_noise_gate_factor,
        )
    };

    // Целевой RMS для речи: 0.12 
    // (оптимален для Whisper, особенно для русского языка)
    let target_rms = 0.12;
    
    // Автоматически подстраиваем gain на основе входного RMS
    if rms_input > 0.001 {
        let mut adaptive_gain = target_rms / rms_input;
        
        // Ограничиваем диапазон: от 0.5 до 10.0
        adaptive_gain = adaptive_gain.clamp(0.5, 10.0);
        
        // ПРЕДОТВРАЩЕНИЕ КЛИППИНГА: если peak * gain > peak_threshold, снижаем gain
        if peak_input * adaptive_gain > peak_threshold {
            let max_gain_for_peak = (peak_threshold / peak_input).max(0.5);
            adaptive_gain = adaptive_gain.min(max_gain_for_peak);
            log::info!(
                "CLIPPING PREVENTION: Reduced gain from target {:.2} to {:.2} (peak was {:.6})",
                target_rms / rms_input, adaptive_gain, peak_input
            );
        }
        
        gain = adaptive_gain;
        log::debug!(
            "ADAPTIVE: Input RMS {:.6}, peak {:.6} → gain {:.2} (target RMS: {:.2})",
            rms_input, peak_input, gain, target_rms
        );
    }
    
    // Noise threshold = 15% от входного RMS 
    // (мягкий noise gate для сохранения деталей речи)
    let noise_threshold = (rms_input * 0.15).min(0.01);
    
    let processed = process_and_filter(
        input_data,
        noise_threshold,
        gain,
        soft_gate_factor,
    );

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

/// Применяет мягкий noise gate и gain к каждому сэмплу.
/// 
/// Реализует лучшие практики:
/// - Сэмплы с амплитудой меньше noise_threshold не полностью обнуляются,
///   а снижаются с множителем soft_gate_factor (обычно 0.2) для сохранения деталей
/// - Остальные сэмплы умножаются на gain и ограничиваются диапазоном [-1.0, 1.0]
/// - Без многократного перекодирования (F32 - формат без потерь)
/// 
/// Параметры:
/// * `input_data` - входные сэмплы
/// * `noise_threshold` - порог шума (абсолютное значение)
/// * `gain` - коэффициент усиления
/// * `soft_gate_factor` - множитель для тихих сэмплов (обычно 0.2 = 20%)
fn process_and_filter(
    input_data: &[f32],
    noise_threshold: f32,
    gain: f32,
    soft_gate_factor: f32,
) -> Vec<f32> {
    let rms_input = calculate_rms(input_data);
    let peak_input = input_data.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    
    log::debug!("Input: RMS={:.6}, Peak={:.6}, Threshold={:.6}", rms_input, peak_input, noise_threshold);

    let processed: Vec<f32> = input_data
        .iter()
        .map(|&sample| {
            // Мягкий noise gate: применяем мягкий множитель для очень тихих сэмплов
            // Это сохраняет детали речи, включая тихие согласные и фрикативы
            if sample.abs() < noise_threshold {
                sample * soft_gate_factor  // Оставляем % (обычно 20%) от тихих сэмплов
            } else {
                sample
            }
        })
        .map(|sample| {
            // Применяем усиление и ограничиваем диапазон (предотвращение клиппинга)
            let amplified = sample * gain;
            amplified.clamp(-1.0, 1.0)
        })
        .collect();

    let rms_output = calculate_rms(&processed);
    let peak_output = processed.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    
    log::debug!(
        "Output: RMS={:.6}, Peak={:.6} (gain={:.2}, threshold={:.6})",
        rms_output, peak_output, gain, noise_threshold
    );
    
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