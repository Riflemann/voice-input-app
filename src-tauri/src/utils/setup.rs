/// Setup and initialization module for first-time application setup
/// Handles downloading and installing Whisper models automatically

use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use crate::utils::error::AppError;
use tokio::io::AsyncWriteExt;

const MODELS_DIR: &str = "whisper_models";
const DEFAULT_MODEL: &str = "base"; // Default model to install
const PROJECT_MODELS_DIR: &str = "models"; // Project models directory
const MODELS_DIR_ENV: &str = "WHISPER_MODELS_DIR";
const MODEL_URLS: &[(&str, &str)] = &[
    ("tiny", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"),
    ("base", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"),
    ("small", "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin"),
];

/// Get the models directory path (AppData or env override)
pub fn get_models_dir(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    if let Ok(dir) = std::env::var(MODELS_DIR_ENV) {
        return Ok(PathBuf::from(dir));
    }

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::IoError(format!("Failed to get app data dir: {}", e)))?;
    
    let models_dir = app_data_dir.join(MODELS_DIR);
    Ok(models_dir)
}

/// Check if model exists in project models directory
fn model_exists_in_project(model_name: &str) -> bool {
    let filename = format!("ggml-{}.bin", model_name);
    
    // Check in project models directory
    let project_paths = vec![
        PathBuf::from(PROJECT_MODELS_DIR).join(&filename),
        PathBuf::from("..").join(PROJECT_MODELS_DIR).join(&filename),
    ];
    
    for path in project_paths {
        if let Ok(metadata) = fs::metadata(&path) {
            if metadata.len() > 0 {
                return true;
            }
        }
    }
    
    false
}

/// Check if model exists in a specific directory
fn model_exists_in_dir(dir: &Path, model_name: &str) -> bool {
    let path = dir.join(format!("ggml-{}.bin", model_name));
    match fs::metadata(&path) {
        Ok(metadata) => metadata.len() > 0,
        Err(_) => false,
    }
}

/// Check if a model file exists in AppData
pub fn model_exists(app_handle: &AppHandle, model_name: &str) -> Result<bool, AppError> {
    let models_dir = get_models_dir(app_handle)?;
    Ok(model_exists_in_dir(&models_dir, model_name))
}

/// Check if default model is installed (in project or AppData)
pub fn is_default_model_installed(app_handle: &AppHandle) -> Result<bool, AppError> {
    // First check if model exists in project directory
    if model_exists_in_project(DEFAULT_MODEL) {
        log::debug!("Default model '{}' found in project directory", DEFAULT_MODEL);
        return Ok(true);
    }
    
    // Then check in AppData
    model_exists(app_handle, DEFAULT_MODEL)
}

/// Initialize models directory
pub fn init_models_dir(app_handle: &AppHandle) -> Result<(), AppError> {
    let models_dir = get_models_dir(app_handle)?;
    
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| AppError::IoError(format!("Failed to create models dir: {}", e)))?;
    }
    
    Ok(())
}

/// Set models dir env var if not already set
pub fn ensure_models_dir_env(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    if let Ok(dir) = std::env::var(MODELS_DIR_ENV) {
        return Ok(PathBuf::from(dir));
    }

    let models_dir = get_models_dir(app_handle)?;
    std::env::set_var(MODELS_DIR_ENV, &models_dir);
    Ok(models_dir)
}

/// Get list of available models
pub fn get_available_models() -> Vec<(&'static str, &'static str)> {
    MODEL_URLS.to_vec()
}

/// Download a model to the app models directory
pub async fn download_model(
    app_handle: &AppHandle,
    model_name: &str,
) -> Result<PathBuf, AppError> {
    if std::env::var("SKIP_MODEL_DOWNLOAD").ok().as_deref() == Some("1") {
        return Err(AppError::InvalidModel("Model download is disabled (SKIP_MODEL_DOWNLOAD=1)".to_string()));
    }

    // If model exists in project directory, reuse it
    if model_exists_in_project(model_name) {
        let filename = format!("ggml-{}.bin", model_name);
        let project_path = PathBuf::from(PROJECT_MODELS_DIR).join(&filename);
        if project_path.exists() {
            return Ok(project_path);
        }
        let project_path = PathBuf::from("..").join(PROJECT_MODELS_DIR).join(&filename);
        if project_path.exists() {
            return Ok(project_path);
        }
    }

    // Check if model already exists and is non-empty
    if model_exists(app_handle, model_name)? {
        return Ok(get_models_dir(app_handle)?.join(format!("ggml-{}.bin", model_name)));
    }

    // Create models directory
    init_models_dir(app_handle)?;
    let models_dir = get_models_dir(app_handle)?;
    let model_path = models_dir.join(format!("ggml-{}.bin", model_name));
    if let Ok(metadata) = fs::metadata(&model_path) {
        if metadata.len() == 0 {
            let _ = fs::remove_file(&model_path);
        }
    }

    // Find the model URL
    let url = MODEL_URLS
        .iter()
        .find(|(name, _)| *name == model_name)
        .map(|(_, url)| url)
        .ok_or_else(|| AppError::InvalidModel(format!("Unknown model: {}", model_name)))?;

    log::info!("Downloading model '{}' from {}", model_name, url);

    let client = reqwest::Client::new();
    let response = client
        .get(*url)
        .send()
        .await
        .map_err(|e| AppError::IoError(format!("Failed to download model: {}", e)))?
        .error_for_status()
        .map_err(|e| AppError::IoError(format!("Download error: {}", e)))?;

    let mut file = tokio::fs::File::create(&model_path)
        .await
        .map_err(|e| AppError::IoError(format!("Failed to create model file: {}", e)))?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::IoError(format!("Download stream error: {}", e)))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| AppError::IoError(format!("Failed to write model file: {}", e)))?;
    }

    file.flush()
        .await
        .map_err(|e| AppError::IoError(format!("Failed to flush model file: {}", e)))?;

    Ok(model_path)
}

/// Ensure default model is installed
pub async fn ensure_default_model(app_handle: &AppHandle) -> Result<(), AppError> {
    if !is_default_model_installed(app_handle)? {
        log::info!("Default model '{}' not found. Downloading...", DEFAULT_MODEL);
        download_model(app_handle, DEFAULT_MODEL).await?;
    }

    let models_dir = get_models_dir(app_handle)?;
    if model_exists_in_dir(&models_dir, DEFAULT_MODEL) || model_exists_in_project(DEFAULT_MODEL) {
        log::info!("Default model '{}' found and ready", DEFAULT_MODEL);
        return Ok(());
    }

    Err(AppError::InvalidModel(format!(
        "Model '{}' not found after download. Check network and storage permissions.",
        DEFAULT_MODEL
    )))
}

/// Get setup status
#[derive(Debug, serde::Serialize)]
pub struct SetupStatus {
    pub models_initialized: bool,
    pub default_model_installed: bool,
    pub available_models: Vec<String>,
    pub installed_models: Vec<String>,
}

/// Get current setup status
pub async fn get_setup_status(app_handle: &AppHandle) -> Result<SetupStatus, AppError> {
    let models_dir = get_models_dir(app_handle)?;
    
    let available_models: Vec<String> = MODEL_URLS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    // Check for installed models in both project and AppData directories
    let mut installed_models = Vec::new();
    
    // Check project models directory
    for (model_name, _) in MODEL_URLS {
        if model_exists_in_project(model_name) {
            installed_models.push(format!("ggml-{}", model_name));
        }
    }
    
    // Check AppData directory
    if models_dir.exists() {
        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|ext| ext == "bin").unwrap_or(false) {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let model_name = stem.to_string();
                        if !installed_models.contains(&model_name) {
                            installed_models.push(model_name);
                        }
                    }
                }
            }
        }
    }

    Ok(SetupStatus {
        models_initialized: models_dir.exists() || model_exists_in_project(DEFAULT_MODEL),
        default_model_installed: is_default_model_installed(app_handle)?,
        available_models,
        installed_models,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_urls() {
        assert!(!MODEL_URLS.is_empty());
        assert_eq!(DEFAULT_MODEL, "base");
    }

    #[test]
    fn test_available_models() {
        let models = get_available_models();
        assert!(models.len() >= 3);
        assert!(models.iter().any(|(name, _)| *name == "tiny"));
        assert!(models.iter().any(|(name, _)| *name == "base"));
    }
}
