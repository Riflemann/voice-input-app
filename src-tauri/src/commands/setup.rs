/// System initialization and setup commands
use crate::utils::setup::{self, SetupStatus};

/// Initialize application on first run
/// Downloads and installs required Whisper models
#[tauri::command]
pub async fn initialize_app(app_handle: tauri::AppHandle) -> Result<SetupStatus, String> {
    // Create models directory
    setup::init_models_dir(&app_handle)
        .map_err(|e| format!("Failed to initialize models directory: {}", e))?;

    setup::ensure_models_dir_env(&app_handle)
        .map_err(|e| format!("Failed to set models dir env: {}", e))?;

    // Ensure default model is installed
    setup::ensure_default_model(&app_handle)
        .await
        .map_err(|e| format!("Failed to setup default model: {}", e))?;

    // Get and return setup status
    let status = setup::get_setup_status(&app_handle)
        .await
        .map_err(|e| format!("Failed to get setup status: {}", e))?;

    Ok(status)
}

/// Get current setup status
#[tauri::command]
pub async fn get_setup_status(app_handle: tauri::AppHandle) -> Result<SetupStatus, String> {
    setup::get_setup_status(&app_handle)
        .await
        .map_err(|e| format!("Failed to get setup status: {}", e))
}

/// Download a specific model
#[tauri::command]
pub async fn download_model(
    app_handle: tauri::AppHandle,
    model_name: String,
) -> Result<String, String> {
    let path = setup::download_model(&app_handle, &model_name)
        .await
        .map_err(|e| format!("Failed to download model: {}", e))?;

    Ok(path
        .to_str()
        .unwrap_or("Unknown")
        .to_string())
}

/// Get list of available models
#[tauri::command]
pub fn get_available_models() -> Result<Vec<(String, String)>, String> {
    let models = setup::get_available_models();
    let result: Vec<(String, String)> = models
        .iter()
        .map(|(name, url)| (name.to_string(), url.to_string()))
        .collect();
    Ok(result)
}
