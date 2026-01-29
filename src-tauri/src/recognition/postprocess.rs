/// Постобработка распознанного текста
/// 
/// Выполняет:
/// - Удаление лишних пробелов
/// - Капитализацию первого символа
/// - Удаление повторяющихся фраз (hallucinations)
pub fn process_text(text: &str) -> String {
    let mut result = text.trim().to_string();

    // Удаляем маркеры неслышимой/фоновой активности (например, [музыка])
    result = remove_non_speech_markers(&result);
    
    // Удаляем множественные пробелы
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }
    
    // Капитализация первой буквы
    if let Some(first_char) = result.chars().next() {
        if first_char.is_lowercase() {
            let mut chars = result.chars();
            chars.next();
            result = first_char.to_uppercase().collect::<String>() + chars.as_str();
        }
    }
    
    // Удаляем повторяющиеся последовательности (hallucinations)
    result = remove_repetitions(&result);
    
    result
}

/// Удаляет известные маркеры фоновой активности, которые Whisper иногда возвращает
fn remove_non_speech_markers(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let markers = [
        "[музыка]",
        "[music]",
        "[аплодисменты]",
        "[applause]",
        "[смех]",
        "[laughter]",
        "[шум]",
        "[noise]",
        "[тишина]",
        "[silence]",
        "[фон]",
        "[background]",
        "[background noise]",
    ];

    let filtered: Vec<String> = text
        .split_whitespace()
        .filter(|token| {
            let cleaned = token
                .trim_matches(|c: char| c == ',' || c == '.' || c == ';' || c == ':' || c == '!' || c == '?')
                .to_lowercase();
            !markers.iter().any(|m| m == &cleaned)
        })
        .map(|s| s.to_string())
        .collect();

    filtered.join(" ")
}

/// Удаляет повторяющиеся последовательности слов
fn remove_repetitions(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    
    if words.len() < 4 {
        return text.to_string();
    }
    
    let mut result = Vec::new();
    let mut i = 0;
    
    while i < words.len() {
        let mut found_repetition = false;
        
        // Проверяем повторения длиной от 2 до 10 слов
        for rep_len in (2..=10.min(words.len() - i)).rev() {
            if i + rep_len * 2 > words.len() {
                continue;
            }
            
            let slice1 = &words[i..i + rep_len];
            let slice2 = &words[i + rep_len..i + rep_len * 2];
            
            if slice1 == slice2 {
                // Найдено повторение, добавляем только один раз
                result.extend_from_slice(slice1);
                i += rep_len * 2;
                found_repetition = true;
                break;
            }
        }
        
        if !found_repetition {
            result.push(words[i]);
            i += 1;
        }
    }
    
    result.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_text_basic() {
        assert_eq!(process_text("  hello   world  "), "Hello world");
    }
    
    #[test]
    fn test_remove_repetitions() {
        let text = "это тест это тест и еще текст";
        let result = remove_repetitions(text);
        assert_eq!(result, "это тест и еще текст");
    }

    #[test]
    fn test_remove_non_speech_markers() {
        let text = "[музыка] привет [music]";
        let result = remove_non_speech_markers(text);
        assert_eq!(result, "привет");
    }
}
