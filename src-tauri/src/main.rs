use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Stream,
};
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::{State, Emitter};

struct AudioCapture {
    stream: Option<Stream>,
    is_recording: bool,
    buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    start_time: Option<Instant>,
    volume_level: f32,
}

impl Default for AudioCapture {
    fn default() -> Self {
        Self {
            stream: None,
            is_recording: false,
            buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: 44100,
            channels: 2,
            start_time: None,
            volume_level: 1.0,
        }
    }
}

// Команда для получения списка доступных устройств
#[tauri::command]
async fn get_input_devices() -> Result<Vec<(String, String)>, String> {
    let host = cpal::default_host();
    let devices = host.input_devices()
        .map_err(|e| format!("Error getting devices: {}", e))?;
    
    let mut result = Vec::new();
    for device in devices {
        let name = device.name().unwrap_or_else(|_| "Unknown".to_string());
        let id = format!("{:?}", device);
        result.push((id, name));
    }
    
    Ok(result)
}

// Захват с выбором устройства
#[tauri::command]
async fn start_capture_with_device(
    mut state: State<'_, Mutex<AudioCapture>>,
    device_name: String,
    window: tauri::Window,
) -> Result<(), String> {
    let mut capture = state.lock().unwrap();
    
    if capture.is_recording {
        return Err("Already recording".to_string());
    }
    
    let host = cpal::default_host();
    let device = host.input_devices()
        .unwrap()
        .find(|d| d.name().unwrap_or_default() == device_name)
        .ok_or_else(|| "Device not found".to_string())?;
    
    let config = device.default_input_config()
        .map_err(|e| format!("Config error: {}", e))?;
    
    capture.sample_rate = config.sample_rate().0;
    capture.channels = config.channels();
    
    let buffer = Arc::clone(&capture.buffer);
    let window_clone = window.clone();
    
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                process_audio_data(data, &buffer, &window_clone);
            },
            move |err| {
                eprintln!("Stream error: {}", err);
            },
            Some(Duration::from_millis(100)),
        ),
        _ => return Err("Unsupported sample format".to_string()),
    };
    
    let stream = stream.map_err(|e| format!("Stream creation failed: {}", e))?;
    stream.play().unwrap();
    
    capture.stream = Some(stream);
    capture.is_recording = true;
    capture.start_time = Some(Instant::now());
    
    Ok(())
}

// Функция обработки аудио данных
fn process_audio_data(data: &[f32], buffer: &Arc<Mutex<Vec<f32>>>, window: &tauri::Window) {
    let mut buffer_guard = buffer.lock().unwrap();
    
    // Можно добавить обработку (нормализация, фильтрация)
    for &sample in data {
        buffer_guard.push(sample);
    }
    
    // Ограничиваем размер буфера (например, 10 секунд аудио)
    let max_samples = 44100 * 10; // 10 секунд при 44.1кГц
    if buffer_guard.len() > max_samples {
        buffer_guard.drain(0..buffer_guard.len() - max_samples);
    }
    
    // Отправляем метрики в UI
    if buffer_guard.len() % 1024 == 0 { // Каждые 1024 сэмпла
        let rms = calculate_rms(&buffer_guard);
        let _ = window.emit("audio-level", rms);
    }
}

// Расчет RMS (уровень громкости)
fn calculate_rms(data: &[f32]) -> f32 {
    if data.is_empty() {
        return 0.0;
    }
    
    let sum_squares: f32 = data.iter().map(|&x| x * x).sum();
    (sum_squares / data.len() as f32).sqrt()
}

// Сохранение в WAV файл
#[tauri::command]
async fn save_audio_to_file(
    state: State<'_, Mutex<AudioCapture>>,
    file_path: String,
) -> Result<(), String> {
    use hound::{WavWriter, WavSpec};
    
    let capture = state.lock().unwrap();
    let buffer = capture.buffer.lock().unwrap();
    
    let spec = WavSpec {
        channels: capture.channels,
        sample_rate: capture.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    
    let mut writer = WavWriter::create(file_path, spec)
        .map_err(|e| format!("Failed to create WAV file: {}", e))?;
    
    // Конвертируем f32 в i16 для WAV
    for &sample in buffer.iter() {
        let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
        writer.write_sample(sample_i16)
            .map_err(|e| format!("Failed to write sample: {}", e))?;
    }
    
    writer.finalize()
        .map_err(|e| format!("Failed to finalize WAV: {}", e))?;
    
    Ok(())
}