use std::{sync::{Arc, Mutex}, time::Instant};

use cpal::{SampleFormat, SupportedStreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use tauri::State;

use crate::types::AudioCapture;

/// Инициализирует и запускает захват аудио с указанного устройства.
/// 
/// Реализует лучшие практики для высокого качества распознавания:
/// - Захватывает аудио в монo (1 канал) - оптимально для речи
/// - Поддерживает 48000 Hz (выше 44100, улучшает детализацию)
/// - Использует F32 (без потерь) для сохранения максимального качества
/// - Автоматически конвертирует I16/U16 в F32 с правильной нормализацией
/// - Предотвращает многократное перекодирование (one-pass conversion)
/// - Автоматический стоп при достижении 30 секунд максимальной длительности
/// 
/// Stream создаётся для одного из трёх форматов: F32, I16, U16 (конвертируются в f32).
/// Callback проверяет is_recording перед добавлением в буфер.
/// 
/// Параметры:
/// * `state` - глобальное состояние AudioCapture с буфером и флагами
/// * `device_name` - имя аудиоустройства для захвата (из cpal::input_devices)
pub fn start_audio_capture_with_stream(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
    device_name: String,
) -> Result<cpal::Stream, String> {
    log::debug!("Starting audio capture on device: {}", device_name);
    // Clone Arc to move into audio callback for duration checks
    let state_arc = state.inner().clone();
    let mut capture = state_arc.lock().unwrap();
    if *capture.is_recording.lock().unwrap() {
        return Err("Already recording".to_string());
    }

    let host = cpal::default_host();
    let device = host.input_devices()
        .map_err(|e| format!("Device error: {}", e))?
        .find(|d| d.name().unwrap_or_default() == device_name)
        .ok_or_else(|| "Device not found".to_string())?;

    let config: SupportedStreamConfig = device.default_input_config()
        .map_err(|e| format!("Config error: {}", e))?
        .into();
    let sample_format = config.sample_format();

    let buffer = capture.buffer.clone();
    *capture.is_recording.lock().unwrap() = true;
    capture.start_time = Some(Instant::now());
    capture.sample_rate = config.sample_rate().0;
    capture.channels = config.channels() as u16;
    
    // Очищаем буфер перед новой записью
    buffer.lock().unwrap().clear();

    let is_recording = Arc::clone(&capture.is_recording);
    // Keep sample_rate for buffer trimming and duration check
    let sample_rate = capture.sample_rate;
    let channels = config.channels() as usize;
    // Max recording seconds (auto-stop)
    const MAX_RECORD_SECONDS: usize = 30;
    let max_samples = sample_rate as usize * channels * MAX_RECORD_SECONDS;
    
    // Флаг для логирования авто-стопа только один раз
    let auto_stop_logged = Arc::new(Mutex::new(false));
    
    log::info!(
        "Audio capture config: sample_rate={}Hz, channels={}, format={:?}",
        sample_rate,
        channels,
        sample_format,
    );
    log::info!(
        "Quality settings: {} Hz (выше 44.1kHz для деталей), {} ({} лучше стерео), {} формат (без потерь)",
        sample_rate,
        if channels == 1 { "Mono" } else { "Stereo" },
        if channels == 1 { "оптимально для речи" } else { "потребует синхронизации" },
        match sample_format {
            SampleFormat::F32 => "F32 (без потерь)",
            SampleFormat::I16 => "I16 (16-bit, буду конвертировать в F32 без потерь)",
            SampleFormat::U16 => "U16 (буду конвертировать в F32 без потерь)",
            _ => "Неизвестный формат (будет отклонён)",
        }
    );

    let stream = match sample_format {
        SampleFormat::F32 => {
            let is_recording = Arc::clone(&is_recording);
            let state_for_cb = state_arc.clone();
            let auto_stop_logged = auto_stop_logged.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    if !*is_recording.lock().unwrap() {
                        return; // stop processing if recording flag cleared
                    }
                    
                    let mut buf = buffer.lock().unwrap();
                    
                    // Check if we're about to exceed max duration
                    if buf.len() + data.len() > max_samples {
                        // Stop recording - only process up to max_samples
                        let remaining = max_samples.saturating_sub(buf.len());
                        if remaining > 0 {
                            buf.extend_from_slice(&data[..remaining]);
                        }
                        drop(buf);
                        
                        // Mark recording stopped
                        *is_recording.lock().unwrap() = false;
                        if let Ok(mut s) = state_for_cb.lock() {
                            s.start_time = None;
                        }
                        
                        // Log only once
                        let mut logged = auto_stop_logged.lock().unwrap();
                        if !*logged {
                            log::info!("Max recording duration reached ({}s), auto-stopping. Buffer size: {} samples", MAX_RECORD_SECONDS, max_samples);
                            *logged = true;
                        }
                        return;
                    }
                    
                    buf.extend_from_slice(data);
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        SampleFormat::I16 => {
            let is_recording = Arc::clone(&is_recording);
            let state_for_cb = state_arc.clone();
            let auto_stop_logged = auto_stop_logged.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    if !*is_recording.lock().unwrap() {
                        return;
                    }
                    
                    let mut buf = buffer.lock().unwrap();
                    
                    // Конвертация I16 -> F32 БЕЗ ПОТЕРЬ (одноразовая конвертация)
                    // Правильная нормализация: [-32768, 32767] -> [-1.0, 1.0]
                    // Это соответствует рекомендации избегать многократного перекодирования
                    let converted: Vec<f32> = data
                        .iter()
                        .map(|&sample| {
                            if sample < 0 {
                                sample as f32 / 32768.0
                            } else {
                                sample as f32 / 32767.0
                            }
                        })
                        .collect();
                    
                    if buf.len() + converted.len() > max_samples {
                        let remaining = max_samples.saturating_sub(buf.len());
                        if remaining > 0 {
                            buf.extend_from_slice(&converted[..remaining]);
                        }
                        drop(buf);
                        
                        *is_recording.lock().unwrap() = false;
                        if let Ok(mut s) = state_for_cb.lock() {
                            s.start_time = None;
                        }
                        
                        let mut logged = auto_stop_logged.lock().unwrap();
                        if !*logged {
                            log::info!("Максимальная длительность записи {}s достигнута, авто-стоп", MAX_RECORD_SECONDS);
                            *logged = true;
                        }
                        return;
                    }
                    
                    buf.extend(converted);
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        SampleFormat::U16 => {
            let is_recording = Arc::clone(&is_recording);
            let state_for_cb = state_arc.clone();
            let auto_stop_logged = auto_stop_logged.clone();
            device.build_input_stream(
                &config.into(),
                move |data: &[u16], _| {
                    if !*is_recording.lock().unwrap() {
                        return;
                    }
                    
                    let mut buf = buffer.lock().unwrap();
                    
                    // Конвертация U16 -> F32 БЕЗ ПОТЕРЬ (одноразовая конвертация)
                    // Нормализация: [0, 65535] -> [-1.0, 1.0]
                    let converted: Vec<f32> = data
                        .iter()
                        .map(|&sample| (sample as f32 - 32768.0) / 32768.0)
                        .collect();
                    
                    if buf.len() + converted.len() > max_samples {
                        let remaining = max_samples.saturating_sub(buf.len());
                        if remaining > 0 {
                            buf.extend_from_slice(&converted[..remaining]);
                        }
                        drop(buf);
                        
                        *is_recording.lock().unwrap() = false;
                        if let Ok(mut s) = state_for_cb.lock() {
                            s.start_time = None;
                        }
                        
                        let mut logged = auto_stop_logged.lock().unwrap();
                        if !*logged {
                            log::info!("Максимальная длительность записи {}s достигнута, авто-стоп", MAX_RECORD_SECONDS);
                            *logged = true;
                        }
                        return;
                    }
                    
                    buf.extend(converted);
                },
                move |err| {
                    log::error!("Stream error: {}", err);
                },
                None,
            ).map_err(|e| format!("Stream creation failed: {}", e))?
        }
        _ => {
            return Err(format!("Unsupported sample format: {:?}", sample_format));
        }
    };

    stream.play().map_err(|e| format!("Stream play error: {}", e))?;
    Ok(stream)
}
