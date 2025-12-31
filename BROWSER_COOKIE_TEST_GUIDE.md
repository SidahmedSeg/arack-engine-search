# Browser Cookie Test Guide

## Test Date: December 19, 2025

## Objective
Verify that session cookies set on `arack.io` are properly shared with `mail.arack.io` after the cookie domain fix.

---

## Pre-Test Setup

**IMPORTANT: Start with a clean state**

1. Open browser (Chrome, Firefox, or Safari)
2. Open DevTools (F12 or Right-click → Inspect)
3. Go to Application tab → Cookies
4. **Delete ALL cookies for arack.io domain**
5. Close DevTools
6. Close and reopen browser (fresh session)

---

## Test Procedure

### TEST 1: Login and Cookie Creation

**Steps:**
1. Navigate to: `https://arack.io/auth/login`
2. Open DevTools → Application → Cookies → `https://arack.io`
3. Enter login credentials for existing user
4. Complete login process
5. After successful login, check cookies again

**Expected Results:**
- ✅ Cookie named `ory_kratos_session` should appear
- ✅ Domain should show: `.arack.io` OR `arack.io` (both acceptable)
- ✅ Path should be: `/`
- ✅ HttpOnly should be: ✅ (checked)
- ✅ Secure should be: ✅ (checked)
- ✅ SameSite should be: `Lax`

**Record Results:**
```
□ ory_kratos_session cookie found: YES / NO
□ Cookie Domain: ________________
□ HttpOnly: YES / NO
□ Secure: YES / NO
□ SameSite: ________________
```

---

### TEST 2: Cookie Visibility on Subdomain

**Steps:**
1. While still logged in, navigate to: `https://mail.arack.io`
2. Open DevTools → Application → Cookies → `https://mail.arack.io`
3. Look for the `ory_kratos_session` cookie

**Expected Results:**
- ✅ Same `ory_kratos_session` cookie should appear
- ✅ Cookie value should be identical to the one from arack.io
- ✅ Domain should still show: `.arack.io` or `arack.io`

**Record Results:**
```
□ ory_kratos_session cookie visible on mail.arack.io: YES / NO
□ Cookie value matches arack.io: YES / NO
□ Cookie Domain: ________________
```

**Screenshot Location:** Take screenshot of DevTools Cookies panel

---

### TEST 3: API Authentication

**Steps:**
1. Stay on `https://mail.arack.io`
2. Open DevTools → Console tab
3. Run this command:

```javascript
fetch('https://api-mail.arack.io/api/mail/account/me', {
  credentials: 'include',
  headers: { 'Content-Type': 'application/json' }
})
.then(r => r.json())
.then(data => {
  console.log('✅ SUCCESS:', data);
  document.body.innerHTML = '<pre style="background:#d4edda;padding:20px;margin:20px;border-radius:8px;color:#155724">' +
    'TEST PASSED!\n\n' +
    'Email: ' + data.account.email_address + '\n' +
    'Status: Authenticated\n' +
    'Cookie sharing: WORKING' +
    '</pre>';
})
.catch(err => {
  console.error('❌ ERROR:', err);
  document.body.innerHTML = '<pre style="background:#f8d7da;padding:20px;margin:20px;border-radius:8px;color:#721c24">' +
    'TEST FAILED!\n\n' +
    'Error: ' + err.message + '\n' +
    'Cookie sharing: NOT WORKING' +
    '</pre>';
})
```

**Expected Results:**
- ✅ Console shows: `✅ SUCCESS:` followed by user data
- ✅ Page shows green success box with email address
- ✅ No "No session cookie found" error

**Record Results:**
```
□ API call successful: YES / NO
□ Error message (if any): ________________
□ User email displayed: ________________
```

---

### TEST 4: Network Tab Cookie Verification

**Steps:**
1. Stay on `https://mail.arack.io`
2. Open DevTools → Network tab
3. Check "Preserve log"
4. Run the API call again (repeat TEST 3 command)
5. Click on the `account/me` request in Network tab
6. Go to "Headers" section
7. Scroll to "Request Headers"

**Expected Results:**
- ✅ Request Headers should include: `Cookie: ory_kratos_session=...`
- ✅ Cookie value should be present (not empty)

**Record Results:**
```
□ Cookie header present in request: YES / NO
□ Cookie header value: ________________ (first 20 chars)
```

**Screenshot Location:** Take screenshot of Network tab Request Headers

---

### TEST 5: Cross-Subdomain Persistence

**Steps:**
1. Open a NEW browser tab (keep session active)
2. Navigate to: `https://admin.arack.io`
3. Open DevTools → Application → Cookies → `https://admin.arack.io`
4. Check for `ory_kratos_session` cookie

**Expected Results:**
- ✅ Same `ory_kratos_session` cookie should appear
- ✅ No login prompt should appear
- ✅ User should be authenticated automatically

**Record Results:**
```
□ Cookie visible on admin.arack.io: YES / NO
□ Automatically authenticated: YES / NO
```

---

### TEST 6: Logout Cookie Cleanup

**Steps:**
1. On any subdomain (arack.io, mail.arack.io, or admin.arack.io), logout
2. Open DevTools → Application → Cookies
3. Check cookies on:
   - `https://arack.io`
   - `https://mail.arack.io`
   - `https://admin.arack.io`

**Expected Results:**
- ✅ `ory_kratos_session` cookie should be deleted from ALL subdomains
- ✅ Only `csrf_token_*` cookies may remain

**Record Results:**
```
□ Session cookie deleted from arack.io: YES / NO
□ Session cookie deleted from mail.arack.io: YES / NO
□ Session cookie deleted from admin.arack.io: YES / NO
```

---

## Test Results Summary

### Overall Status: PASS / FAIL / PARTIAL

| Test | Status | Notes |
|------|--------|-------|
| TEST 1: Cookie Creation | ☐ PASS ☐ FAIL | |
| TEST 2: Subdomain Visibility | ☐ PASS ☐ FAIL | |
| TEST 3: API Authentication | ☐ PASS ☐ FAIL | |
| TEST 4: Network Cookie Header | ☐ PASS ☐ FAIL | |
| TEST 5: Cross-Subdomain Persistence | ☐ PASS ☐ FAIL | |
| TEST 6: Logout Cleanup | ☐ PASS ☐ FAIL | |

---

## Issues Encountered

**If any test fails, record details here:**

```
Issue Description:
_________________________________________________

Browser Used:
_________________________________________________

Error Messages:
_________________________________________________

Cookie Domain Shown:
_________________________________________________

Additional Notes:
_________________________________________________
```

---

## Server-Side Test Results (Already Completed)

✅ **Kratos Configuration:** Domain set to `.arack.io` (with leading dot)
✅ **Nginx Cookie Rewriting:** Configured for api.arack.io and auth.arack.io
✅ **Cookie Attributes:** HttpOnly, SameSite=Lax, Path=/
✅ **Set-Cookie Headers:** Properly sending Domain=arack.io

**Expected Outcome:**
All browser tests should **PASS** based on server configuration.

---

## Quick Test (5 minutes)

If you want a quick verification without the full test suite:

1. **Clear all arack.io cookies**
2. **Login at https://arack.io/auth/login**
3. **Check DevTools → Cookies → Domain field**
   - Should show: `.arack.io` or `arack.io`
4. **Navigate to https://mail.arack.io**
5. **Check DevTools → Cookies**
   - Should see the same session cookie
6. **Run in Console:**
   ```javascript
   fetch('https://api-mail.arack.io/api/mail/account/me', {credentials: 'include'})
   .then(r => r.json())
   .then(console.log)
   ```
7. **Check Console output:**
   - ✅ Should show user account data
   - ❌ Should NOT show "No session cookie found"

---

## Screenshots Required

Please take screenshots of:

1. **Cookie panel on arack.io after login** (showing domain field)
2. **Cookie panel on mail.arack.io** (showing same cookie)
3. **Network tab showing Cookie request header**
4. **Console output from API test**

---

## Troubleshooting

### If cookies don't appear on mail.arack.io:

1. **Check Domain field** - Must be `.arack.io` or `arack.io` (not `api.arack.io`)
2. **Clear browser cache** - Sometimes old cookies interfere
3. **Try incognito/private mode** - Eliminates extension interference
4. **Try different browser** - Chrome, Firefox, Safari
5. **Check for ad blockers** - May interfere with cookies

### If API call fails:

1. **Check Network tab** - Look for Cookie header in request
2. **Check Console errors** - CORS or authentication errors
3. **Verify you're logged in** - Session cookie must exist
4. **Check cookie expiry** - Cookie may have expired

---

## Next Steps After Testing

**If ALL tests PASS:**
- ✅ Cookie fix is working correctly
- ✅ Mark issue as resolved
- ✅ Document the solution

**If ANY test FAILS:**
- ❌ Record which test failed
- ❌ Note the cookie Domain value shown
- ❌ Provide screenshots
- ❌ Report back for further debugging

---

**Tester Name:** ________________

**Date/Time:** ________________

**Browser:** ________________

**OS:** ________________
