# Voice Input App

Voice Input App — кроссплатформенное настольное приложение для голосового ввода, обработки аудио и распознавания речи. В качестве бэкенда используется Tauri (Rust), а фронтенд реализован на React с TypeScript, что обеспечивает быструю и отзывчивую работу.

## Возможности

- **Голосовой ввод**: запись и обработка аудиопотока с устройства
- **Распознавание речи**: Whisper.cpp для локального распознавания (99 языков)
- **Постобработка**: автоматическая очистка текста от повторений и шумов
- **Настройки**: гибкая настройка параметров аудио и выбор моделей
- **Кроссплатформенность**: Windows, macOS и Linux
- **Современный интерфейс**: React + Zustand + Tailwind CSS

## Технологии

### Фронтенд
- **Фреймворк**: React + TypeScript
- **Состояние**: Zustand
- **Стили**: Tailwind CSS
- **Сборка**: Vite
- **Каталог**: `src/`

### Бэкенд
- **Фреймворк**: Tauri 2.0
- **Язык**: Rust
- **Аудио**: CPAL для захвата
- **Распознавание**: Whisper.cpp (whisper-rs)
- **Каталог**: `src-tauri/`

## Быстрый старт

### Требования
- Node.js (v18+)
- Rust (stable)
- **LLVM** (для компиляции Whisper)
- **CMake** (для сборки Whisper)

### Установка модели (обязательно!)

Приложение использует модель Whisper для распознавания речи. Модель нужно скачать перед запуском:

```bash
# Создать папку для моделей
mkdir models

# Скачать модель Whisper.cpp (Base, 141 MB)
# Windows PowerShell:
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "models\ggml-base.bin"

# или macOS/Linux (curl):
curl -o models/ggml-base.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
```

### Запуск приложения

```bash
# Установка npm зависимостей
npm install

# Dev режим (Vite + Tauri)
npm run tauri dev

# Сборка (создать bundle)
npm run build
npm run tauri build
```

### Доступные модели

Можно использовать другие модели Whisper:
- **tiny** (39 MB) — быстро, низкое качество
- **base** (141 MB) — баланс качества и скорости ✅ **рекомендуется**
- **small** (465 MB) — лучше качество, медленнее
- **medium** (1.5 GB) — еще лучше качество
- **large** (2.9 GB) — лучшее качество

Замени ссылку в команде выше на нужную модель с [HuggingFace](https://huggingface.co/ggerganov/whisper.cpp).

# Запуск
npm run tauri dev
```

### Ручная установка

1. **Установите LLVM и CMake**:
   ```powershell
   winget install LLVM.LLVM
   winget install Kitware.CMake
   ```

2. **Клонируйте репозиторий**:
   ```bash
   git clone <repository-url>
   cd voice-input-app
   ```

3. **Установите зависимости**:
   ```bash
   npm install
   ```

4. **Скачайте модель Whisper** (см. [models/README.md](models/README.md)):
   ```powershell
   mkdir models
   Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "models\ggml-base.bin"
   ```

5. **Запустите**:
   ```bash
   npm run tauri dev
   ```

## Сборка

Собрать приложение для продакшена:
```bash
npm run tauri build
```

## Структура каталогов

```
voice-input-app/
├── src/                # Код фронтенда
│   ├── components/     # React-компоненты
│   ├── hooks/          # Пользовательские хуки
│   ├── stores/         # Zustand-хранилища
│   ├── utils/          # Утилиты
├── src-tauri/          # Код бэкенда (Rust)
│   ├── audio/          # Логика обработки аудио
│   ├── commands/       # Команды Tauri
│   ├── recognition/    # Логика распознавания речи
│   ├── utils/          # Вспомогательные модули
├── package.json        # Node.js зависимости и скрипты
├── Cargo.toml          # Rust зависимости
├── vite.config.js      # Конфиг Vite
├── tailwind.config.js  # Конфиг Tailwind CSS
```

## Разработка

### Фронтенд
- Редактируйте React-компоненты в `src/components/`.
- Используйте Zustand для управления состоянием в `src/stores/`.
- Добавляйте хуки в `src/hooks/`.

### Бэкенд
- Добавляйте команды Tauri в `src-tauri/src/commands/`.
- Реализуйте логику обработки аудио в `src-tauri/src/audio/`.
- Расширяйте функционал распознавания в `src-tauri/src/recognition/`.

## Вклад

Пулл-реквесты приветствуются! Порядок работы:
1. Форкните репозиторий.
2. Создайте ветку для фичи или исправления.
3. Закомитьте изменения и запушьте ветку.
4. Откройте Pull Request.

## Лицензия

Проект лицензирован под MIT. См. файл [LICENSE](LICENSE) для деталей.

## Благодарности

- [Tauri](https://tauri.app/) — за фреймворк бэкенда.
- [React](https://reactjs.org/) — за фронтенд.
- [Zustand](https://zustand-demo.pmnd.rs/) — за управление состоянием.
- [Tailwind CSS](https://tailwindcss.com/) — за стили.
- [cpal](https://github.com/RustAudio/cpal) — за работу с аудио в Rust.
