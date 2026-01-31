// Типы ошибок

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Invalid model: {0}")]
    InvalidModel(String),
}
