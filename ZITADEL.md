# Zitadel Integration Guide

**Single Source of Truth for Authentication**

This document covers all Zitadel OIDC authentication integration for the Arack platform.

---

## Overview

Arack uses **Zitadel** as the identity provider for all services:
- **Search App** (arack.io)
- **Email App** (mail.arack.io)
- **Admin Dashboard** (admin.arack.io)

### Key URLs

| Service | URL | Purpose |
|---------|-----|---------|
| Zitadel Console | https://auth.arack.io/ui/console | Admin UI |
| Authorization | https://auth.arack.io/oauth/v2/authorize | OAuth2 authorize |
| Token | https://auth.arack.io/oauth/v2/token | OAuth2 token exchange |
| User Info | https://auth.arack.io/oidc/v1/userinfo | Get user details |
| JWKS | https://auth.arack.io/oauth/v2/keys | JWT signing keys |
| Discovery | https://auth.arack.io/.well-known/openid-configuration | OIDC discovery |

---

## Architecture

### Authentication Flow

```
┌─────────────┐                                    ┌──────────────┐
│   Frontend  │                                    │   Zitadel    │
│  (arack.io) │                                    │ auth.arack.io│
└──────┬──────┘                                    └──────┬───────┘
       │                                                  │
       │ 1. User clicks "Login"                          │
       │────────────────────────────────────────────────▶│
       │                                                  │
       │ 2. GET /oauth/v2/authorize                      │
       │    + client_id + redirect_uri                   │
       │    + scope=openid profile email                 │
       │    + code_challenge (PKCE)                      │
       │────────────────────────────────────────────────▶│
       │                                                  │
       │ 3. Zitadel login page                           │
       │◀────────────────────────────────────────────────│
       │                                                  │
       │ 4. User authenticates                           │
       │────────────────────────────────────────────────▶│
       │                                                  │
       │ 5. Redirect with authorization code             │
       │◀────────────────────────────────────────────────│
       │                                                  │
       ▼                                                  │
┌─────────────┐                                          │
│   Backend   │                                          │
│ (API Server)│                                          │
└──────┬──────┘                                          │
       │                                                  │
       │ 6. POST /oauth/v2/token (code + PKCE verifier)  │
       │────────────────────────────────────────────────▶│
       │                                                  │
       │ 7. Access token + ID token                      │
       │◀────────────────────────────────────────────────│
       │                                                  │
       │ 8. GET /oidc/v1/userinfo                        │
       │────────────────────────────────────────────────▶│
       │                                                  │
       │ 9. User profile data                            │
       │◀────────────────────────────────────────────────│
```

### Custom Registration Flow

The platform uses a 3-step custom registration flow that creates users via Zitadel's Management API:

```
┌─────────────┐      ┌─────────────┐      ┌──────────────┐      ┌──────────────┐
│   Frontend  │      │Search Service│      │   Zitadel    │      │Email Service │
│  (arack.io) │      │ api.arack.io │      │ auth.arack.io│      │api-mail.arack│
└──────┬──────┘      └──────┬──────┘      └──────┬───────┘      └──────┬───────┘
       │                    │                    │                     │
       │ 1. POST /api/auth/suggest-usernames     │                     │
       │────────────────────▶│                   │                     │
       │                     │                   │                     │
       │ 2. Username options │                   │                     │
       │◀────────────────────│                   │                     │
       │                     │                   │                     │
       │ 3. POST /api/auth/register              │                     │
       │    {first_name, last_name, username,    │                     │
       │     password, date_of_birth, gender}    │                     │
       │────────────────────▶│                   │                     │
       │                     │                   │                     │
       │                     │ 4. POST /v2beta/users/human             │
       │                     │   (Management API)│                     │
       │                     │──────────────────▶│                     │
       │                     │                   │                     │
       │                     │ 5. User created   │                     │
       │                     │   (user_id)       │                     │
       │                     │◀──────────────────│                     │
       │                     │                   │                     │
       │                     │ 6. POST /internal/mail/provision        │
       │                     │─────────────────────────────────────────▶
       │                     │                   │                     │
       │                     │ 7. Email account created               │
       │                     │◀─────────────────────────────────────────
       │                     │                   │                     │
       │ 8. Success response │                   │                     │
       │◀────────────────────│                   │                     │
```

---

## Configuration

### Environment Variables

```bash
# Zitadel OIDC Configuration
ZITADEL_ISSUER_URL=https://auth.arack.io

# OAuth Client IDs (PKCE flow - no secrets needed)
ZITADEL_CLIENT_ID_SEARCH=353040740571480068
ZITADEL_CLIENT_ID_EMAIL=353040882842271748
ZITADEL_CLIENT_ID_ADMIN=353041000131788804

# OAuth Redirect URIs
ZITADEL_REDIRECT_URI_SEARCH=https://api.arack.io/auth/callback
ZITADEL_REDIRECT_URI_EMAIL=https://api-mail.arack.io/auth/callback
ZITADEL_REDIRECT_URI_ADMIN=https://admin.arack.io/auth/callback

# Management API Token (for custom registration)
ZITADEL_MGMT_TOKEN=<service-account-access-token>
```

### Service Account Setup

For the Management API (user creation), you need a service account:

1. **Service Account Details:**
   - KEY_ID: `353106685046292493`
   - USER_ID: `353043841739128836`
   - Private key stored in: `get_zitadel_token.sh`

2. **Generate Access Token:**
   ```bash
   # The get_zitadel_token.sh script generates a JWT and exchanges it
   ./get_zitadel_token.sh
   ```

3. **Token Exchange Flow:**
   - Create JWT signed with service account private key
   - Audience must be `http://auth.arack.io` (dev mode) or `https://auth.arack.io` (prod)
   - POST to `/oauth/v2/token` with `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer`
   - Returns access token valid for 12 hours

---

## Backend Integration

### Module Structure

```
src/lib/zitadel/
├── mod.rs          # Module exports
├── models.rs       # Data structures (UserInfo, webhooks)
├── client.rs       # OAuth2 client with PKCE
├── middleware.rs   # JWT validation middleware
└── management.rs   # Management API client (user creation)
```

### Key Components

**ZitadelClient** (`client.rs`):
- OAuth2 authorization URL generation
- PKCE code challenge/verifier
- Token exchange
- User info retrieval

**ZitadelManagementClient** (`management.rs`):
- Create users via v2beta API
- Delete users (rollback on failure)
- Check user existence
- Supports internal Docker networking with Host header override

### API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/auth/suggest-usernames` | POST | Get username suggestions |
| `/api/auth/check-username` | POST | Check username availability |
| `/api/auth/register` | POST | Custom registration (creates Zitadel user) |

### Database Schema

```sql
-- user_preferences table
CREATE TABLE user_preferences (
    id UUID PRIMARY KEY,
    zitadel_user_id TEXT UNIQUE,      -- Zitadel user ID
    kratos_identity_id UUID,           -- Legacy (deprecated)
    username TEXT,
    date_of_birth DATE,
    gender TEXT,
    theme TEXT DEFAULT 'light',
    results_per_page INTEGER DEFAULT 20,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

---

## Docker Networking

When running inside Docker, the search-service needs to call Zitadel via internal networking but with the correct Host header:

```rust
// For internal URL like http://zitadel:8080
// Set Host header to auth.arack.io
let client = ZitadelManagementClient::with_host_override(
    "http://zitadel:8080".to_string(),
    access_token,
    Some("auth.arack.io".to_string()),
);
```

**Docker Compose Configuration:**
```yaml
search-service:
  environment:
    - ZITADEL_ISSUER_URL=https://auth.arack.io
    - ZITADEL_MGMT_TOKEN=${ZITADEL_MGMT_TOKEN}
```

---

## Troubleshooting

### Common Issues

**1. "password authentication failed"**
- PostgreSQL password mismatch between docker-compose and actual database
- Solution: Reset password or update DATABASE_URL

**2. "Audience must contain client_id"**
- JWT audience format mismatch
- Dev mode: Use `http://auth.arack.io`
- Production: Use `https://auth.arack.io`

**3. "QUERY-AG4gs: An internal error occurred"**
- Zitadel internal issue during user search
- Usually transient, registration continues anyway

**4. "Failed to provision email account"**
- Email service is down
- User is created in Zitadel but rolled back
- Fix email service and retry registration

### Verification Commands

```bash
# Check Zitadel is running
curl https://auth.arack.io/debug/healthz

# Test OAuth discovery
curl https://auth.arack.io/.well-known/openid-configuration

# Check search-service logs for Zitadel
docker logs search_engine_search_service | grep -i zitadel

# Verify user created in database
docker exec search_engine_postgres psql -U postgres -d engine_search \
  -c "SELECT zitadel_user_id, username FROM user_preferences WHERE zitadel_user_id IS NOT NULL;"
```

---

## Zitadel Admin Console

### Access
- URL: https://auth.arack.io/ui/console
- Organization: Arack

### Key Entities

**Projects:**
- Arack Platform (contains all OAuth apps)

**Applications:**
1. **Arack Search** - Client ID: 353040740571480068
2. **Arack Email** - Client ID: 353040882842271748
3. **Arack Admin** - Client ID: 353041000131788804

**Service Users:**
- Management Service Account (for API operations)

---

## Migration Notes

### From Ory (Kratos/Hydra)

The system previously used Ory Kratos for identity and Hydra for OAuth. Key differences:

| Aspect | Ory | Zitadel |
|--------|-----|---------|
| Identity Storage | Kratos | Zitadel Users |
| OAuth Provider | Hydra | Zitadel OIDC |
| User ID Field | `kratos_identity_id` | `zitadel_user_id` |
| Registration | Kratos self-service | Custom via Management API |
| Admin Console | CLI-based | Web UI |

Legacy `kratos_identity_id` column remains for backward compatibility but is deprecated.

---

## Version Notes

- **Zitadel Version:** v2.62.1
- **Note:** Actions V2 requires Zitadel v3.0.0+ (not currently available)
- **Current Actions:** Using legacy Zitadel Actions (webhooks configured in console)

---

## Related Files

- `src/lib/zitadel/` - Rust integration module
- `search/api/registration.rs` - Custom registration handler
- `search/api/username.rs` - Username suggestions
- `get_zitadel_token.sh` - Token generation script
- `.env.example` - Environment variable reference
