# Поток событий в voice-input-app

## Архитектура событий

Приложение использует **event-driven** подход для асинхронной связи между Rust backend и React frontend через Tauri события.

## Полный цикл: запись → обработка → распознавание

```
ЗАПИСЬ АУДИО (1-30 сек)
    ↓
ОБРАБОТКА АУДИО (adaptive gain, noise gate)
    ↓
ПЕРЕОТПРОБИРОВАНИЕ & ПАДДИНГ (если нужно)
    ↓
РАСПОЗНАВАНИЕ WHISPER (inference)
    ↓
ПОСТОБРАБОТКА ТЕКСТА (удаление маркеров)
    ↓
ОТОБРАЖЕНИЕ РЕЗУЛЬТАТА (на frontend)
```

Подробное описание каждого этапа в [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md).

## Последовательность событий

### 1. Запись аудио

**Файлы:**
- Frontend: [`src/components/RecordButton/index.tsx`](../src/components/RecordButton/index.tsx)
- Backend: [`src-tauri/src/audio/capture.rs`](../src-tauri/src/audio/capture.rs)

```
Frontend: Button "Нажмите для записи"
    ↓
Frontend: invoke('start_recording')
    ↓
Backend: start_audio_capture_with_stream()
    ├─ Инициализирует CPAL устройство
    ├─ Получает stream callbacks с аудиоданными
    └─ Сохраняет сэмплы в Arc<RwLock<Vec<f32>>>
    ↓ (продолжается запись до 30 сек или нажатия stop)
Frontend: invoke('stop_recording')
    ↓
Backend: stop_recording_inner()
    ├─ Закрывает CPAL stream
    ├─ Извлекает полный буфер аудиоданных
    └─ Отправляет в background worker на обработку
```

**Функции:**
- `start_recording()` — инициирует захват в [`src-tauri/src/commands/audio.rs`](../src-tauri/src/commands/audio.rs)
- `start_audio_capture_with_stream()` — основной цикл в [`src-tauri/src/audio/capture.rs`](../src-tauri/src/audio/capture.rs)
- `stop_recording_inner()` — завершает запись в [`src-tauri/src/commands/audio.rs`](../src-tauri/src/commands/audio.rs)

### 2. Обработка аудио (background worker)

**Файлы:**
- Worker: [`src-tauri/src/audio/worker.rs`](../src-tauri/src/audio/worker.rs)
- Обработка: [`src-tauri/src/audio/processor.rs`](../src-tauri/src/audio/processor.rs)
- Состояние: [`src-tauri/src/types.rs`](../src-tauri/src/types.rs)

```
Backend: queue_for_processing(audio_samples)
    ↓ (отправляет в mpsc channel)
Background Tokio Worker: worker::run()
    ├─ Получает сэмплы из channel
    ├─ Сохраняет pre-processed.wav (исходное аудио)
    ├─ process_audio(samples, &buffer, &state)
    │  ├─ Вычисляет RMS входного сигнала
    │  ├─ Адаптивное вычисление gain (target_rms 0.08)
    │  ├─ Применяет noise gate (если нужно)
    │  ├─ Усиливает сэмплы (sample * gain)
    │  └─ Логирует RMS выходного сигнала
    ├─ Сохраняет post-processed.wav (обработанное аудио)
    └─ emit('processing-finished', [prePath, postPath])
        ↓ (асинхронно, без блокирования)
```

**Параметры обработки:**
- Target RMS: 0.08 (оптимально для Whisper)
- Gain range: 0.5-3.0x (адаптивно)
- Noise threshold: 0.0 (отключен для сохранения качества)

Подробно: [AUDIO_PIPELINE.md → Обработка сигнала](AUDIO_PIPELINE.md#-обработка-сигнала)

### 3. Распознавание речи

**Файлы:**
- Frontend listener: [`src/components/RecordButton/useRecord.ts`](../src/components/RecordButton/useRecord.ts)
- Backend recognition: [`src-tauri/src/commands/recognition.rs`](../src-tauri/src/commands/recognition.rs)
- Whisper inference: [`src-tauri/src/recognition/whisper.rs`](../src-tauri/src/recognition/whisper.rs)
- Постобработка: [`src-tauri/src/recognition/postprocess.rs`](../src-tauri/src/recognition/postprocess.rs)

```
Frontend: listen('processing-finished')
    ↓ (получает пути к WAV файлам)
Frontend: setWavPaths(prePath, postPath)
Frontend: setIsProcessing(false)
Frontend: recognize(postPath) → invoke('recognize_audio')
    ↓
Backend: recognize_audio(audio_path)
    ├─ recognize() function
    │  ├─ load_audio_samples(path) — читает .wav файл
    │  ├─ Переотпробирование (если нужно) — 16 кГц target
    │  ├─ Паддинг минимум 1.1 сек
    │  ├─ load_model() — первый раз, затем переиспользуется
    │  └─ whisper inference — распознавание текста
    │
    ├─ postprocess_text(raw_text)
    │  ├─ Удаляет [МУЗЫКА], [MUSIC], [АПЛОДИСМЕНТЫ]
    │  ├─ Удаляет излишние повторения
    │  ├─ Капитализирует первую букву
    │  └─ Очищает от странных символов
    │
    ├─ emit('recognition-completed', {text, audio_path})
    │  ↓ (асинхронно)
    └─ return Ok(clean_text)
        ↓ (в parallel)
```

**Frontend обработка результата:**
```
Promise resolves: setText(result)      ← Быстрое обновление
Event received: listen('recognition-completed') → setText(text)  ← Дублирование (insurance)
```

**Модель и языки:**
- Model path: `../models/ggml-base.bin`
- Автоматическое определение языка (включает русский)
- Подробно: [WHISPER_MODELS.md](WHISPER_MODELS.md)

## События (Tauri)

### `processing-finished`

**Источник:** [`src-tauri/src/audio/worker.rs`](../src-tauri/src/audio/worker.rs) → `emit_processing_finished()`

**Payload:** 
```typescript
{
  pre_path: string,    // Путь к pre-processed.wav
  post_path: string    // Путь к post-processed.wav
}
```

**Назначение:** Уведомляет frontend что аудио обработано и готово к распознаванию

**Слушатель:** 
```typescript
// src/components/RecordButton/useRecord.ts
listen('processing-finished', (event) => {
  setWavPaths(event.payload.pre_path, event.payload.post_path);
  // Затем вызывает recognize(event.payload.post_path)
})
```

**Время срабатывания:** Сразу после сохранения WAV файлов (< 100 мс)

### `recognition-completed`

**Источник:** [`src-tauri/src/commands/recognition.rs`](../src-tauri/src/commands/recognition.rs) → `emit()`

**Payload:**
```typescript
{
  text: string,        // Распознанный текст (clean)
  audio_path: string   // Путь к исходному .wav файлу
}
```

**Назначение:** Уведомляет frontend что распознавание завершено

**Слушатель:**
```typescript
// src/components/RecordButton/useRecord.ts
listen('recognition-completed', (event) => {
  setText(event.payload.text);
  // store.setLastResult()
  // store.setLastResultEmpty()
})
```

**Время срабатывания:** После Whisper inference (0.5-5 сек в зависимости от модели)

**Примечание:** Результат дублируется из Promise и Event — это гарантирует получение данных даже при сбое Promise

## Диаграмма потока данных

```
┌──────────┐  start  ┌─────────┐  stop  ┌──────────────┐
│ Frontend │────────→│ Backend │───────→│ stop_inner() │
└──────────┘         └─────────┘        └──────┬───────┘
                                               │
                                               ↓
                                    ┌──────────────────┐
                                    │ queue samples    │
                                    │ (mpsc channel)   │
                                    └────────┬─────────┘
                                             │
                                             ↓
                          ┌──────────────────────────────────┐
                          │   Background Worker (tokio)      │
                          │                                  │
                          │  1. Save pre-processed.wav       │
                          │  2. process_audio()              │
                          │  3. Save post-processed.wav      │
                          │  4. emit('processing-finished')  │
                          └──────────────┬───────────────────┘
                                         │
                                         ↓
                          ┌─────────────────────────────┐
                          │  Frontend Event Listener    │
                          │                             │
                          │  setWavPaths()              │
                          │  setIsProcessing(false)     │
                          │  recognize(postPath) ────┐  │
                          └─────────────────────────┼──┘
                                                    │
                                                    ↓
                          ┌─────────────────────────────────┐
                          │  Backend: recognize_audio()     │
                          │                                 │
                          │  1. Load audio file             │
                          │  2. [stub] simulate recognition │
                          │  3. emit('recognition-completed')│
                          │  4. return Ok(text)             │
                          └──────────────┬──────────────────┘
                                         │
                                         ↓
                          ┌─────────────────────────────┐
                          │  Frontend: two paths        │
                          │                             │
                          │  Promise: setText(result)   │
                          │  Event: setText(text)       │
                          └─────────────────────────────┘
```

## Преимущества event-driven подхода

1. **Decoupling:** Backend не блокирует frontend — воркер работает асинхронно
2. **Progress tracking:** Frontend может показывать статус обработки через события
3. **Flexibility:** Можно добавлять новых слушателей без изменения backend
4. **Error isolation:** Ошибки в одном слушателе не влияют на другие

## Альтернативные подходы

- **Promise-only:** Использовать только `invoke()` без событий (текущий подход использует оба)
- **Polling:** Frontend периодически проверяет статус (неэффективно)
- **WebSocket:** Для более сложных real-time сценариев (избыточно для текущих требований)
