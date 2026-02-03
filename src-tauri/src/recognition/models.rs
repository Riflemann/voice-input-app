use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use whisper_rs::{WhisperContext, WhisperContextParameters};

/// Статическое хранилище загруженной модели Whisper
static WHISPER_MODEL: Lazy<Arc<Mutex<Option<WhisperContext>>>> = 
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// Путь к директории с моделями (по умолчанию)
const DEFAULT_MODELS_DIR: &str = "models";
const MODELS_DIR_ENV: &str = "WHISPER_MODELS_DIR";

/// Имена поддерживаемых моделей
#[derive(Debug, Clone)]
pub enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
    Large,
}

impl ModelSize {
    /// Возвращает имя файла модели
    pub fn filename(&self) -> &str {
        match self {
            ModelSize::Tiny => "ggml-tiny.bin",
            ModelSize::Base => "ggml-base.bin",
            ModelSize::Small => "ggml-small.bin",
            ModelSize::Medium => "ggml-medium.bin",
            ModelSize::Large => "ggml-large-v3.bin",
        }
    }
    
    /// Парсит размер модели из строки
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tiny" => Some(ModelSize::Tiny),
            "base" => Some(ModelSize::Base),
            "small" => Some(ModelSize::Small),
            "medium" => Some(ModelSize::Medium),
            "large" => Some(ModelSize::Large),
            _ => None,
        }
    }
}

/// Загружает модель Whisper из файла
pub fn load_model(model_path: &Path) -> Result<WhisperContext, String> {
    log::info!("Loading Whisper model from: {:?}", model_path);
    if let Ok(cwd) = std::env::current_dir() {
        log::info!("Current working directory: {:?}", cwd);
    }
    if let Ok(metadata) = model_path.metadata() {
        log::info!("File size: {} MB", metadata.len() / 1024 / 1024);
    }
    
    if !model_path.exists() {
        return Err(format!(
            "Model file not found: {:?}\n\nМодель должна скачиваться автоматически при первом запуске.\n\
            Проверьте подключение к интернету и права на запись в папку моделей.\n\
            Текущий путь модели: {:?}",
            model_path,
            model_path
        ));
    }
    
    log::info!("Creating Whisper context (this may take 10-30 seconds)...");
    let ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("Invalid model path")?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load Whisper model: {}", e))?;
    
    log::info!("Whisper model loaded successfully");
    Ok(ctx)
}

/// Инициализирует и загружает модель в глобальное хранилище
pub fn initialize_model(model_size: ModelSize) -> Result<(), String> {
    log::info!("Initializing Whisper model: {:?}", model_size);
    let model_path = get_model_path(model_size)?;
    
    log::info!("Acquiring model mutex lock...");
    let mut model = WHISPER_MODEL.lock()
        .map_err(|e| format!("Failed to lock model mutex: {}", e))?;

    if model.is_some() {
        log::info!("Whisper model already initialized, skipping re-load");
        return Ok(());
    }

    log::info!("Model not initialized yet. Loading and storing..." );
    let ctx = load_model(&model_path)?;
    *model = Some(ctx);
    log::info!("Model stored in global state successfully");
    
    Ok(())
}

/// Возвращает путь к модели
pub fn get_model_path(model_size: ModelSize) -> Result<PathBuf, String> {
    // Пытаемся найти модель в нескольких местах
    let mut possible_paths = Vec::new();

    if let Ok(dir) = std::env::var(MODELS_DIR_ENV) {
        possible_paths.push(PathBuf::from(dir).join(model_size.filename()));
    }

    possible_paths.extend(vec![
        PathBuf::from(DEFAULT_MODELS_DIR).join(model_size.filename()),
        PathBuf::from("..").join(DEFAULT_MODELS_DIR).join(model_size.filename()),
        PathBuf::from("src").join("assets").join("models").join(model_size.filename()),
        PathBuf::from("../src/assets/models").join(model_size.filename()),
    ]);
    
    for path in possible_paths {
        if path.exists() {
            return Ok(path);
        }
    }
    
    // Возвращаем дефолтный путь с инструкцией
    Ok(PathBuf::from(DEFAULT_MODELS_DIR).join(model_size.filename()))
}

/// Получает доступ к загруженной модели
pub fn get_model() -> Result<Arc<Mutex<Option<WhisperContext>>>, String> {
    Ok(WHISPER_MODEL.clone())
}
