# Ory Hydra Migration Fix - Complete Summary

**Date:** 2025-12-19
**Status:** ✅ FIXED AND OPERATIONAL
**Issue:** Hydra migrations failing with "relation 'networks' already exists"
**Root Cause:** Hydra configured to use wrong database (shared with Kratos)
**Solution:** Changed Hydra DSN to use dedicated `hydra_db` database

---

## 🔍 Problem Analysis

### Original Issue

When attempting to use Ory Hydra for OIDC authentication (as discovered during JMAP auth research), Hydra was not functional:

```
ERROR: relation "networks" already exists (SQLSTATE 42P07)
error executing migrations/20150101000001000000_networks.postgres.up.sql
```

**Health check failed:**
```json
{
  "error": "migrations have not yet been fully applied",
  "status": "Instance is not yet ready"
}
```

### Root Cause Investigation

1. **Database Conflict:**
   - Hydra DSN: `postgres://postgres:postgres@postgres:5432/engine_search`
   - Kratos DSN: `postgres://postgres:postgres@postgres:5432/kratos_db`
   - **Problem:** Hydra was pointing to `engine_search` instead of its own database

2. **Migration Table Conflict:**
   - Both Kratos and Hydra use `schema_migration` table for tracking
   - `engine_search` database had 315 Kratos migration entries
   - Hydra tried to apply 195 migrations but saw conflicts
   - First migration (create `networks` table) failed because Kratos already created it

3. **Database State:**
   - `engine_search` DB: Contains search service, email service, user preferences, Kratos tables
   - `kratos_db` DB: Kratos identity/session tables
   - `hydra_db` DB: **Existed but unused** (created earlier but Hydra wasn't configured to use it)

---

## 🔧 Fix Applied

### 1. Backup Configuration

```bash
cd /opt/arack
cp docker-compose.yml docker-compose.yml.backup-hydra-fix-20251219-225207
```

### 2. Update Hydra DSN

**Changed in `docker-compose.yml`:**

**Before:**
```yaml
environment:
  - DSN=postgres://postgres:postgres@postgres:5432/engine_search?sslmode=disable
```

**After:**
```yaml
environment:
  - DSN=postgres://postgres:postgres@postgres:5432/hydra_db?sslmode=disable
```

**Two locations updated:**
- `hydra-migrate` service (migration runner)
- `hydra` service (main server)

### 3. Restart and Migrate

```bash
# Stop old Hydra containers
docker compose stop hydra hydra-migrate
docker compose rm -f hydra hydra-migrate

# Run migrations against clean hydra_db
docker compose up -d hydra-migrate

# Start Hydra server
docker compose up -d hydra
```

---

## ✅ Verification Results

### Migration Success

**All 195 Hydra migrations applied successfully:**

```
time=2025-12-19T22:52:21Z level=info msg=> client_add_logout_skip_consent_column applied successfully
Successfully applied migrations!
```

**Migration versions:** 20150101000001000000 → 20240129174410000001

### Health Check

```bash
curl http://localhost:4444/health/ready
```

**Response:**
```json
{
  "status": "ok"
}
```

### OIDC Discovery

```bash
curl http://localhost:4444/.well-known/openid-configuration
```

**Response includes:**
```json
{
  "issuer": "http://127.0.0.1:4444/",
  "authorization_endpoint": "http://127.0.0.1:4444/oauth2/auth",
  "token_endpoint": "http://127.0.0.1:4444/oauth2/token",
  "jwks_uri": "http://127.0.0.1:4444/.well-known/jwks.json",
  "userinfo_endpoint": "http://127.0.0.1:4444/userinfo",
  "subject_types_supported": ["public"],
  "response_types_supported": ["code", "code id_token", "id_token", ...],
  "grant_types_supported": ["authorization_code", "implicit", "client_credentials", "refresh_token"]
}
```

### Database Tables

**20 Hydra tables created in `hydra_db`:**
- `hydra_client` - OAuth2 client configurations
- `hydra_jwk` - JSON Web Keys
- `hydra_oauth2_access` - Access tokens
- `hydra_oauth2_authentication_session` - Authentication sessions
- `hydra_oauth2_code` - Authorization codes
- `hydra_oauth2_flow` - OAuth2 flows
- `hydra_oauth2_jti_blacklist` - Token blacklist
- `hydra_oauth2_logout_request` - Logout requests
- `hydra_oauth2_oidc` - OIDC-specific data
- `hydra_oauth2_pkce` - PKCE verification
- `hydra_oauth2_refresh` - Refresh tokens
- And 9 more tables

### Service Status

```
✅ search_engine_hydra          - Up, ports 4444-4445
✅ search_engine_kratos         - Up, ports 4433-4434
✅ search_engine_search_service - Up, port 3000
✅ search_engine_email_service  - Up, port 3001
✅ search_engine_stalwart       - Up, port 8081 (JMAP/SMTP)
✅ search_engine_postgres       - Healthy
✅ search_engine_redis          - Healthy
✅ search_engine_meilisearch    - Up
✅ search_engine_centrifugo     - Up
```

---

## 📊 Database Architecture (Final State)

### Three Separate Databases

| Database | Purpose | Services |
|----------|---------|----------|
| **engine_search** | Main application data | Search service, Email service, User preferences |
| **kratos_db** | Identity & session management | Ory Kratos |
| **hydra_db** | OAuth2/OIDC provider | Ory Hydra |

**Why separate databases?**
- ✅ Isolation - Each service owns its data
- ✅ No migration conflicts - Independent migration tracking
- ✅ Security - Hydra/Kratos can't access application data
- ✅ Scalability - Can scale databases independently
- ✅ Best practice - Microservices pattern

---

## 🎯 Next Steps: Implement OIDC Authentication for Email Service

Now that Hydra is functional, we can implement **standards-compliant OIDC authentication** for the email service instead of the Master User workaround.

### Option 1: Master User Pattern (Current Plan - Simple)

**Pros:**
- ✅ Simple implementation (~1.5 hours)
- ✅ Works immediately with existing infrastructure
- ✅ No frontend changes needed
- ✅ Industry-standard proxy pattern

**Cons:**
- ⚠️ Uses admin credentials server-side (secure but not ideal)
- ⚠️ Not standards-compliant OAuth flow

**Implementation:** See `JMAP_AUTH_FIX_PLAN_V2.md`

### Option 2: OIDC via Hydra (Now Possible - Standards-Compliant)

**Pros:**
- ✅ Standards-compliant OAuth 2.0 / OIDC flow
- ✅ True unified authentication (ONE password)
- ✅ Token-based authentication
- ✅ Scalable to other services
- ✅ Audit trail of all authentications

**Cons:**
- ⚠️ More complex implementation (~2-3 days)
- ⚠️ Requires email service code changes
- ⚠️ Requires Stalwart OIDC backend configuration
- ⚠️ Requires frontend OAuth flow implementation

### Recommended Approach: OIDC (Now That Hydra Works)

**Architecture:**
```
User Login Flow:
1. User logs in via Kratos (email/password)
2. Kratos creates session
3. Frontend redirects to Hydra for OAuth authorization
4. Hydra issues access token
5. Frontend stores token
6. Email API uses token to access JMAP via Stalwart OIDC backend

JMAP Authentication Flow:
1. Email API receives request with session cookie
2. Validate session with Kratos
3. Get access token for user from Hydra
4. Authenticate to Stalwart JMAP using OAUTHBEARER mechanism
5. Stalwart validates token with Hydra
6. Return mailbox data
```

**Key Components to Implement:**

1. **Hydra OAuth Client Registration:**
   ```bash
   # Register email service as OAuth client
   curl -X POST http://localhost:4445/admin/clients \
     -H "Content-Type: application/json" \
     -d '{
       "client_id": "email-service",
       "client_secret": "GENERATE_SECURE_SECRET",
       "grant_types": ["authorization_code", "refresh_token"],
       "response_types": ["code"],
       "redirect_uris": ["https://mail.arack.io/oauth/callback"],
       "scope": "openid email profile",
       "token_endpoint_auth_method": "client_secret_post"
     }'
   ```

2. **Stalwart OIDC Backend Configuration:**
   ```toml
   # /opt/arack/stalwart/config.toml
   [directory."oidc"]
   type = "oidc"
   endpoint.url = "http://hydra:4444/userinfo"
   endpoint.method = "userinfo"
   fields.email = "email"
   fields.username = "preferred_username"
   ```

3. **Email Service Code Changes:**
   - Add Hydra client (OAuth2 library)
   - Implement token exchange flow
   - Update JMAP authentication to use Bearer tokens
   - Add token refresh logic

4. **Search Service Hydra Integration:**
   - Add login/consent handlers at `/api/hydra/login` and `/api/hydra/consent`
   - Return Kratos session data to Hydra for token issuance

**Detailed Implementation Plan:** See `STALWART_AUTH_OPTIONS_ANALYSIS.md` - Option 2

---

## 📋 Decision Point

**User needs to decide:** Master User (quick) vs OIDC (proper)?

### If Master User:
- Proceed with `JMAP_AUTH_FIX_PLAN_V2.md`
- Email features working in ~1.5 hours
- Can migrate to OIDC later

### If OIDC:
- Create detailed implementation plan
- ~2-3 days of work
- Proper unified auth from the start

---

## 📁 Related Documentation

**Created during this investigation:**
- `EMAIL_FEATURES_TEST_REPORT.md` - Email service status
- `JMAP_AUTH_FIX_PLAN.md` - Initial diagnosis
- `JMAP_AUTH_FIX_PLAN_V2.md` - Master User approach
- `STALWART_AUTH_OPTIONS_ANALYSIS.md` - All authentication options
- `HYDRA_FIX_COMPLETE.md` - This file

**Key Findings:**
- Hydra WAS installed but not configured correctly
- Now Hydra is functional and ready for OIDC implementation
- User's unified auth vision is achievable

---

## 🔄 Rollback Plan (If Needed)

**If Hydra causes issues:**

```bash
# Revert to old configuration
cd /opt/arack
cp docker-compose.yml.backup-hydra-fix-20251219-225207 docker-compose.yml

# Restart services
docker compose stop hydra hydra-migrate
docker compose rm -f hydra hydra-migrate
docker compose up -d
```

**Risk:** Very low - Hydra is isolated and not yet integrated with other services

---

## ✅ Summary

**What was broken:**
- Hydra migrations failing due to database conflict with Kratos

**What was fixed:**
- Changed Hydra DSN from `engine_search` to `hydra_db`
- Applied all 195 Hydra migrations successfully
- Hydra fully operational with OIDC discovery endpoint working

**Current state:**
- ✅ Hydra ready for OIDC integration
- ✅ Email service still using default password (needs auth fix)
- ✅ All infrastructure components healthy

**Next step:**
- User decides: Quick fix (Master User) or proper solution (OIDC)?

---

**Status:** ✅ **HYDRA FIX COMPLETE - READY FOR OIDC IMPLEMENTATION**
