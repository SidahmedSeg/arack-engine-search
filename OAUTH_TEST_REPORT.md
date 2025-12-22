# OAuth End-to-End Test Report

**Test Date:** December 20, 2025
**Environment:** Production (https://mail.arack.io)
**Status:** ✅ System Ready - Manual Testing Required

---

## Automated Test Results

### ✅ 1. Frontend Accessibility

**Test:** Verify frontend is accessible and loads correctly

**Result:** PASS ✅
- Main page: https://mail.arack.io - HTTP 200 OK
- Settings page: https://mail.arack.io/settings - HTTP 200 OK
- OAuth callback page: https://mail.arack.io/oauth/callback - HTTP 200 OK
- All pages render correctly with proper HTML structure

### ✅ 2. OAuth UI Components

**Test:** Verify OAuth UI is visible on settings page

**Result:** PASS ✅

**Verified Components:**
- ✅ "Email Account Connection" section visible
- ✅ "OAuth Authentication" card displayed
- ✅ "Not Connected" status with Link2Off icon
- ✅ "Connect Email Account" button present
- ✅ Benefits list visible:
  - "Secure authentication without sharing passwords"
  - "Automatic token refresh for seamless access"
  - "Granular permission control"
  - "Easy revocation from settings"

### ✅ 3. Backend API Health

**Test:** Verify email API is running and healthy

**Result:** PASS ✅

**Response:**
```json
{
  "status": "ok",
  "service": "email-service",
  "version": "0.3.0",
  "phase": "3"
}
```

**API URL:** https://api-mail.arack.io

### ✅ 4. OAuth Endpoints

**Test:** Verify OAuth endpoints respond correctly

**Result:** PASS ✅

#### OAuth Status Endpoint
- **URL:** https://api-mail.arack.io/api/mail/oauth/status
- **Without Auth:** Returns `{"error":"No session cookie found"}` (Expected ✅)
- **Status Code:** 200

#### OAuth Authorize Endpoint
- **URL:** https://api-mail.arack.io/api/mail/oauth/authorize
- **Without Auth:** Returns HTTP 401 Unauthorized (Expected ✅)
- **Behavior:** Correctly requires authentication before initiating OAuth flow

#### OAuth Callback Page
- **URL:** https://mail.arack.io/oauth/callback
- **Response:** Shows "Connecting your account..." processing state
- **Status Code:** 200

### ✅ 5. Hydra OAuth Server Configuration

**Test:** Verify Hydra client is properly configured

**Result:** PASS ✅

**Client Configuration:**
```
CLIENT ID:       email-service
GRANT TYPES:     authorization_code, refresh_token
RESPONSE TYPES:  code
SCOPE:           openid email profile offline_access
AUDIENCE:        email-api
REDIRECT URIS:
  - https://mail.arack.io/oauth/callback (Production)
  - http://localhost:5173/oauth/callback (Development)
```

**Hydra Container:** search_engine_hydra (oryd/hydra:v2.2.0)
**Status:** Running (17 hours uptime)
**Ports:** 4444 (public), 4445 (admin)

### ✅ 6. Database Schema

**Test:** Verify OAuth tokens table exists

**Result:** PASS ✅

**Table:** `email.email_oauth_tokens`
**Current Token Count:** 0 (Expected - no OAuth flows completed yet)

**Schema Verified:**
- ✅ Migration 010 applied successfully
- ✅ Table exists with correct structure
- ✅ Foreign key to email_accounts configured

### ✅ 7. Frontend Service

**Test:** Verify frontend service is running

**Result:** PASS ✅

**PM2 Status:**
```
ID:    9
Name:  email-frontend
Mode:  fork
PID:   3633640
Port:  5006
Status: online
Memory: 51.4mb
```

**Entry Point:** `/opt/arack/frontend-email/build/index.js`
**Adapter:** @sveltejs/adapter-node

---

## Manual Testing Required

The following steps **require browser interaction** and cannot be automated. Please complete these tests manually:

### Step 1: Login to Kratos

1. Open browser in incognito/private mode
2. Navigate to: **https://auth.arack.io**
3. Login with your credentials
4. Verify you get a session cookie (`ory_kratos_session`)

### Step 2: Access Settings Page

1. Navigate to: **https://mail.arack.io/settings**
2. **Verify:**
   - Page loads without errors
   - OAuth section shows "Loading..." briefly
   - Then shows "Not Connected" state
   - "Connect Email Account" button is enabled

### Step 3: Initiate OAuth Flow

1. Click: **"Connect Email Account"** button
2. **Expected:**
   - Browser redirects to `https://api-mail.arack.io/api/mail/oauth/authorize`
   - Backend redirects to Hydra OAuth server
   - URL changes to Hydra domain (may be localhost or Hydra container)

3. **If you get an error at this step:**
   - Check browser console for errors
   - Verify you're logged in to Kratos
   - Check email service logs: `docker logs search_engine_email_service --tail 50`

### Step 4: Grant OAuth Consent

1. On Hydra consent screen:
   - Review requested scopes:
     - `openid` - Basic identity information
     - `email` - Email address
     - `profile` - User profile information
   - Click **"Accept"** or **"Allow"**

2. **Expected:**
   - Hydra redirects to `https://mail.arack.io/oauth/callback?code=...`
   - Callback page shows "Connecting your account..." with spinning loader
   - Backend exchanges authorization code for access token
   - Token is stored in database
   - Page shows "Account Connected!" with green checkmark
   - Auto-redirects to `/settings` after 2 seconds

3. **If callback fails:**
   - Check URL for `error` parameter
   - Check callback page for error message
   - Check backend logs for OAuth token exchange errors

### Step 5: Verify Connected State

Back on settings page:

1. **Verify OAuth section shows:**
   - ✅ Green checkmark icon
   - "Account Connected" heading
   - "Your email is securely connected via OAuth" message
   - Connection details box with:
     - Scopes: `openid email profile`
     - Expires: `in X days` (calculated from token expiry)
   - "Disconnect Account" button (red/destructive style)

2. **Take a screenshot** of the connected state for documentation

### Step 6: Verify Token in Database

SSH into VPS and run:

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

docker exec search_engine_postgres psql -U postgres -d engine_search -c "
SELECT
  id,
  kratos_identity_id,
  scope,
  expires_at,
  created_at
FROM email.email_oauth_tokens
ORDER BY created_at DESC
LIMIT 1;
"
```

**Expected Output:**
```
                  id                  |         kratos_identity_id          |         scope         |        expires_at          |         created_at
--------------------------------------+-------------------------------------+-----------------------+----------------------------+----------------------------
 <uuid>                               | <your kratos identity uuid>         | openid email profile  | 2025-12-27 XX:XX:XX+00     | 2025-12-20 XX:XX:XX+00
```

**Verify:**
- Token exists with your Kratos identity ID
- Scopes match what was requested
- Expiry is set (typically 7 days from creation)
- Created timestamp matches when you completed OAuth flow

### Step 7: Test Disconnect (Optional)

1. Click **"Disconnect Account"** button
2. **Expected:**
   - Browser confirmation dialog appears: "Are you sure you want to disconnect your email account?"
   - Click **"OK"** to confirm

3. **After confirmation:**
   - UI immediately updates to "Not Connected" state
   - "Connect Email Account" button reappears
   - Benefits list is visible again

4. **Verify in database:**
```bash
docker exec search_engine_postgres psql -U postgres -d engine_search -c "
SELECT COUNT(*) as token_count FROM email.email_oauth_tokens;
"
```

**Expected:** `token_count = 0` (token should be deleted)

---

## Test Results Summary

| Test | Status | Notes |
|------|--------|-------|
| Frontend loads | ✅ PASS | All pages accessible |
| OAuth UI visible | ✅ PASS | Settings page shows OAuth card |
| Backend API healthy | ✅ PASS | Email service running |
| OAuth endpoints | ✅ PASS | Status, authorize, callback all responding |
| Hydra client configured | ✅ PASS | Client exists with correct settings |
| Database schema | ✅ PASS | OAuth tokens table exists |
| Frontend service | ✅ PASS | PM2 running on port 5006 |
| **Manual: Login** | ⏳ PENDING | Requires browser |
| **Manual: OAuth flow** | ⏳ PENDING | Requires browser + Hydra interaction |
| **Manual: Token verification** | ⏳ PENDING | After OAuth flow completes |
| **Manual: Disconnect** | ⏳ PENDING | Optional test |

---

## Known Issues / Notes

### 1. Centrifugo WebSocket Connection

The settings page shows "Offline" status with "Disconnected from Centrifugo" messages in logs.

**Impact:** Does not affect OAuth functionality - this is for real-time email updates
**Status:** Expected behavior if Centrifugo is not running
**Action:** Can be ignored for OAuth testing

### 2. Account Info Not Loaded

Settings page shows "Email Address: Not loaded" and empty Account ID.

**Cause:** Requires authenticated session and email account provisioned
**Impact:** Does not affect OAuth testing
**Action:** Will populate after user is logged in and has email account

---

## Next Steps After Manual Testing

Once manual testing is complete:

1. **Document Results:**
   - Take screenshots of each step
   - Note any errors encountered
   - Record token expiry time

2. **Test Token Refresh:**
   - Wait for token to expire (or manually set short expiry)
   - Verify automatic refresh works
   - Check logs for refresh attempts

3. **Test Email API with OAuth:**
   - Try listing mailboxes using OAuth token
   - Try sending email using OAuth token
   - Verify API calls succeed with OAuth authentication

4. **Security Testing:**
   - Test with expired token
   - Test with invalid token
   - Test PKCE flow security
   - Test redirect URI validation

5. **Performance Testing:**
   - Test OAuth flow under load
   - Monitor token storage performance
   - Check Hydra response times

---

## Troubleshooting Commands

### Check Email Service Logs
```bash
docker logs search_engine_email_service --tail 100 --follow
```

### Check Hydra Logs
```bash
docker logs search_engine_hydra --tail 100 --follow
```

### Check Frontend Logs
```bash
pm2 logs email-frontend --lines 50
```

### Check Database Connections
```bash
docker exec search_engine_postgres psql -U postgres -d engine_search -c "
SELECT COUNT(*) FROM pg_stat_activity WHERE datname = 'engine_search';
"
```

### Test OAuth Authorize with Session Cookie

If you have a valid `ory_kratos_session` cookie:

```bash
curl -v -L \
  -H "Cookie: ory_kratos_session=YOUR_SESSION_COOKIE_HERE" \
  https://api-mail.arack.io/api/mail/oauth/authorize
```

Should redirect to Hydra consent screen.

---

## Contact / Support

If you encounter issues during manual testing:

1. Check the troubleshooting section above
2. Review logs from email service and Hydra
3. Verify Kratos session is valid
4. Check browser console for JavaScript errors
5. Verify all services are running: `docker ps`

---

**Test Report Complete**
**Automated Tests:** 7/7 PASS ✅
**Manual Tests Required:** 4 tests pending
**System Status:** READY FOR MANUAL TESTING 🚀
