use tauri::{State};
use tokio::{task, time::sleep};
use std::{sync::{Arc, Mutex}, time::Duration};
use crate::AudioCapture;
use crate::audio::capture::{start_audio_capture_with_stream};
use tokio::sync::mpsc::Sender;

#[tauri::command]
pub fn start_recording(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
    processing_sender: State<'_, Sender<Vec<f32>>>,
    device: String,
) -> Result<String, String> {
    let stream = start_audio_capture_with_stream(state, device)?;
    
    log::info!("Audio capture started successfully.");

    // Stream будет дропнут, но callback продолжит работать пока is_recording == true
    // Callback проверяет флаг is_recording внутри себя
    std::mem::forget(stream); // Предотвращаем drop, stream будет жить до конца программы
    
    // TODO: Добавить таймаут если нужно (требует другой архитектуры)
    // Пока пользователь должен сам останавливать запись

    // Захват аудио запущен; обработка будет выполняться при остановке
    Ok("Recording started".to_string())
}

#[tauri::command]
pub fn stop_recording(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
    processing_sender: State<'_, Sender<Vec<f32>>>,
) -> Result<(), String> {
    // wrapper for the tauri command: forward to the inner function using the inner Arc and sender
    stop_recording_inner(state.inner().clone(), processing_sender.inner().clone())
}

fn stop_recording_inner(
    state_arc: Arc<Mutex<AudioCapture>>,
    sender: Sender<Vec<f32>>,
) -> Result<(), String> {
    // Извлекаем и останавливаем запись
    let samples = extract_audio_samples(&state_arc)?;
    
    // Проверяем наличие данных
    validate_samples(&samples)?;
    
    // Отправляем в очередь обработки
    queue_for_processing(sender, samples)
}

/// Останавливает запись и извлекает аудио сэмплы из буфера
fn extract_audio_samples(state_arc: &Arc<Mutex<AudioCapture>>) -> Result<Vec<f32>, String> {
    let mut capture = state_arc
        .lock()
        .map_err(|_| "Failed to acquire lock on AudioCapture state".to_string())?;

    // Проверяем, что запись активна
    let is_recording = *capture
        .is_recording
        .lock()
        .map_err(|_| "Failed to lock is_recording flag".to_string())?;
    
    if !is_recording {
        return Err("Recording is not in progress".to_string());
    }

    // Останавливаем запись
    *capture
        .is_recording
        .lock()
        .map_err(|_| "Failed to lock is_recording flag".to_string())? = false;
    capture.start_time = None;

    // Извлекаем сэмплы из буфера (без клонирования)
    let samples = capture
        .buffer
        .lock()
        .map_err(|_| "Failed to lock audio buffer".to_string())
        .map(|mut buf| std::mem::take(&mut *buf))?;
    
    log::info!("Recording stopped, extracted {} samples", samples.len());
    Ok(samples)
}

/// Проверяет что буфер содержит данные
fn validate_samples(samples: &[f32]) -> Result<(), String> {
    if samples.is_empty() {
        log::warn!("Recording stopped but buffer is empty");
        return Err("No audio data recorded".to_string());
    }
    Ok(())
}

/// Отправляет сэмплы в очередь обработки
fn queue_for_processing(sender: Sender<Vec<f32>>, samples: Vec<f32>) -> Result<(), String> {
    log::info!("Queueing {} samples for background processing", samples.len());
    
    sender
        .try_send(samples)
        .map_err(|e| format!("Failed to queue processing: {}", e))?;
    
    log::info!("Audio processing queued successfully");
    Ok(())
}