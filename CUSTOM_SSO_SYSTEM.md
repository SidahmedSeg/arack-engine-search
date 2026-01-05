# Arack Custom SSO System - Single Source of Truth

**Version:** 1.0
**Last Updated:** January 5, 2025
**Status:** Production

---

## Overview

Arack uses a **custom Single Sign-On (SSO) system** that provides authentication across all subdomains (`arack.io`, `mail.arack.io`, `admin.arack.io`). This system replaces the previous Zitadel/Kratos/Hydra stack with a simpler, self-hosted solution.

### Key Characteristics

| Feature | Implementation |
|---------|---------------|
| **Authentication** | Email/Password with Argon2id hashing |
| **Tokens** | RS256 JWT (access + refresh tokens) |
| **Session Storage** | Redis with 30-day TTL |
| **Cookie** | `arack_session` on `.arack.io` domain |
| **OIDC Compatible** | Yes (for Stalwart email server) |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           CLIENTS                                    │
├─────────────────┬─────────────────┬─────────────────────────────────┤
│  arack.io       │  mail.arack.io  │  admin.arack.io                 │
│  (Main Site)    │  (Email App)    │  (Admin Dashboard)              │
│                 │                 │                                 │
│  Login Page     │  Uses SSO       │  Uses SSO                       │
│  /auth/login    │  Client Lib     │  Client Lib                     │
└────────┬────────┴────────┬────────┴────────────────┬────────────────┘
         │                 │                         │
         │     arack_session cookie (.arack.io)      │
         └─────────────────┼─────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    ACCOUNT SERVICE                                   │
│                  account.arack.io:3002                               │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ LocalAuth    │  │ JWT Service  │  │ Session      │               │
│  │ Service      │  │              │  │ Service      │               │
│  │              │  │ RS256 Keys   │  │              │               │
│  │ - Register   │  │ - Sign       │  │ - Create     │               │
│  │ - Login      │  │ - Validate   │  │ - Get        │               │
│  │ - Validate   │  │ - JWKS       │  │ - Refresh    │               │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘               │
│         │                 │                 │                        │
│         ▼                 ▼                 ▼                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐               │
│  │ PostgreSQL   │  │ RSA Keys     │  │ Redis        │               │
│  │ (Users)      │  │ /app/keys/   │  │ (Sessions)   │               │
│  └──────────────┘  └──────────────┘  └──────────────┘               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    STALWART MAIL SERVER                              │
│                    smtp.arack.io:8080                                │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  OAuth Configuration (config.toml):                                  │
│  - directory = "oidc"                                                │
│  - userinfo_url = "https://account.arack.io/userinfo"               │
│  - JWKS validation via account.arack.io/.well-known/jwks.json       │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Components

### 1. Account Service (Go)

**Location:** `account-service/`
**Port:** 3002
**Container:** `search_engine_account_service`
**URL:** `https://account.arack.io`

#### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/login` | POST | Email/password login |
| `/api/register` | POST | User registration |
| `/api/session` | GET | Get current session (requires cookie) |
| `/api/logout` | POST | Destroy session |
| `/.well-known/jwks.json` | GET | Public keys for JWT validation |
| `/.well-known/openid-configuration` | GET | OIDC discovery document |
| `/userinfo` | GET | Get user info (requires Bearer token) |
| `/oauth/token` | POST | Refresh tokens |
| `/health` | GET | Health check |

#### Configuration (Environment Variables)

```bash
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3002

# Database
DATABASE_URL=postgresql://...

# Redis
REDIS_URL=redis://...

# JWT
JWT_PRIVATE_KEY_PATH=/app/keys/private.pem
JWT_PUBLIC_KEY_PATH=/app/keys/public.pem
JWT_ISSUER=https://account.arack.io
JWT_AUDIENCE=https://arack.io
JWT_ACCESS_TTL=24h
JWT_REFRESH_TTL=168h  # 7 days

# Session
SESSION_TTL=720h  # 30 days

# Cookie
COOKIE_NAME=arack_session
COOKIE_DOMAIN=.arack.io
COOKIE_SECURE=true
COOKIE_HTTP_ONLY=true
COOKIE_SAME_SITE=lax
COOKIE_MAX_AGE=2592000  # 30 days

# Stalwart (Email provisioning)
STALWART_URL=http://stalwart:8080
STALWART_ADMIN_USER=admin
STALWART_ADMIN_PASSWORD=...
```

### 2. Frontend SSO Client (TypeScript)

**Location:** `frontend-email/src/lib/auth/sso.ts`

Used by `mail.arack.io` and other subdomains to validate sessions.

```typescript
// Key functions:
getSession()      // Check if user has active session
login(returnUrl)  // Redirect to login page
logout()          // Destroy session
getUserInfo()     // Get user details
getAccessToken()  // Get JWT for API calls
```

### 3. Login Page (SvelteKit)

**Location:** `frontend-search/src/routes/auth/login/+page.svelte`
**URL:** `https://arack.io/auth/login`

Two-step login flow (Google-style):
1. Enter email → Next
2. Enter password → Sign in

Supports `?return_url=` parameter for post-login redirect.

---

## Authentication Flow

### Login Flow

```
1. User → arack.io/auth/login

2. User enters email/password

3. Frontend → POST account.arack.io/api/login
   {
     "email": "user@example.com",
     "password": "..."
   }

4. Account Service:
   a. Validate credentials against PostgreSQL
   b. Generate JWT tokens (access + refresh)
   c. Create session in Redis
   d. Set arack_session cookie (domain: .arack.io)

5. Response:
   {
     "success": true,
     "user": { "id": "...", "email": "...", "name": "..." },
     "accessToken": "eyJ...",
     "refreshToken": "eyJ...",
     "expiresIn": 86400
   }

6. Frontend redirects to return_url (or /)
```

### Cross-Domain SSO Flow

```
1. User visits mail.arack.io (not logged in)

2. hooks.server.ts checks for arack_session cookie

3. If no cookie:
   → Redirect to arack.io/auth/login?return_url=https://mail.arack.io/inbox

4. If cookie exists:
   → Validate with account.arack.io/api/session
   → If valid: allow access
   → If invalid: clear cookie, redirect to login

5. After login at arack.io:
   → Cookie is set on .arack.io domain
   → User redirected back to mail.arack.io/inbox
   → Cookie now accessible, session valid
```

### Token Refresh Flow

```
1. Session service checks if tokens need refresh
   (when token expiry < 5 minutes away)

2. If refresh needed:
   → POST account.arack.io/oauth/token
   {
     "grant_type": "refresh_token",
     "refresh_token": "eyJ..."
   }

3. Response with new tokens:
   {
     "access_token": "eyJ...",
     "refresh_token": "eyJ...",
     "expires_in": 86400
   }

4. Session updated in Redis with new tokens
```

---

## JWT Token Structure

### Access Token Claims

```json
{
  "iss": "https://account.arack.io",
  "sub": "user-uuid",
  "aud": ["https://arack.io"],
  "exp": 1704499200,
  "iat": 1704412800,
  "nbf": 1704412800,
  "jti": "unique-token-id",
  "email": "user@example.com",
  "name": "User Name"
}
```

### Key Details

| Property | Value |
|----------|-------|
| Algorithm | RS256 |
| Key Size | 2048-bit RSA |
| Key ID | First 8 bytes of modulus (base64url) |
| Access TTL | 24 hours |
| Refresh TTL | 7 days |

---

## Session Storage (Redis)

```
Key:    session:{uuid}
Value:  {
  "id": "session-uuid",
  "user": {
    "id": "user-uuid",
    "email": "user@example.com",
    "name": "User Name"
  },
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "token_expires_at": "2025-01-06T00:00:00Z",
  "created_at": "2025-01-05T00:00:00Z",
  "last_accessed_at": "2025-01-05T12:00:00Z"
}
TTL:    30 days (refreshed on access)
```

---

## Stalwart OIDC Integration

The email server (Stalwart) uses OAuth/OIDC to authenticate users for IMAP/SMTP/JMAP access.

### Configuration (config.toml)

```toml
[storage]
directory = "oidc"

[directory.oidc]
type = "oidc"
userinfo-url = "https://account.arack.io/userinfo"
userinfo-token-header = "Authorization"
cache-ttl = "1h"

[session.auth]
directory = ["oidc", "internal"]
mechanisms = ["plain", "login", "oauthbearer"]
```

### Flow

1. Email client sends OAuth Bearer token
2. Stalwart calls `account.arack.io/userinfo` with token
3. Account service validates JWT and returns user info
4. Stalwart authenticates user for IMAP/JMAP access

---

## Database Schema

### Users Table (PostgreSQL)

```sql
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    gender VARCHAR(20),
    birth_date DATE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### Password Hashing

- Algorithm: Argon2id
- Memory: 64 MB
- Iterations: 3
- Parallelism: 2
- Salt: 16 bytes random
- Hash length: 32 bytes

---

## Security Considerations

### Cookie Security

| Attribute | Value | Purpose |
|-----------|-------|---------|
| HttpOnly | true | Prevent XSS access |
| Secure | true | HTTPS only |
| SameSite | Lax | CSRF protection |
| Domain | .arack.io | Cross-subdomain sharing |

### Token Security

- RS256 asymmetric signing (private key never exposed)
- Short-lived access tokens (24h)
- Refresh tokens stored server-side only
- Token refresh happens transparently
- JWKS endpoint for public key distribution

### Session Security

- Sessions stored in Redis (not client-side)
- Session ID is opaque UUID (no sensitive data)
- Automatic expiry after 30 days of inactivity
- Logout destroys session server-side

---

## RED LINES - NEVER MODIFY

### 1. Cookie Configuration

**File:** `account-service/internal/config/config.go`

```go
// NEVER CHANGE THESE DEFAULTS
COOKIE_NAME=arack_session      // All apps depend on this name
COOKIE_DOMAIN=.arack.io        // Required for cross-subdomain SSO
```

**Why:** Changing the cookie name or domain will break SSO across all subdomains.

### 2. JWT Issuer/Audience

**File:** `account-service/internal/config/config.go`

```go
JWT_ISSUER=https://account.arack.io
JWT_AUDIENCE=https://arack.io
```

**Why:** Stalwart validates these claims. Changing them breaks email authentication.

### 3. RSA Keys Location

**File:** `account-service/internal/service/jwt.go`

```go
JWT_PRIVATE_KEY_PATH=/app/keys/private.pem
JWT_PUBLIC_KEY_PATH=/app/keys/public.pem
```

**Why:** These keys sign all tokens. Losing them invalidates all existing sessions.

**Recovery:** Keys are stored in Docker volume. If lost, all users must re-login.

### 4. OIDC Endpoints

**File:** `account-service/internal/handler/oauth.go`

| Endpoint | Purpose |
|----------|---------|
| `/.well-known/jwks.json` | Token validation (Stalwart) |
| `/.well-known/openid-configuration` | OIDC discovery |
| `/userinfo` | User info for OAuth clients |

**Why:** Stalwart and future OAuth clients depend on these exact paths.

### 5. Session Cookie in hooks.server.ts

**File:** `frontend-email/src/hooks.server.ts`

```typescript
const sessionCookie = cookies.get('arack_session');  // MUST match COOKIE_NAME
```

**Why:** Mismatch causes authentication failures.

### 6. Stalwart OIDC Configuration

**File:** `/opt/arack/ory/stalwart/config.toml` (VPS)

```toml
[storage]
directory = "oidc"  # MUST be "oidc", NOT "internal"

[directory.oidc]
userinfo-url = "https://account.arack.io/userinfo"  # MUST match account-service
```

**Why:** Changing this breaks email authentication completely.

**Recovery:**
```bash
ssh root@213.199.59.206
cp /opt/arack/ory/stalwart/config.toml.backup_oidc_fix /opt/arack/ory/stalwart/config.toml
docker restart arack_stalwart
```

---

## Troubleshooting

### "Not authenticated" Error

1. Check if `arack_session` cookie exists (DevTools → Application → Cookies)
2. Verify cookie domain is `.arack.io`
3. Check account-service logs: `docker logs search_engine_account_service`
4. Verify Redis is running: `docker exec search_engine_redis redis-cli ping`

### "Session expired" Error

1. Clear cookies and re-login
2. Check Redis session: `docker exec search_engine_redis redis-cli keys "session:*"`
3. Verify JWT tokens haven't expired

### Email Authentication Fails

1. Check Stalwart logs: `docker logs arack_stalwart`
2. Verify JWKS endpoint: `curl https://account.arack.io/.well-known/jwks.json`
3. Check userinfo endpoint: `curl -H "Authorization: Bearer <token>" https://account.arack.io/userinfo`

### SSO Not Working Across Subdomains

1. Verify cookie domain is `.arack.io` (with leading dot)
2. Check CORS configuration in account-service
3. Ensure all subdomains use HTTPS

---

## File Locations

| Component | Location |
|-----------|----------|
| Account Service (Go) | `account-service/` |
| JWT Service | `account-service/internal/service/jwt.go` |
| Session Service | `account-service/internal/service/session.go` |
| Local Auth Service | `account-service/internal/service/local_auth.go` |
| Config | `account-service/internal/config/config.go` |
| Handlers | `account-service/internal/handler/` |
| SSO Client (TS) | `frontend-email/src/lib/auth/sso.ts` |
| Login Page | `frontend-search/src/routes/auth/login/+page.svelte` |
| Register Page | `frontend-search/src/routes/auth/register/+page.svelte` |
| Hooks (mail) | `frontend-email/src/hooks.server.ts` |
| Stalwart Config | `ory/stalwart/config.toml` |

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-05 | Initial documentation, migration from Zitadel complete |
