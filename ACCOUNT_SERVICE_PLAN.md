# Account Service Implementation Plan

## Overview

Central authentication service for true SSO across all arack.io subdomains.

**Service:** account.arack.io
**Language:** Go
**Purpose:** Manages OAuth flow, sessions, and provides unified authentication for all apps

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           AFTER IMPLEMENTATION                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                        account.arack.io (NEW)                           │
│                        ┌─────────────────────┐                          │
│                        │   Go Service        │                          │
│                        │   - OAuth with      │                          │
│                        │     Zitadel         │                          │
│                        │   - Session in      │                          │
│                        │     Redis           │                          │
│                        │   - Cookie on       │                          │
│                        │     .arack.io       │                          │
│                        └──────────┬──────────┘                          │
│                                   │                                      │
│                    Sets cookie: arack_session=xxx                       │
│                    Domain: .arack.io (shared!)                          │
│                                   │                                      │
│         ┌─────────────────────────┼─────────────────────────┐           │
│         │                         │                         │           │
│         ▼                         ▼                         ▼           │
│    arack.io               mail.arack.io             admin.arack.io      │
│    (search)                 (email)                   (admin)           │
│         │                         │                         │           │
│         │    All read same cookie, call /api/session        │           │
│         │                         │                         │           │
│         ▼                         ▼                         ▼           │
│    api.arack.io           api-mail.arack.io                             │
│    (search-service)       (email-service)                               │
│    JWT validation         JWT validation                                │
│    NO CHANGES             NO CHANGES                                    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/login` | GET | Start OAuth flow, accepts `return_url` param |
| `/callback` | GET | OAuth callback from Zitadel |
| `/api/session` | GET | Get current user + access_token |
| `/api/logout` | POST | Destroy session, clear cookie |
| `/health` | GET | Health check |

## Session Response

```json
{
  "user_id": "12345",
  "email": "user@arack.io",
  "name": "John Doe",
  "picture": "https://...",
  "access_token": "eyJhbG..."
}
```

## Project Structure

```
account-service/
├── go.mod
├── go.sum
├── Dockerfile
├── cmd/
│   └── server/
│       └── main.go              # Entry point
├── internal/
│   ├── config/
│   │   └── config.go            # Configuration
│   ├── domain/
│   │   ├── session.go           # Session entity
│   │   └── user.go              # User value object
│   ├── repository/
│   │   ├── session.go           # Interface
│   │   └── redis/
│   │       └── session.go       # Redis implementation
│   ├── service/
│   │   ├── auth.go              # OAuth orchestration
│   │   └── session.go           # Session management
│   ├── handler/
│   │   ├── handler.go           # Routes setup
│   │   ├── login.go             # /login
│   │   ├── callback.go          # /callback
│   │   ├── session.go           # /api/session
│   │   └── logout.go            # /api/logout
│   └── middleware/
│       └── logging.go
└── pkg/
    └── httputil/
        └── response.go          # JSON helpers
```

## Dependencies

```go
require (
    github.com/go-chi/chi/v5 v5.0.12      // Router
    github.com/go-chi/cors v1.2.1          // CORS
    github.com/coreos/go-oidc/v3 v3.10.0   // OIDC (battle-tested)
    golang.org/x/oauth2 v0.18.0            // OAuth2
    github.com/redis/go-redis/v9 v9.5.1    // Redis
    github.com/rs/zerolog v1.32.0          // Logging
    github.com/kelseyhightower/envconfig v1.4.0  // Config
    github.com/google/uuid v1.6.0          // UUID
)
```

## Environment Variables

```bash
# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=3002

# Redis
REDIS_URL=redis://redis:6379

# OAuth (Zitadel)
ZITADEL_ISSUER_URL=https://auth.arack.io
ZITADEL_CLIENT_ID=<new-client-id>
OAUTH_REDIRECT_URL=https://account.arack.io/callback
OAUTH_SCOPES=openid,profile,email,offline_access

# Session
SESSION_TTL=720h                    # 30 days
TOKEN_REFRESH_THRESHOLD=5m

# Cookie
COOKIE_NAME=arack_session
COOKIE_DOMAIN=.arack.io
COOKIE_SECURE=true
COOKIE_HTTP_ONLY=true
COOKIE_MAX_AGE=2592000              # 30 days in seconds
```

## Implementation Tasks

### Phase 1: Account Service (Go)
- [ ] Create project structure
- [ ] Implement config loading
- [ ] Implement domain entities (Session, User)
- [ ] Implement Redis session repository
- [ ] Implement OAuth service (Zitadel integration)
- [ ] Implement session service
- [ ] Implement HTTP handlers
- [ ] Create Dockerfile
- [ ] Test locally

### Phase 2: Infrastructure
- [ ] Add to docker-compose.yml
- [ ] Add nginx config for account.arack.io
- [ ] Create Zitadel OAuth application
- [ ] Deploy to VPS

### Phase 3: Frontend Updates
- [ ] Update frontend-search auth logic
- [ ] Update frontend-admin auth logic
- [ ] Update frontend (email) auth logic
- [ ] Remove direct Zitadel OAuth code
- [ ] Test SSO flow

### Phase 4: Testing & Cleanup
- [ ] Test full SSO flow (login once, access all)
- [ ] Test logout (logs out everywhere)
- [ ] Test token refresh
- [ ] Remove old OAuth code from frontends
- [ ] Update documentation

## Docker Compose Addition

```yaml
# Add to docker-compose.yml
account-service:
  build:
    context: ./account-service
    dockerfile: Dockerfile
  container_name: search_engine_account_service
  ports:
    - "3002:3002"
  environment:
    - SERVER_HOST=0.0.0.0
    - SERVER_PORT=3002
    - REDIS_URL=redis://redis:6379
    - ZITADEL_ISSUER_URL=https://auth.arack.io
    - ZITADEL_CLIENT_ID=${ZITADEL_CLIENT_ID_ACCOUNT}
    - OAUTH_REDIRECT_URL=https://account.arack.io/callback
    - COOKIE_DOMAIN=.arack.io
    - COOKIE_NAME=arack_session
  depends_on:
    redis:
      condition: service_healthy
  restart: unless-stopped
  networks:
    - search_network
```

## Nginx Configuration

```nginx
# Add to nginx/sites-enabled/arack.io.conf

# =============================================================================
# Account Service - account.arack.io
# =============================================================================

server {
    listen 80;
    server_name account.arack.io;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://$server_name$request_uri;
    }
}

server {
    listen 443 ssl http2;
    server_name account.arack.io;

    ssl_certificate /etc/letsencrypt/live/arack.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/arack.io/privkey.pem;

    location / {
        proxy_pass http://account-service:3002;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Cookie support
        proxy_set_header Cookie $http_cookie;
        proxy_cookie_path / /;
    }
}
```

## Frontend Changes

### New Auth Module (shared across all frontends)

```typescript
// src/lib/auth.ts

const ACCOUNT_SERVICE_URL = 'https://account.arack.io';

export interface User {
  user_id: string;
  email: string;
  name: string;
  picture?: string;
  access_token: string;
}

export async function getSession(): Promise<User | null> {
  try {
    const res = await fetch(`${ACCOUNT_SERVICE_URL}/api/session`, {
      credentials: 'include'
    });
    if (res.ok) {
      return await res.json();
    }
    return null;
  } catch {
    return null;
  }
}

export function login(returnUrl?: string): void {
  const url = new URL(`${ACCOUNT_SERVICE_URL}/login`);
  url.searchParams.set('return_url', returnUrl || window.location.href);
  window.location.href = url.toString();
}

export async function logout(): Promise<void> {
  await fetch(`${ACCOUNT_SERVICE_URL}/api/logout`, {
    method: 'POST',
    credentials: 'include'
  });
  window.location.href = 'https://arack.io';
}

export function isAuthenticated(user: User | null): boolean {
  return user !== null;
}
```

### Updated API Client

```typescript
// src/lib/api.ts

import { getSession, type User } from './auth';

class API {
  private user: User | null = null;

  async init(): Promise<User | null> {
    this.user = await getSession();
    return this.user;
  }

  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json'
    };
    if (this.user?.access_token) {
      headers['Authorization'] = `Bearer ${this.user.access_token}`;
    }
    return headers;
  }

  async search(query: string, limit = 10, offset = 0) {
    const params = new URLSearchParams({ q: query, limit: String(limit), offset: String(offset) });
    const res = await fetch(`https://api.arack.io/api/search?${params}`, {
      headers: this.getHeaders()
    });
    return res.json();
  }

  // ... other methods
}

export const api = new API();
```

## Backend Services - NO CHANGES NEEDED

Search service and email service continue to:
1. Accept `Authorization: Bearer <token>` header
2. Validate JWT against Zitadel's JWKS
3. No code changes required

## SSO Flow After Implementation

```
1. User visits arack.io (not logged in)
   └─► GET account.arack.io/api/session
       └─► No cookie → 401
   └─► Show login button

2. User clicks Login
   └─► Redirect to account.arack.io/login?return_url=https://arack.io
       └─► account.arack.io generates PKCE, state
       └─► Redirect to auth.arack.io/oauth/v2/authorize

3. User logs in at Zitadel
   └─► Zitadel redirects to account.arack.io/callback?code=xxx

4. account.arack.io/callback
   └─► Exchange code for tokens
   └─► Create session in Redis
   └─► Set cookie: arack_session=<session_id>; Domain=.arack.io
   └─► Redirect to return_url (arack.io)

5. User is logged in on arack.io
   └─► GET account.arack.io/api/session (cookie sent automatically)
   └─► Returns { user_id, email, name, access_token }
   └─► Frontend stores access_token for API calls

═══════════════════════════════════════════════════════════════════

6. User opens mail.arack.io (new tab)
   └─► GET account.arack.io/api/session (SAME cookie sent!)
   └─► Returns { user_id, email, name, access_token }
   └─► User is INSTANTLY logged in! (TRUE SSO)
```

## Zitadel Configuration

Create new application in Zitadel:
- Name: `Account Service`
- Type: Web Application
- Grant Types: Authorization Code
- Auth Method: PKCE (no client secret)
- Redirect URIs: `https://account.arack.io/callback`
- Post Logout URIs: `https://arack.io`

## Success Criteria

1. ✅ User logs in once at account.arack.io
2. ✅ Cookie set on .arack.io domain
3. ✅ arack.io reads cookie, gets session → logged in
4. ✅ mail.arack.io reads same cookie, gets session → logged in (NO redirect!)
5. ✅ Logout from any app → logged out everywhere
6. ✅ Token refresh happens automatically in account-service
7. ✅ Backend services continue working with JWT validation

## Timeline

- Phase 1 (Account Service): 1-2 days
- Phase 2 (Infrastructure): 2-4 hours
- Phase 3 (Frontend Updates): 4-6 hours
- Phase 4 (Testing): 2-4 hours

**Total: 2-3 days**
