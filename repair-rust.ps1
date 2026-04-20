$ErrorActionPreference = "Stop"

function Write-Step($msg) {
    Write-Host ""
    Write-Host "==> $msg" -ForegroundColor Cyan
}

function Ensure-Dir($path) {
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Force -Path $path | Out-Null
    }
}

$UserProfileDir = $env:USERPROFILE
$CargoDir = Join-Path $UserProfileDir ".cargo"
$RustupDir = Join-Path $UserProfileDir ".rustup"
$CargoBin = Join-Path $CargoDir "bin"
$RustupExe = Join-Path $CargoBin "rustup.exe"
$RustupInitExe = Join-Path (Get-Location) "rustup-init.exe"

Write-Step "Environment"
Write-Host "USERPROFILE = $UserProfileDir"
Write-Host "CARGO_HOME  = $CargoDir"
Write-Host "RUSTUP_HOME = $RustupDir"

Write-Step "Kill lingering rust processes"
Get-Process cargo, rustc, rustup, clippy-driver, rust-analyzer -ErrorAction SilentlyContinue |
    Stop-Process -Force -ErrorAction SilentlyContinue

Write-Step "Prepare base directories"
Ensure-Dir $CargoDir
Ensure-Dir $RustupDir

Write-Step "Repair permissions"
icacls $CargoDir /inheritance:e /grant "${env:USERNAME}:(OI)(CI)F" /T | Out-Null
icacls $RustupDir /inheritance:e /grant "${env:USERNAME}:(OI)(CI)F" /T | Out-Null

Write-Step "Clean broken temp/download state"
Remove-Item -Recurse -Force (Join-Path $RustupDir "tmp") -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force (Join-Path $RustupDir "downloads") -ErrorAction SilentlyContinue
Ensure-Dir (Join-Path $RustupDir "tmp")
Ensure-Dir (Join-Path $RustupDir "downloads")

$DoFullReset = Read-Host "Do full reset of ~/.cargo and ~/.rustup? (y/N)"
if ($DoFullReset -match '^(y|Y)$') {
    Write-Step "Full reset requested"

    if (Test-Path $RustupExe) {
        Write-Host "Uninstalling existing rustup..."
        & $RustupExe self uninstall -y
    }

    Write-Step "Delete old rust directories"
    Remove-Item -Recurse -Force $RustupDir -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $CargoDir -ErrorAction SilentlyContinue

    Ensure-Dir $CargoDir
    Ensure-Dir $RustupDir

    icacls $CargoDir /inheritance:e /grant "${env:USERNAME}:(OI)(CI)F" /T | Out-Null
    icacls $RustupDir /inheritance:e /grant "${env:USERNAME}:(OI)(CI)F" /T | Out-Null
}

Write-Step "Find installer"
if (-not (Test-Path $RustupInitExe)) {
    Write-Host "rustup-init.exe not found in current directory."
    Write-Host "Downloading installer..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $RustupInitExe
}

Write-Step "Install Rust 1.85.0"
& $RustupInitExe -y --profile minimal --default-toolchain 1.85.0

Write-Step "Add cargo bin to current session PATH"
$env:PATH = "$CargoBin;$env:PATH"

Write-Step "Post-install verification"
Write-Host "cargo path:"
Get-Command cargo -ErrorAction SilentlyContinue | Format-List -Property Source, Name

Write-Host "rustc path:"
Get-Command rustc -ErrorAction SilentlyContinue | Format-List -Property Source, Name

Write-Host ""
Write-Host "cargo --version"
cargo --version

Write-Host ""
Write-Host "rustc --version"
rustc --version

Write-Host ""
Write-Host "rustup show"
rustup show

Write-Step "Toolchain bin contents"
$ToolchainDir = Join-Path $RustupDir "toolchains\1.85.0-x86_64-pc-windows-msvc\bin"
if (Test-Path $ToolchainDir) {
    Get-ChildItem $ToolchainDir | Select-Object Name, Length, LastWriteTime
} else {
    Write-Warning "Expected toolchain bin directory not found: $ToolchainDir"
}

Write-Step "Done"
Write-Host "If a new terminal still cannot find cargo, close PowerShell and reopen it."
