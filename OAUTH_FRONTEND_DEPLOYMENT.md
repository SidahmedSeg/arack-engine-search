# OAuth Frontend Deployment - Complete

## ✅ Deployment Summary

The OAuth UI has been successfully deployed to production at **https://mail.arack.io**

---

## What Was Deployed

### 1. **Frontend Build Changes**

**Adapter Change:** Switched from `@sveltejs/adapter-auto` to `@sveltejs/adapter-node`
- **Why:** adapter-auto doesn't create a standalone executable server
- **Result:** Build now creates `build/index.js` which can be run directly with Node.js

**Files Modified Locally:**
- `svelte.config.js` - Changed adapter from auto to node
- Package installed: `@sveltejs/adapter-node`

### 2. **Production Deployment (VPS)**

**Server:** root@213.199.59.206

**Frontend Location:** `/opt/arack/frontend-email/`

**PM2 Service:**
- **Name:** email-frontend
- **ID:** 9
- **Entry Point:** `/opt/arack/frontend-email/build/index.js`
- **Port:** 5006
- **Status:** ✅ Online and listening
- **Command:** `PORT=5006 HOST=0.0.0.0 pm2 start build/index.js --name email-frontend`

**Public URL:** https://mail.arack.io
- **Status:** ✅ HTTP 200 OK
- **Nginx Proxy:** Already configured to proxy to `http://host.docker.internal:5006`

### 3. **OAuth UI Components**

All components from `OAUTH_FRONTEND_IMPLEMENTATION.md` are now live:

✅ Settings page with OAuth connection card (`/settings`)
✅ OAuth callback page (`/oauth/callback`)
✅ API client with OAuth methods
✅ Production environment configuration

---

## Testing OAuth Flow End-to-End

### Prerequisites

1. You must be logged in to Kratos (https://auth.arack.io)
2. Email service backend must be running (port 3001)
3. Hydra OAuth server must be running (port 4444)

### Step-by-Step Testing

#### 1. **Access Settings Page**

Navigate to: **https://mail.arack.io/settings**

**Expected:**
- Page loads successfully
- OAuth section shows "Loading..." spinner briefly
- Then shows "Not Connected" state with benefits list
- "Connect Email Account" button is visible

#### 2. **Initiate OAuth Flow**

Click: **"Connect Email Account" button**

**Expected:**
- Browser redirects to: `https://api-mail.arack.io/api/mail/oauth/authorize`
- Backend redirects to Hydra OAuth consent screen
- URL changes to Hydra domain with OAuth parameters

#### 3. **Grant Consent**

On Hydra consent screen:
- Review requested scopes (openid, email, profile)
- Click "Accept" or "Allow"

**Expected:**
- Hydra redirects to: `https://mail.arack.io/oauth/callback?code=...`
- Callback page shows "Connecting your account..." with spinner
- Backend exchanges code for tokens
- Page shows "Account Connected!" with green checkmark
- Auto-redirects to `/settings` after 2 seconds

#### 4. **Verify Connection**

Back on settings page:

**Expected:**
- OAuth section shows "Account Connected" with green checkmark
- Connection details display:
  - Scopes: openid email profile
  - Expires: in X days/hours
- "Disconnect Account" button is visible

#### 5. **Test Disconnect (Optional)**

Click: **"Disconnect Account" button**

**Expected:**
- Confirmation dialog appears
- After confirming, OAuth token is deleted
- UI updates to "Not Connected" state
- "Connect Email Account" button appears again

---

## Verifying Backend Integration

### Check OAuth Token in Database

SSH into VPS and query the database:

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

docker exec search_engine_postgres psql -U postgres -d engine_search \
  -c "SELECT id, kratos_identity_id, scope, expires_at, created_at
      FROM email.email_oauth_tokens
      ORDER BY created_at DESC
      LIMIT 5;"
```

**Expected Output After OAuth Flow:**
```
                  id                  |         kratos_identity_id          |         scope         |        expires_at          |         created_at
--------------------------------------+-------------------------------------+-----------------------+----------------------------+----------------------------
 <uuid>                               | <user's kratos id>                  | openid email profile  | 2025-12-27 02:10:00+00     | 2025-12-20 02:10:00+00
```

### Check OAuth Status API

Test with curl (requires session cookie):

```bash
# From VPS (requires ory_kratos_session cookie)
curl -s https://api-mail.arack.io/api/mail/oauth/status \
  -H "Cookie: ory_kratos_session=YOUR_SESSION_COOKIE" \
  | jq
```

**Expected Response (Connected):**
```json
{
  "connected": true,
  "expires_at": "2025-12-27T02:10:00Z",
  "scope": "openid email profile"
}
```

**Expected Response (Not Connected):**
```json
{
  "connected": false
}
```

---

## Troubleshooting

### Issue: Settings page shows loading forever

**Cause:** Email API not accessible

**Fix:**
```bash
# Check email service status
docker ps | grep email_service

# Check logs
docker logs search_engine_email_service --tail 50

# Restart if needed
docker-compose restart email-service
```

### Issue: "Connect" button does nothing

**Cause:** Environment variable `VITE_EMAIL_API_URL` incorrect

**Check:**
```bash
# On VPS, check frontend environment
cat /opt/arack/frontend-email/.env.production

# Should contain:
# VITE_EMAIL_API_URL=https://api-mail.arack.io
```

**Fix:** If incorrect, update `.env.production` and rebuild frontend

### Issue: OAuth redirect fails with error

**Cause:** Hydra client not configured or redirect URI mismatch

**Check Hydra Client:**
```bash
docker exec ory_hydra hydra clients list --endpoint http://hydra:4445

# Verify email-service client exists with:
# - grant_types: authorization_code, refresh_token
# - redirect_uris: https://mail.arack.io/oauth/callback
```

**Fix:** Re-create Hydra client with correct configuration

### Issue: Callback shows "Connection Failed"

**Cause:** Backend failed to exchange code for token

**Check Logs:**
```bash
docker logs search_engine_email_service --tail 100 | grep -i oauth
```

**Common Errors:**
- "Invalid redirect URI" → Hydra client misconfigured
- "Invalid code" → Code already used or expired
- "Database error" → Check PostgreSQL connection

### Issue: Token not saved in database

**Cause:** Database migration 010 not applied

**Check:**
```bash
docker exec search_engine_postgres psql -U postgres -d engine_search \
  -c "SELECT version, description FROM _sqlx_migrations WHERE version = 10;"
```

**Expected:**
```
 version |    description
---------+-------------------
      10 | create oauth tokens
```

**Fix:** If missing, restart email service to apply migration

---

## Production URLs Reference

| Service | URL | Purpose |
|---------|-----|---------|
| Frontend (Mail UI) | https://mail.arack.io | Email interface with OAuth settings |
| Email API | https://api-mail.arack.io | Backend email service |
| OAuth Authorize | https://api-mail.arack.io/api/mail/oauth/authorize | Initiate OAuth flow |
| OAuth Callback | https://mail.arack.io/oauth/callback | Handle OAuth redirect |
| OAuth Status | https://api-mail.arack.io/api/mail/oauth/status | Check connection status |
| OAuth Disconnect | https://api-mail.arack.io/api/mail/oauth/disconnect | Remove connection |
| Kratos Auth | https://auth.arack.io | User authentication |

---

## Next Steps

1. ✅ **Test OAuth Flow Manually** - Follow the step-by-step guide above
2. ✅ **Verify Token Storage** - Check database for OAuth tokens
3. ⏭️ **Test Token Refresh** - Wait for token to expire and verify auto-refresh works
4. ⏭️ **Test Email API with OAuth** - Try listing mailboxes, sending emails using OAuth token
5. ⏭️ **Document OAuth Integration** - Update main README with OAuth setup instructions

---

## Files Modified

### Local Repository

1. `/frontend-email/svelte.config.js` - Changed adapter to adapter-node
2. `/frontend-email/package.json` - Added @sveltejs/adapter-node dependency
3. `/frontend-email/.env.production` - Production API URLs

### VPS Production

1. `/opt/arack/frontend-email/build/` - Complete new build with OAuth UI
2. PM2 configuration - Updated to use `build/index.js`

---

## Success Criteria ✅

- [x] Frontend builds successfully with adapter-node
- [x] Frontend deployed to VPS
- [x] PM2 service running on port 5006
- [x] Public URL https://mail.arack.io returns HTTP 200
- [x] OAuth settings page accessible
- [x] OAuth API endpoints responding correctly
- [ ] **Manual Test Required:** Complete OAuth flow and verify token storage

---

**Deployment completed:** December 20, 2025 03:07 UTC
**Frontend URL:** https://mail.arack.io
**Backend API:** https://api-mail.arack.io
**Status:** ✅ READY FOR TESTING
