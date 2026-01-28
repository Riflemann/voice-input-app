use std::{sync::{Arc, Mutex, RwLock}, time::{SystemTime, UNIX_EPOCH}};
use tokio::sync::mpsc::Receiver;
use hound;
use crate::AudioCapture;
use crate::audio::processor::process_audio;
use tauri::{AppHandle, Manager, Emitter};

pub async fn run(mut rx: Receiver<Vec<f32>>, capture: Arc<Mutex<AudioCapture>>, app: AppHandle) {
    log::info!("Audio worker started");

    while let Some(samples) = rx.recv().await {
        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let pre_path = std::env::temp_dir().join(format!("pre_{}.wav", ts));
        let post_path = std::env::temp_dir().join(format!("post_{}.wav", ts));

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
