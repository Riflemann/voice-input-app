use std::path::PathBuf;
use std::sync::Arc;
use std::fs;

/// Временный кэш для хранения WAV файлов.
/// 
/// Создаёт уникальную директорию при инициализации и автоматически удаляет её при Drop.
/// Директория существует только пока приложение запущено.
pub struct AudioCache {
    cache_dir: PathBuf,
}

impl AudioCache {
    /// Создаёт новую кэш-директорию в системной temp папке.
    /// 
    /// Имя директории: `voice-input-app-{process_id}`
    pub fn new() -> Result<Self, String> {
        let process_id = std::process::id();
        let cache_dir = std::env::temp_dir().join(format!("voice-input-app-{}", process_id));
        
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        
        log::info!("Audio cache created at: {:?}", cache_dir);
        
        Ok(Self { cache_dir })
    }
    
    /// Возвращает путь к кэш-директории.
    #[allow(dead_code)]
    pub fn dir(&self) -> &PathBuf {
        &self.cache_dir
    }
    
    /// Генерирует путь для нового WAV файла с timestamp префиксом.
    /// 
    /// Параметры:
    /// * `prefix` - префикс имени файла (например, "pre" или "post")
    pub fn generate_wav_path(&self, prefix: &str) -> PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        self.cache_dir.join(format!("{}_{}.wav", prefix, ts))
    }
    
    /// Очищает все файлы в кэш-директории.
    #[allow(dead_code)]
    pub fn clear(&self) -> Result<(), String> {
        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path).map_err(|e| e.to_string())?;
                }
            }
            log::debug!("Cache cleared");
        }
        Ok(())
    }
}

impl Drop for AudioCache {
    fn drop(&mut self) {
        if self.cache_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&self.cache_dir) {
                log::warn!("Failed to cleanup cache directory: {}", e);
            } else {
                log::info!("Cache directory cleaned up: {:?}", self.cache_dir);
            }
        }
    }
}

/// Обёртка для безопасного использования AudioCache в многопоточной среде.
pub type SharedAudioCache = Arc<AudioCache>;
