#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_process_and_filter() {
        let input_data = vec![0.01, 0.03, -0.05, 0.5, -0.9];
        let result = process_and_filter(&input_data);
        assert_eq!(result.len(), input_data.len());
        assert_eq!(result[0], 0.0); // Ниже порога шума
        assert!(result[1] > 0.0); // Усилено
        assert!(result[3] <= 1.0); // Ограничено (clamped)
    }

    #[test]
    fn test_manage_buffer() {
        let mut buffer = vec![0.0; MAX_SAMPLES - 100];
        let window = tauri::Window::default(); // Мок или заменить тестовым дублёром
        manage_buffer(&mut buffer, &window);
        assert!(buffer.len() <= MAX_SAMPLES);

        buffer.extend(vec![0.1; 200]);
        manage_buffer(&mut buffer, &window);
        assert_eq!(buffer.len(), MAX_SAMPLES);
    }

    #[test]
    fn test_calculate_rms() {
        let data = vec![0.0, 1.0, -1.0, 0.5, -0.5];
        let rms = calculate_rms(&data);
        assert!((rms - 0.707).abs() < 0.01); // Приближённое значение
    }
}