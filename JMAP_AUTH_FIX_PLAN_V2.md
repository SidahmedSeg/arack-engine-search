# JMAP Authentication Fix Plan V2 - Official Solution

**Date:** 2025-12-19
**Issue:** Users should have ONE password for everything (unified auth)
**Root Cause:** Current system requires separate passwords for Kratos and Stalwart
**Solution:** Use Stalwart **Master User** feature for admin impersonation

---

## 🔍 Problem Analysis (Based on Official Docs)

### Current Broken Architecture

**User Registration Flow:**
1. User registers at `arack.io/auth/register`
   - Email: `test@arack.io`
   - Password: `"MySecurePass123"` ← **Stored in Kratos only**
2. Kratos webhook triggers email provisioning
3. Email service creates Stalwart account with password: `"ChangeMe123!"` ← **WRONG!**
4. User tries to access email → **Authentication fails** ❌

**Why This is Wrong:**
- User expects ONE password for everything
- But Stalwart has a different password
- Email service can't authenticate to JMAP with user's real password
- Kratos webhook **doesn't send user's password** (security design)

---

## ✅ Correct Architecture (Stalwart Master User)

According to [Stalwart Administrator Documentation](https://stalw.art/docs/auth/authorization/administrator/):

> **Master User:** "A special account with full access to all mailboxes on the server"
>
> **Mailbox Impersonation:** "Once enabled, the master user can access any mailbox using the format: `<account_name>%<master_user>`"

### How It Works:

1. **User Registration:**
   - User: `test@arack.io` + password `"MySecurePass123"`
   - Kratos stores password (hashed) ✅
   - Webhook creates Stalwart account with **random password** (never used) ✅

2. **User Accesses Email:**
   - User logs in with `"MySecurePass123"` → Gets Kratos session cookie ✅
   - Email API validates session with Kratos ✅
   - Email API authenticates to JMAP as: **`test%admin`** with **admin password** ✅
   - Stalwart grants access to test's mailbox ✅

**User never authenticates to Stalwart directly!** The email service acts as a **trusted proxy**.

---

## 📋 Implementation Plan

### Phase 1: Enable Master User in Stalwart (10 minutes)

**Current Status:**
```json
{
  "fallback-admin.user": "admin",
  "fallback-admin.secret": "adminpassword"
}
```

**Master user:** ❌ Not configured

**Action:** Configure master user via Stalwart Settings API

#### Step 1.1: Configure Master User

**Method:** Use [Stalwart Management API](https://stalw.art/docs/api/management/overview/) to update settings

**API Call:**
```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Configure master user
curl -X POST -u "admin:adminpassword" \
  http://localhost:8081/api/settings \
  -H "Content-Type: application/json" \
  -d '[
    {
      "type": "set",
      "key": "authentication.master.user",
      "value": "admin"
    },
    {
      "type": "set",
      "key": "authentication.master.secret",
      "value": "adminpassword"
    }
  ]'
```

**Configuration Format** (from [official docs](https://stalw.art/docs/auth/authorization/administrator/)):
```toml
[authentication.master]
user = "admin"
secret = "adminpassword"  # Or use hashed password: "$6$..."
```

#### Step 1.2: Reload Stalwart Configuration

```bash
# Reload config to apply changes
curl -X GET -u "admin:adminpassword" \
  http://localhost:8081/api/reload
```

#### Step 1.3: Test Master User Authentication

```bash
# Test accessing yacine.wanik's mailbox via master user
curl -s -u "yacine.wanik%admin:adminpassword" \
  http://localhost:8081/jmap/session | python3 -m json.tool
```

**Expected Result:**
```json
{
  "username": "yacine.wanik",
  "accounts": {
    "l": {
      "name": "yacine.wanik",
      "isPersonal": true,
      ...
    }
  },
  ...
}
```

---

### Phase 2: Update Email Service Code (30 minutes)

#### Change 1: Update `get_jmap_session` Helper

**File:** `email/api/mod.rs` (line 400-436)

**BEFORE:**
```rust
async fn get_jmap_session(
    jmap_client: &JmapClient,
    email: &str,
    password: &str,  // ← User password (we don't have this!)
) -> Result<(JmapAuth, String), (StatusCode, Json<serde_json::Value>)> {
    let username = email.split('@').next().unwrap_or(email);

    let auth = JmapAuth::Basic {
        username: username.to_string(),
        password: password.to_string(),  // ← WRONG!
    };
    // ...
}
```

**AFTER:**
```rust
async fn get_jmap_session(
    jmap_client: &JmapClient,
    stalwart_admin_user: &str,      // ← "admin"
    stalwart_admin_password: &str,  // ← "adminpassword"
    email: &str,
) -> Result<(JmapAuth, String), (StatusCode, Json<serde_json::Value>)> {
    // Extract username from email (part before @)
    let username = email.split('@').next().unwrap_or(email);

    // Use master user impersonation format: username%master
    let master_login = format!("{}%{}", username, stalwart_admin_user);

    info!("JMAP authentication as master user for: {}", email);

    let auth = JmapAuth::Basic {
        username: master_login,                    // ← "yacine.wanik%admin"
        password: stalwart_admin_password.to_string(),  // ← Admin password
    };

    // Get JMAP session
    match jmap_client.get_session(&auth).await {
        Ok(session) => {
            let account_id = session
                .primary_accounts
                .get("urn:ietf:params:jmap:mail")
                .cloned()
                .unwrap_or_else(|| {
                    session.accounts.keys().next().cloned().unwrap_or_default()
                });
            Ok((auth, account_id))
        }
        Err(e) => {
            error!("Failed to get JMAP session for {}: {}", email, e);
            Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({ "error": "JMAP authentication failed" })),
            ))
        }
    }
}
```

#### Change 2: Update AppState to Include Admin Credentials

**File:** `email/api/mod.rs` (line 37-49)

**BEFORE:**
```rust
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub jmap_client: JmapClient,
    pub search_client: EmailSearchClient,
    pub centrifugo_client: CentrifugoClient,
    pub stalwart_admin_client: StalwartAdminClient,
    pub default_email_password: String,  // ← REMOVE THIS
    pub kratos_client: KratosClient,
    #[cfg(feature = "email")]
    pub openai_client: Client<OpenAIConfig>,
}
```

**AFTER:**
```rust
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub jmap_client: JmapClient,
    pub search_client: EmailSearchClient,
    pub centrifugo_client: CentrifugoClient,
    pub stalwart_admin_client: StalwartAdminClient,
    pub stalwart_admin_user: String,      // ← ADD: "admin"
    pub stalwart_admin_password: String,  // ← ADD: "adminpassword"
    pub kratos_client: KratosClient,
    #[cfg(feature = "email")]
    pub openai_client: Client<OpenAIConfig>,
}
```

#### Change 3: Update `create_router` Function

**File:** `email/api/mod.rs` (line 52-75)

**BEFORE:**
```rust
pub fn create_router(
    db_pool: PgPool,
    redis_client: redis::Client,
    jmap_client: JmapClient,
    search_client: EmailSearchClient,
    centrifugo_client: CentrifugoClient,
    stalwart_admin_client: StalwartAdminClient,
    default_email_password: String,  // ← REMOVE
    kratos_client: KratosClient,
    openai_client: Client<OpenAIConfig>,
) -> Router {
    let state = Arc::new(AppState {
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        default_email_password,  // ← REMOVE
        kratos_client,
        openai_client,
    });
    // ...
}
```

**AFTER:**
```rust
pub fn create_router(
    db_pool: PgPool,
    redis_client: redis::Client,
    jmap_client: JmapClient,
    search_client: EmailSearchClient,
    centrifugo_client: CentrifugoClient,
    stalwart_admin_client: StalwartAdminClient,
    stalwart_admin_user: String,      // ← ADD
    stalwart_admin_password: String,  // ← ADD
    kratos_client: KratosClient,
    openai_client: Client<OpenAIConfig>,
) -> Router {
    let state = Arc::new(AppState {
        db_pool,
        redis_client,
        jmap_client,
        search_client,
        centrifugo_client,
        stalwart_admin_client,
        stalwart_admin_user,      // ← ADD
        stalwart_admin_password,  // ← ADD
        kratos_client,
        openai_client,
    });
    // ...
}
```

#### Change 4: Update All JMAP Call Sites

**Update `list_mailboxes` (line 507):**
```rust
// BEFORE
let (auth, account_id) = match get_jmap_session(
    &state.jmap_client,
    &email,
    &state.default_email_password,  // ← REMOVE
).await {

// AFTER
let (auth, account_id) = match get_jmap_session(
    &state.jmap_client,
    &state.stalwart_admin_user,      // ← ADD
    &state.stalwart_admin_password,  // ← ADD
    &email,
).await {
```

**Repeat for:**
- `list_messages` (line 676)
- `send_message` (line 801)
- `get_message` (line 750)
- `create_mailbox` (line 564)

#### Change 5: Update `main.rs` (Email Service Entry Point)

**File:** `email/main.rs` (or wherever `create_router` is called)

**BEFORE:**
```rust
let default_email_password = std::env::var("DEFAULT_EMAIL_PASSWORD")
    .expect("DEFAULT_EMAIL_PASSWORD must be set");

let router = email::api::create_router(
    db_pool.clone(),
    redis_client.clone(),
    jmap_client.clone(),
    search_client.clone(),
    centrifugo_client.clone(),
    stalwart_admin_client.clone(),
    default_email_password,  // ← REMOVE
    kratos_client.clone(),
    openai_client,
);
```

**AFTER:**
```rust
let stalwart_admin_user = std::env::var("STALWART_ADMIN_USER")
    .expect("STALWART_ADMIN_USER must be set");
let stalwart_admin_password = std::env::var("STALWART_ADMIN_PASSWORD")
    .expect("STALWART_ADMIN_PASSWORD must be set");

let router = email::api::create_router(
    db_pool.clone(),
    redis_client.clone(),
    jmap_client.clone(),
    search_client.clone(),
    centrifugo_client.clone(),
    stalwart_admin_client.clone(),
    stalwart_admin_user,      // ← ADD
    stalwart_admin_password,  // ← ADD
    kratos_client.clone(),
    openai_client,
);
```

---

### Phase 3: Update Provisioning (15 minutes)

**File:** `email/provisioning/mod.rs` (line 102-148)

**Change:** Use random password during account creation (user will never use it)

**BEFORE:**
```rust
pub async fn provision_email_account_full(
    db_pool: &PgPool,
    stalwart_client: &StalwartAdminClient,
    jmap_client: &JmapClient,
    payload: KratosWebhookPayload,
    default_password: &str,  // ← REMOVE this parameter
) -> Result<ProvisioningResponse> {
    // ...
    let stalwart_principal_id = stalwart_client
        .create_account(
            email,
            default_password,  // ← User never uses this!
            display_name.as_deref(),
            Some(5_368_709_120),
        )
        .await?;
    // ...
}
```

**AFTER:**
```rust
pub async fn provision_email_account_full(
    db_pool: &PgPool,
    stalwart_client: &StalwartAdminClient,
    jmap_client: &JmapClient,
    payload: KratosWebhookPayload,
    // ← Removed default_password parameter
) -> Result<ProvisioningResponse> {
    // ...

    // Generate a random password (user will never use it - master user access instead)
    let random_password = uuid::Uuid::new_v4().to_string();

    info!(
        "Creating Stalwart account for {} with random password (master user will be used for access)",
        email
    );

    let stalwart_principal_id = stalwart_client
        .create_account(
            email,
            &random_password,  // ← Random password, never used
            display_name.as_deref(),
            Some(5_368_709_120),
        )
        .await?;
    // ...
}
```

**Update provisioning webhook handler (email/api/mod.rs line 222-228):**

**BEFORE:**
```rust
match provisioning::provision_email_account_full(
    &state.db_pool,
    &state.stalwart_admin_client,
    &state.jmap_client,
    payload.clone(),
    &state.default_email_password,  // ← REMOVE
).await {
```

**AFTER:**
```rust
match provisioning::provision_email_account_full(
    &state.db_pool,
    &state.stalwart_admin_client,
    &state.jmap_client,
    payload.clone(),
    // ← Removed default_email_password
).await {
```

---

### Phase 4: Remove `DEFAULT_EMAIL_PASSWORD` Environment Variable (5 minutes)

#### Change 1: Update `docker-compose.yml`

**File:** `/opt/arack/docker-compose.yml`

**BEFORE:**
```yaml
email-service:
  environment:
    - STALWART_ADMIN_USER=admin
    - STALWART_ADMIN_PASSWORD=adminpassword
    - DEFAULT_EMAIL_PASSWORD=ChangeMe123!  # ← REMOVE THIS LINE
```

**AFTER:**
```yaml
email-service:
  environment:
    - STALWART_ADMIN_USER=admin
    - STALWART_ADMIN_PASSWORD=adminpassword
    # DEFAULT_EMAIL_PASSWORD removed - using master user instead
```

#### Change 2: Update `.env.production`

**File:** `/opt/arack/.env.production`

**Remove:**
```bash
DEFAULT_EMAIL_PASSWORD=8yH1v9eW24DSaDbUpioHag==  # ← DELETE THIS
```

---

## 🚀 Deployment Steps

### Step 1: Configure Stalwart Master User (VPS - 10 minutes)

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Configure master user
curl -X POST -u "admin:adminpassword" \
  http://localhost:8081/api/settings \
  -H "Content-Type: application/json" \
  -d '[
    {
      "type": "set",
      "key": "authentication.master.user",
      "value": "admin"
    },
    {
      "type": "set",
      "key": "authentication.master.secret",
      "value": "adminpassword"
    }
  ]'

# Reload config
curl -X GET -u "admin:adminpassword" http://localhost:8081/api/reload

# Test master user login
curl -s -u "yacine.wanik%admin:adminpassword" \
  http://localhost:8081/jmap/session | python3 -m json.tool | head -30

# Should return session data for yacine.wanik ✅
```

### Step 2: Build Updated Email Service (Local - 10 minutes)

```bash
# On local machine
cd "/Users/intelifoxdz/RS Projects/Engine_search"

# Build for Linux
docker build --platform linux/amd64 -f Dockerfile.email -t email-service:latest .

# Save and upload
docker save email-service:latest | gzip | \
  ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206 'gunzip | docker load'
```

### Step 3: Update docker-compose.yml (VPS - 5 minutes)

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Edit docker-compose.yml
nano /opt/arack/docker-compose.yml

# Remove DEFAULT_EMAIL_PASSWORD line from email-service environment
# Save and exit (Ctrl+X, Y, Enter)
```

### Step 4: Restart Email Service (VPS - 2 minutes)

```bash
# Stop email service
docker stop search_engine_email_service
docker rm search_engine_email_service

# Start with new configuration
cd /opt/arack
docker-compose up -d email-service

# Check logs
docker logs -f search_engine_email_service
# Look for: "JMAP authentication as master user for: ..."
```

### Step 5: Test Email API (VPS - 5 minutes)

```bash
# Get session cookie from browser (DevTools → Application → Cookies)
COOKIE="ory_kratos_session=YOUR_SESSION_HERE"

# Test mailboxes
curl -H "Cookie: $COOKIE" https://api-mail.arack.io/api/mail/mailboxes | python3 -m json.tool

# Expected response:
# {
#   "mailboxes": [
#     {"id": "...", "name": "Inbox", "role": "inbox", ...},
#     {"id": "...", "name": "Sent", "role": "sent", ...},
#     ...
#   ],
#   "total": 5
# }
```

---

## ✅ Success Criteria

**Fix is successful when:**

1. ✅ Stalwart master user configured and working
2. ✅ Master user login `yacine.wanik%admin:adminpassword` returns JMAP session
3. ✅ `GET /api/mail/mailboxes` returns mailbox list (not auth error)
4. ✅ `GET /api/mail/messages` returns messages (or empty list)
5. ✅ No 401 Unauthorized errors in email service logs
6. ✅ User can register and access email with ONE password
7. ✅ `DEFAULT_EMAIL_PASSWORD` environment variable removed

---

## 🔄 Rollback Plan

**If deployment fails:**

### Code Rollback

```bash
# Revert Docker image
docker stop search_engine_email_service
docker rm search_engine_email_service
docker tag email-service:old email-service:latest
docker-compose up -d email-service
```

### Stalwart Config Rollback

```bash
# Remove master user config
curl -X POST -u "admin:adminpassword" \
  http://localhost:8081/api/settings \
  -H "Content-Type: application/json" \
  -d '[
    {"type": "delete", "key": "authentication.master.user"},
    {"type": "delete", "key": "authentication.master.secret"}
  ]'

# Reload
curl -X GET -u "admin:adminpassword" http://localhost:8081/api/reload
```

---

## 📊 Benefits of This Approach

### User Experience ✅
- **ONE password for everything** (Kratos login)
- User never knows Stalwart exists
- Seamless experience across search and email

### Security ✅
- User passwords never leave Kratos
- Random Stalwart passwords never used
- Admin credentials secured server-side
- Master user access logged in Stalwart

### Architecture ✅
- Clean separation of concerns
- Email service acts as trusted proxy
- Follows [Stalwart best practices](https://stalw.art/docs/auth/authorization/administrator/)
- No password synchronization needed

### Maintainability ✅
- Simple code (remove password management logic)
- Fewer environment variables
- Easier debugging (one auth path)

---

## 📚 References

**Official Documentation:**
- [Stalwart Administrator Guide](https://stalw.art/docs/auth/authorization/administrator/) - Master user configuration
- [Stalwart Management API](https://stalw.art/docs/api/management/overview/) - Settings API
- [Stalwart OpenAPI Spec](https://github.com/stalwartlabs/stalwart/blob/main/api/v1/openapi.yml) - API endpoints
- [Stalwart JMAP Documentation](https://stalw.art/docs/email/jmap/) - JMAP authentication

**Key Concepts:**
- Master user impersonation format: `username%master`
- Configuration via REST API: `POST /api/settings`
- Reload config: `GET /api/reload`

---

## ⏱️ Estimated Timeline

| Phase | Time | Description |
|-------|------|-------------|
| **Phase 1** | 10 min | Configure Stalwart master user |
| **Phase 2** | 30 min | Update email service code |
| **Phase 3** | 15 min | Update provisioning |
| **Phase 4** | 5 min | Remove DEFAULT_EMAIL_PASSWORD |
| **Deployment** | 30 min | Build, upload, test |
| **Total** | **1.5 hours** | Full implementation |

---

**Status:** ✅ Ready to implement
**Risk Level:** Low (can rollback easily)
**Impact:** High (fixes fundamental architecture issue)
