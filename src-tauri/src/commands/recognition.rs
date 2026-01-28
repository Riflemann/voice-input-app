use std::path::Path;
use crate::AudioCapture;
use tauri::{AppHandle, Emitter, State};
use std::sync::{Arc, Mutex};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RecognitionResult {
    pub text: String,
    pub audio_path: String,
}

/// Распознавание речи с помощью Whisper модели
/// 
/// # Аргументы
/// * `audio_path` - путь к WAV файлу (обработанный аудиофайл)
/// * `capture_state` - состояние AudioCapture (для параметров как sample_rate)
/// * `app` - AppHandle для эмита события
/// 
/// # Возвращает
/// Распознанный текст или ошибка
#[tauri::command]
pub async fn recognize_audio(
    audio_path: String,
    app: AppHandle,
) -> Result<String, String> {
    log::info!("Starting recognition for: {}", audio_path);
    
    let path = Path::new(&audio_path);
    
    // Проверяем, что файл существует
    if !path.exists() {
        return Err(format!("Audio file not found: {}", audio_path));
    }
    
    // STUB: Имитируем распознавание речи
    // TODO: Интегрировать whisper-rs когда libclang будет доступен
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    
    let full_text = format!("Тестовое распознавание успешно. Файл: {}", file_name);
    
    log::info!("Recognition completed. Text length: {}", full_text.len());
    
    // Эмитим событие recognition-completed с результатом
    let result = RecognitionResult {
        text: full_text.clone(),
        audio_path: audio_path.clone(),
    };
    
    app.emit("recognition-completed", &result)
        .map_err(|e| format!("Failed to emit recognition-completed event: {}", e))?;
    
    log::info!("Emitted recognition-completed event");
    
    Ok(full_text)
}
