// Управление аудио
fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(AudioCapture::default()))
        .invoke_handler(tauri::generate_handler![
            init_audio_capture,
            start_capture,
            stop_capture,
            get_audio_data,
            clear_audio_buffer,
            get_input_devices,
            start_capture_with_device,
            save_audio_to_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}