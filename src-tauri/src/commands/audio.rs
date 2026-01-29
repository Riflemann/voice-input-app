use tauri::State;
use std::sync::{Arc, Mutex};
use crate::types::AudioCapture;
use crate::audio::capture::start_audio_capture_with_stream;
use tokio::sync::mpsc::Sender;

/// Возвращает текущий статус записи.
/// 
/// Параметры:
/// * `state` - глобальное состояние AudioCapture
#[tauri::command]
pub fn get_recording_status(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
) -> Result<bool, String> {
    let capture = state.lock()
        .map_err(|_| "Failed to lock state".to_string())?;
    let is_recording = *capture.is_recording.lock()
        .map_err(|_| "Failed to lock is_recording".to_string())?;
    Ok(is_recording)
}

/// Запускает запись аудио с микрофона.
/// 
/// Инициализирует аудиозахват через cpal stream, который продолжает работать пока is_recording == true.
/// Stream забывается через mem::forget чтобы callback продолжал работу до явной остановки.
/// Авто-стоп происходит через 30 секунд внутри audio callback (см. capture.rs).
/// 
/// Параметры:
/// * `state` - глобальное состояние AudioCapture с буфером и флагами записи
/// * `device` - имя аудиоустройства для захвата
#[tauri::command]
pub fn start_recording(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
    device: String,
) -> Result<String, String> {
    let stream = start_audio_capture_with_stream(state, device)?;
    
    log::info!("Audio capture started successfully.");

    std::mem::forget(stream);
    Ok("Recording started".to_string())
}

/// Останавливает запись аудио и отправляет захваченные сэмплы на обработку.
/// 
/// Извлекает сэмплы из буфера, валидирует их и отправляет в фоновый worker через mpsc канал.
/// 
/// Параметры:
/// * `state` - глобальное состояние AudioCapture
/// * `processing_sender` - канал для отправки сэмплов в background worker
#[tauri::command]
pub fn stop_recording(
    state: State<'_, Arc<Mutex<AudioCapture>>>,
    processing_sender: State<'_, Sender<Vec<f32>>>,
) -> Result<(), String> {
    stop_recording_inner(state.inner().clone(), processing_sender.inner().clone())
}

/// Внутренняя функция остановки записи: координирует извлечение, валидацию и постановку в очередь.
/// 
/// Параметры:
/// * `state_arc` - Arc на состояние AudioCapture
/// * `sender` - mpsc канал для отправки сэмплов
fn stop_recording_inner(
    state_arc: Arc<Mutex<AudioCapture>>,
    sender: Sender<Vec<f32>>,
) -> Result<(), String> {
    let samples = extract_audio_samples(&state_arc)?;
    validate_samples(&samples)?;
    queue_for_processing(sender, samples)
}

/// Останавливает запись и извлекает аудио сэмплы из буфера.
/// 
/// Устанавливает is_recording в false, очищает start_time и забирает буфер через mem::take.
/// 
/// Параметры:
/// * `state_arc` - ссылка на Arc с состоянием AudioCapture
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

/// Проверяет что буфер содержит данные.
/// 
/// Возвращает ошибку если массив сэмплов пуст.
/// 
/// Параметры:
/// * `samples` - срез аудиосэмплов для проверки
fn validate_samples(samples: &[f32]) -> Result<(), String> {
    if samples.is_empty() {
        log::warn!("Recording stopped but buffer is empty");
        return Err("No audio data recorded".to_string());
    }
    Ok(())
}

/// Отправляет сэмплы в очередь обработки через mpsc канал.
/// 
/// Использует try_send для неблокирующей отправки. Если канал полон, возвращает ошибку.
/// 
/// Параметры:
/// * `sender` - mpsc sender для передачи данных в background worker
/// * `samples` - вектор аудиосэмплов (f32) для обработки
fn queue_for_processing(sender: Sender<Vec<f32>>, samples: Vec<f32>) -> Result<(), String> {
    log::info!("Queueing {} samples for background processing", samples.len());
    
    sender
        .try_send(samples)
        .map_err(|e| format!("Failed to queue processing: {}", e))?;
    
    log::info!("Audio processing queued successfully");
    Ok(())
}