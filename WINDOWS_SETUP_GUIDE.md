# Windows Build Setup Guide

## Current Status
❌ CMake not installed
❌ NASM not installed
✅ Rust 1.90.0 installed

## Required Steps

### 1️⃣ Open PowerShell as Administrator
- Press `Win + X`
- Select "Windows PowerShell (Admin)" or "Terminal (Admin)"

### 2️⃣ Install Rust with MSVC toolchain
```powershell
rustup default stable-x86_64-pc-windows-msvc
```

### 3️⃣ Install Visual Studio Build Tools
```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e `
  --source winget `
  --override "--add Microsoft.VisualStudio.Component.VC.Tools.x86.x64"
```

**Note**: This may take 5-10 minutes. The download is ~2GB.

### 4️⃣ Install CMake
```powershell
winget install Kitware.CMake
```

### 5️⃣ Install NASM (assembler)
```powershell
winget install NASM
```

### 6️⃣ Optional: Install Perl (for OpenSSL)
```powershell
winget install StrawberryPerl
```

### 7️⃣ Restart PowerShell/Terminal
**IMPORTANT**: Close and reopen your terminal to refresh PATH environment variables.

### 8️⃣ Verify Installation
```powershell
cmake --version
# Expected: cmake version 3.x.x

nasm -v
# Expected: NASM version 2.x.x

cl
# Expected: Microsoft (R) C/C++ Optimizing Compiler Version...

rustc --version
# Expected: rustc 1.90.0 or later
```

### 9️⃣ Clean and Build
```powershell
cd C:\Users\gamin\documents\flow

# Clean previous build artifacts
cargo clean

# Build entire workspace
cargo build --workspace

# Expected output: Compiling... Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

### 🔟 Run Tests
```powershell
# Run all tests with shuffle (3 times as requested)
cargo test --workspace -- --shuffle

# Run again with different seed
cargo test --workspace --test '*' -- --shuffle

# Run a third time
cargo test --workspace --lib -- --nocapture --shuffle
```

## Alternative: Use Chocolatey (If winget fails)

If `winget` is not available, use Chocolatey:

### Install Chocolatey
```powershell
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

### Install Build Tools
```powershell
choco install cmake nasm visualstudio2022buildtools -y
```

## Troubleshooting

### Issue: "cmake not found" after installation
**Solution**: Restart your terminal or run:
```powershell
$env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
```

### Issue: "NASM not found" after installation
**Solution**: Add NASM to PATH manually:
```powershell
$env:Path += ";C:\Program Files\NASM"
```

### Issue: "linker `link.exe` not found"
**Solution**: Install Visual Studio Build Tools (step 3) and restart terminal.

### Issue: Still getting aws-lc-sys build errors
**Solution**:
1. Ensure all tools are in PATH (run step 8)
2. Run `cargo clean`
3. Try building a single package: `cargo build -p vibe-api`

## Expected Build Output

After successful setup, you should see:
```
   Compiling sqlx v0.8.2
   Compiling axum v0.7.7
   Compiling vibe-api v0.1.0 (C:\Users\gamin\documents\flow\apps\api)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 45.23s
```

## Test Execution

Phase 1 + Phase 2 tests should all pass:
```
running 45 tests
test auth::jwt::tests::test_generate_and_validate_access_token ... ok
test auth::role_guard::tests::test_require_admin_with_admin_role ... ok
test phase2_auth_test::test_signup_success ... ok
test phase2_auth_test::test_role_guard_admin_required ... ok
test phase2_auth_test::test_admin_can_list_users ... ok
...

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Next Steps After Successful Build

1. **Run migrations**:
   ```powershell
   # Ensure DATABASE_URL is set in .env
   sqlx migrate run
   ```

2. **Start the server**:
   ```powershell
   cargo run
   ```

3. **Access Swagger UI**:
   - Open browser: `http://localhost:3000/swagger-ui`
   - Test endpoints with role-based access control

4. **Test role-based auth**:
   - Register user: POST `/auth/signup` with `{"email": "user@test.com", "password": "password123", "name": "Test User"}`
   - Register admin: POST `/auth/signup` with `{"email": "admin@test.com", "password": "password123", "name": "Admin", "role": "admin"}`
   - Try accessing `/users` with each token
   - Verify user gets 403 Forbidden, admin gets 200 OK

## Summary

**Total Setup Time**: ~15-20 minutes (including downloads)
**Disk Space Required**: ~3-4 GB (Visual Studio Build Tools + dependencies)
**One-Time Setup**: These tools work for all future Rust projects on Windows

---

**After completing these steps, all Windows build issues will be resolved and you can run the full test suite!**
