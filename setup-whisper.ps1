# 🔧 Скрипт установки зависимостей для Whisper.cpp

Write-Host "🎯 Установка зависимостей для Whisper.cpp..." -ForegroundColor Cyan
Write-Host ""

# Проверяем права администратора
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

# === 1. Установка LLVM ===
Write-Host "📦 Шаг 1/3: Проверка LLVM..." -ForegroundColor Yellow
$llvmPath = "C:\Program Files\LLVM\bin"

if (Test-Path $llvmPath) {
    Write-Host "✅ LLVM уже установлен" -ForegroundColor Green
} else {
    Write-Host "⚙️  Установка LLVM..." -ForegroundColor Yellow
    winget install LLVM.LLVM --silent
    Write-Host "✅ LLVM установлен" -ForegroundColor Green
}

# === 2. Установка CMake ===
Write-Host ""
Write-Host "📦 Шаг 2/3: Проверка CMake..." -ForegroundColor Yellow
$cmakePath = "C:\Program Files\CMake\bin"

if (Test-Path $cmakePath) {
    Write-Host "✅ CMake уже установлен" -ForegroundColor Green
} else {
    Write-Host "⚙️  Установка CMake..." -ForegroundColor Yellow
    winget install Kitware.CMake --silent
    Write-Host "✅ CMake установлен" -ForegroundColor Green
}

# === 3. Обновление PATH ===
Write-Host ""
Write-Host "📦 Шаг 3/3: Настройка переменных окружения..." -ForegroundColor Yellow

# Обновляем PATH для текущей сессии
$env:Path = "$llvmPath;$cmakePath;" + $env:Path

# Проверяем текущий PATH
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")

$needsUpdate = $false
if ($currentPath -notlike "*$llvmPath*") {
    $needsUpdate = $true
}
if ($currentPath -notlike "*$cmakePath*") {
    $needsUpdate = $true
}

if ($needsUpdate) {
    Write-Host "⚠️  Требуется добавить LLVM и CMake в PATH" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Выберите способ:" -ForegroundColor Cyan
    Write-Host "  1) Автоматически (требуется перезапуск терминала)" -ForegroundColor White
    Write-Host "  2) Показать инструкцию для ручной настройки" -ForegroundColor White
    Write-Host "  3) Пропустить (использовать только для текущей сессии)" -ForegroundColor White
    Write-Host ""
    
    $choice = Read-Host "Ваш выбор (1-3)"
    
    switch ($choice) {
        "1" {
            try {
                # Получаем текущий PATH пользователя
                $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
                
                # Добавляем пути, если их еще нет
                if ($userPath -notlike "*$llvmPath*") {
                    $userPath += ";$llvmPath"
                }
                if ($userPath -notlike "*$cmakePath*") {
                    $userPath += ";$cmakePath"
                }
                
                # Устанавливаем новый PATH
                [Environment]::SetEnvironmentVariable("Path", $userPath, "User")
                
                Write-Host "✅ PATH обновлен" -ForegroundColor Green
                Write-Host "⚠️  Пожалуйста, ПЕРЕЗАПУСТИТЕ терминал/IDE для применения изменений" -ForegroundColor Yellow
            }
            catch {
                Write-Host "❌ Ошибка обновления PATH: $_" -ForegroundColor Red
                Write-Host "Попробуйте вариант 2 (ручная настройка)" -ForegroundColor Yellow
            }
        }
        "2" {
            Write-Host ""
            Write-Host "📋 Ручная настройка PATH:" -ForegroundColor Cyan
            Write-Host "1. Откройте 'Система' -> 'Дополнительные параметры системы'" -ForegroundColor White
            Write-Host "2. Нажмите 'Переменные среды'" -ForegroundColor White
            Write-Host "3. В разделе 'Переменные пользователя' найдите 'Path'" -ForegroundColor White
            Write-Host "4. Нажмите 'Изменить' и добавьте следующие пути:" -ForegroundColor White
            Write-Host "   - $llvmPath" -ForegroundColor Yellow
            Write-Host "   - $cmakePath" -ForegroundColor Yellow
            Write-Host "5. Нажмите OK и перезапустите терминал/IDE" -ForegroundColor White
            Write-Host ""
        }
        "3" {
            Write-Host "⚠️  PATH обновлен только для текущей сессии" -ForegroundColor Yellow
            Write-Host "После перезапуска терминала нужно будет запустить этот скрипт снова" -ForegroundColor Yellow
        }
    }
} else {
    Write-Host "✅ PATH уже содержит нужные пути" -ForegroundColor Green
}

# === Проверка ===
Write-Host ""
Write-Host "🔍 Проверка установки..." -ForegroundColor Cyan

try {
    $clangVersion = & clang --version 2>&1 | Select-Object -First 1
    Write-Host "✅ LLVM: $clangVersion" -ForegroundColor Green
}
catch {
    Write-Host "❌ LLVM не найден в PATH" -ForegroundColor Red
    Write-Host "   Перезапустите терминал или добавьте вручную" -ForegroundColor Yellow
}

try {
    $cmakeVersion = & cmake --version 2>&1 | Select-Object -First 1
    Write-Host "✅ CMake: $cmakeVersion" -ForegroundColor Green
}
catch {
    Write-Host "❌ CMake не найден в PATH" -ForegroundColor Red
    Write-Host "   Перезапустите терминал или добавьте вручную" -ForegroundColor Yellow
}

# === Следующие шаги ===
Write-Host ""
Write-Host "🎉 Установка завершена!" -ForegroundColor Green
Write-Host ""
Write-Host "📝 Следующие шаги:" -ForegroundColor Cyan
Write-Host "1. Скачайте модель Whisper:" -ForegroundColor White
Write-Host "   mkdir models -ErrorAction SilentlyContinue" -ForegroundColor Gray
Write-Host "   Invoke-WebRequest -Uri 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin' -OutFile 'models\ggml-base.bin'" -ForegroundColor Gray
Write-Host ""
Write-Host "2. Соберите проект:" -ForegroundColor White
Write-Host "   cargo check --manifest-path src-tauri/Cargo.toml" -ForegroundColor Gray
Write-Host ""
Write-Host "3. Запустите приложение:" -ForegroundColor White
Write-Host "   npm run tauri dev" -ForegroundColor Gray
Write-Host ""
Write-Host "📚 Подробнее см. WHISPER_SETUP.md" -ForegroundColor Cyan
