use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy};
use crate::recognition::models::{get_model, initialize_model, ModelSize};
use crate::recognition::postprocess;

/// Инициализирует Whisper с заданной моделью
pub fn init(model_size: ModelSize) -> Result<(), String> {
    initialize_model(model_size)
}

/// Распознает речь из WAV файла
/// 
/// Параметры:
/// * `audio_path` - путь к WAV файлу (должен быть 16kHz, mono, 16-bit)
/// * `language` - язык распознавания ("ru", "en", "auto" для автоопределения)
pub fn recognize(audio_path: &Path, language: &str) -> Result<String, String> {
    log::info!("Recognizing audio from: {:?}, language: {}", audio_path, language);
    
    // Проверяем файл
    if !audio_path.exists() {
        return Err(format!("Audio file not found: {:?}", audio_path));
    }
    
    // Загружаем аудио данные
    log::info!("Loading audio samples from file...");
    let audio_data = load_audio_samples(audio_path)?;
    log::info!("Audio loaded: {} samples", audio_data.len());
    let audio_data = pad_audio_min_duration(audio_data, 16000, 1.1);
    
    // Получаем модель
    log::info!("Getting Whisper model lock...");
    let model_lock = get_model()?;
    log::info!("Locking model mutex...");
    let mut model_guard = model_lock.lock()
        .map_err(|e| format!("Failed to lock model: {}", e))?;
    
    log::info!("Checking if model is initialized...");
    if model_guard.is_none() {
        log::warn!("Whisper model not initialized yet. Initializing default model (base)...");
        drop(model_guard);

        initialize_model(ModelSize::Base)?;

        log::info!("Re-locking model mutex after initialization...");
        model_guard = model_lock.lock()
            .map_err(|e| format!("Failed to lock model after init: {}", e))?;
    }

    let ctx = model_guard.as_mut()
        .ok_or("Whisper model not initialized. Initialization failed.")?;
    log::info!("Model is ready, proceeding with recognition...");
    
    // Создаем параметры распознавания
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    
    // Настраиваем параметры
    params.set_n_threads(4);
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    
    // Устанавливаем язык
    if language != "auto" {
        params.set_language(Some(language));
    }
    
    // Запускаем распознавание
    log::info!("Creating Whisper state...");
    let mut state = ctx.create_state()
        .map_err(|e| format!("Failed to create Whisper state: {}", e))?;
    log::info!("Whisper state created, starting recognition...");
    
    state.full(params, &audio_data)
        .map_err(|e| format!("Whisper recognition failed: {}", e))?;
    log::info!("Whisper recognition finished, extracting segments...");
    
    // Собираем результат
    let num_segments = state.full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;
    
    let mut full_text = String::new();
    
    for i in 0..num_segments {
        let segment = state.full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment {}: {}", i, e))?;
        full_text.push_str(&segment);
        full_text.push(' ');
    }
    
    let full_text = full_text.trim().to_string();
    
    log::info!("Recognition completed. Text length: {}", full_text.len());
    
    // Постобработка текста
    let processed_text = postprocess::process_text(&full_text);
    if processed_text.trim().is_empty() && !full_text.trim().is_empty() {
        log::info!(
            "Postprocess removed all text. Raw result was: {}",
            full_text
        );
    }
    
    Ok(processed_text)
}

/// Загружает аудио данные из WAV файла и конвертирует в формат для Whisper
/// (16kHz, mono, f32 в диапазоне [-1.0, 1.0])
fn load_audio_samples(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;
    
    let spec = reader.spec();
    log::debug!("WAV spec: {:?}", spec);
    
    // Whisper требует 16kHz mono
    let needs_resample = spec.sample_rate != 16000;
    if needs_resample {
        log::warn!(
            "Audio sample rate is {} Hz, resampling to 16000 Hz for Whisper.",
            spec.sample_rate
        );
    }
    
    if spec.channels != 1 {
        log::warn!(
            "Audio has {} channels, but Whisper expects mono. \
            Only first channel will be used.",
            spec.channels
        );
    }
    
    // Читаем семплы и нормализуем в диапазон [-1.0, 1.0]
    let samples: Result<Vec<f32>, _> = match spec.sample_format {
        hound::SampleFormat::Float => {
            reader.samples::<f32>()
                .map(|s| s.map_err(|e| format!("Sample read error: {}", e)))
                .collect()
        }
        hound::SampleFormat::Int => {
            match spec.bits_per_sample {
                16 => {
                    reader.samples::<i16>()
                        .map(|s| s.map(|s| s as f32 / 32768.0)
                            .map_err(|e| format!("Sample read error: {}", e)))
                        .collect()
                }
                32 => {
                    reader.samples::<i32>()
                        .map(|s| s.map(|s| s as f32 / 2147483648.0)
                            .map_err(|e| format!("Sample read error: {}", e)))
                        .collect()
                }
                _ => Err(format!("Unsupported bits per sample: {}", spec.bits_per_sample)),
            }
        }
    };
    
    let samples = samples?;
    
    // Если стерео, берем только первый канал
    let mono_samples = if spec.channels > 1 {
        samples.iter()
            .step_by(spec.channels as usize)
            .copied()
            .collect()
    } else {
        samples
    };
    
    let resampled = if needs_resample {
        log::info!("Resampling from {} Hz to 16000 Hz...", spec.sample_rate);
        let result = resample_linear(&mono_samples, spec.sample_rate, 16000);
        log::info!("Resampling completed: {} -> {} samples", mono_samples.len(), result.len());
        result
    } else {
        mono_samples
    };
    
    log::debug!("Loaded {} audio samples", resampled.len());
    
    Ok(resampled)
}

/// Простая линейная ресэмплизация до целевой частоты
fn resample_linear(input: &[f32], src_rate: u32, dst_rate: u32) -> Vec<f32> {
    if input.is_empty() || src_rate == dst_rate {
        return input.to_vec();
    }

    let src_rate = src_rate as f64;
    let dst_rate = dst_rate as f64;
    let ratio = src_rate / dst_rate;
    let out_len = ((input.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = (i as f64) * ratio;
        let idx = src_pos.floor() as usize;
        let frac = src_pos - (idx as f64);

        let s0 = input.get(idx).copied().unwrap_or(0.0);
        let s1 = input.get(idx + 1).copied().unwrap_or(s0);

        output.push(s0 + (s1 - s0) * (frac as f32));
    }

    output
}

/// Дополняет аудио тишиной до минимальной длительности
fn pad_audio_min_duration(mut input: Vec<f32>, sample_rate: u32, min_seconds: f32) -> Vec<f32> {
    if input.is_empty() {
        return input;
    }

    let min_samples = (sample_rate as f32 * min_seconds).ceil() as usize;
    if input.len() < min_samples {
        let to_add = min_samples - input.len();
        log::warn!("Audio too short ({} samples). Padding with {} samples of silence.", input.len(), to_add);
        input.extend(std::iter::repeat(0.0).take(to_add));
    }

    input
}
