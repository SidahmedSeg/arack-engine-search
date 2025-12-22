# Cookie Sharing Issue - Diagnosis & Fix Plan

## Problem Summary

**User Report:**
- `arack.io` has cookie: `ory_st_4hHSBoxmgrL23PTh31mIJ4ORQ5CEsleW`
- `mail.arack.io` has **NO cookies** (empty)
- API calls from `mail.arack.io` return: `{"error":"No session cookie found"}`

**Expected Behavior:**
- User logs in on `arack.io` → Session cookie set with `Domain=arack.io`
- User navigates to `mail.arack.io` → Browser automatically sends session cookie
- API calls work with authenticated session

---

## Root Cause Analysis

### Issue #1: Wrong Cookie Type

**The `ory_st_` cookie is NOT a session cookie!**

- `ory_st_*` = Kratos **state token** (temporary, used during authentication flows)
- `ory_kratos_session` = Kratos **session cookie** (persistent, used for authenticated requests)

**Implication:**
- The user may not have completed login yet (only initialized the flow)
- OR the session cookie exists but has a different name/domain

### Issue #2: Cookie Domain Configuration

**Research Finding:**

From [How to share the cookies between subdomains | ABP.IO](https://abp.io/community/articles/how-to-share-the-cookies-between-subdomains-jfrzggc2):

> **"Cookies explicitly set with a domain appear in devtools with a leading period (e.g., .example.com), while cookies set without a domain value appear without the leading period (e.g., example.com)."**

**Critical Distinction:**
- `Domain=arack.io` (in config) → Browser shows as `.arack.io` IF it shares to subdomains
- `Domain=arack.io` (in config) → Browser shows as `arack.io` (no dot) IF it's domain-specific only

**Current Status:**
- Our Kratos config has: `cookies.domain: arack.io`
- Need to verify: What does the browser ACTUALLY show in DevTools?

### Issue #3: Frontend Configuration

**Frontend-Email (.env on VPS):**
```env
VITE_EMAIL_API_URL=https://api-mail.arack.io  ✅ Correct
VITE_AUTH_URL=https://auth.arack.io           ✅ Correct
```

**API Client Configuration:**
```typescript
withCredentials: true  ✅ Correct (line 147 in client.ts)
```

**Nginx Configuration:**
```nginx
location / {
    proxy_pass http://host.docker.internal:5006;  ✅ Correct
}
```

### Issue #4: Potential Domain Mismatch

**Hypothesis:**

Browsers may interpret `Domain=arack.io` differently:
1. **RFC 6265 (modern standard):** `Domain=arack.io` shares with subdomains
2. **Older browsers:** May require `Domain=.arack.io` (with leading dot)
3. **Kratos behavior:** May set cookie without domain attribute if validation fails

**Evidence from Logs:**

```
cookies: map[domain:arack.io path:/ same_site:Lax secure:true]
additionalProperties "secure" not allowed
```

The `secure: true` is causing validation errors, which might prevent proper cookie setting!

---

## Diagnostic Steps

### Step 1: Verify Actual Cookie Domain in Browser

**Test in Chrome/Firefox DevTools:**

1. Clear all cookies for `arack.io`
2. Go to `https://arack.io/auth/login`
3. Complete login with existing user credentials
4. Open DevTools → Application → Cookies → `https://arack.io`
5. Check ALL cookies and note:
   - Cookie Name
   - Cookie Value (first 20 chars)
   - **Domain** (with or without leading dot?)
   - Path
   - Expires/Max-Age
   - HttpOnly
   - Secure
   - SameSite

**Expected Cookies:**
```
Name: ory_kratos_session
Domain: .arack.io  ← CRITICAL: Should have leading dot for subdomain sharing
Path: /
HttpOnly: ✅
Secure: ✅
SameSite: Lax
```

**If Domain shows `arack.io` (no dot):**
- Cookie is NOT shared with subdomains
- This is the root cause

**If Domain shows `.arack.io` (with dot):**
- Cookie SHOULD be shared with subdomains
- Issue is elsewhere

### Step 2: Check Cookies on mail.arack.io

After login on `arack.io`:

1. Navigate to `https://mail.arack.io` (in same browser tab or new tab)
2. Open DevTools → Application → Cookies → `https://mail.arack.io`
3. Check if `ory_kratos_session` cookie appears

**Expected:**
- Same `ory_kratos_session` cookie visible
- Domain: `.arack.io`

**If no cookies:**
- Confirms domain-specific cookie (not shared)

### Step 3: Test API Call with Cookie

From `mail.arack.io` console:

```javascript
fetch('https://api-mail.arack.io/api/mail/account/me', {
  credentials: 'include',
  headers: { 'Content-Type': 'application/json' }
})
.then(r => r.json())
.then(console.log)
.catch(console.error)
```

**Expected Success:**
```json
{
  "account": {
    "id": "...",
    "email_address": "user@arack.io",
    ...
  },
  "quota_percentage": 5.2
}
```

**Expected Error (if cookie not shared):**
```json
{
  "error": "No session cookie found"
}
```

### Step 4: Check Network Tab for Cookie Headers

1. Open DevTools → Network tab
2. From `mail.arack.io`, trigger an API call
3. Click on the request → Headers tab
4. Check **Request Headers**:
   - Look for `Cookie:` header
   - Should include `ory_kratos_session=...`

**If Cookie header is missing:**
- Browser is not sending the cookie
- Confirms domain mismatch

### Step 5: Verify Kratos Configuration

On VPS, check if Kratos is actually using the updated config:

```bash
docker exec search_engine_kratos cat /etc/config/kratos/kratos.yml | grep -A 5 "cookies:"
```

**Expected:**
```yaml
cookies:
  domain: arack.io
  path: /
  same_site: Lax
```

**Check for `secure: true`:**
- Should NOT be present (causes validation error)
- If present, Kratos may ignore cookie domain setting

### Step 6: Check Kratos Logs for Cookie Setting

```bash
docker logs search_engine_kratos --tail 100 | grep -i "set-cookie\|cookie\|domain"
```

**Look for:**
- Validation errors about `secure` flag
- Cookie domain being set correctly
- Any warnings about cookie configuration

---

## Root Causes Identified

Based on the diagnostics, here are the most likely root causes:

### Cause A: Leading Dot Missing (Domain Specificity)

**Issue:**
- Kratos sets cookie with `Domain=arack.io` (no leading dot)
- Browser interprets this as domain-specific, not subdomain-sharing
- Cookie only accessible on `arack.io`, not `mail.arack.io`

**Fix:**
- Change Kratos config to `domain: .arack.io` (with leading dot)
- Restart Kratos
- Test cookie sharing

**Likelihood:** 🔴 **HIGH** (70%)

### Cause B: Validation Errors Preventing Cookie Setting

**Issue:**
- Kratos config has validation errors (e.g., `secure: true` not allowed)
- Kratos ignores invalid config and falls back to defaults
- Default domain might be specific to request origin

**Fix:**
- Remove ALL invalid config options
- Ensure config passes validation
- Restart Kratos and test

**Likelihood:** 🟡 **MEDIUM** (20%)

### Cause C: Session Cookie Not Created Yet

**Issue:**
- User only initialized login flow (has `ory_st_` state token)
- User did NOT complete login (no `ory_kratos_session` created)
- Session cookie only created AFTER successful authentication

**Fix:**
- Complete full login flow
- Verify `ory_kratos_session` cookie exists
- Then test subdomain sharing

**Likelihood:** 🟢 **LOW** (10%)

---

## Recommended Fix Plan

### Fix Option 1: Add Leading Dot to Cookie Domain (RECOMMENDED)

**Why:**
- Explicit leading dot `.arack.io` is the standard for subdomain sharing
- Widely supported across all browsers
- Removes ambiguity in browser cookie handling

**Steps:**

1. **Update Kratos Configuration:**

```yaml
cookies:
  domain: .arack.io  # Add leading dot
  path: /
  same_site: Lax
  # Remove secure: true (not supported in this version)

session:
  cookie:
    domain: .arack.io  # Add leading dot
    path: /
    same_site: Lax
    persistent: false
    # Remove secure: true (not supported in this version)
```

2. **Deploy to VPS:**

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Backup current config
cp /opt/arack/ory/kratos/kratos.yml /opt/arack/ory/kratos/kratos.yml.backup.$(date +%Y%m%d_%H%M%S)

# Edit configuration
nano /opt/arack/ory/kratos/kratos.yml
# Change: domain: arack.io → domain: .arack.io (both occurrences)

# Restart Kratos
cd /opt/arack
docker compose restart kratos

# Verify logs
docker logs search_engine_kratos --tail 50 | grep -i domain
```

3. **Test:**

```bash
# Test cookie domain in response headers
curl -i 'https://auth.arack.io/self-service/login/browser' 2>&1 | grep -i 'set-cookie'

# Expected: Domain=.arack.io (with leading dot)
```

4. **Browser Test:**
   - Clear all `arack.io` cookies
   - Login at `https://arack.io/auth/login`
   - Check DevTools → Cookies → Domain should show `.arack.io`
   - Navigate to `https://mail.arack.io`
   - Verify same cookie visible

**Risk:** 🟢 **LOW**
- Standard approach
- Easy rollback (restore config backup)
- No code changes required

---

### Fix Option 2: Use Ory Proxy (Alternative)

**Why:**
- Ory provides a reverse proxy that handles cookie domain issues
- Proxy automatically manages session tokens across subdomains
- Production-grade solution used by Ory Network

**Steps:**

1. **Install Ory CLI on VPS**
2. **Configure Ory Proxy** to run in front of Kratos
3. **Update Nginx** to proxy to Ory Proxy instead of Kratos directly
4. **Test** cookie sharing through proxy

**Risk:** 🟡 **MEDIUM**
- More complex setup
- Additional service to maintain
- May have performance overhead

**Recommendation:** Only use if Fix Option 1 fails

---

### Fix Option 3: Custom Cookie Middleware (Last Resort)

**Why:**
- Full control over cookie attributes
- Can set cookies via custom middleware in search/email services
- Works around Kratos limitations

**Steps:**

1. **Implement cookie relay middleware** in Rust backend
2. **Forward Kratos session** to custom cookie with correct domain
3. **Update API authentication** to use custom cookie
4. **Test** across subdomains

**Risk:** 🔴 **HIGH**
- Complex implementation
- Bypasses Kratos security features
- Maintenance burden

**Recommendation:** Avoid unless absolutely necessary

---

## Testing Checklist

After applying fix, verify:

- [ ] **Browser DevTools** shows `Domain: .arack.io` (with leading dot)
- [ ] Login on `arack.io` creates `ory_kratos_session` cookie
- [ ] Cookie visible on `mail.arack.io` after navigation
- [ ] Cookie visible on `admin.arack.io` after navigation
- [ ] API call from `mail.arack.io` includes cookie in request headers
- [ ] API returns user data (not "No session cookie found" error)
- [ ] Logout on any subdomain clears cookie across all subdomains
- [ ] Session persists across browser tabs on different subdomains

---

## Verification Commands

### Check Cookie Domain in Browser

```javascript
// On arack.io after login:
document.cookie.split(';').forEach(c => console.log(c.trim()))

// Look for: ory_kratos_session=...
```

### Test API with Cookie from mail.arack.io

```javascript
// On mail.arack.io:
fetch('https://api-mail.arack.io/api/mail/account/me', {
  credentials: 'include'
})
.then(r => r.json())
.then(data => {
  console.log('SUCCESS:', data);
})
.catch(err => {
  console.error('ERROR:', err);
})
```

### Check Cookie in Network Tab

```javascript
// On mail.arack.io:
fetch('https://api-mail.arack.io/api/mail/mailboxes', {
  credentials: 'include'
})
.then(r => {
  console.log('Response headers:', [...r.headers.entries()]);
  return r.json();
})
.then(console.log)
```

Then check Network tab → Request Headers → Should see `Cookie: ory_kratos_session=...`

---

## Expected Outcome

**After Fix:**

1. User logs in on `arack.io`
2. Cookie set: `ory_kratos_session` with `Domain=.arack.io`
3. User navigates to `mail.arack.io`
4. Cookie automatically available
5. API calls work with authentication
6. No "No session cookie found" errors

**Cookie Attributes:**
```
Name: ory_kratos_session
Domain: .arack.io  ← Leading dot is key!
Path: /
Secure: Yes (auto via HTTPS)
HttpOnly: Yes
SameSite: Lax
```

---

## Next Steps

1. **First:** Run diagnostic steps to confirm root cause
2. **Then:** Apply Fix Option 1 (add leading dot to domain)
3. **Test:** Verify cookie sharing across subdomains
4. **Monitor:** Check for any authentication errors in logs
5. **Document:** Update configuration documentation with findings

---

## References

- [How to share the cookies between subdomains | ABP.IO](https://abp.io/community/articles/how-to-share-the-cookies-between-subdomains-jfrzggc2)
- [Which Domains and Subdomains Can Access Browser Cookies?](https://www.uptimia.com/questions/which-domains-and-subdomains-can-access-browser-cookies)
- [How to Access Cross-Domain Cookies: A Comprehensive Guide](https://captaincompliance.com/education/how-to-access-cross-domain-cookies-a-comprehensive-guide/)
- [Ory Kratos Multi-Domain Cookies](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies)

---

**Status:** 📋 **DIAGNOSTIC PLAN READY**

**Recommended Action:** Have user check browser DevTools to confirm cookie domain, then apply leading dot fix
