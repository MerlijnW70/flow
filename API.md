# vibe-api API Documentation

Base URL: `http://localhost:3000` (development) or your Railway URL (production)

## Authentication

Most endpoints require a JWT token. Include it in the Authorization header:

```http
Authorization: Bearer <your_access_token>
```

## Endpoints

### Health & Monitoring

#### GET /health
Health check endpoint for monitoring.

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime_seconds": 3600
  }
}
```

#### GET /ready
Readiness check for load balancers.

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "ready": true,
    "checks": [
      {
        "name": "api",
        "healthy": true
      }
    ]
  }
}
```

#### GET /metrics
Prometheus metrics endpoint.

**Response:** 200 OK (Prometheus format)

---

### Authentication

#### POST /auth/register
Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "name": "John Doe"
}
```

**Validation:**
- Email must be valid format
- Password must be at least 8 characters
- Name must be 2-100 characters

**Response:** 201 Created
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGc...",
    "refresh_token": "eyJhbGc...",
    "token_type": "Bearer",
    "expires_in": 86400,
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe"
    }
  }
}
```

#### POST /auth/login
Login with existing credentials.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGc...",
    "refresh_token": "eyJhbGc...",
    "token_type": "Bearer",
    "expires_in": 86400,
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe"
    }
  }
}
```

#### POST /auth/refresh
Refresh access token using refresh token.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGc..."
}
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGc...",
    "refresh_token": "eyJhbGc...",
    "token_type": "Bearer",
    "expires_in": 86400,
    "user": {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user@example.com",
      "name": "John Doe"
    }
  }
}
```

---

### Users

All user endpoints require authentication.

#### GET /users/me
Get current user profile.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "name": "John Doe",
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z",
    "last_login": "2025-01-01T12:00:00Z"
  }
}
```

#### PATCH /users/me
Update current user profile.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "name": "Jane Doe"
}
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "email": "user@example.com",
    "name": "Jane Doe",
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-02T00:00:00Z",
    "last_login": "2025-01-01T12:00:00Z"
  }
}
```

#### PUT /users/me/password
Change password.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "current_password": "OldPass123!",
  "new_password": "NewPass123!"
}
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": null,
  "message": "Password changed successfully"
}
```

#### DELETE /users/me
Delete current user account.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Response:** 204 No Content

#### GET /users
List all users (paginated).

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `page` (optional, default: 1) - Page number
- `per_page` (optional, default: 20) - Items per page

**Response:** 200 OK
```json
{
  "success": true,
  "data": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "email": "user1@example.com",
      "name": "User One",
      "created_at": "2025-01-01T00:00:00Z",
      "updated_at": "2025-01-01T00:00:00Z",
      "last_login": "2025-01-01T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

---

### AI (Feature: ai)

AI endpoints require authentication.

#### POST /ai/chat
Send a chat message to AI.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "message": "Hello, how are you?",
  "provider": "openai",
  "model": "gpt-4",
  "temperature": 0.7,
  "max_tokens": 2000,
  "system_prompt": "You are a helpful assistant."
}
```

**Fields:**
- `message` (required) - User message
- `provider` (optional, default: "openai") - AI provider: "openai", "anthropic", or "local"
- `model` (optional) - Model name (provider-specific)
- `temperature` (optional) - Creativity level (0.0-2.0)
- `max_tokens` (optional) - Maximum response length
- `system_prompt` (optional) - System instructions

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "response": "Hello! I'm doing well, thank you for asking.",
    "provider": "openai",
    "model": "gpt-4",
    "tokens_used": 25
  }
}
```

#### POST /ai/chat/stream
Stream AI responses (Server-Sent Events).

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request Body:** Same as `/ai/chat`

**Response:** 200 OK (SSE stream)
```
data: {"content": "Hello", "done": false}

data: {"content": "! I'm", "done": false}

data: {"content": " doing well", "done": true}
```

#### POST /ai/embeddings
Generate text embeddings (OpenAI only).

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "text": "Generate embeddings for this text",
  "model": "text-embedding-3-small"
}
```

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "embedding": [0.123, -0.456, 0.789, ...],
    "model": "text-embedding-3-small",
    "dimensions": 1536
  }
}
```

---

### Storage (Feature: storage)

File upload endpoints require authentication.

#### POST /storage/upload
Upload a file.

**Headers:**
```http
Authorization: Bearer <access_token>
Content-Type: multipart/form-data
```

**Form Data:**
- `file` (required) - File to upload

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "file_id": "123e4567-e89b-12d3-a456-426614174000",
    "file_name": "document.pdf",
    "file_size": 1048576,
    "content_type": "application/pdf",
    "url": "https://bucket.s3.amazonaws.com/uploads/..."
  }
}
```

#### GET /storage/presigned-upload
Get presigned URL for direct upload.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `file_name` (required) - Name of file to upload
- `content_type` (required) - MIME type
- `expires_in` (optional, default: 3600) - URL expiry in seconds

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "url": "https://bucket.s3.amazonaws.com/...",
    "expires_in_seconds": 3600
  }
}
```

#### GET /storage/presigned-download/:file_id
Get presigned URL for download.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `file_name` (required) - Original file name
- `expires_in` (optional, default: 3600) - URL expiry in seconds

**Response:** 200 OK
```json
{
  "success": true,
  "data": {
    "url": "https://bucket.s3.amazonaws.com/...",
    "expires_in_seconds": 3600
  }
}
```

#### DELETE /storage/:file_id
Delete a file.

**Headers:**
```http
Authorization: Bearer <access_token>
```

**Query Parameters:**
- `file_name` (required) - Original file name

**Response:** 204 No Content

---

### WebSocket (Feature: websocket)

#### GET /ws
Establish WebSocket connection.

**Query Parameters:**
- `user_id` (optional) - Associate connection with user

**Connection URL:**
```
ws://localhost:3000/ws?user_id=123e4567-e89b-12d3-a456-426614174000
```

**Message Types:**

Ping/Pong:
```json
{"type": "ping"}
```

Text message:
```json
{
  "type": "text",
  "content": "Hello, world!"
}
```

Join room:
```json
{
  "type": "join",
  "room": "chat-room-1"
}
```

Leave room:
```json
{
  "type": "leave",
  "room": "chat-room-1"
}
```

Broadcast to room:
```json
{
  "type": "broadcast",
  "room": "chat-room-1",
  "content": "Message to all in room"
}
```

---

## Error Responses

All errors follow a consistent format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message"
  }
}
```

### Error Codes

| Code | Status | Description |
|------|--------|-------------|
| `AUTHENTICATION_ERROR` | 401 | Invalid or expired token |
| `AUTHORIZATION_ERROR` | 403 | Insufficient permissions |
| `VALIDATION_ERROR` | 400 | Invalid input data |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource already exists |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected error |

### Example Error Response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation error: email: Invalid email address"
  }
}
```

---

## Rate Limiting

API endpoints are rate-limited to prevent abuse. Current limits:
- 100 requests per second per IP address

When rate limit is exceeded:
- **Status:** 429 Too Many Requests
- **Retry-After header:** Seconds until reset

---

## Versioning

Currently API version 1.0. Future versions will use URL versioning:
- `/v1/users/me`
- `/v2/users/me`

---

## CORS

Allowed origins are configured via `CORS_ORIGINS` environment variable.

Default (development): `http://localhost:3000`

---

## Examples

### cURL Examples

Register:
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!","name":"Test User"}'
```

Login:
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!"}'
```

Get profile:
```bash
curl -X GET http://localhost:3000/users/me \
  -H "Authorization: Bearer <token>"
```

### JavaScript/TypeScript Examples

```typescript
// Register
const response = await fetch('http://localhost:3000/auth/register', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    email: 'test@example.com',
    password: 'Test123!',
    name: 'Test User'
  })
});

const data = await response.json();
const accessToken = data.data.access_token;

// Get profile
const profile = await fetch('http://localhost:3000/users/me', {
  headers: { 'Authorization': `Bearer ${accessToken}` }
});
```

---

## Support

For API support or questions:
- GitHub Issues: [your-repo/issues](https://github.com/yourusername/vibe-api/issues)
- Documentation: See README.md
