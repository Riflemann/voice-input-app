# Поток событий в voice-input-app

## Архитектура событий

Приложение использует event-driven подход для связи между Rust backend и React frontend через Tauri события.

## Последовательность событий

### 1. Запись аудио
```
Frontend: invoke('start_recording') 
    ↓
Backend: audio::start_recording()
    ↓ (30s или пользователь нажал stop)
Frontend: invoke('stop_recording')
    ↓
Backend: audio::stop_recording_inner()
```

### 2. Обработка аудио (background worker)
```
Backend: queue_for_processing() → sender.try_send(samples)
    ↓
Worker: worker::run() получает samples
    ↓
Worker: сохраняет pre-processed.wav
    ↓
Worker: process_audio() (noise gate, gain)
    ↓
Worker: сохраняет post-processed.wav
    ↓
Worker: emit('processing-finished', [prePath, postPath])
```

### 3. Распознавание речи
```
Frontend: listen('processing-finished')
    ↓
Frontend: setWavPaths(), setIsProcessing(false)
    ↓
Frontend: recognize(postPath) → invoke('recognize_audio')
    ↓
Backend: recognition::recognize_audio()
    ↓
Backend: [stub] sleep(500ms) + format text
    ↓
Backend: emit('recognition-completed', {text, audio_path})
    ↓
Backend: return Ok(text)
    ↓ (параллельно)
Frontend: Promise resolves → setText(result)
Frontend: listen('recognition-completed') → setText(text)
```

## События

### `processing-finished`
- **Источник:** `src-tauri/src/audio/worker.rs`
- **Payload:** `[string, string]` — `[prePath, postPath]`
- **Назначение:** Уведомляет frontend что WAV файлы сохранены и готовы к распознаванию
- **Слушатель:** `src/components/RecordButton/useRecord.ts`

### `recognition-completed`
- **Источник:** `src-tauri/src/commands/recognition.rs`
- **Payload:** `RecognitionResult { text: string, audio_path: string }`
- **Назначение:** Уведомляет frontend что распознавание завершено
- **Слушатель:** `src/components/RecordButton/useRecord.ts`
- **Примечание:** Дублирует результат из promise, но гарантирует event-driven уведомление

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
