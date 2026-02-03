param(
    [switch]$SkipNode,
    [switch]$SkipRust,
    [switch]$SkipLLVM,
    [switch]$SkipCMake
)

if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Error "winget not found. Install App Installer from Microsoft Store."
    exit 1
}

if (-not $SkipNode) {
    winget install OpenJS.NodeJS.LTS --accept-package-agreements --accept-source-agreements
}

if (-not $SkipRust) {
    winget install Rustlang.Rustup --accept-package-agreements --accept-source-agreements
}

if (-not $SkipLLVM) {
    winget install LLVM.LLVM --accept-package-agreements --accept-source-agreements
}

if (-not $SkipCMake) {
    winget install Kitware.CMake --accept-package-agreements --accept-source-agreements
}

Write-Host "Done. You may need to restart the terminal to refresh PATH."