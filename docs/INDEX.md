# Документация Voice Input App

Полная документация проекта с описанием архитектуры, аудиопайплайна и руководств по разработке.

## 🎯 Быстрые ссылки

**Первый запуск?** → [README.md](../README.md#быстрый-старт)  
**Проблемы?** → [FAQ.md](FAQ.md)  
**Как работает аудио?** → [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md)  
**Модели Whisper?** → [WHISPER_MODELS.md](WHISPER_MODELS.md)  

## 📖 Документы по темам

### 🔊 Аудио и распознавание

| Документ | Содержание | Для кого |
|----------|-----------|----------|
| **[AUDIO_PIPELINE.md](AUDIO_PIPELINE.md)** | <br/>• Полный цикл обработки аудио<br/>• Адаптивное усиление (gain)<br/>• Noise gate и фильтрование<br/>• Параметры и их назначение<br/>• Модули: capture, processor, whisper<br/> | Разработчики,<br/>интересующиеся<br/>как работает аудио |
| **[WHISPER_MODELS.md](WHISPER_MODELS.md)** | <br/>• Выбор и сравнение моделей<br/>• Установка (tiny, base, small, large)<br/>• Поддерживаемые языки<br/>• Время обработки и точность<br/>• Интеграция в приложение<br/>• Troubleshooting<br/> | Пользователи,<br/>разработчики<br/>выбирающие модель |
| **[event-flow.md](event-flow.md)** | <br/>• Архитектура событий Tauri<br/>• Поток данных frontend ↔ backend<br/>• Events: processing-finished, recognition-completed<br/>• Диаграммы и временные рамки<br/>• Frontend слушатели<br/> | Разработчики<br/>интегрирующие<br/>новые функции |

### ❓ Помощь и вопросы

| Документ | Содержание | Для кого |
|----------|-----------|----------|
| **[FAQ.md](FAQ.md)** | <br/>• Установка и запуск<br/>• Качество распознавания<br/>• Проблемы с аудио<br/>• Производительность<br/>• Разработка и тестирование<br/>• Debugging и логирование<br/> | Все:<br/>пользователи,<br/>разработчики |

### 📚 Основная информация

| Документ | Содержание | Для кого |
|----------|-----------|----------|
| **[README.md](../README.md)** | <br/>• Описание приложения<br/>• Быстрый старт<br/>• Установка зависимостей<br/>• Структура проекта<br/>• Быстрый запуск (dev & build)<br/> | Все начиная<br/>с проекта |

## 🏗️ Архитектура на одной странице

```
┌─────────────────────────────────────────────────────────────┐
│                      РЕЧЬ В МИКРОФОН                         │
└────────────────┬────────────────────────────────────────────┘
                 │
                 ↓ (CPAL захват)
         ┌───────────────┐
         │  CAPTURE      │ [capture.rs]
         │  16 кГц, F32  │
         └───────┬───────┘
                 │
                 ↓ (обработка в реальном времени)
         ┌───────────────┐
         │ PROCESSOR     │ [processor.rs]
         │ Adaptive Gain │
         │ Noise Gate    │
         └───────┬───────┘
                 │
                 ↓ (background worker)
         ┌───────────────────┐
         │ WORKER            │ [worker.rs]
         │ Сохранить WAV     │
         │ emit event        │
         └───────┬───────────┘
                 │
                 ↓ (Frontend listener)
         ┌──────────────────────┐
         │ RECOGNITION TRIGGER  │ [useRecognition.ts]
         │ invoke recognize()   │
         └───────┬──────────────┘
                 │
                 ↓ (Whisper inference)
         ┌───────────────────┐
         │ WHISPER INFERENCE │ [whisper.rs]
         │ 99 языков         │
         │ Базовая модель    │
         └───────┬───────────┘
                 │
                 ↓ (очистка результата)
         ┌──────────────────────┐
         │ POSTPROCESSING       │ [postprocess.rs]
         │ Удаление маркеров    │
         │ Очистка текста       │
         └───────┬──────────────┘
                 │
                 ↓ (emit event + promise resolve)
         ┌──────────────────────┐
         │ FRONTEND DISPLAY     │ [TextDisplay.tsx]
         │ Результат в UI       │
         └──────────────────────┘
```

## 🔍 Поиск по функциям

### Я хочу ...

| Задача | Документ | Файл | Функция |
|--------|----------|------|---------|
| Установить приложение | [README.md](../README.md#быстрый-старт) | — | — |
| Выбрать модель Whisper | [WHISPER_MODELS.md](WHISPER_MODELS.md) | `src-tauri/src/recognition/models.rs` | `initialize_model()` |
| Понять аудиопайплайн | [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md) | `src-tauri/src/audio/` | `process_audio()` |
| Изменить gain/threshold | [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md#параметры-обработки) | `src-tauri/src/audio/processor.rs` | `process_audio()` |
| Добавить фильтр | [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md#-обработка-сигнала) | `src-tauri/src/audio/processor.rs` | `process_and_filter()` |
| Модифицировать текст | [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md#-постобработка) | `src-tauri/src/recognition/postprocess.rs` | `cleanup_recognized_text()` |
| Добавить язык | [WHISPER_MODELS.md](WHISPER_MODELS.md#поддерживаемые-языки) | `src-tauri/src/recognition/whisper.rs` | `recognize()` |
| Понять события | [event-flow.md](event-flow.md) | `src-tauri/src/audio/worker.rs` | `emit()` |
| Отладить проблемы | [FAQ.md](FAQ.md#разработка) | `src-tauri/src/` | — |
| Собрать продакшен | [FAQ.md](FAQ.md#как-собрать-для-продакшена) | `package.json` | `tauri build` |

## 📂 Структура документов

```
docs/
├── README.md               ← Этот файл (навигация)
├── AUDIO_PIPELINE.md       ← Архитектура аудио
├── WHISPER_MODELS.md       ← Модели и установка
├── event-flow.md           ← События и асинхронность
└── FAQ.md                  ← Вопросы и ответы
```

## 🔗 Перекрестные ссылки

### AUDIO_PIPELINE.md ссылается на:
- WHISPER_MODELS.md (выбор модели)
- event-flow.md (как события доходят до frontend)
- FAQ.md (troubleshooting аудио)

### WHISPER_MODELS.md ссылается на:
- AUDIO_PIPELINE.md (как аудио попадает в Whisper)
- FAQ.md (проблемы с моделями)

### event-flow.md ссылается на:
- AUDIO_PIPELINE.md (обработка аудио)
- WHISPER_MODELS.md (модель и язык)
- FAQ.md (debugging событий)

### FAQ.md ссылается на:
- README.md (быстрый старт)
- AUDIO_PIPELINE.md (параметры обработки)
- WHISPER_MODELS.md (выбор модели)
- event-flow.md (архитектура)

## 💡 Практические примеры

### Пример 1: "Мне нужна лучшая точность"

1. Прочитайте [WHISPER_MODELS.md → Доступные модели](WHISPER_MODELS.md#доступные-модели)
2. Выберите small или medium
3. Следуйте [WHISPER_MODELS.md → Установка моделей](WHISPER_MODELS.md#установка-моделей)
4. Отредактируйте [src-tauri/src/recognition/models.rs](../src-tauri/src/recognition/models.rs)
5. Пересобрите: `npm run tauri dev`

### Пример 2: "Аудио звучит неправильно"

1. Проверьте [FAQ.md → Проблемы с аудио](FAQ.md#проблемы-с-аудио)
2. Смотрите [AUDIO_PIPELINE.md → Обработка сигнала](AUDIO_PIPELINE.md#-обработка-сигнала)
3. Читайте логи: смотрите "Input RMS" и "Output RMS" в консоли
4. Если нужно отрегулировать: [AUDIO_PIPELINE.md → Параметры обработки](AUDIO_PIPELINE.md#параметры-обработки)

### Пример 3: "Хочу добавить новую функцию"

1. Определите где: [AUDIO_PIPELINE.md → Этапы обработки](AUDIO_PIPELINE.md#этапы-обработки) или [event-flow.md](event-flow.md)
2. Найдите файл в соответствующем модуле
3. Добавьте функцию
4. Зарегистрируйте как Tauri command если нужен frontend доступ
5. Вызовите из frontend: [event-flow.md → Frontend](event-flow.md#3-распознавание-речи)

## 🚀 Следующие шаги

**Новичок в проекте?**
1. Начните с [README.md](../README.md)
2. Установите и запустите: [README.md → Быстрый старт](../README.md#быстрый-старт)
3. Прочитайте [AUDIO_PIPELINE.md](AUDIO_PIPELINE.md) — общее понимание
4. Смотрите [event-flow.md](event-flow.md) — как части взаимодействуют

**Есть проблема?**
1. Смотрите [FAQ.md](FAQ.md) — ваш вопрос вероятно там
2. Если нет — создайте issue на GitHub с описанием

**Хотите разработку?**
1. Определитесь где менять ([AUDIO_PIPELINE.md](AUDIO_PIPELINE.md) или [event-flow.md](event-flow.md))
2. Смотрите соответствующий документ для деталей
3. Отредактируйте нужный файл
4. Пересобрите: `npm run tauri dev`
5. Тестируйте!

---

**Версия документации:** v1.0  
**Последнее обновление:** January 2026  
**Актуальное состояние:** ✅ Актуально для текущей версии приложения
