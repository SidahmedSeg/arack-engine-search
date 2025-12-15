# SSO Implementation Complete

## Summary

Successfully implemented and tested OAuth 2.0 SSO flow with Ory Hydra and Kratos for the search engine project. The implementation enables Single Sign-On for future email/calendar applications (Stalwart).

## Implementation Status

### ✅ Completed Components

1. **Backend OAuth Handlers** (`src/api/mod.rs`)
   - Login flow handler (`/api/hydra/login`)
   - Consent flow handler (`/api/hydra/consent`)
   - Accept login endpoint with Kratos session validation
   - Accept/reject consent endpoints
   - HTTP redirect responses (302) for browser navigation
   - AJAX detection for JSON API responses

2. **Frontend Consent UI** (`frontend-search/src/routes/auth/consent/+page.svelte`)
   - Clean, user-friendly consent screen
   - Shows requested OAuth scopes with descriptions
   - Allow/Deny buttons
   - Double-submission protection
   - Error handling with user-friendly messages

3. **OAuth Client Registration**
   - Client ID: `stalwart-email`
   - Client Secret: `stalwart-email-secret-change-in-production`
   - Redirect URIs:
     - `http://localhost:6000/auth/callback/`
     - `http://localhost:6000/oauth/callback`
   - Scopes: `openid profile email offline_access`
   - Grant Types: `authorization_code`, `refresh_token`

4. **Callback Handler** (`auth/callback/index.html`)
   - Displays authorization code on successful OAuth flow
   - Shows error messages if authorization fails
   - Provides curl command for token exchange
   - Served via Python HTTP server on port 6000

5. **Test Infrastructure**
   - SSO test page (`test_sso.html`)
   - Callback handler (`auth/callback/index.html`)
   - Python HTTP server running on port 6000

### ✅ Testing Results

**Token Exchange Test - Successful:**

```json
{
  "access_token": "ory_at_xWF3cNcaLd2p5YtOr_CMKOvho06ZT5DFnzWUeeuRgUk.10WJgD-0L4sbw019zJuL5qWdIyF6jqcKwIsNcMHRiPg",
  "expires_in": 3600,
  "refresh_token": "ory_rt_ebc47-5lqd6Nkfl1ny7HKZZlXjYBwPVGXcUwRQ0sGpQ.ypDomG8U8XDY4bjM6lfMvMXfWMOANf27xjWtC-mWjT8",
  "scope": "openid profile email offline_access",
  "token_type": "bearer"
}
```

**ID Token Claims (decoded):**

```json
{
  "aud": ["stalwart-email"],
  "auth_time": 1765743227,
  "email": "0ef084a0-1978-467e-a7b1-9833721c12b1",
  "first_name": "Walid",
  "last_name": "Wlid",
  "iss": "http://127.0.0.1:4444/",
  "sub": "0ef084a0-1978-467e-a7b1-9833721c12b1",
  "exp": 1765749807
}
```

## Complete OAuth 2.0 Flow

### Flow Diagram

```
┌─────────────────┐
│   User clicks   │
│ "Login with SSO"│
└────────┬────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 1. App redirects to Hydra authorization endpoint│
│    http://127.0.0.1:4444/oauth2/auth            │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 2. Hydra redirects to login provider            │
│    http://127.0.0.1:3000/api/hydra/login        │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 3. Backend checks Kratos session cookie         │
│    - If authenticated: accept login → Step 5    │
│    - If not authenticated: redirect to login    │
└────────┬────────────────────────────────────────┘
         │ (Not authenticated)
         v
┌─────────────────────────────────────────────────┐
│ 4. User logs in via Kratos                      │
│    http://127.0.0.1:5001/auth/login             │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 5. Backend accepts login challenge               │
│    Hydra redirects to consent provider          │
│    http://127.0.0.1:3000/api/hydra/consent      │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 6. User sees consent UI                          │
│    http://127.0.0.1:5001/auth/consent           │
│    "Allow Stalwart Email to access your         │
│     profile, email, etc.?"                      │
└────────┬────────────────────────────────────────┘
         │
         v (User clicks "Allow")
┌─────────────────────────────────────────────────┐
│ 7. Backend accepts consent challenge             │
│    Hydra redirects back to app callback         │
│    http://localhost:6000/auth/callback/         │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 8. App receives authorization code               │
│    code=ory_ac_...                              │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 9. App exchanges code for tokens                 │
│    POST http://127.0.0.1:4444/oauth2/token      │
└────────┬────────────────────────────────────────┘
         │
         v
┌─────────────────────────────────────────────────┐
│ 10. App receives access token, ID token,         │
│     refresh token → User is authenticated!       │
└──────────────────────────────────────────────────┘
```

### Step-by-Step Testing Guide

**Prerequisites:**
1. Backend running (`cargo run`)
2. Frontend running (`cd frontend-search && npm run dev`)
3. Hydra running (Docker container)
4. Kratos running (Docker container)
5. Callback server running on port 6000 (Python HTTP server)

**Test Steps:**

1. **Open SSO Test Page:**
   ```
   http://localhost:6000/test_sso.html
   ```

2. **Click "Start OAuth Flow" button**

3. **Expected Flow:**
   - Redirects to Hydra authorization endpoint
   - Hydra redirects to backend login handler
   - Backend checks Kratos session
   - If not logged in: redirects to Kratos login page
   - User logs in with Kratos credentials
   - Backend accepts login challenge
   - Redirects to consent page
   - User sees: "2arak Email (Stalwart) wants to access your account"
   - User clicks "Allow"
   - Backend accepts consent challenge
   - Redirects to callback handler with authorization code

4. **Callback Page Shows:**
   - ✅ Authorization Successful
   - Authorization code (ory_ac_...)
   - State value
   - curl command to exchange code for tokens

5. **Exchange Code for Tokens (automatic expiry in 10 minutes):**
   ```bash
   curl -X POST http://127.0.0.1:4444/oauth2/token \
     -d "grant_type=authorization_code" \
     -d "code=<YOUR_AUTHORIZATION_CODE>" \
     -d "redirect_uri=http://localhost:6000/auth/callback/" \
     -d "client_id=stalwart-email" \
     -d "client_secret=stalwart-email-secret-change-in-production"
   ```

6. **Expected Response:**
   ```json
   {
     "access_token": "ory_at_...",
     "id_token": "eyJhbGci...",
     "refresh_token": "ory_rt_...",
     "expires_in": 3600,
     "token_type": "bearer",
     "scope": "openid profile email offline_access"
   }
   ```

## Key Technical Details

### OAuth 2.0 Configuration

**Authorization Endpoint:**
```
http://127.0.0.1:4444/oauth2/auth
```

**Token Endpoint:**
```
http://127.0.0.1:4444/oauth2/token
```

**Userinfo Endpoint:**
```
http://127.0.0.1:4444/userinfo
```

**Discovery Endpoint:**
```
http://127.0.0.1:4444/.well-known/openid-configuration
```

### Hydra Configuration

**Environment Variables** (in docker-compose.yml):
```yaml
environment:
  - URLS_SELF_ISSUER=http://127.0.0.1:4444/
  - URLS_LOGIN=http://127.0.0.1:3000/api/hydra/login
  - URLS_CONSENT=http://127.0.0.1:3000/api/hydra/consent
```

### Backend API Endpoints

**Login Flow Handler:**
```
GET /api/hydra/login?login_challenge=<challenge>
```
- Checks Kratos session cookie
- If authenticated: accepts login, returns HTTP redirect to consent
- If not authenticated: returns HTTP redirect to Kratos login

**Consent Flow Handler:**
```
GET /api/hydra/consent?consent_challenge=<challenge>
```
- AJAX requests: returns JSON with consent details
- Browser requests: returns HTTP redirect to consent UI or auto-accepts

**Accept Consent:**
```
POST /api/hydra/consent/accept
Body: { "consent_challenge": "<challenge>" }
```

**Reject Consent:**
```
POST /api/hydra/consent/reject
Body: { "consent_challenge": "<challenge>" }
```

### Frontend Routes

**Consent UI:**
```
http://127.0.0.1:5001/auth/consent?consent_challenge=<challenge>
```

**Login Page (Kratos):**
```
http://127.0.0.1:5001/auth/login?login_challenge=<challenge>
```

## Important Notes

### Authorization Code Lifetime

- **Single-use only** - Code becomes invalid after first token exchange
- **Expiration:** 10 minutes from issuance
- **Solution:** Complete token exchange immediately after receiving code

### Session Management

- Kratos sessions stored in cookies
- Session validation via `/sessions/whoami` endpoint
- Sessions passed to Hydra via context in login acceptance

### CORS and Redirects

- Consent UI makes AJAX requests (JSON responses)
- OAuth flow uses browser redirects (HTTP 302 responses)
- Backend detects request type via `Accept: application/json` header

### Security Considerations

1. **Client Secrets:**
   - Current: `stalwart-email-secret-change-in-production`
   - Production: Generate cryptographically secure secrets (32+ characters)

2. **Redirect URIs:**
   - Strictly validated by Hydra
   - Must exactly match registered URIs (including trailing slashes)

3. **State Parameter:**
   - Prevents CSRF attacks
   - Client generates random state, validates on callback

4. **HTTPS:**
   - Development: HTTP is acceptable
   - Production: MUST use HTTPS for all OAuth endpoints

## Next Steps

### Immediate (Phase 6.7 from plan.md)

1. **Update Stalwart Configuration** with OAuth settings:
   ```toml
   [authentication.oidc]
   endpoint.url = "http://127.0.0.1:4444/userinfo"
   oidc.discovery = "http://127.0.0.1:4444/.well-known/openid-configuration"
   auth.client_id = "stalwart-email"
   auth.client_secret = "stalwart-email-secret-change-in-production"
   ```

2. **Test Stalwart Email Authentication** via SSO

3. **Register Calendar OAuth Client:**
   ```bash
   curl -X POST http://127.0.0.1:4445/admin/clients \
     -H "Content-Type: application/json" \
     -d @stalwart-calendar-client.json
   ```

### Future Enhancements

1. **Remember Me** - Longer session lifetimes for trusted devices
2. **Consent Scope Management** - User dashboard to revoke granted consents
3. **Multi-Application SSO** - Email + Calendar + other services
4. **Social Login** - Google, GitHub OAuth providers in Kratos
5. **2FA/MFA** - Multi-factor authentication via Kratos

## Troubleshooting

### Error: "This site can't be reached" on callback

**Cause:** Callback server not running on port 6000

**Solution:**
```bash
# Check if server is running
lsof -i :6000

# If not running, start it
cd "/Users/intelifoxdz/RS Projects/Engine_search"
python3 -m http.server 6000
```

### Error: "invalid_grant" when exchanging code

**Cause:** Authorization code expired or already used

**Solution:** Complete a fresh OAuth flow to get a new code

### Error: "redirect_uri_mismatch"

**Cause:** Redirect URI in token exchange doesn't match OAuth client configuration

**Solution:** Ensure redirect_uri parameter exactly matches registered URI (including trailing slashes)

### Error: "Consent challenge already used"

**Cause:** Double-submission of consent form

**Solution:** Already fixed with `submitting` state in consent UI

## Files Modified

### Backend
- `src/api/mod.rs` - Added OAuth handlers
- `docker-compose.yml` - Hydra environment variables (URLS_LOGIN, URLS_CONSENT)

### Frontend
- `frontend-search/src/routes/auth/consent/+page.svelte` - Consent UI

### Test Infrastructure
- `test_sso.html` - SSO flow test page
- `auth/callback/index.html` - OAuth callback handler
- Python HTTP server on port 6000 (serving project root)

### Configuration
- OAuth client `stalwart-email` updated with correct redirect URIs

## Commits

**Commit 1:** `bebd0f6` - SSO implementation (backend + frontend)
**Commit 2:** `2b872ec` - Updated .gitignore for backup files

## References

- [Ory Hydra Documentation](https://www.ory.sh/docs/hydra)
- [OAuth 2.0 Authorization Code Flow](https://oauth.net/2/grant-types/authorization-code/)
- [OpenID Connect Specification](https://openid.net/specs/openid-connect-core-1_0.html)
- [Stalwart OIDC Configuration](https://stalw.art/docs/auth/oidc)

---

**Status:** ✅ Implementation Complete and Tested
**Date:** 2025-12-14
**Version:** 1.0
