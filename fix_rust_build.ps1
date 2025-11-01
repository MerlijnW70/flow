<#
Fix Rust build issues on Windows:
   Clears invalid [target.*] sections
   Creates .cargo/config.toml with linker/env vars
   Forces Swagger UI offline mode
   Installs rust-lld if missing
   Cleans + rebuilds workspace
#>

Write-Host " Starting Rust workspace fix..." -ForegroundColor Cyan

# --- Paths --------------------------------------------------------------------
$workspace = "C:\dev\flow"
$configDir = "$workspace\.cargo"
$configFile = "$configDir\config.toml"
$tmpDir = "C:\cargo_tmp"

# --- Step 1: Ensure target dir exists -----------------------------------------
if (-Not (Test-Path $tmpDir)) {
    New-Item -ItemType Directory -Path $tmpDir | Out-Null
}
Write-Host " Using target dir: $tmpDir"

# --- Step 2: Clean Cargo.toml -------------------------------------------------
$manifest = "$workspace\Cargo.toml"
if (Test-Path $manifest) {
    $content = Get-Content $manifest -Raw
    $newContent = $content -replace '(\[target[^\]]*\][\s\S]*)', ''
    $newContent | Set-Content $manifest -Encoding UTF8
    Write-Host " Cleaned invalid [target.*] section in workspace Cargo.toml"
}

# --- Step 3: Create .cargo/config.toml ----------------------------------------
if (-Not (Test-Path $configDir)) {
    New-Item -ItemType Directory -Path $configDir | Out-Null
}

@"
[build]
target-dir = "C:\\cargo_tmp"

[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"
rustflags = ["-C", "link-arg=/INCREMENTAL:NO"]

[env]
SWAGGER_UI_OFFLINE = "true"
SWAGGER_UI_DOWNLOAD_USE_REQWEST = "true"
"@ | Set-Content $configFile -Encoding UTF8

Write-Host "  Created Cargo config file at $configFile"

# --- Step 4: Grant permissions ------------------------------------------------
icacls $tmpDir /grant "$env:USERNAME:F" /T | Out-Null

# --- Step 5: Install rust-lld if missing -------------------------------------
$rustrpath = (Get-Command rust-lld.exe -ErrorAction SilentlyContinue)
if (-not $rustrpath) {
    Write-Host " Installing LLVM tools (rust-lld)..."
    rustup component add llvm-tools-preview
} else {
    Write-Host " rust-lld found at: $($rustrpath.Source)"
}

# --- Step 6: Build cleanly ----------------------------------------------------
Set-Location $workspace
Write-Host " Cleaning project..."
cargo clean

Write-Host "  Building project (single thread)..."
cargo build -j1

Write-Host ""
Write-Host " Done! Your Rust build should now succeed without file locks or Swagger UI TLS errors." -ForegroundColor Green
Write-Host " Build artifacts: $tmpDir"
