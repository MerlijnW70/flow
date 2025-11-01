# Railway Deployment Skill

A comprehensive skill for deploying Rust applications to Railway with automated configuration, diagnostics, and optimization.

## What's Included

### Scripts (`scripts/`)

1. **generate_railway_config.py** - Automated railway.toml generator
   - Presets: web, api, worker, custom
   - Generates optimized configurations for different Rust application types
   - Usage: `python scripts/generate_railway_config.py web > railway.toml`

2. **diagnose_deployment.py** - Pre-deployment diagnostic tool
   - Checks Cargo.toml configuration
   - Validates port binding
   - Identifies common Railway deployment issues
   - Usage: `python scripts/diagnose_deployment.py`

### References (`references/`)

1. **optimizations.md** - Railway-specific Rust optimizations
   - Build optimization strategies
   - Docker/Nixpacks layer caching
   - Runtime performance tuning
   - Memory and database optimization
   - Cold start reduction techniques

2. **troubleshooting.md** - Comprehensive troubleshooting guide
   - Common build failures and solutions
   - Runtime error diagnosis
   - Database connection issues
   - Performance problem resolution
   - Quick fixes checklist

3. **environment-variables.md** - Environment variable management patterns
   - Structured configuration patterns
   - Railway built-in variables
   - Secret management
   - Feature flag patterns
   - Validation strategies

### Assets (`assets/`)

1. **railway-web.toml** - Web server configuration template
2. **railway-api.toml** - REST API service configuration template
3. **railway-worker.toml** - Background worker configuration template
4. **Dockerfile** - Multi-stage Rust Dockerfile optimized for Railway
5. **.dockerignore** - Railway-optimized Docker ignore patterns

## Key Features

- **Automated Configuration**: Generate railway.toml files for different Rust app types
- **Pre-deployment Diagnostics**: Catch common issues before deployment
- **Comprehensive Troubleshooting**: Detailed solutions for Railway deployment problems
- **Railway-Specific Optimizations**: Performance tuning specifically for Railway platform
- **Environment Management**: Patterns for managing Railway environment variables
- **Template Files**: Ready-to-use configuration templates

## When to Use This Skill

Use this skill when:
- Deploying Rust applications to Railway
- Configuring Railway services for Rust
- Debugging Railway deployment issues
- Optimizing Rust apps for Railway platform
- Managing Railway environment variables
- Setting up Railway CI/CD for Rust projects

## Critical Requirements for Railway

1. **Port Binding**: Always bind to `0.0.0.0:$PORT`
2. **Health Checks**: Implement health check endpoints
3. **Binary Configuration**: Ensure Cargo.toml binary names match railway.toml
4. **Environment Variables**: Use Railway dashboard or railway.toml, not .env files
5. **Build Optimization**: Configure release profile for optimal performance

## Quick Start

1. Generate railway.toml: `python scripts/generate_railway_config.py web`
2. Run diagnostics: `python scripts/diagnose_deployment.py`
3. Fix any reported issues
4. Deploy to Railway
5. If issues occur, consult troubleshooting guide

## Installation

Install this skill in Claude by uploading the `railway-deployment.skill` file through the Claude skills interface.

Once installed, Claude will automatically use this skill when you ask about:
- Railway deployment for Rust
- Debugging Railway issues
- Optimizing Rust apps for Railway
- Configuring Railway services
