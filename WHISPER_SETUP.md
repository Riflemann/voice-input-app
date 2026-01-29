# Настройка Whisper.cpp для Voice Input App

## 🎯 Что было сделано

Интегрировал Whisper.cpp для распознавания речи:

1. ✅ Добавлен `whisper-rs = "0.12"` в Cargo.toml
2. ✅ Реализована система загрузки моделей ([models.rs](src-tauri/src/recognition/models.rs))
3. ✅ Создан Whisper сервис ([whisper.rs](src-tauri/src/recognition/whisper.rs))
4. ✅ Добавлена постобработка текста ([postprocess.rs](src-tauri/src/recognition/postprocess.rs))
5. ✅ Обновлены Tauri команды ([recognition.rs](src-tauri/src/commands/recognition.rs))

## 🔧 Установка LLVM (libclang)

Whisper-rs требует libclang для компиляции. Установите LLVM:

### Windows

**Вариант 1: через winget (рекомендуется)**
```powershell
winget install LLVM.LLVM
```

**Вариант 2: ручная установка**
1. Скачайте LLVM: https://github.com/llvm/llvm-project/releases/latest
2. Выберите `LLVM-*-win64.exe`
3. Запустите установщик
4. ✅ Обязательно выберите "Add LLVM to system PATH"

После установки перезапустите PowerShell и проверьте:
```powershell
clang --version
```

### macOS
```bash
brew install llvm
```

Затем добавьте в `~/.zshrc` или `~/.bash_profile`:
```bash
export PATH="/usr/local/opt/llvm/bin:$PATH"
export LDFLAGS="-L/usr/local/opt/llvm/lib"
export CPPFLAGS="-I/usr/local/opt/llvm/include"
```

### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install llvm-dev libclang-dev clang
```

### Linux (Fedora)
```bash
sudo dnf install llvm-devel clang-devel
```

## 📥 Скачивание моделей Whisper

После установки LLVM скачайте модель для распознавания.

**Быстрый старт (модель Base, 142 MB):**

Windows PowerShell:
```powershell
mkdir models -ErrorAction SilentlyContinue
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "models\ggml-base.bin"
```

Linux/macOS:
```bash
mkdir -p models
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -o models/ggml-base.bin
```

Подробнее о моделях см. [models/README.md](models/README.md)

## 🚀 Сборка и запуск

После установки LLVM и скачивания модели:

```bash
# Проверка компиляции
cargo check --manifest-path src-tauri/Cargo.toml

# Запуск в режиме разработки
npm run tauri dev

# Сборка для production
npm run tauri build
```

## 💻 Использование в коде

### Frontend (TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/tauri'

// 1. Инициализация модели при запуске приложения
async function initializeWhisper() {
  try {
    await invoke('init_whisper', { 
      modelSize: 'base' // или 'tiny', 'small', 'medium', 'large'
    })
    console.log('Whisper initialized')
  } catch (error) {
    console.error('Failed to initialize Whisper:', error)
  }
}

// 2. Распознавание речи
async function recognizeAudio(audioPath: string) {
  try {
    const text = await invoke<string>('recognize_audio', { 
      audioPath 
    })
    console.log('Recognized:', text)
    return text
  } catch (error) {
    console.error('Recognition failed:', error)
    throw error
  }
}

// 3. Прослушивание событий распознавания
import { listen } from '@tauri-apps/api/event'

listen('recognition-completed', (event) => {
  const { text, audio_path } = event.payload
  console.log('Recognition completed:', text)
})
```

### Backend (Rust)

```rust
use crate::recognition::{whisper, models};

// Инициализация при старте приложения
fn setup_whisper() -> Result<(), String> {
    whisper::init(models::ModelSize::Base)?;
    Ok(())
}

// Распознавание
fn recognize_file(path: &Path) -> Result<String, String> {
    whisper::recognize(path, "ru")
}
```

## 🎛️ Конфигурация

Модели ищутся в следующих папках (по порядку):
1. `models/`
2. `src/assets/models/`
3. `../src/assets/models/`

Языки распознавания:
- `"ru"` - русский
- `"en"` - английский
- `"auto"` - автоопределение

## 🔍 Диагностика проблем

### Ошибка: "couldn't find any valid shared libraries matching: ['clang.dll', 'libclang.dll']"

**Решение:**
1. Убедитесь что LLVM установлен: `clang --version`
2. Перезапустите терминал/IDE после установки
3. Если не помогает, установите переменную окружения:
   ```powershell
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   ```
4. Или добавьте в переменные окружения Windows постоянно

### Ошибка: "Model file not found"

**Решение:**
Скачайте модель в папку `models/`:
```powershell
mkdir models -ErrorAction SilentlyContinue
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "models\ggml-base.bin"
```

### Ошибка: "Whisper model not initialized"

**Решение:**
Вызовите `init_whisper` перед использованием:
```typescript
await invoke('init_whisper', { modelSize: 'base' })
```

## 📊 Производительность

| Модель | Размер | ~Время на 30 сек аудио | RAM |
|--------|--------|------------------------|-----|
| Tiny   | 75 MB  | 1-2 сек                | ~300 MB |
| Base   | 142 MB | 2-4 сек                | ~500 MB |
| Small  | 466 MB | 5-10 сек               | ~1 GB |
| Medium | 1.5 GB | 15-30 сек              | ~2.5 GB |
| Large  | 3 GB   | 30-60 сек              | ~5 GB |

*Время указано для CPU (Intel i7/AMD Ryzen 7). С GPU будет значительно быстрее.*

## 🎯 Следующие шаги

1. ✅ Установите LLVM
2. ✅ Скачайте модель
3. ✅ Соберите проект: `cargo check --manifest-path src-tauri/Cargo.toml`
4. ✅ Запустите: `npm run tauri dev`
5. ✅ Добавьте инициализацию Whisper в [App.tsx](src/App.tsx)
6. 🔜 Настройте UI для выбора моделей
7. 🔜 Добавьте индикатор прогресса распознавания
8. 🔜 Реализуйте кэширование результатов

## 📚 Дополнительная информация

- [Whisper.cpp GitHub](https://github.com/ggerganov/whisper.cpp)
- [Whisper-rs docs](https://docs.rs/whisper-rs)
- [Модели Whisper](https://huggingface.co/ggerganov/whisper.cpp)
- [OpenAI Whisper](https://github.com/openai/whisper)
