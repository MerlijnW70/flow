# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-01

### Added
- Initial project structure with modular architecture
- Authentication module with JWT and Argon2 password hashing
- User management module (CRUD operations)
- AI integration module with OpenAI, Anthropic, and local model support
- S3-compatible storage module for file uploads
- WebSocket module for real-time communication
- Background jobs module with cron scheduling
- Prometheus metrics and health check endpoints
- Rate limiting middleware with Governor
- CORS support
- Database migrations for PostgreSQL
- Comprehensive error handling with custom error types
- Input validation with validator crate
- Structured logging with tracing
- Docker support with multi-stage builds
- Railway deployment configuration
- CI/CD with GitHub Actions
- Comprehensive documentation (README, API, Architecture, Contributing)
- Setup scripts for Linux/macOS and Windows
- Integration test framework

### Security
- JWT-based stateless authentication
- Argon2 password hashing
- SQL injection prevention with SQLx parameterized queries
- Rate limiting to prevent abuse
- Input validation on all endpoints

## [0.0.1] - 2025-01-01

### Added
- Project initialization
- Basic project structure

[Unreleased]: https://github.com/yourusername/vibe-api/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/vibe-api/releases/tag/v0.1.0
[0.0.1]: https://github.com/yourusername/vibe-api/releases/tag/v0.0.1
