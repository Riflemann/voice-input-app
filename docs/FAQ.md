# FAQ и Troubleshooting

Ответы на часто задаваемые вопросы и решение проблем.

## Содержание

1. [Установка и запуск](#установка-и-запуск)
2. [Качество распознавания](#качество-распознавания)
3. [Проблемы с аудио](#проблемы-с-аудио)
4. [Производительность](#производительность)
5. [Разработка](#разработка)

## Установка и запуск

### В: Как установить приложение?

О: Следуйте инструкциям в [README.md](../README.md#быстрый-старт):

1. Установите Node.js (v18+) и Rust
2. Установите LLVM и CMake
3. Скачайте модель Whisper
4. Запустите `npm run tauri dev`

Подробнее: [docs/WHISPER_MODELS.md](WHISPER_MODELS.md#быстрый-старт)

### В: "Model file not found" при запуске

О: Необходимо скачать модель вручную:

```powershell
mkdir models
Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "models\ggml-base.bin"
```

Убедитесь, что файл находится в `models/ggml-base.bin` относительно корня проекта.

### В: На macOS/Linux не работает

О: Убедитесь, что установлены зависимости:

```bash
# macOS
brew install llvm cmake

# Ubuntu/Debian
sudo apt-get install llvm cmake build-essential

# Затем переустановите зависимости
cargo clean
npm install
npm run tauri dev
```

## Качество распознавания

### В: Распознавание работает неправильно (неправильные слова)

О: Возможные причины и решения:

1. **Используется tiny модель** (39 МБ)
   - ↳ Переключитесь на base (141 МБ): лучше качество
   - Инструкция: [docs/WHISPER_MODELS.md → Интеграция в приложение](WHISPER_MODELS.md#интеграция-в-приложение)

2. **Плохое качество записи**
   - ↳ Говорите четче
   - ↳ Ближе к микрофону (15-20 см)
   - ↳ Минимизируйте фоновый шум
   - ↳ Проверьте уровни в [AUDIO_PIPELINE.md → Параметры обработки](AUDIO_PIPELINE.md#параметры-обработки)

3. **Используется неподходящий язык**
   - ↳ Приложение должно автоматически определять русский язык
   - ↳ Если не работает, явно укажите язык в коде (см. [docs/WHISPER_MODELS.md → Поддерживаемые языки](WHISPER_MODELS.md#поддерживаемые-языки))

### В: Распознавание очень медленное

О: Это зависит от модели. Нормальные времена:

| Модель | 10 сек аудио | 30 сек аудио |
|--------|--------------|--------------|
| tiny | 0.5 сек | 1.5 сек |
| base | 1.5 сек | 4 сек |
| small | 5 сек | 15 сек |

Если медленнее:
- Закройте другие приложения (особенно видео, браузеры)
- Переключитесь на tiny или base модель
- Проверьте использование ЦПУ в Task Manager

### В: Результат "Речь не обнаружена" или пусто

О: Причины:

1. **Аудио не было записано**
   - ↳ Убедитесь, что микрофон работает
   - ↳ Говорите достаточно громко
   - ↳ Минимум 1.1 сек записи требуется

2. **Aудио слишком тихое**
   - ↳ Проверьте адаптивное усиление: [AUDIO_PIPELINE.md → Адаптивное усиление](AUDIO_PIPELINE.md#адаптивное-усиление)
   - ↳ Смотрите логи в консоли: "ADAPTIVE: Input RMS"

3. **Модель не загрузилась**
   - ↳ Посмотрите логи бэкенда в консоли Tauri
   - ↳ Проверьте что файл модели существует: `ls models/ggml-base.bin`

## Проблемы с аудио

### В: Аудио звучит как "писк" или шум

О: Это происходит, когда:

1. **Неправильный формат аудио**
   - ↳ Причина была в неправильной нормализации i16 → f32
   - ↳ Уже исправлено в текущей версии

2. **Слишком высокое усиление (gain)**
   - ↳ Обычно автоматическое (adaptive), но проверьте в логах
   - ↳ Максимум: 3.0x (см. [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md))

3. **Устройство захватывает на неправильной частоте**
   - ↳ Обычно 16 кГц (правильно)
   - ↳ Смотрите логи: "capture.rs: Device sample rate"

### В: Микрофон не захватывает звук

О: Проверьте:

1. **Микрофон включен?**
   ```powershell
   # Windows: Settings → Sound → Volume mixer
   # Или с PowerShell:
   Get-AudioDevice -List
   ```

2. **Права доступа?**
   - Windows: Settings → Privacy & Security → Microphone
   - macOS: System Preferences → Security & Privacy → Microphone

3. **Микрофон выбран в приложении?**
   - Смотрите селектор устройства (если есть в UI)
   - По умолчанию выбирается default device

### В: Слышно двойной звук в записи (эхо)

О: Это означает что:

1. **Может быть включен мониторинг микрофона**
   - ↳ Выключите в audio settings операционной системы
   - ↳ Отключите "Listen to this device" в Windows

2. **Две разные записи смешиваются**
   - ↳ Убедитесь что только одно приложение записывает
   - ↳ Закройте другие приложения (Discord, OBS, и т.д.)

## Производительность

### В: Приложение зависает при записи

О: Причины и решения:

1. **Недостаточно ОЗУ**
   - ↳ Требуется минимум 1 ГБ свободной памяти
   - ↳ Проверьте в Task Manager: Memory

2. **Проблема с буфером CPAL**
   - ↳ Обычно автоматически обрабатывается
   - ↳ Максимум 30 сек записи (см. [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md))

3. **Бэкенд перегружен**
   - ↳ Не записывайте одновременно несколько потоков
   - ↳ Дайте время на обработку между записями

### В: Использует слишком много памяти

О: Это нормально при использовании больших моделей:

| Модель | ОЗУ при загрузке | ОЗУ при обработке |
|--------|-----------------|------------------|
| tiny | 100 МБ | 200 МБ |
| base | 300 МБ | 500 МБ |
| small | 1 ГБ | 1.5 ГБ |
| medium | 3 ГБ | 4 ГБ |
| large | 5 ГБ | 6+ ГБ |

Если слишком много:
- Закройте другие приложения
- Переключитесь на меньшую модель (tiny, base)

## Разработка

### В: Как добавить новую функцию в бэкенд?

О: Основные этапы:

1. **Добавьте команду Tauri** в [`src-tauri/src/commands/`](../src-tauri/src/commands/)
   ```rust
   #[tauri::command]
   pub fn my_command(arg: String) -> String {
       format!("Hello, {}", arg)
   }
   ```

2. **Экспортируйте** в [`src-tauri/src/commands/mod.rs`](../src-tauri/src/commands/mod.rs)
   ```rust
   pub mod my_module;
   ```

3. **Вызовите из frontend** в TypeScript
   ```typescript
   const result = await invoke('my_command', { arg: 'world' })
   ```

Подробнее: [.copilot-instructions.md](.copilot-instructions.md#примеры--walkthrough)

### В: Как модифицировать аудиопайплайн?

О: Различные компоненты:

1. **Изменить параметры обработки** (gain, threshold)
   - Файл: [`src-tauri/src/audio/processor.rs`](../src-tauri/src/audio/processor.rs)
   - Функция: `process_audio()`
   - Подробнее: [AUDIO_PIPELINE.md → Параметры обработки](AUDIO_PIPELINE.md#параметры-обработки)

2. **Добавить новый фильтр**
   - Файл: [`src-tauri/src/audio/processor.rs`](../src-tauri/src/audio/processor.rs)
   - Функция: `process_and_filter()`

3. **Изменить обработку текста**
   - Файл: [`src-tauri/src/recognition/postprocess.rs`](../src-tauri/src/recognition/postprocess.rs)
   - Функция: `cleanup_recognized_text()`

### В: Как отладить проблемы?

О: Используйте логирование:

1. **Backend логи** (Rust)
   ```rust
   log::info!("Debug message: {:?}", value);
   log::warn!("Warning!");
   log::error!("Error!");
   ```

2. **Frontend логи** (TypeScript)
   ```typescript
   console.log('Debug:', value)
   console.error('Error:', error)
   ```

3. **Смотрите логи при запуске**
   ```bash
   npm run tauri dev
   # Логи выводятся в терминал
   ```

### В: Как запустить тесты?

О: Для Rust:

```bash
# Все тесты
cargo test

# Конкретный тест
cargo test processor --lib

# С логами
RUST_LOG=debug cargo test -- --nocapture
```

### В: Как собрать для продакшена?

О: Используйте:

```bash
npm run tauri build
```

Это создаст exe для Windows, dmg для macOS, и AppImage для Linux в папке `src-tauri/target/release/bundle/`.

Убедитесь что:
- Модель Whisper есть в `models/`
- Все зависимости установлены
- Нет ошибок компиляции: `npm run build && cargo check`

---

**Смотрите также:**
- [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md) — Архитектура аудио
- [WHISPER_MODELS.md](WHISPER_MODELS.md) — Модели и установка
- [event-flow.md](event-flow.md) — Поток событий
