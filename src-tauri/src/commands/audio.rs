use std::sync::Mutex;

use tauri::State;

use crate::AudioCapture;


async fn stop_and_save_audio(
	state: State<'_, Mutex<AudioCapture>>,
	file_path: String,
) -> Result<(), String> {
	use hound::{WavWriter, WavSpec};

	let mut capture: std::sync::MutexGuard<'_, AudioCapture> = state.lock().unwrap();
	capture.is_recording = false;
	// Сохраняем буфер
	let buffer: std::sync::MutexGuard<'_, Vec<f32>> = capture.buffer.lock().unwrap();
	let spec: WavSpec = WavSpec {
		channels: capture.channels,
		sample_rate: capture.sample_rate,
		bits_per_sample: 16,
		sample_format: hound::SampleFormat::Int,
	};
	let mut writer = WavWriter::create(file_path, spec)
		.map_err(|e| format!("Failed to create WAV: {}", e))?;
	for &sample in buffer.iter() {
		let sample_i16 = (sample * 32767.0).clamp(-32768.0, 32767.0) as i16;
		writer.write_sample(sample_i16)
			.map_err(|e| format!("Failed to write sample: {}", e))?;
	}
	writer.finalize().map_err(|e| format!("Finalize error: {}", e))?;
	Ok(())
}