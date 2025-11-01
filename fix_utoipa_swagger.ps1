<#
===============================================================================
 Ultimate Rust Build Fix  utoipa-swagger-ui Offline Build
===============================================================================
Fixes:
   "curl schannel CRYPT_E_NO_REVOCATION_CHECK"
   "failed to run custom build command for utoipa-swagger-ui"
   "exit code 101"

What it does:
   Edits both Cargo.toml files to disable utoipa-swagger-ui's download feature
   Adds [patch.crates-io] override to enforce globally
   Deletes Cargo registry caches and old build artifacts
   Re-fetches dependencies and verifies features
   Builds and runs your Axum API (vibe-api)
===============================================================================
#>

Write-Host " Starting full utoipa-swagger-ui offline build fix..." -ForegroundColor Cyan

# -------------------------------------------------------------------------
# Paths
# -------------------------------------------------------------------------
$workspace = "C:\dev\flow"
$apiManifest = "$workspace\apps\api\Cargo.toml"
$rootManifest = "$workspace\Cargo.toml"
$tmpDir = "C:\cargo_tmp"

# -------------------------------------------------------------------------
# Step 1  Fix dependency line in apps/api/Cargo.toml
# -------------------------------------------------------------------------
Write-Host " Ensuring utoipa-swagger-ui dependency is correct..."
$depLine = 'utoipa-swagger-ui = { version = "8.1.0", default-features = false, features = ["axum"] }'
$content = Get-Content $apiManifest -Raw
if ($content -notmatch 'utoipa-swagger-ui') {
    Write-Host " Adding missing utoipa-swagger-ui dependency..."
    Add-Content $apiManifest "`n$depLine"
} else {
    $newContent = $content -replace 'utoipa-swagger-ui\s*=.*', $depLine
    $newContent | Set-Content $apiManifest -Encoding UTF8
    Write-Host " Updated utoipa-swagger-ui dependency line"
}

# -------------------------------------------------------------------------
# Step 2  Add [patch.crates-io] override in workspace root
# -------------------------------------------------------------------------
Write-Host " Ensuring global [patch.crates-io] override..."
$patchLine = 'utoipa-swagger-ui = { version = "8.1.0", default-features = false, features = ["axum"] }'
$rootContent = Get-Content $rootManifest -Raw
if ($rootContent -notmatch '\[patch\.crates-io\]') {
    Add-Content $rootManifest "`n[patch.crates-io]`n$patchLine"
    Write-Host " Added [patch.crates-io] override section"
} else {
    $newRoot = $rootContent -replace 'utoipa-swagger-ui\s*=.*', $patchLine
    $newRoot | Set-Content $rootManifest -Encoding UTF8
    Write-Host " Updated existing [patch.crates-io] section"
}

# -------------------------------------------------------------------------
# Step 3  Full cleanup of cache and targets
# -------------------------------------------------------------------------
Write-Host " Cleaning Cargo registry, cache, and build artifacts..."
cargo clean
Remove-Item -Recurse -Force "$env:USERPROFILE\.cargo\registry\src" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force "$env:USERPROFILE\.cargo\registry\cache" -ErrorAction SilentlyContinue
if (Test-Path $tmpDir) { Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue }

# -------------------------------------------------------------------------
# Step 4  Recreate target directory
# -------------------------------------------------------------------------
if (-not (Test-Path $tmpDir)) {
    New-Item -ItemType Directory -Path $tmpDir | Out-Null
    Write-Host " Created $tmpDir"
}

# -------------------------------------------------------------------------
# Step 5  Fetch fresh dependencies
# -------------------------------------------------------------------------
Write-Host " Fetching fresh dependencies..."
cargo fetch

# -------------------------------------------------------------------------
# Step 6  Verify that 'download' feature is disabled
# -------------------------------------------------------------------------
Write-Host " Checking Cargo features..."
$features = cargo tree -e features | Select-String "utoipa-swagger-ui"
Write-Host $features
if ($features -match "download") {
    Write-Host "  'download' feature still detected  enforcing override globally" -ForegroundColor Yellow
    Add-Content $rootManifest "`n[patch.crates-io]`n$patchLine"
} else {
    Write-Host " Confirmed: 'download' feature disabled" -ForegroundColor Green
}

# -------------------------------------------------------------------------
# Step 7  Build project cleanly
# -------------------------------------------------------------------------
Write-Host "  Building project (single-thread)..."
$build = Start-Process cargo -ArgumentList "build", "-j1" -NoNewWindow -PassThru -Wait

if ($build.ExitCode -eq 0) {
    Write-Host " Build succeeded!" -ForegroundColor Green
    Write-Host " Starting API server..."
    Start-Process cargo -ArgumentList "run", "-p", "vibe-api" -NoNewWindow
} else {
    Write-Host " Build failed with exit code $($build.ExitCode)" -ForegroundColor Red
    Write-Host " Run 'cargo tree -e features | Select-String utoipa-swagger-ui' to verify no 'download' feature remains."
}

Write-Host " Artifacts in: $tmpDir"
Write-Host " Fix process completed."
