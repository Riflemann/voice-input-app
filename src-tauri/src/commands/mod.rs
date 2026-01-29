pub mod audio;
pub mod device;
pub mod recognition;
pub mod system;

// Экспорт всех команд для регистрации в Tauri
pub use audio::*;
pub use device::*;
pub use recognition::*;
pub use system::*;

