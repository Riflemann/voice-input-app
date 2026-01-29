use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc::Receiver;
use hound;
use crate::types::AudioCapture;
use crate::audio::processor::process_audio;
use crate::utils::cache::SharedAudioCache;
use tauri::{AppHandle, Emitter};

/// Background worker для обработки аудио в отдельном потоке.
/// 
/// Принимает сэмплы из mpsc канала, сохраняет их в pre-processed WAV,
/// применяет process_audio для фильтрации/усиления, сохраняет post-processed WAV
/// и эмитит событие 'processing-finished' с путями к файлам.
/// 
/// Параметры:
/// * `rx` - mpsc receiver для получения сэмплов
/// * `capture` - Arc на состояние AudioCapture для чтения sample_rate/channels
/// * `cache` - Arc на AudioCache для генерации путей к временным WAV файлам
/// * `app` - AppHandle для отправки событий во frontend
pub async fn run(
    mut rx: Receiver<Vec<f32>>, 
    capture: Arc<Mutex<AudioCapture>>, 
    cache: SharedAudioCache,
    app: AppHandle
) {
    log::info!("Audio worker started");

    while let Some(samples) = rx.recv().await {
        let pre_path = cache.generate_wav_path("pre");
        let post_path = cache.generate_wav_path("post");

        // Read sample rate and channels from capture
        let (sample_rate, channels) = {
            let cap = capture.lock().unwrap();
            (cap.sample_rate, cap.channels)
        };

        // Save pre-processing WAV
        if let Err(e) = write_wav_f32_path(&pre_path, &samples, sample_rate, channels) {
            log::error!("Failed to write pre WAV: {}", e);
        } else {
            log::info!("Wrote pre WAV: {:?}", pre_path);
        }

        // Prepare output buffer for processed data
        let out_buf: Arc<RwLock<Vec<f32>>> = Arc::new(RwLock::new(Vec::new()));

        // run processor in blocking thread
        let capture_clone = capture.clone();
        let samples_clone = samples.clone();
        let out_clone = out_buf.clone();

        let _ = tokio::task::spawn_blocking(move || {
            process_audio(&samples_clone, &out_clone, capture_clone);
        }).await;

        // Read processed samples
        let processed = match out_buf.read() {
            Ok(v) => v.clone(),
            Err(e) => {
                log::error!("Failed to read processed buffer: {}", e);
                Vec::new()
            }
        };

        // Save post-processing WAV
        if let Err(e) = write_wav_f32_path(&post_path, &processed, sample_rate, channels) {
            log::error!("Failed to write post WAV: {}", e);
        } else {
            log::info!("Wrote post WAV: {:?}", post_path);
        }

        // Emit event to front-end with paths
        let payload = (pre_path.to_string_lossy().to_string(), post_path.to_string_lossy().to_string());
        log::info!("Emitting processing-finished event with payload: {:?}", payload);
        
        if let Err(e) = app.emit("processing-finished", payload) {
            log::error!("Failed to emit processing-finished event: {}", e);
        } else {
            log::info!("Successfully emitted processing-finished event");
        }
    }

    log::info!("Audio worker exiting");
}

/// Записывает f32 сэмплы в WAV файл с 16-битным PCM форматом.
/// 
/// Конвертирует f32 [-1.0, 1.0] в i16 с ограничением диапазона.
/// 
/// Параметры:
/// * `path` - путь к выходному WAV файлу
/// * `samples` - срез аудиосэмплов (f32)
/// * `sample_rate` - частота дискретизации
/// * `channels` - количество каналов
fn write_wav_f32_path(path: &std::path::Path, samples: &[f32], sample_rate: u32, channels: u16) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec).map_err(|e| e.to_string())?;

    for &s in samples.iter() {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(v).map_err(|e| e.to_string())?;
    }

    writer.finalize().map_err(|e| e.to_string())?;
    Ok(())
}
