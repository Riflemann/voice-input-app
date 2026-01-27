use tauri::State;
use std::sync::{Arc, Mutex};
use crate::AudioCapture;
use crate::audio::service::{start, stop_recording};

#[tauri::command]
fn start_recording(state: State<Arc<Mutex<AudioCapture>>>) -> Result<String, String> {
    let mut recording_state = state.lock().map_err(|_| "Failed to acquire lock")?;
    if recording_state.is_recording {
        return Err("Recording is already in progress".to_string());
    }

    // Add logic to start audio recording here

    recording_state.is_recording = true;
    Ok("Recording started".to_string())
}

#[tauri::command]
fn stop_recording(state: State<Arc<Mutex<AudioCapture>>>) -> Result<String, String> {
    let mut recording_state = state.lock().map_err(|_| "Failed to acquire lock")?;
    if !recording_state.is_recording {
        return Err("No recording in progress".to_string());
    }

    // Add logic to stop audio recording here

    recording_state.is_recording = false;
    Ok("Recording stopped".to_string())
}