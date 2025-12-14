# Phase 6 & 7 Completion Summary

## Phase 6: Hydra SSO for Stalwart Integration - ✅ COMPLETE

### Overview
Implemented OAuth 2.0 Single Sign-On (SSO) using Ory Hydra to enable unified authentication for Stalwart email and calendar applications.

### Completed Tasks

#### 6.1 Hydra Configuration ✅
- **File**: `docker-compose.yml`
- **Changes**:
  - Added `URLS_LOGIN=http://127.0.0.1:3000/api/hydra/login`
  - Added `URLS_CONSENT=http://127.0.0.1:3000/api/hydra/consent`
  - Added `OAUTH2_EXPOSE_INTERNAL_ERRORS=true` (development only)
- **Status**: Hydra restarted and running with new configuration

#### 6.2 OAuth Client Registration ✅
- **Clients Registered**:
  1. **stalwart-email**
     - Client ID: `stalwart-email`
     - Redirect URIs: `http://localhost:6000/auth/callback`, `http://localhost:6000/oauth/callback`
     - Scopes: `openid profile email offline_access`
     - Grant Types: `authorization_code`, `refresh_token`

  2. **stalwart-calendar**
     - Client ID: `stalwart-calendar`
     - Redirect URIs: `http://localhost:7000/auth/callback`, `http://localhost:7000/oauth/callback`
     - Scopes: `openid profile email offline_access`
     - Grant Types: `authorization_code`, `refresh_token`

#### 6.3 Backend Hydra Handlers ✅
- **File**: `src/api/mod.rs` (lines 1311-1617)
- **Handlers Implemented**:
  1. `handle_hydra_login` - Bridges Hydra login to Kratos authentication
  2. `accept_hydra_login` - Accepts login challenge with Kratos session data
  3. `handle_hydra_consent` - Manages consent flow (auto-accept or show UI)
  4. `accept_consent` - API endpoint to accept consent from UI
  5. `reject_consent` - API endpoint to reject consent from UI
  6. `accept_hydra_consent_internal` - Internal function to call Hydra admin API

- **Routes**:
  - `GET /api/hydra/login` - Hydra login provider endpoint
  - `GET /api/hydra/consent` - Hydra consent provider endpoint
  - `POST /api/hydra/consent/accept` - Accept consent
  - `POST /api/hydra/consent/reject` - Reject consent

#### 6.4 Consent UI ✅
- **File**: `frontend-search/src/routes/auth/consent/+page.svelte`
- **Features**:
  - Displays requesting application name
  - Shows requested OAuth scopes with human-readable descriptions
  - Provides Allow/Deny buttons
  - Handles auto-consent for repeat authorizations
  - Dark mode support
  - Loading states and error handling

## Phase 7: Cleanup - ✅ COMPLETE

### 7.1 Email Verification ✅
- **Status**: Already enabled in Kratos configuration
- **File**: `ory/kratos/kratos.yml` (lines 65-70)
- **Method**: Code-based verification
- **Flow**: After registration → Email with verification code → User verifies → Account activated

### 7.2 Custom Auth Module ❌ NOT REMOVED
- **Reason**: Still required for admin authentication and invitation system
- **Current Usage**:
  - Admin middleware (`auth::middleware::require_admin`)
  - User repository for admin operations
  - Password hashing for invitations
  - Admin dashboard authentication
- **Future**: Will be removed when admin users are migrated to Kratos

### 7.3 Documentation ✅
- Created `PHASE6_7_COMPLETE.md` (this file)
- Documented SSO flow and OAuth client configuration
- Provided testing instructions

## SSO Flow Documentation

### Authorization Code Flow

```
┌─────────────────┐
│ Email/Calendar  │
│   Application   │
└────────┬────────┘
         │ 1. Initiate OAuth
         ▼
┌─────────────────┐
│  Ory Hydra      │
│  (OAuth Server) │
└────────┬────────┘
         │ 2. Redirect to login?
         ▼
┌─────────────────┐
│ Backend API     │
│ /hydra/login    │
└────────┬────────┘
         │ 3. Check Kratos session
         ▼
    ┌────┴────┐
    │ Authed? │
    └────┬────┘
     Yes │ No
    ┌────▼────┐        ┌─────────────┐
    │ Accept  │        │ Redirect to │
    │ Login   │        │ Kratos Login│
    └────┬────┘        └──────┬──────┘
         │                    │ 4. User logs in
         │                    │
         │ 5. Redirect to consent
         ▼
┌─────────────────┐
│ Backend API     │
│ /hydra/consent  │
└────────┬────────┘
         │ 6. Check if already consented
         ▼
    ┌────┴────┐
    │ Skip?   │
    └────┬────┘
     Yes │ No
    ┌────▼────┐        ┌─────────────┐
    │ Auto    │        │ Show        │
    │ Accept  │        │ Consent UI  │
    └────┬────┘        └──────┬──────┘
         │                    │ 7. User grants consent
         │                    │
         │ 8. Return authorization code
         ▼
┌─────────────────┐
│ Email/Calendar  │
│   Application   │
└────────┬────────┘
         │ 9. Exchange code for tokens
         ▼
┌─────────────────┐
│  Ory Hydra      │
│  (Token)        │
└────────┬────────┘
         │ 10. Return access + ID + refresh tokens
         ▼
┌─────────────────┐
│ Email/Calendar  │
│   Application   │
│   (Authenticated)│
└─────────────────┘
```

## Testing Instructions

### Test OAuth Flow

1. **Start Email Application** (example):
```bash
# Navigate to email app OAuth initiation
open "http://127.0.0.1:4444/oauth2/auth?client_id=stalwart-email&response_type=code&scope=openid%20profile%20email&redirect_uri=http://localhost:6000/auth/callback&state=random-state"
```

2. **Expected Flow**:
   - Redirected to Kratos login (if not authenticated)
   - After login → Redirected to consent UI
   - User sees: "2arak Email (Stalwart) wants to access your account"
   - User clicks "Allow"
   - Redirected back to email app with authorization code
   - Email app exchanges code for tokens

3. **Verify Token Exchange**:
```bash
CODE="authorization-code-from-callback"

curl -X POST http://127.0.0.1:4444/oauth2/token \
  -d "grant_type=authorization_code" \
  -d "code=$CODE" \
  -d "redirect_uri=http://localhost:6000/auth/callback" \
  -d "client_id=stalwart-email" \
  -d "client_secret=stalwart-email-secret-change-in-production" \
  | jq .
```

4. **Expected Response**:
```json
{
  "access_token": "ory_at_...",
  "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "ory_rt_...",
  "token_type": "bearer",
  "expires_in": 3600
}
```

### Verify ID Token Claims

```bash
# Decode ID token (use jwt.io or this command)
ID_TOKEN="id-token-from-response"
echo $ID_TOKEN | cut -d'.' -f2 | base64 -d | jq .
```

**Expected Claims**:
```json
{
  "sub": "user-id-from-kratos",
  "email": "user@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "iss": "http://127.0.0.1:4444/",
  "aud": ["stalwart-email"],
  "exp": 1702843200,
  "iat": 1702839600
}
```

## Configuration Files

### Hydra Environment Variables
```yaml
# docker-compose.yml
environment:
  - DSN=postgres://postgres:postgres@postgres:5432/engine_search?sslmode=disable
  - URLS_SELF_ISSUER=http://127.0.0.1:4444/
  - URLS_LOGIN=http://127.0.0.1:3000/api/hydra/login
  - URLS_CONSENT=http://127.0.0.1:3000/api/hydra/consent
  - SECRETS_SYSTEM=CHANGE-THIS-IN-PRODUCTION-MINIMUM-32-CHARACTERS-LONG
  - OIDC_SUBJECT_IDENTIFIERS_SUPPORTED_TYPES=public
  - OAUTH2_EXPOSE_INTERNAL_ERRORS=true
```

### OAuth Client Configuration
Stored in Hydra database (`engine_search` schema). View with:
```bash
curl -s http://127.0.0.1:4445/admin/clients | jq '.[] | {client_id, client_name, redirect_uris}'
```

## Security Considerations

### Production Deployment
1. **Change Secrets**:
   - `SECRETS_SYSTEM` in Hydra configuration
   - OAuth client secrets
   - Kratos cookie secrets

2. **Use HTTPS**:
   - All URLs must use HTTPS in production
   - Update Hydra issuer URL
   - Update redirect URIs

3. **Token Lifetimes**:
   - Access tokens: 1 hour (default)
   - Refresh tokens: configurable (currently: remember for 7 days)
   - ID tokens: 1 hour (default)

4. **Disable Debug Mode**:
   - Remove `OAUTH2_EXPOSE_INTERNAL_ERRORS=true`
   - Change Kratos log level from `debug` to `info`

## Next Steps

### Future Enhancements
1. **Stalwart Configuration**:
   - Configure Stalwart Mail Server to use Hydra as OIDC provider
   - Set up OAUTHBEARER SASL mechanism
   - Configure CalDAV/CardDAV with OAuth

2. **Admin Migration**:
   - Migrate admin users to Kratos
   - Remove custom auth module entirely
   - Update admin dashboard authentication

3. **Enhanced Security**:
   - Implement 2FA/MFA via Kratos
   - Add passkey support
   - Implement device trust

4. **User Management**:
   - Add admin UI for OAuth client management
   - Implement user consent revocation
   - Add audit logs for OAuth grants

## Success Criteria ✅

- [x] Hydra configured with login/consent endpoints
- [x] OAuth clients registered (stalwart-email, stalwart-calendar)
- [x] Login handler bridges Hydra to Kratos authentication
- [x] Consent handler manages user approval flow
- [x] Consent UI displays scopes and handles user input
- [x] Email verification enabled in Kratos
- [x] Documentation updated

## Conclusion

Phase 6 & 7 are complete! The system now supports OAuth 2.0 SSO for external applications (Stalwart email/calendar) while maintaining Kratos-based authentication for search users. Admin authentication remains on the custom auth module pending future migration.

The foundation is in place for a production-ready SSO infrastructure that can be extended to support additional OAuth clients and enhanced security features.
