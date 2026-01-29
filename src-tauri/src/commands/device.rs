use cpal::{default_host, traits::{DeviceTrait, HostTrait}};
use serde::Serialize;
use pipe_trait::Pipe;

#[derive(Serialize)]
pub struct InputDevice {
    pub name: String,
}

/// Возвращает список всех доступных аудио входных устройств.
/// 
/// Использует cpal для получения списка устройств от системного хоста.
/// Фильтрует устройства, которые не могут предоставить имя.
#[tauri::command]
pub fn get_input_device_names() -> Result<Vec<InputDevice>, String> {
	log::debug!("Retrieving input devices");
	let host = default_host();
	log::debug!("Using host: {:?}", host.id());

	host.input_devices()
        .map_err(|e| format!("Failed to retrieve input devices: {}", e))?
        .filter_map(|device| device.name().ok().map(|name| InputDevice { name }))
        .collect::<Vec<_>>()
        .pipe(Ok)
}

/// Возвращает системное аудио входное устройство по умолчанию.
/// 
/// Если устройство по умолчанию недоступно, возвращает ошибку.
#[tauri::command]
pub async fn get_default_input_device_name() -> Result<InputDevice, String> {
	log::debug!("Retrieving input devices");
	let host = default_host();
	log::debug!("Using host: {:?}", host.id());

	let device = host.default_input_device().ok_or_else(|| {
		format!("Device error: No default input device found")
	})?;

	let device_name = device.name().map_err(|e| format!("Failed to get device name: {}", e))?;

	Ok(InputDevice { name: device_name })
}