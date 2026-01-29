use std::path::Path;
use tauri::{AppHandle, Emitter};
use serde::Serialize;
use crate::recognition::whisper;

#[derive(Debug, Clone, Serialize)]
pub struct RecognitionResult {
    pub text: String,
    pub audio_path: String,
}

/// Выполняет распознавание речи из WAV файла с использованием Whisper.
/// 
/// После распознавания эмитит событие 'recognition-completed' с результатом.
/// 
/// Параметры:
/// * `audio_path` - путь к обработанному WAV файлу (16kHz, mono)
/// * `app` - AppHandle для отправки событий во frontend
#[tauri::command]
pub async fn recognize_audio(
    audio_path: String,
    app: AppHandle,
) -> Result<String, String> {
    log::info!("Starting Whisper recognition for: {}", audio_path);
    
    let path = Path::new(&audio_path);
    
    // Проверяем, что файл существует
    if !path.exists() {
        return Err(format!("Audio file not found: {}", audio_path));
    }
    
    // Запускаем распознавание в отдельном потоке, чтобы не блокировать
    let path_owned = path.to_path_buf();
    let full_text = tokio::task::spawn_blocking(move || {
        // По умолчанию используем русский язык
        // TODO: сделать язык настраиваемым через параметры
        whisper::recognize(&path_owned, "ru")
    })
    .await
    .map_err(|e| format!("Recognition task failed: {}", e))??;
    
    log::info!("Recognition completed. Text length: {}", full_text.len());
    log::debug!("Recognized text: {}", full_text);
    
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

/// Инициализирует модель Whisper
/// 
/// Параметры:
/// * `model_size` - размер модели ("tiny", "base", "small", "medium", "large")
#[tauri::command]
pub async fn init_whisper(model_size: String) -> Result<String, String> {
    log::info!("Initializing Whisper with model: {}", model_size);
    
    let model = crate::recognition::models::ModelSize::from_str(&model_size)
        .ok_or(format!("Invalid model size: {}", model_size))?;
    
    // Инициализация в отдельном потоке
    tokio::task::spawn_blocking(move || {
        whisper::init(model)
    })
    .await
    .map_err(|e| format!("Init task failed: {}", e))??;
    
    log::info!("Whisper initialized successfully");
    Ok(format!("Whisper model '{}' initialized", model_size))
}
