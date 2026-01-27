use std::sync::Mutex;
use tauri::{State, window};
use crate::AudioCapture;
use crate::audio::capture::{start_audio_capture_with_stream, stop_and_save_audio};
use crate::audio::processor::process_audio;

pub fn start(
    state: State<'_, Mutex<AudioCapture>>, 
    device_name: String, 
    window: window::Window
) {
    let stream = start_audio_capture_with_stream(state, device_name);
    match stream {
        Ok(audio_stream) => {
            log::info!("Audio capture started successfully.");

            
            process_audio(&audio_data, , &window);
        },
        Err(e) => log::error!("Failed to start audio capture: {}", e),
    } 
}

pub fn stop_recording(state: State<'_, Mutex<AudioCapture>>) -> Result<(), String> {
    let mut capture = state.lock().map_err(|_| "Failed to acquire lock on AudioCapture state")?;

    if !capture.is_recording {
        return Err("Recording is not in progress".to_string());
    }

    capture.is_recording = false;
    capture.start_time = None;

    log::info!("Recording stopped successfully.");
    Ok(())
}