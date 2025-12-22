# JMAP Authentication Fix Plan

**Date:** 2025-12-19
**Issue:** Email API endpoints (mailboxes, messages) return "Authentication failed" error
**Root Cause Identified:** Multiple issues with JMAP authentication

---

## 🔍 Root Cause Analysis

### Issue 1: Mixed Account Passwords ❌

**Problem:** Different accounts have different passwords in Stalwart

**Evidence:**
```
ID 13: mus.dali@arack.io       - Password: ChangeMe123!  ✅
ID 12: wassim.yessam@arack.io  - Password: ChangeMe123!  ✅
ID 11: yacine.wanik@arack.io   - Password: ChangeMe123!  ✅
ID 8:  omar.djedi@arack.io     - Password: TestPass123   ❌ DIFFERENT!
```

**Impact:**
- Email service uses `DEFAULT_EMAIL_PASSWORD=ChangeMe123!`
- omar.djedi@arack.io has password `TestPass123` (from earlier testing)
- JMAP auth fails for omar.djedi with wrong password

**Why This Happened:**
- Earlier testing created accounts with different passwords
- Current provisioning uses `ChangeMe123!` consistently
- Old accounts remain with old passwords

---

### Issue 2: 401 Unauthorized Errors in Logs ❌

**Problem:** Email service logs show authentication failures

**Evidence:**
```
ERROR Failed to get JMAP session: 401 Unauthorized
      "You have to authenticate first."
```

**But manual testing works:**
```bash
# From within Docker network - WORKS ✅
curl -u "yacine.wanik:ChangeMe123!" http://stalwart:8080/jmap/session
# Returns: {"username":"yacine.wanik","accounts":{"l":{...}}}
```

**Possible Causes:**
1. JMAP client not sending Basic Auth headers correctly
2. Stalwart rejecting auth for some reason
3. Code path using wrong password
4. Session cookie pointing to wrong user account

---

### Issue 3: Session Cookie Maps to Wrong Account ⚠️

**Problem:** User logs in as one account, but gets different account's data

**Evidence:**
```bash
# Cookie for yacine.wanik@arack.io
curl -H "Cookie: ory_kratos_session=..." https://api-mail.arack.io/api/mail/account/me

# Returns:
{
  "account": {
    "email_address": "omar.djedi@arack.io",  ← WRONG USER!
    "kratos_identity_id": "3794d157-c2c5-4201-accb-5e08b7aacf96",
    ...
  }
}
```

**Impact:**
- User A logs in, sees User B's account
- Privacy/security issue
- JMAP auth fails because omar.djedi has wrong password

---

## 🎯 Fix Plan

### Fix 1: Reset All Account Passwords to Match DEFAULT_EMAIL_PASSWORD

**Priority:** HIGH
**Effort:** 10 minutes
**Risk:** Low (can be rolled back)

**Actions:**

1. **Update all existing accounts to use `ChangeMe123!` password:**

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Reset omar.djedi password (the problem account)
curl -X PATCH -u "admin:adminpassword" \
  http://localhost:8081/api/principal/omar.djedi \
  -H "Content-Type: application/json" \
  -d '{"secrets": ["ChangeMe123!"]}'

# Check all other old accounts and reset if needed
# Get full list
curl -s -u "admin:adminpassword" http://localhost:8081/api/principal | \
  python3 -c "
import sys, json
data = json.load(sys.stdin)
for item in data.get('data', {}).get('items', []):
    if item.get('type') == 'individual':
        password = item.get('secrets', ['N/A'])[0]
        if password != 'ChangeMe123!':
            print(f\"NEEDS RESET: {item['name']} - Current: {password}\")
"

# Reset any that don't match
curl -X PATCH -u "admin:adminpassword" \
  http://localhost:8081/api/principal/ACCOUNT_NAME \
  -H "Content-Type: application/json" \
  -d '{"secrets": ["ChangeMe123!"]}'
```

**Verification:**
```bash
# Test JMAP auth for omar.djedi
curl -s -u "omar.djedi:ChangeMe123!" http://localhost:8081/jmap/session | python3 -m json.tool
# Should return session data
```

---

### Fix 2: Investigate Session Cookie Mapping Issue

**Priority:** HIGH
**Effort:** 30 minutes
**Risk:** Medium (might require code changes)

**Investigation Steps:**

1. **Verify Kratos session returns correct user:**
```bash
# Get the actual session from Kratos
COOKIE="ory_kratos_session=ory_st_biylAlYqeuUdVunSE1s5EsVGEfObSxgG"
curl -H "Cookie: $COOKIE" https://api.arack.io/api/auth/whoami | python3 -m json.tool

# Check what Kratos says the identity is
```

2. **Check if registration webhook created wrong account mapping:**
```sql
-- On VPS, check email_accounts table
docker exec search_engine_postgres psql -U postgres -d engine_search -c "
SELECT
  ea.kratos_identity_id,
  ea.email_address,
  ea.created_at
FROM email.email_accounts ea
ORDER BY ea.created_at DESC
LIMIT 5;
"

-- Cross-reference with Kratos identities
-- If mismatch found, update the record
```

3. **If mapping is wrong, fix database:**
```sql
-- Update incorrect mapping (example)
UPDATE email.email_accounts
SET kratos_identity_id = 'CORRECT_KRATOS_ID'
WHERE email_address = 'omar.djedi@arack.io';
```

---

### Fix 3: Add Debug Logging to JMAP Client

**Priority:** MEDIUM
**Effort:** 15 minutes
**Risk:** Low (logging only)

**Code Changes:**

**File:** `email/jmap/mod.rs`

**Before** (line 52-78):
```rust
pub async fn get_session(&self, auth: &JmapAuth) -> Result<JmapSession> {
    let url = format!("{}/jmap/session", self.base_url);

    debug!("Fetching JMAP session from {}", url);

    let request = self.client.get(&url);
    let response = self
        .apply_auth(request, auth)
        .send()
        .await
        .context("Failed to fetch JMAP session")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("JMAP session request failed: {} - {}", status, error_text);
    }
    // ...
}
```

**After:**
```rust
pub async fn get_session(&self, auth: &JmapAuth) -> Result<JmapSession> {
    let url = format!("{}/jmap/session", self.base_url);

    // Add detailed auth logging
    match auth {
        JmapAuth::Bearer(token) => {
            debug!("Fetching JMAP session from {} with Bearer token (length: {})", url, token.len());
        }
        JmapAuth::Basic { username, password } => {
            debug!("Fetching JMAP session from {} with Basic auth (username: {}, password length: {})",
                   url, username, password.len());
        }
    }

    let request = self.client.get(&url);
    let response = self
        .apply_auth(request, auth)
        .send()
        .await
        .context("Failed to fetch JMAP session")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();

        // Log the failure with auth details
        match auth {
            JmapAuth::Basic { username, .. } => {
                error!("JMAP authentication failed for user '{}': {} - {}", username, status, error_text);
            }
            _ => {
                error!("JMAP authentication failed: {} - {}", status, error_text);
            }
        }

        anyhow::bail!("JMAP session request failed: {} - {}", status, error_text);
    }
    // ...
}
```

**Rebuild and Deploy:**
```bash
# On local machine
cd "/Users/intelifoxdz/RS Projects/Engine_search"
docker build --platform linux/amd64 -f Dockerfile.email -t email-service:latest .

# Upload to VPS
docker save email-service:latest | gzip | ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206 'gunzip | docker load'

# On VPS
docker stop search_engine_email_service
docker rm search_engine_email_service
docker-compose up -d email-service

# Check logs
docker logs -f search_engine_email_service
```

---

### Fix 4: Test with Actual User Login Flow

**Priority:** HIGH
**Effort:** 10 minutes
**Risk:** None (testing only)

**Test Steps:**

1. **Clear browser cookies completely**
2. **Register NEW test user:**
   - Go to https://arack.io/auth/register
   - Email: `test.jmap@arack.io`
   - Password: (any password for Kratos)
3. **Check provisioning created account:**
```bash
# Check Stalwart account created
curl -s -u "admin:adminpassword" http://localhost:8081/api/principal | grep "test.jmap"

# Should show: password = ChangeMe123!
```
4. **Test mailbox API:**
```bash
# Get session cookie from browser DevTools
COOKIE="ory_kratos_session=YOUR_SESSION_HERE"

curl -H "Cookie: $COOKIE" https://api-mail.arack.io/api/mail/mailboxes | python3 -m json.tool
```

**Expected Result:**
```json
{
  "mailboxes": [
    {"id": "...", "name": "Inbox", "role": "inbox", ...},
    {"id": "...", "name": "Sent", "role": "sent", ...},
    {"id": "...", "name": "Drafts", "role": "drafts", ...},
    {"id": "...", "name": "Trash", "role": "trash", ...},
    {"id": "...", "name": "Junk", "role": "junk", ...}
  ],
  "total": 5
}
```

---

## 📊 Summary of Issues & Fixes

| Issue | Status | Fix | Priority | Effort |
|-------|--------|-----|----------|--------|
| Mixed account passwords | ❌ Identified | Reset all to ChangeMe123! | HIGH | 10 min |
| Session maps to wrong user | ⚠️ Suspected | Investigate Kratos/DB mapping | HIGH | 30 min |
| 401 Unauthorized errors | ❌ Confirmed | Reset passwords + add logging | HIGH | 15 min |
| JMAP client logging | ⚠️ Missing | Add debug logs | MEDIUM | 15 min |

---

## 🚀 Execution Order

### Step 1: Quick Win - Reset Passwords (10 minutes)

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Reset omar.djedi (confirmed wrong password)
curl -X PATCH -u "admin:adminpassword" \
  http://localhost:8081/api/principal/omar.djedi \
  -H "Content-Type: application/json" \
  -d '{"secrets": ["ChangeMe123!"]}'

# Verify
curl -s -u "omar.djedi:ChangeMe123!" http://localhost:8081/jmap/session | python3 -m json.tool | grep username
```

### Step 2: Test Mailboxes API (5 minutes)

```bash
# Use existing session cookie
COOKIE="ory_kratos_session=ory_st_biylAlYqeuUdVunSE1s5EsVGEfObSxgG"
curl -H "Cookie: $COOKIE" https://api-mail.arack.io/api/mail/mailboxes | python3 -m json.tool
```

**If still fails** → Continue to Step 3

### Step 3: Investigate Session Mapping (30 minutes)

```bash
# Check Kratos session
curl -H "Cookie: $COOKIE" https://api.arack.io/api/auth/whoami

# Check database mapping
docker exec search_engine_postgres psql -U postgres -d engine_search -c "
SELECT kratos_identity_id, email_address
FROM email.email_accounts
WHERE email_address IN ('yacine.wanik@arack.io', 'omar.djedi@arack.io');
"
```

### Step 4: Add Logging & Rebuild (optional, if issues persist)

- Add debug logging to JMAP client
- Rebuild Docker image
- Deploy and test

---

## ✅ Success Criteria

**Fix is successful when:**

1. All Stalwart accounts have password `ChangeMe123!`
2. `GET /api/mail/mailboxes` returns mailbox list (not auth error)
3. `GET /api/mail/messages` returns empty or populated message list
4. User's session cookie maps to correct email account
5. No 401 Unauthorized errors in email service logs

---

## 🔄 Rollback Plan

**If fixes cause issues:**

1. **Password reset rollback:**
   - Passwords can be changed back individually via Stalwart Admin API
   - No data loss, instant rollback

2. **Code changes rollback:**
   - Revert Docker image to previous version
   - `docker-compose up -d email-service`

3. **Database changes rollback:**
   - If any UPDATE statements run, have SELECT first to save old values
   - Restore with UPDATE back to original values

---

## 📝 Next Steps After Fix

Once JMAP authentication works:

1. Test send email functionality
2. Test email search (currently stub)
3. Implement draft management endpoints
4. Implement trash management endpoints
5. Configure OpenAI API key for AI features

---

**Estimated Total Time:** 1-2 hours (including testing and verification)
**Risk Level:** Low to Medium
**Requires Restart:** Yes (email service only, search service unaffected)
