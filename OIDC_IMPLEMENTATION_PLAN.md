# OIDC Implementation Plan - Complete Guide

**Date:** 2025-12-20
**Goal:** Implement OAuth 2.0 / OIDC authentication for email service using Ory Hydra
**Estimated Time:** 2-3 days
**Complexity:** High (but production-grade)

---

## 📋 Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Prerequisites](#prerequisites)
3. [Phase 1: Hydra Client Registration](#phase-1-hydra-client-registration)
4. [Phase 2: Kratos-Hydra Integration](#phase-2-kratos-hydra-integration)
5. [Phase 3: Stalwart OIDC Backend](#phase-3-stalwart-oidc-backend)
6. [Phase 4: Email Service Code Changes](#phase-4-email-service-code-changes)
7. [Phase 5: Frontend OAuth Flow](#phase-5-frontend-oauth-flow)
8. [Phase 6: Testing & Verification](#phase-6-testing--verification)
9. [Phase 7: Production Deployment](#phase-7-production-deployment)
10. [Rollback Plan](#rollback-plan)

---

## Architecture Overview

### Current Flow (Broken)

```
User → Kratos (login) → Session Cookie → Email API → JMAP (DEFAULT_PASSWORD) ❌
```

### Target Flow (OIDC)

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Authentication                          │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 1. User logs in via Kratos (email/password)                         │
│    → POST /api/auth/flows/login                                      │
│    → Kratos validates credentials                                    │
│    → Creates Kratos session                                          │
│    → Returns session cookie                                          │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 2. Frontend initiates OAuth flow with Hydra                         │
│    → Redirect to /oauth2/auth with client_id                        │
│    → Hydra checks if user authenticated (via session)               │
│    → Hydra redirects to login handler if not authenticated          │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 3. Login Handler (Search Service)                                   │
│    → GET /api/hydra/login?login_challenge=xxx                       │
│    → Verify Kratos session exists                                   │
│    → Accept login challenge with user subject                       │
│    → PUT /admin/oauth2/auth/requests/login/accept?login_challenge   │
│    → Hydra returns redirect_to URL                                  │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 4. Consent Handler (Search Service)                                 │
│    → GET /api/hydra/consent?consent_challenge=xxx                   │
│    → Accept consent for requested scopes (openid, email)            │
│    → PUT /admin/oauth2/auth/requests/consent/accept                 │
│    → Hydra returns redirect_to URL                                  │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 5. Hydra issues authorization code                                  │
│    → Redirect to callback: /oauth/callback?code=xxx                 │
│    → Frontend exchanges code for tokens                             │
│    → POST /oauth2/token (code + client credentials)                 │
│    → Hydra returns: access_token, refresh_token, id_token           │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 6. Email API uses access token                                      │
│    → GET /api/mail/mailboxes (with session cookie)                  │
│    → Email service validates session                                 │
│    → Email service gets access token for user                        │
│    → Email service authenticates to Stalwart with Bearer token      │
└─────────────────────────────────────────────────────────────────────┘
                                    ↓
┌─────────────────────────────────────────────────────────────────────┐
│ 7. Stalwart validates token with Hydra                              │
│    → Stalwart calls Hydra userinfo endpoint with Bearer token       │
│    → GET /userinfo (Authorization: Bearer xxx)                       │
│    → Hydra validates token and returns user claims                  │
│    → Stalwart grants access to user's mailbox                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Interaction Map

```
┌──────────────┐
│   Browser    │
│ (Frontend)   │
└──────┬───────┘
       │
       ├─────────────────┐
       │                 │
       v                 v
┌──────────────┐  ┌──────────────┐
│    Kratos    │  │    Hydra     │
│  (Identity)  │  │   (OAuth)    │
└──────┬───────┘  └──────┬───────┘
       │                 │
       │                 │
       v                 v
┌─────────────────────────────────┐
│      Search Service (Rust)       │
│  - Login Handler                 │
│  - Consent Handler               │
│  - Token Management              │
└─────────────────────────────────┘
                 │
                 v
┌─────────────────────────────────┐
│      Email Service (Rust)        │
│  - Token-based JMAP auth         │
│  - Session validation            │
└──────────────┬──────────────────┘
               │
               v
┌─────────────────────────────────┐
│        Stalwart Mail Server      │
│  - OIDC backend                  │
│  - Userinfo endpoint validation  │
└─────────────────────────────────┘
```

---

## Prerequisites

### ✅ Already Complete

- [x] Ory Hydra installed and running (ports 4444, 4445)
- [x] Hydra migrations applied successfully
- [x] Ory Kratos running with session management
- [x] Email service with JMAP client
- [x] Stalwart mail server operational
- [x] PostgreSQL databases (engine_search, kratos_db, hydra_db)

### 📦 Rust Dependencies Needed

Add to `Cargo.toml`:

```toml
[dependencies]
# OAuth/OIDC client libraries
oauth2 = "4.4"
reqwest = { version = "0.11", features = ["json"] }
jsonwebtoken = "9.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.21"

# For token storage/caching
tokio = { version = "1.35", features = ["full"] }
```

### 🔑 Environment Variables Needed

Add to `.env.production` on VPS:

```bash
# Hydra configuration
HYDRA_ADMIN_URL=http://hydra:4445
HYDRA_PUBLIC_URL=http://127.0.0.1:4444
HYDRA_CLIENT_ID=email-service
HYDRA_CLIENT_SECRET=<GENERATED_SECRET>

# Stalwart OIDC
STALWART_OIDC_ENABLED=true
```

---

## Phase 1: Hydra Client Registration

**Goal:** Register email service as an OAuth2 client in Hydra

**Time Estimate:** 30 minutes

### Step 1.1: Generate Client Secret

```bash
# On VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Generate secure secret (32 characters minimum)
openssl rand -base64 32
# Example output: 7X9Ks2Lm4Np6Rq8Tv0Uw3Yx5Za1Bc4Df6Gh9Jk2M=
```

**Save this secret** - you'll need it in multiple places.

### Step 1.2: Register Email Service Client

```bash
# Create Hydra OAuth2 client for email service
curl -X POST http://localhost:4445/admin/clients \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "email-service",
    "client_name": "Email Service",
    "client_secret": "7X9Ks2Lm4Np6Rq8Tv0Uw3Yx5Za1Bc4Df6Gh9Jk2M=",
    "grant_types": ["authorization_code", "refresh_token"],
    "response_types": ["code"],
    "redirect_uris": [
      "https://mail.arack.io/oauth/callback",
      "http://localhost:5173/oauth/callback"
    ],
    "scope": "openid email profile offline_access",
    "token_endpoint_auth_method": "client_secret_post",
    "audience": ["email-api"]
  }'
```

**Expected Response:**
```json
{
  "client_id": "email-service",
  "client_name": "Email Service",
  "grant_types": ["authorization_code", "refresh_token"],
  "response_types": ["code"],
  "redirect_uris": ["https://mail.arack.io/oauth/callback", ...],
  "scope": "openid email profile offline_access",
  "created_at": "2025-12-20T...",
  "updated_at": "2025-12-20T..."
}
```

### Step 1.3: Register Stalwart as Resource Server

```bash
# Stalwart needs to validate tokens
# Register as trusted JWT issuer
curl -X POST http://localhost:4445/admin/trust/grants/jwt-bearer/issuers \
  -H "Content-Type: application/json" \
  -d '{
    "issuer": "http://127.0.0.1:4444/",
    "subject": "email-service",
    "scope": ["email"],
    "jwks_uri": "http://hydra:4444/.well-known/jwks.json",
    "expires_at": "2030-01-01T00:00:00Z"
  }'
```

### Step 1.4: Verify Client Registration

```bash
# List all clients
curl http://localhost:4445/admin/clients | python3 -m json.tool

# Get specific client
curl http://localhost:4445/admin/clients/email-service | python3 -m json.tool
```

**✅ Verification:**
- Client `email-service` appears in list
- `redirect_uris` includes both production and development URLs
- `scope` includes `openid email profile offline_access`

---

## Phase 2: Kratos-Hydra Integration

**Goal:** Implement login and consent handlers in Search service

**Time Estimate:** 4-6 hours

### Step 2.1: Add Hydra Admin Client to Search Service

**File:** `src/api/mod.rs`

**Add at top of file:**

```rust
use oauth2::{
    basic::BasicClient,
    AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};

// Hydra admin API types
#[derive(Debug, Deserialize, Serialize)]
struct LoginRequest {
    challenge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginAcceptRequest {
    subject: String,
    remember: bool,
    remember_for: i64,
    acr: String,
    context: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginAcceptResponse {
    redirect_to: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentRequest {
    challenge: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_scope: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    requested_access_token_audience: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentAcceptRequest {
    grant_scope: Vec<String>,
    grant_access_token_audience: Vec<String>,
    remember: bool,
    remember_for: i64,
    session: ConsentSessionData,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentSessionData {
    id_token: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
struct ConsentAcceptResponse {
    redirect_to: String,
}
```

### Step 2.2: Add Hydra Configuration to AppState

**File:** `src/api/mod.rs`

**Update `AppState` struct:**

```rust
pub struct AppState {
    pub pool: PgPool,
    pub redis: redis::Client,
    pub meilisearch: MeilisearchClient,
    pub crawler: Arc<Crawler>,
    pub kratos_client: reqwest::Client,
    pub kratos_url: String,
    pub hydra_admin_url: String,  // NEW
    pub hydra_client: reqwest::Client,  // NEW
}
```

**In `main.rs`, update state initialization:**

```rust
let hydra_admin_url = env::var("HYDRA_ADMIN_URL")
    .unwrap_or_else(|_| "http://hydra:4445".to_string());

let state = AppState {
    pool: pool.clone(),
    redis: redis_client.clone(),
    meilisearch: meilisearch_client.clone(),
    crawler,
    kratos_client: reqwest::Client::new(),
    kratos_url,
    hydra_admin_url,  // NEW
    hydra_client: reqwest::Client::new(),  // NEW
};
```

### Step 2.3: Implement Login Handler

**File:** `src/api/mod.rs`

**Add new route handler:**

```rust
/// Hydra login handler
/// GET /api/hydra/login?login_challenge=xxx
async fn hydra_login_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let login_challenge = params
        .get("login_challenge")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing login_challenge parameter"})),
            )
        })?;

    // Get login request info from Hydra
    let login_req_url = format!(
        "{}/admin/oauth2/auth/requests/login?login_challenge={}",
        state.hydra_admin_url, login_challenge
    );

    let login_info: LoginRequest = state
        .hydra_client
        .get(&login_req_url)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to get login request from Hydra: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to communicate with auth server"})),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse Hydra login request: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid auth server response"})),
            )
        })?;

    // Check if user already has Kratos session
    let session_cookie = jar
        .get("ory_kratos_session")
        .ok_or_else(|| {
            // No session - redirect to Kratos login
            info!("No Kratos session found, user needs to login");
            (
                StatusCode::FOUND,
                Json(json!({
                    "redirect_to": format!("https://arack.io/auth/login?return_to={}",
                        urlencoding::encode(&format!("/api/hydra/login?login_challenge={}", login_challenge)))
                })),
            )
        })?;

    // Validate Kratos session
    let whoami_url = format!("{}/sessions/whoami", state.kratos_url);
    let whoami_response = state
        .kratos_client
        .get(&whoami_url)
        .header("Cookie", format!("ory_kratos_session={}", session_cookie.value()))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to validate Kratos session: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Session validation failed"})),
            )
        })?;

    if !whoami_response.status().is_success() {
        // Invalid session - redirect to login
        return Err((
            StatusCode::FOUND,
            Json(json!({
                "redirect_to": format!("https://arack.io/auth/login?return_to={}",
                    urlencoding::encode(&format!("/api/hydra/login?login_challenge={}", login_challenge)))
            })),
        ));
    }

    let session_data: serde_json::Value = whoami_response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse whoami response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid session data"})),
            )
        })?;

    // Extract user identity
    let identity_id = session_data["identity"]["id"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing identity ID in session");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid session data"})),
            )
        })?;

    let email = session_data["identity"]["traits"]["email"]
        .as_str()
        .ok_or_else(|| {
            error!("Missing email in identity traits");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid user data"})),
            )
        })?;

    info!("User {} authenticated via Kratos, accepting login", email);

    // Accept login challenge
    let accept_url = format!(
        "{}/admin/oauth2/auth/requests/login/accept?login_challenge={}",
        state.hydra_admin_url, login_challenge
    );

    let accept_body = LoginAcceptRequest {
        subject: identity_id.to_string(),
        remember: true,
        remember_for: 3600 * 24 * 7, // 7 days
        acr: "0".to_string(), // Authentication Context Class Reference
        context: json!({
            "email": email,
            "identity_id": identity_id
        }),
    };

    let accept_response: LoginAcceptResponse = state
        .hydra_client
        .put(&accept_url)
        .json(&accept_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to accept login: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to accept login"})),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse accept response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid auth server response"})),
            )
        })?;

    // Redirect user to Hydra's redirect URL
    Ok((
        StatusCode::FOUND,
        [("Location", accept_response.redirect_to)],
    ))
}
```

### Step 2.4: Implement Consent Handler

**File:** `src/api/mod.rs`

```rust
/// Hydra consent handler
/// GET /api/hydra/consent?consent_challenge=xxx
async fn hydra_consent_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let consent_challenge = params
        .get("consent_challenge")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "Missing consent_challenge parameter"})),
            )
        })?;

    // Get consent request info from Hydra
    let consent_req_url = format!(
        "{}/admin/oauth2/auth/requests/consent?consent_challenge={}",
        state.hydra_admin_url, consent_challenge
    );

    let consent_info: ConsentRequest = state
        .hydra_client
        .get(&consent_req_url)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to get consent request from Hydra: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to communicate with auth server"})),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse Hydra consent request: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid auth server response"})),
            )
        })?;

    info!(
        "User {} consenting to scopes: {:?}",
        consent_info.subject.as_deref().unwrap_or("unknown"),
        consent_info.requested_scope
    );

    // Auto-accept consent for email service (trusted first-party app)
    let accept_url = format!(
        "{}/admin/oauth2/auth/requests/consent/accept?consent_challenge={}",
        state.hydra_admin_url, consent_challenge
    );

    let accept_body = ConsentAcceptRequest {
        grant_scope: consent_info.requested_scope.unwrap_or_default(),
        grant_access_token_audience: consent_info
            .requested_access_token_audience
            .unwrap_or_default(),
        remember: true,
        remember_for: 3600 * 24 * 30, // 30 days
        session: ConsentSessionData {
            id_token: json!({
                "email": consent_info.subject,
            }),
        },
    };

    let accept_response: ConsentAcceptResponse = state
        .hydra_client
        .put(&accept_url)
        .json(&accept_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to accept consent: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to accept consent"})),
            )
        })?
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse accept response: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Invalid auth server response"})),
            )
        })?;

    // Redirect user to Hydra's redirect URL
    Ok((
        StatusCode::FOUND,
        [("Location", accept_response.redirect_to)],
    ))
}
```

### Step 2.5: Add Routes to Router

**File:** `src/api/mod.rs`

**In the `create_router()` function:**

```rust
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ... existing routes ...

        // Hydra OAuth handlers
        .route("/api/hydra/login", get(hydra_login_handler))
        .route("/api/hydra/consent", get(hydra_consent_handler))

        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(/* ... */)
                // ... rest of CORS config
        )
}
```

### Step 2.6: Update Hydra Configuration

**File:** `/opt/arack/docker-compose.yml` (on VPS)

**Update Hydra environment variables:**

```yaml
hydra:
  image: oryd/hydra:v2.2.0
  container_name: search_engine_hydra
  ports:
    - "4444:4444"
    - "4445:4445"
  environment:
    - DSN=postgres://postgres:postgres@postgres:5432/hydra_db?sslmode=disable
    - URLS_SELF_ISSUER=https://auth.arack.io  # CHANGED from http://127.0.0.1:4444
    - URLS_LOGIN=https://api.arack.io/api/hydra/login  # CHANGED
    - URLS_CONSENT=https://api.arack.io/api/hydra/consent  # CHANGED
    - SECRETS_SYSTEM=CHANGE-THIS-IN-PRODUCTION-MINIMUM-32-CHARACTERS-LONG
    - OIDC_SUBJECT_IDENTIFIERS_SUPPORTED_TYPES=public
    - OAUTH2_EXPOSE_INTERNAL_ERRORS=true
  command: serve all --dev
  restart: unless-stopped
  networks:
    - search_network
  depends_on:
    - hydra-migrate
    - kratos
```

### Step 2.7: Build and Deploy Search Service

```bash
# On local machine
cd "/Users/intelifoxdz/RS Projects/Engine_search"

# Add missing dependency
cargo add urlencoding

# Build with offline mode
SQLX_OFFLINE=true cargo build --release --bin search-service

# Upload to VPS
scp -i ~/.ssh/id_rsa_arack target/release/search-service root@213.199.59.206:/opt/arack/

# On VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Restart services
docker stop search_engine_search_service
docker rm search_engine_search_service
docker compose up -d search-service

# Restart Hydra with new config
docker compose up -d hydra

# Check logs
docker logs search_engine_search_service -f
docker logs search_engine_hydra -f
```

**✅ Verification:**

```bash
# Test login handler (should redirect or show error about missing challenge)
curl -i https://api.arack.io/api/hydra/login

# Test consent handler (should redirect or show error about missing challenge)
curl -i https://api.arack.io/api/hydra/consent
```

---

## Phase 3: Stalwart OIDC Backend

**Goal:** Configure Stalwart to validate access tokens via Hydra

**Time Estimate:** 1-2 hours

### Step 3.1: Update Stalwart Configuration

**File:** `/opt/arack/stalwart/config.toml` (on VPS)

**Add OIDC directory configuration:**

```toml
# OIDC Authentication Backend
[directory."oidc"]
type = "oidc"

# Hydra userinfo endpoint
[directory."oidc".endpoint]
url = "http://hydra:4444/userinfo"
method = "userinfo"

# Field mappings
[directory."oidc".fields]
email = "email"
username = "preferred_username"
name = "name"

# Use OIDC as primary auth for JMAP
[jmap.auth]
directory = "oidc"

# Fallback to internal directory for admin
[smtp.auth]
directory = ["oidc", "sql"]
```

### Step 3.2: Test Token Validation Flow

**First, get a test token from Hydra:**

```bash
# Manual OAuth flow test
# This will fail initially but helps verify endpoint connectivity

# 1. Get authorization code (will redirect)
curl -i "http://localhost:4444/oauth2/auth?\
client_id=email-service&\
response_type=code&\
scope=openid+email+profile&\
redirect_uri=https://mail.arack.io/oauth/callback&\
state=test123"

# 2. After getting code from redirect, exchange for token
curl -X POST http://localhost:4444/oauth2/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=authorization_code&\
code=AUTHORIZATION_CODE_HERE&\
redirect_uri=https://mail.arack.io/oauth/callback&\
client_id=email-service&\
client_secret=7X9Ks2Lm4Np6Rq8Tv0Uw3Yx5Za1Bc4Df6Gh9Jk2M="

# 3. Test userinfo endpoint with access token
curl http://localhost:4444/userinfo \
  -H "Authorization: Bearer ACCESS_TOKEN_HERE"
```

### Step 3.3: Restart Stalwart

```bash
# On VPS
docker restart search_engine_stalwart

# Check logs for OIDC configuration
docker logs search_engine_stalwart -f

# Should see:
# [INFO] OIDC directory configured: oidc
# [INFO] Userinfo endpoint: http://hydra:4444/userinfo
```

**✅ Verification:**

```bash
# Check Stalwart can reach Hydra
docker exec search_engine_stalwart curl -s http://hydra:4444/health/ready
# Should return: {"status":"ok"}

# Check OIDC discovery
docker exec search_engine_stalwart curl -s http://hydra:4444/.well-known/openid-configuration | python3 -m json.tool
```

---

## Phase 4: Email Service Code Changes

**Goal:** Update email service to use OAuth tokens for JMAP authentication

**Time Estimate:** 6-8 hours

### Step 4.1: Add OAuth Token Management Module

**Create new file:** `src/email/oauth.rs`

```rust
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i64,
    refresh_token: Option<String>,
    scope: Option<String>,
}

pub struct OAuthTokenManager {
    hydra_token_url: String,
    client_id: String,
    client_secret: String,
    http_client: reqwest::Client,
    // Cache: user_identity_id -> TokenPair
    token_cache: Arc<RwLock<HashMap<String, TokenPair>>>,
}

impl OAuthTokenManager {
    pub fn new(
        hydra_public_url: String,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self {
            hydra_token_url: format!("{}/oauth2/token", hydra_public_url),
            client_id,
            client_secret,
            http_client: reqwest::Client::new(),
            token_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get valid access token for user (from cache or refresh)
    pub async fn get_access_token(&self, user_id: &str) -> Result<String> {
        // Check cache first
        {
            let cache = self.token_cache.read().await;
            if let Some(token_pair) = cache.get(user_id) {
                // Token still valid for at least 5 minutes
                if token_pair.expires_at > Utc::now() + Duration::minutes(5) {
                    return Ok(token_pair.access_token.clone());
                }
            }
        }

        // Token expired or not in cache - need to refresh or get new token
        // In production, this would use refresh token
        // For now, we'll need the user to re-authenticate
        anyhow::bail!("No valid access token found for user {}", user_id);
    }

    /// Store token pair in cache
    pub async fn store_token(&self, user_id: String, token_response: TokenResponse) {
        let expires_at = Utc::now() + Duration::seconds(token_response.expires_in);

        let token_pair = TokenPair {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at,
            token_type: token_response.token_type,
        };

        let mut cache = self.token_cache.write().await;
        cache.insert(user_id, token_pair);
    }

    /// Refresh access token using refresh token
    pub async fn refresh_token(&self, user_id: &str) -> Result<String> {
        let refresh_token = {
            let cache = self.token_cache.read().await;
            cache
                .get(user_id)
                .and_then(|t| t.refresh_token.clone())
                .ok_or_else(|| anyhow::anyhow!("No refresh token available"))?
        };

        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response: TokenResponse = self
            .http_client
            .post(&self.hydra_token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?
            .json()
            .await
            .context("Failed to parse token response")?;

        let access_token = response.access_token.clone();
        self.store_token(user_id.to_string(), response).await;

        Ok(access_token)
    }

    /// Exchange authorization code for tokens
    pub async fn exchange_code(
        &self,
        code: String,
        redirect_uri: String,
        user_id: String,
    ) -> Result<TokenPair> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response: TokenResponse = self
            .http_client
            .post(&self.hydra_token_url)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange authorization code")?
            .json()
            .await
            .context("Failed to parse token response")?;

        let token_pair = TokenPair {
            access_token: response.access_token.clone(),
            refresh_token: response.refresh_token.clone(),
            expires_at: Utc::now() + Duration::seconds(response.expires_in),
            token_type: response.token_type.clone(),
        };

        // Store in cache
        self.store_token(user_id, response).await;

        Ok(token_pair)
    }
}
```

### Step 4.2: Update Email Service AppState

**File:** `src/email/api/mod.rs`

```rust
use crate::email::oauth::OAuthTokenManager;

pub struct AppState {
    pub db: PgPool,
    pub redis: Arc<redis::Client>,
    pub jmap_client: Arc<JmapClient>,
    pub stalwart_admin_url: String,
    pub stalwart_admin_user: String,
    pub stalwart_admin_password: String,
    pub default_email_domain: String,
    pub default_email_password: String,
    pub openai_api_key: Option<String>,
    pub centrifugo_client: Arc<CentrifugoClient>,
    pub kratos_url: String,
    pub oauth_manager: Arc<OAuthTokenManager>,  // NEW
}
```

**In email service `main.rs`:**

```rust
use email::oauth::OAuthTokenManager;

#[tokio::main]
async fn main() -> Result<()> {
    // ... existing setup ...

    let hydra_public_url = env::var("HYDRA_PUBLIC_URL")
        .unwrap_or_else(|_| "http://hydra:4444".to_string());
    let hydra_client_id = env::var("HYDRA_CLIENT_ID")
        .unwrap_or_else(|_| "email-service".to_string());
    let hydra_client_secret = env::var("HYDRA_CLIENT_SECRET")
        .context("HYDRA_CLIENT_SECRET must be set")?;

    let oauth_manager = Arc::new(OAuthTokenManager::new(
        hydra_public_url,
        hydra_client_id,
        hydra_client_secret,
    ));

    let state = AppState {
        // ... existing fields ...
        oauth_manager,
    };

    // ... rest of main ...
}
```

### Step 4.3: Add OAuth Callback Endpoint

**File:** `src/email/api/mod.rs`

```rust
#[derive(Debug, Deserialize)]
struct OAuthCallbackQuery {
    code: String,
    state: Option<String>,
}

/// OAuth callback endpoint
/// GET /api/mail/oauth/callback?code=xxx&state=xxx
async fn oauth_callback(
    State(state): State<Arc<AppState>>,
    Query(query): Query<OAuthCallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Validate Kratos session
    let session_cookie = jar.get("ory_kratos_session").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "No session found"})),
        )
    })?;

    // Get user identity from Kratos
    let kratos_url = format!("{}/sessions/whoami", state.kratos_url);
    let response = reqwest::Client::new()
        .get(&kratos_url)
        .header("Cookie", format!("ory_kratos_session={}", session_cookie.value()))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to validate session: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Session validation failed"})),
            )
        })?;

    if !response.status().is_success() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid session"})),
        ));
    }

    let session_data: Value = response.json().await.map_err(|e| {
        error!("Failed to parse session: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Invalid session data"})),
        )
    })?;

    let identity_id = session_data["identity"]["id"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Missing identity ID"})),
            )
        })?;

    // Exchange authorization code for tokens
    let token_pair = state
        .oauth_manager
        .exchange_code(
            query.code,
            "https://mail.arack.io/oauth/callback".to_string(),
            identity_id.to_string(),
        )
        .await
        .map_err(|e| {
            error!("Failed to exchange authorization code: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Token exchange failed"})),
            )
        })?;

    info!("Successfully exchanged authorization code for user {}", identity_id);

    // Redirect to email app with success
    Ok((
        StatusCode::FOUND,
        [("Location", "https://mail.arack.io?auth=success")],
    ))
}
```

### Step 4.4: Update JMAP Authentication to Use Tokens

**File:** `src/email/api/mod.rs`

**Update `get_jmap_session` function:**

```rust
async fn get_jmap_session(
    jmap_client: &JmapClient,
    oauth_manager: &OAuthTokenManager,
    user_id: &str,
    email: &str,
) -> Result<(JmapAuth, String), (StatusCode, Json<serde_json::Value>)> {
    // Get access token for user
    let access_token = oauth_manager
        .get_access_token(user_id)
        .await
        .map_err(|e| {
            error!("Failed to get access token for user {}: {}", user_id, e);
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "No valid access token. Please re-authenticate.",
                    "auth_url": format!("https://auth.arack.io/oauth2/auth?\
                        client_id=email-service&\
                        response_type=code&\
                        scope=openid+email+profile+offline_access&\
                        redirect_uri=https://mail.arack.io/oauth/callback&\
                        state={}", user_id)
                })),
            )
        })?;

    // Use Bearer token authentication for JMAP
    let auth = JmapAuth::Bearer(access_token);

    // Get JMAP session
    let session = jmap_client.get_session(&auth).await.map_err(|e| {
        error!("Failed to get JMAP session: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Failed to connect to mail server"})),
        )
    })?;

    let primary_account = session
        .primary_accounts
        .get("urn:ietf:params:jmap:mail")
        .ok_or_else(|| {
            error!("No primary mail account in JMAP session");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "No mail account configured"})),
            )
        })?;

    Ok((auth, primary_account.clone()))
}
```

**Update all endpoint handlers that use `get_jmap_session`:**

```rust
// Example: list_mailboxes
async fn list_mailboxes(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Get account from session
    let account = get_account_from_session(&state, &jar).await?;

    // Get JMAP session with OAuth token
    let (auth, account_id) = get_jmap_session(
        &state.jmap_client,
        &state.oauth_manager,
        &account.kratos_identity_id.to_string(),
        &account.email_address,
    )
    .await?;

    // ... rest of function ...
}
```

### Step 4.5: Add OAuth Initiation Endpoint

**File:** `src/email/api/mod.rs`

```rust
/// Initiate OAuth flow
/// GET /api/mail/oauth/initiate
async fn oauth_initiate(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> Result<impl IntoResponse, (StatusCode, Json<Value>)> {
    // Validate session
    let session_cookie = jar.get("ory_kratos_session").ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Not authenticated"})),
        )
    })?;

    // Get user identity
    let kratos_url = format!("{}/sessions/whoami", state.kratos_url);
    let response = reqwest::Client::new()
        .get(&kratos_url)
        .header("Cookie", format!("ory_kratos_session={}", session_cookie.value()))
        .send()
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Session validation failed"})),
            )
        })?;

    let session_data: Value = response.json().await.map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "Invalid session"})),
        )
    })?;

    let identity_id = session_data["identity"]["id"]
        .as_str()
        .ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Missing identity"})),
            )
        })?;

    // Build OAuth authorization URL
    let auth_url = format!(
        "https://auth.arack.io/oauth2/auth?\
        client_id=email-service&\
        response_type=code&\
        scope=openid+email+profile+offline_access&\
        redirect_uri=https://mail.arack.io/oauth/callback&\
        state={}",
        identity_id
    );

    Ok(Json(json!({
        "auth_url": auth_url,
        "message": "Redirect user to this URL to complete OAuth flow"
    })))
}
```

### Step 4.6: Add Routes

**File:** `src/email/api/mod.rs`

```rust
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ... existing routes ...

        // OAuth routes
        .route("/api/mail/oauth/initiate", get(oauth_initiate))
        .route("/api/mail/oauth/callback", get(oauth_callback))

        .with_state(state)
        // ... rest of config ...
}
```

### Step 4.7: Update JMAP Client to Support Bearer Auth

**File:** `src/email/jmap/mod.rs`

**Update `apply_auth` method:**

```rust
impl JmapClient {
    fn apply_auth(
        &self,
        request: reqwest::RequestBuilder,
        auth: &JmapAuth,
    ) -> reqwest::RequestBuilder {
        match auth {
            JmapAuth::Basic { username, password } => {
                request.basic_auth(username, Some(password))
            }
            JmapAuth::Bearer(token) => {
                request.header("Authorization", format!("Bearer {}", token))
            }
        }
    }
}
```

### Step 4.8: Build and Deploy Email Service

```bash
# On local machine
cd "/Users/intelifoxdz/RS Projects/Engine_search"

# Add dependencies
cargo add chrono
cargo add oauth2

# Build
cargo build --release --bin email-service

# Upload to VPS
docker build --platform linux/amd64 -f Dockerfile.email -t email-service:latest .
docker save email-service:latest | gzip | ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206 'gunzip | docker load'

# On VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Update environment
cat >> /opt/arack/.env.production << 'EOF'
HYDRA_PUBLIC_URL=http://hydra:4444
HYDRA_CLIENT_ID=email-service
HYDRA_CLIENT_SECRET=7X9Ks2Lm4Np6Rq8Tv0Uw3Yx5Za1Bc4Df6Gh9Jk2M=
EOF

# Restart email service
docker stop search_engine_email_service
docker rm search_engine_email_service
docker compose up -d email-service

# Check logs
docker logs search_engine_email_service -f
```

---

## Phase 5: Frontend OAuth Flow

**Goal:** Implement OAuth authorization flow in frontend

**Time Estimate:** 3-4 hours

### Step 5.1: Add OAuth Callback Route

**File:** `frontend-search/src/routes/oauth/callback/+page.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  let status: 'loading' | 'success' | 'error' = 'loading';
  let error: string | null = null;

  onMount(async () => {
    const code = $page.url.searchParams.get('code');
    const state = $page.url.searchParams.get('state');
    const errorParam = $page.url.searchParams.get('error');

    if (errorParam) {
      status = 'error';
      error = $page.url.searchParams.get('error_description') || 'Authorization failed';
      return;
    }

    if (!code) {
      status = 'error';
      error = 'No authorization code received';
      return;
    }

    try {
      // Email service will handle the code exchange
      const response = await fetch(
        `/api/mail/oauth/callback?code=${code}&state=${state || ''}`,
        {
          credentials: 'include', // Include session cookie
          redirect: 'manual'
        }
      );

      if (response.ok || response.type === 'opaqueredirect') {
        status = 'success';
        // Redirect to email app after 2 seconds
        setTimeout(() => {
          goto('/mail');
        }, 2000);
      } else {
        status = 'error';
        const data = await response.json();
        error = data.error || 'Token exchange failed';
      }
    } catch (e) {
      status = 'error';
      error = e instanceof Error ? e.message : 'Unknown error';
    }
  });
</script>

<div class="container">
  {#if status === 'loading'}
    <div class="loading">
      <div class="spinner"></div>
      <h2>Completing authentication...</h2>
    </div>
  {:else if status === 'success'}
    <div class="success">
      <svg class="checkmark" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 52 52">
        <circle class="checkmark__circle" cx="26" cy="26" r="25" fill="none"/>
        <path class="checkmark__check" fill="none" d="M14.1 27.2l7.1 7.2 16.7-16.8"/>
      </svg>
      <h2>Authentication successful!</h2>
      <p>Redirecting to your mailbox...</p>
    </div>
  {:else if status === 'error'}
    <div class="error">
      <h2>Authentication failed</h2>
      <p>{error}</p>
      <button on:click={() => goto('/auth/login')}>
        Try again
      </button>
    </div>
  {/if}
</div>

<style>
  .container {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 2rem;
  }

  .loading, .success, .error {
    text-align: center;
    max-width: 400px;
  }

  .spinner {
    border: 4px solid #f3f3f3;
    border-top: 4px solid #3498db;
    border-radius: 50%;
    width: 50px;
    height: 50px;
    animation: spin 1s linear infinite;
    margin: 0 auto 1rem;
  }

  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  .checkmark {
    width: 100px;
    height: 100px;
    margin: 0 auto 1rem;
  }

  .checkmark__circle {
    stroke: #4caf50;
    stroke-width: 2;
    stroke-dasharray: 166;
    stroke-dashoffset: 166;
    animation: stroke 0.6s cubic-bezier(0.65, 0, 0.45, 1) forwards;
  }

  .checkmark__check {
    stroke: #4caf50;
    stroke-width: 2;
    stroke-dasharray: 48;
    stroke-dashoffset: 48;
    animation: stroke 0.3s cubic-bezier(0.65, 0, 0.45, 1) 0.6s forwards;
  }

  @keyframes stroke {
    100% {
      stroke-dashoffset: 0;
    }
  }

  button {
    margin-top: 1rem;
    padding: 0.75rem 1.5rem;
    background: #3498db;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }

  button:hover {
    background: #2980b9;
  }
</style>
```

### Step 5.2: Add OAuth Initiation to Mail Route

**File:** `frontend-search/src/routes/mail/+page.svelte`

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';

  let loading = true;
  let needsAuth = false;
  let authUrl = '';

  onMount(async () => {
    try {
      // Try to load mailboxes
      const response = await fetch('/api/mail/mailboxes', {
        credentials: 'include'
      });

      if (response.status === 401) {
        const data = await response.json();
        if (data.auth_url) {
          // Need OAuth authorization
          needsAuth = true;
          authUrl = data.auth_url;
          loading = false;
          return;
        }
      }

      if (response.ok) {
        // Success - mailboxes loaded
        loading = false;
        // Continue with normal mail UI
      } else {
        throw new Error('Failed to load mailboxes');
      }
    } catch (error) {
      console.error('Error loading mail:', error);
      loading = false;
    }
  });

  function initiateAuth() {
    window.location.href = authUrl;
  }
</script>

{#if loading}
  <div class="loading">Loading...</div>
{:else if needsAuth}
  <div class="auth-prompt">
    <h2>Email Access Required</h2>
    <p>You need to grant permission to access your email.</p>
    <button on:click={initiateAuth}>
      Authorize Email Access
    </button>
  </div>
{:else}
  <!-- Normal mail UI -->
  <div class="mail-app">
    <!-- Mailbox list, message list, etc. -->
  </div>
{/if}
```

### Step 5.3: Update Email API Client

**File:** `frontend-search/src/lib/api/email.ts`

```typescript
export class EmailAPI {
  private baseUrl = '';

  async getMailboxes() {
    const response = await fetch(`${this.baseUrl}/api/mail/mailboxes`, {
      credentials: 'include'
    });

    if (response.status === 401) {
      const data = await response.json();
      if (data.auth_url) {
        // Redirect to OAuth flow
        window.location.href = data.auth_url;
        throw new Error('OAuth authorization required');
      }
    }

    if (!response.ok) {
      throw new Error('Failed to fetch mailboxes');
    }

    return response.json();
  }

  async initiateOAuth() {
    const response = await fetch(`${this.baseUrl}/api/mail/oauth/initiate`, {
      credentials: 'include'
    });

    if (!response.ok) {
      throw new Error('Failed to initiate OAuth');
    }

    const data = await response.json();
    return data.auth_url;
  }
}
```

---

## Phase 6: Testing & Verification

**Goal:** End-to-end testing of OIDC flow

**Time Estimate:** 2-3 hours

### Step 6.1: Test OAuth Client Registration

```bash
# Verify client exists
curl -s http://localhost:4445/admin/clients/email-service | python3 -m json.tool

# Expected output should include:
# - client_id: "email-service"
# - grant_types: ["authorization_code", "refresh_token"]
# - redirect_uris with mail.arack.io
```

### Step 6.2: Test Login Handler

```bash
# Simulate Hydra calling login handler
# 1. Create a login challenge (via Hydra admin API)
curl -X POST http://localhost:4445/admin/oauth2/auth/requests/login \
  -H "Content-Type: application/json" \
  -d '{
    "subject": "test-user",
    "skip": false,
    "challenge": "test-challenge"
  }'

# 2. Call login handler with challenge
curl -i "https://api.arack.io/api/hydra/login?login_challenge=CHALLENGE_HERE" \
  -H "Cookie: ory_kratos_session=YOUR_SESSION"

# Should redirect or accept login
```

### Step 6.3: Test Full OAuth Flow (Manual)

**Step-by-step manual test:**

1. **Login to Kratos:**
   ```bash
   # Navigate to https://arack.io/auth/login
   # Login with existing user
   # Note session cookie in browser DevTools
   ```

2. **Initiate OAuth Flow:**
   ```bash
   # Navigate to:
   https://auth.arack.io/oauth2/auth?\
   client_id=email-service&\
   response_type=code&\
   scope=openid+email+profile+offline_access&\
   redirect_uri=https://mail.arack.io/oauth/callback&\
   state=test123
   ```

3. **Flow should:**
   - Check if user authenticated (Kratos session exists)
   - Call login handler → Accept login
   - Call consent handler → Accept consent
   - Redirect to callback with authorization code

4. **Verify callback:**
   ```bash
   # Should redirect to: https://mail.arack.io/oauth/callback?code=XXX&state=test123
   # Frontend exchanges code for tokens
   # Email service stores tokens in cache
   ```

5. **Test mailbox access:**
   ```bash
   curl -s https://api-mail.arack.io/api/mail/mailboxes \
     -H "Cookie: ory_kratos_session=YOUR_SESSION" | python3 -m json.tool

   # Should return mailbox list (not authentication error)
   ```

### Step 6.4: Test Token Refresh

```bash
# Wait for token to expire (or manually expire in cache)
# Then call mailboxes again
curl -s https://api-mail.arack.io/api/mail/mailboxes \
  -H "Cookie: ory_kratos_session=YOUR_SESSION"

# Should automatically refresh token and return mailboxes
```

### Step 6.5: Check Stalwart OIDC Validation

```bash
# Check Stalwart logs for OIDC validation
docker logs search_engine_stalwart -f

# Should see:
# [INFO] OIDC token validation request from user
# [INFO] Userinfo endpoint: http://hydra:4444/userinfo
# [INFO] Token valid for user: yacine.wanik@arack.io
```

### Step 6.6: End-to-End Integration Test

**Complete user journey:**

1. User registers at `https://arack.io/auth/register`
2. User navigates to `https://mail.arack.io`
3. App detects no OAuth tokens
4. User clicks "Authorize Email Access"
5. Redirects to Hydra OAuth flow
6. Hydra validates Kratos session
7. Login handler accepts (user already logged in)
8. Consent handler auto-accepts
9. Callback receives authorization code
10. Email service exchanges code for tokens
11. Mailboxes load successfully
12. User sends/receives emails
13. Token refreshes automatically when expired

**Success Criteria:**
- ✅ No "Authentication failed" errors
- ✅ Mailboxes load with real data
- ✅ Can send emails
- ✅ Can read emails
- ✅ Tokens refresh automatically
- ✅ ONE password used throughout (Kratos password)

---

## Phase 7: Production Deployment

**Goal:** Deploy to production with proper security

**Time Estimate:** 2-3 hours

### Step 7.1: Generate Production Secrets

```bash
# Generate strong secrets
HYDRA_CLIENT_SECRET=$(openssl rand -base64 48)
HYDRA_SYSTEM_SECRET=$(openssl rand -base64 48)

echo "HYDRA_CLIENT_SECRET=$HYDRA_CLIENT_SECRET"
echo "HYDRA_SYSTEM_SECRET=$HYDRA_SYSTEM_SECRET"

# Save these securely!
```

### Step 7.2: Update Production Configuration

**File:** `/opt/arack/.env.production`

```bash
# Hydra OAuth
HYDRA_PUBLIC_URL=https://auth.arack.io
HYDRA_ADMIN_URL=http://hydra:4445
HYDRA_CLIENT_ID=email-service
HYDRA_CLIENT_SECRET=<GENERATED_SECRET>

# Stalwart
STALWART_OIDC_ENABLED=true
```

**File:** `/opt/arack/docker-compose.yml`

```yaml
hydra:
  environment:
    - URLS_SELF_ISSUER=https://auth.arack.io
    - URLS_LOGIN=https://api.arack.io/api/hydra/login
    - URLS_CONSENT=https://api.arack.io/api/hydra/consent
    - SECRETS_SYSTEM=${HYDRA_SYSTEM_SECRET}  # Use generated secret
```

### Step 7.3: Update OAuth Client with Production Secret

```bash
# Update client with new secret
curl -X PUT http://localhost:4445/admin/clients/email-service \
  -H "Content-Type: application/json" \
  -d '{
    "client_id": "email-service",
    "client_name": "Email Service",
    "client_secret": "'$HYDRA_CLIENT_SECRET'",
    "grant_types": ["authorization_code", "refresh_token"],
    "response_types": ["code"],
    "redirect_uris": ["https://mail.arack.io/oauth/callback"],
    "scope": "openid email profile offline_access",
    "token_endpoint_auth_method": "client_secret_post"
  }'
```

### Step 7.4: Configure nginx for Hydra

**File:** `/opt/arack/nginx/sites-enabled/arack.io.conf`

**Add Hydra proxy:**

```nginx
# Ory Hydra (auth.arack.io)
server {
    listen 443 ssl http2;
    server_name auth.arack.io;

    ssl_certificate /etc/letsencrypt/live/arack.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/arack.io/privkey.pem;

    # Hydra public endpoints
    location / {
        proxy_pass http://search_engine_hydra:4444;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

**Reload nginx:**

```bash
docker exec arack_nginx nginx -t
docker exec arack_nginx nginx -s reload
```

### Step 7.5: SSL Certificate for auth.arack.io

```bash
# Add auth.arack.io to certificate
certbot certonly --webroot -w /var/www/html \
  -d auth.arack.io \
  --email your@email.com \
  --agree-tos \
  --non-interactive

# Reload nginx
docker exec arack_nginx nginx -s reload
```

### Step 7.6: Restart All Services

```bash
# Restart in correct order
docker compose restart postgres
docker compose restart hydra-migrate
docker compose restart hydra
docker compose restart kratos
docker compose restart search-service
docker compose restart email-service
docker compose restart stalwart

# Verify all healthy
docker ps | grep -E "(hydra|kratos|search|email|stalwart)"
```

### Step 7.7: Production Verification

```bash
# 1. Test Hydra public endpoint
curl -s https://auth.arack.io/.well-known/openid-configuration | python3 -m json.tool

# 2. Test login handler
curl -i https://api.arack.io/api/hydra/login

# 3. Test consent handler
curl -i https://api.arack.io/api/hydra/consent

# 4. Test OAuth initiate
curl -s https://api-mail.arack.io/api/mail/oauth/initiate \
  -H "Cookie: ory_kratos_session=YOUR_SESSION" | python3 -m json.tool

# 5. Full manual OAuth flow via browser
# Navigate to: https://auth.arack.io/oauth2/auth?client_id=email-service&...
```

---

## Rollback Plan

### If OIDC Implementation Fails

**Quick Rollback to Master User Pattern:**

```bash
# 1. Revert email service to previous version
docker stop search_engine_email_service
docker rm search_engine_email_service

# Load backup image (if saved)
docker load < email-service-backup.tar.gz
docker compose up -d email-service

# 2. Revert Stalwart config
# Remove OIDC backend, use default password auth
# Restart Stalwart

# 3. Email features will work with DEFAULT_EMAIL_PASSWORD
```

**Partial Rollback (Keep Hydra for Future):**

```bash
# Keep Hydra running but disable OIDC in email service
# Set environment variable:
USE_OIDC_AUTH=false

# Email service falls back to default password
docker restart search_engine_email_service
```

---

## Troubleshooting Guide

### Issue: Login Handler Returns 500 Error

**Diagnosis:**
```bash
docker logs search_engine_search_service | grep hydra
```

**Common causes:**
- Hydra admin URL incorrect
- Can't reach Hydra from search service
- Invalid login challenge parameter

**Fix:**
```bash
# Test connectivity
docker exec search_engine_search_service curl http://hydra:4445/health/ready

# Verify environment variable
docker exec search_engine_search_service env | grep HYDRA
```

### Issue: Token Exchange Fails

**Diagnosis:**
```bash
docker logs search_engine_email_service | grep -i token
```

**Common causes:**
- Wrong client secret
- Invalid authorization code (expired or already used)
- Wrong redirect URI

**Fix:**
```bash
# Verify client configuration
curl http://localhost:4445/admin/clients/email-service | python3 -m json.tool

# Check client secret matches in:
# 1. Hydra client registration
# 2. Email service environment variable
```

### Issue: Stalwart Rejects Bearer Token

**Diagnosis:**
```bash
docker logs search_engine_stalwart | grep -i oidc
docker logs search_engine_stalwart | grep -i token
```

**Common causes:**
- Stalwart can't reach Hydra userinfo endpoint
- OIDC backend not configured correctly
- Token expired

**Fix:**
```bash
# Test Stalwart → Hydra connectivity
docker exec search_engine_stalwart curl http://hydra:4444/userinfo

# Verify Stalwart config
docker exec search_engine_stalwart cat /opt/stalwart/etc/config.toml | grep -A 10 oidc
```

### Issue: Infinite Redirect Loop

**Diagnosis:**
Browser keeps redirecting between Hydra, login handler, consent handler

**Common causes:**
- Login handler not accepting login properly
- Consent handler not accepting consent
- Missing or invalid subject in accept request

**Fix:**
```bash
# Check login acceptance logs
docker logs search_engine_search_service | grep "accept login"

# Verify subject (identity_id) is being set correctly
# Add debug logging to login handler
```

---

## Security Checklist

Before going to production:

- [ ] Strong client secret generated (48+ characters)
- [ ] Strong Hydra system secret generated
- [ ] Secrets stored in `.env.production`, not in code
- [ ] HTTPS enabled for all public endpoints (auth.arack.io)
- [ ] Token expiry configured appropriately (15-60 min for access tokens)
- [ ] Refresh tokens stored securely (encrypted in cache/database)
- [ ] CORS configured correctly (only allow arack.io domains)
- [ ] Hydra `--dev` flag removed in production
- [ ] Rate limiting on OAuth endpoints
- [ ] Logging of all OAuth flows (for security audits)
- [ ] Session replay attack prevention (validate state parameter)
- [ ] PKCE enabled for public clients (if adding mobile support)

---

## Performance Optimization

### Token Caching Strategy

Current implementation uses in-memory cache (`HashMap`). For production:

**Option 1: Redis Cache**
```rust
// Store tokens in Redis with TTL
redis.set_ex(
    format!("oauth:token:{}", user_id),
    token_json,
    expires_in as usize
).await?;
```

**Option 2: PostgreSQL**
```sql
CREATE TABLE oauth_tokens (
    user_id UUID PRIMARY KEY,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_oauth_tokens_expires ON oauth_tokens(expires_at);
```

### Connection Pooling

Hydra admin client should reuse HTTP connections:

```rust
// Use persistent client
lazy_static! {
    static ref HYDRA_CLIENT: reqwest::Client = reqwest::Client::builder()
        .pool_max_idle_per_host(10)
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
}
```

---

## Success Metrics

**How to know OIDC implementation is successful:**

### Technical Metrics

1. **Authentication Success Rate**
   - Target: >99% of OAuth flows complete successfully
   - Monitor: Hydra admin API `/metrics` endpoint

2. **Token Refresh Rate**
   - Target: <1% of requests require user re-authentication
   - Most requests use cached or refreshed tokens

3. **JMAP Auth Errors**
   - Target: 0 "Authentication failed" errors in email service logs
   - All JMAP requests use valid Bearer tokens

4. **Response Times**
   - Token exchange: <500ms
   - Token validation (Stalwart): <100ms
   - End-to-end OAuth flow: <3 seconds

### User Experience Metrics

1. **Single Sign-On**
   - User logs in once (Kratos)
   - Email access granted via OAuth (no second password)
   - Stays authenticated for 7+ days

2. **Seamless Access**
   - First email access requires one-click "Authorize"
   - Subsequent access is instant (cached tokens)
   - Token refresh happens transparently

3. **Zero Password Confusion**
   - Users only know ONE password (Kratos registration password)
   - No separate email password to remember
   - True unified authentication

---

## Documentation

After implementation, create user-facing documentation:

### For Users

**"How to Access Your Email"**
1. Login to your account at arack.io
2. Navigate to mail.arack.io
3. Click "Authorize Email Access" (first time only)
4. Your mailbox will load automatically

### For Developers

**"OAuth Flow Architecture"**
- Document complete flow with sequence diagrams
- API endpoint documentation
- Token management best practices
- Troubleshooting guide

---

## Next Steps After OIDC Implementation

Once OIDC is working:

1. **Add MFA Support**
   - Hydra supports WebAuthn/TOTP
   - Kratos handles MFA enrollment
   - Transparent to email service

2. **Mobile App Support**
   - Use same OAuth flow
   - PKCE for public clients
   - Deep linking for callback

3. **Third-Party Integrations**
   - Allow other apps to access user's email (with consent)
   - Use same Hydra OAuth infrastructure

4. **Passwordless Authentication**
   - Add WebAuthn to Kratos
   - OAuth flow remains unchanged
   - Email service doesn't need updates

---

## Estimated Timeline

| Phase | Task | Time | Dependencies |
|-------|------|------|--------------|
| 1 | Hydra client registration | 0.5h | None |
| 2 | Kratos-Hydra integration | 6h | Phase 1 |
| 3 | Stalwart OIDC backend | 2h | Phase 1 |
| 4 | Email service OAuth | 8h | Phase 2, 3 |
| 5 | Frontend OAuth flow | 4h | Phase 4 |
| 6 | Testing & verification | 3h | Phase 5 |
| 7 | Production deployment | 3h | Phase 6 |
| **Total** | | **26.5 hours** | ~3-4 days |

**Breakdown by day:**
- **Day 1 (8h):** Phases 1-3 (Infrastructure setup)
- **Day 2 (8h):** Phase 4 (Email service implementation)
- **Day 3 (8h):** Phases 5-6 (Frontend + testing)
- **Day 4 (2-3h):** Phase 7 (Production deployment)

---

## Final Checklist

Before marking as complete:

- [ ] Hydra OAuth client registered
- [ ] Login handler implemented and tested
- [ ] Consent handler implemented and tested
- [ ] Stalwart OIDC backend configured
- [ ] Email service OAuth token management working
- [ ] Frontend OAuth flow complete
- [ ] End-to-end test successful
- [ ] Production secrets generated and stored securely
- [ ] nginx configured for auth.arack.io
- [ ] SSL certificate obtained
- [ ] All services restarted in production
- [ ] Monitoring/logging configured
- [ ] Documentation created
- [ ] Rollback plan tested

---

**Ready to implement?** Start with Phase 1!
