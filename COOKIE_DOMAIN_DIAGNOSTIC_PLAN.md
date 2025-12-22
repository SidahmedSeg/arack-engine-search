# Cookie Domain Diagnostic Plan - After nginx Fix Still Shows api.arack.io

## Current Situation

**Problem:** Browser still shows `Domain: api.arack.io` in cookies despite nginx `proxy_cookie_domain` fix deployed and verified

**What We Know:**
- ✅ nginx config has `proxy_cookie_domain api.arack.io .arack.io` in correct location
- ✅ nginx inside Docker container reloaded successfully
- ✅ Server-side curl test shows cookies with `.arack.io` domain
- ❌ Browser still shows `api.arack.io` domain after fresh login

---

## Root Cause Analysis from Research

Based on official nginx documentation and community troubleshooting, several potential issues emerged:

### Issue 1: Backend Application May Be Setting Cookies Directly

**Source:** [Module ngx_http_proxy_module](https://nginx.org/en/docs/http/ngx_http_proxy_module.html)

**Finding:** `proxy_cookie_domain` ONLY rewrites cookies that come FROM the proxied backend in Set-Cookie headers. If your **application code** sets cookies directly (not via proxy), nginx cannot rewrite them.

**Your Architecture:**
```
Browser → nginx → Rust Search Service (port 3000) → Kratos (port 4433)
```

**Critical Question:** When the Rust backend calls Kratos and gets a response with cookies, does it:
1. **Pass through** the Set-Cookie headers unchanged? (nginx CAN rewrite) ✅
2. **Modify** the cookies before sending to browser? (nginx CANNOT rewrite) ❌
3. **Set new cookies** in application code? (nginx CANNOT rewrite) ❌

### Issue 2: Multiple Set-Cookie Headers

**Source:** [Modifying or deleting cookies sent by upstream server](https://clouddocs.f5.com/training/community/nginx/html/class3/module1/module15.html)

**Finding:** When backend sends multiple Set-Cookie headers, nginx's `proxy_cookie_domain` may not apply to all of them consistently.

**Problem:** If Kratos sends cookies AND your Rust backend adds cookies, the rewriting may only apply to one set.

### Issue 3: Cookie Domain Must Match What Backend Sends

**Source:** [Using Nginx reverse-proxy to set cross-site cookies](https://mickeyabhi1999.medium.com/using-nginx-reverse-proxy-to-set-cross-site-cookies-for-your-web-app-7c9e5e502091)

**Finding:** The first parameter in `proxy_cookie_domain` must EXACTLY match what the backend sends.

**Current Config:**
```nginx
proxy_cookie_domain api.arack.io .arack.io;
```

**This assumes backend sends:** `Domain=api.arack.io`

**But if backend sends:** `Domain=127.0.0.1` or no domain attribute → nginx won't match → no rewrite

### Issue 4: Rust Backend May Be Proxying Incorrectly

**Source:** [Ory Kratos Cookie Settings](https://www.ory.sh/docs/kratos/guides/configuring-cookies)

**Finding:** When proxying Kratos, the application must preserve Set-Cookie headers exactly as Kratos sends them.

**Potential Issue:** Your Rust backend (Axum) might be:
- Reading cookies from Kratos response
- Re-constructing Set-Cookie headers with different domain
- Adding cookies in middleware

---

## Diagnostic Steps Plan

### Step 1: Check What Rust Backend Actually Sends

**Goal:** Capture actual Set-Cookie headers from Rust backend to browser

**Action:**
```bash
# Test the EXACT endpoint browser uses during login
curl -v -X POST "https://api.arack.io/api/auth/flows/login" \
  -H "Content-Type: application/json" \
  -d '{"flow_id":"test","identifier":"test@test.com","password":"test"}' \
  2>&1 | grep -i "set-cookie"
```

**Look For:**
- What domain is in Set-Cookie header?
- Are there multiple Set-Cookie headers?
- What cookies are being set (ory_kratos_session, csrf_token, others)?

### Step 2: Check Rust Backend Source Code

**Goal:** Understand how Rust backend handles Kratos cookies

**Files to Check:**
- `src/api/auth/flows.rs` or similar (the endpoint handling `/api/auth/flows/login`)
- Any middleware that handles cookies
- Response construction code

**Questions:**
1. Does it use `proxy_pass` style forwarding OR call Kratos as an HTTP client?
2. Does it read Set-Cookie from Kratos response and reconstruct them?
3. Does it use any cookie manipulation libraries?
4. Does it add cookies in application code?

### Step 3: Compare nginx Log vs Browser Network Tab

**Goal:** See if nginx is rewriting but browser is caching old cookies

**Actions:**
1. Enable nginx access log to show Set-Cookie headers
2. Make fresh login request from browser
3. Compare:
   - What nginx sends (access log)
   - What browser receives (Network tab → Headers)

### Step 4: Test Direct Kratos Endpoint (Bypass Rust Backend)

**Goal:** Verify cookies work correctly when Rust backend is bypassed

**Action:**
```bash
# Test auth.arack.io which goes DIRECTLY to Kratos (no Rust backend)
curl -c /tmp/direct_kratos.txt -s "https://auth.arack.io/self-service/registration/browser" > /dev/null
cat /tmp/direct_kratos.txt
```

**Expected:** Should show `.arack.io` domain

**Then test in browser:**
1. Go to `https://auth.arack.io/self-service/login/browser`
2. Check cookies in DevTools
3. See if domain is correct

### Step 5: Check for Client-Side Cookie Setting

**Goal:** Rule out JavaScript setting cookies with wrong domain

**Action:**
1. Open browser Network tab
2. Login at `https://arack.io/auth/login`
3. Look for ANY fetch/XHR requests that might set cookies via JavaScript
4. Check frontend source for `document.cookie =` statements

### Step 6: Test with Regex Pattern

**Goal:** Try alternative nginx configuration approach

**Source:** [nginx ticket #2331](https://trac.nginx.org/nginx/ticket/2331)

**Config Change:**
```nginx
# Instead of:
proxy_cookie_domain api.arack.io .arack.io;

# Try regex pattern:
proxy_cookie_domain ~^(.+)\.arack\.io$ .arack.io;

# OR more specific:
proxy_cookie_domain ~\.? api.arack.io .arack.io;
```

---

## Most Likely Root Causes (Ranked)

### 🔴 #1: Rust Backend Modifies Cookies Before Sending to Browser

**Probability:** 90%

**Why:**
- curl tests show `.arack.io` when calling `/self-service/` (Kratos direct)
- Browser shows `api.arack.io` when calling `/api/auth/flows/login` (Rust backend)
- Difference suggests Rust backend is changing cookies

**Evidence Needed:**
- Check Rust source code for cookie handling
- Capture Set-Cookie headers from Rust backend

**Solution If True:**
- Fix Rust backend to NOT modify cookie domain
- OR use dedicated `auth.arack.io` subdomain (bypass Rust backend)

### 🟡 #2: Browser Has Cached Cookies from JavaScript

**Probability:** 30%

**Why:**
- JavaScript can set cookies that override server-set cookies
- SvelteKit might be setting cookies client-side

**Evidence Needed:**
- Check Network tab for client-side cookie setting
- Search frontend code for `document.cookie`

**Solution If True:**
- Remove client-side cookie setting code
- Rely only on server Set-Cookie headers

### 🟢 #3: nginx Regex Pattern Needed

**Probability:** 10%

**Why:**
- Some configurations require regex for domain matching

**Evidence Needed:**
- Test with regex pattern in nginx

**Solution If True:**
- Use `proxy_cookie_domain ~\.? api.arack.io .arack.io;`

---

## Immediate Action Plan

### Phase A: Diagnose (30 minutes)

**Priority 1: Check Rust Backend Response Headers**

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Get a login flow ID
FLOW=$(curl -s "https://api.arack.io/api/auth/flows/login" | grep -o '"id":"[^"]*"' | cut -d'"' -f4)

# Test login submission and capture headers
curl -v -X POST "https://api.arack.io/api/auth/flows/login" \
  -H "Content-Type: application/json" \
  -d "{\"flow_id\":\"$FLOW\",\"identifier\":\"test@test.com\",\"password\":\"wrong\"}" \
  2>&1 | tee /tmp/rust_backend_response.txt

# Check for Set-Cookie headers
grep -i "set-cookie" /tmp/rust_backend_response.txt
```

**Priority 2: Check Rust Source Code**

Look at how the Rust backend handles Kratos responses:
- File: `src/api/auth/flows.rs` (or wherever `/api/auth/flows/login` is implemented)
- Check for cookie manipulation
- Check if it proxies Kratos correctly

**Priority 3: Test Browser with Network Tab**

1. Open browser in Incognito mode (fresh start)
2. Open DevTools → Network tab
3. Navigate to `https://arack.io/auth/login`
4. Enter credentials and login
5. Check Network tab for the POST request to `/api/auth/flows/login`
6. Look at Response Headers → Set-Cookie
7. What domain does it show?

### Phase B: Implement Fix (Based on Findings)

**If Rust Backend is the Issue:**

**Option 1: Fix Rust Code (Best)**
- Modify Rust backend to preserve Kratos Set-Cookie headers exactly
- Don't modify cookie domain in application code

**Option 2: Bypass Rust Backend (Recommended - 3 hours)**
- Migrate to dedicated `auth.arack.io` subdomain
- Frontend calls Kratos directly (no Rust middleware)
- See: `COOKIE_DOMAIN_FIX_PLAN_V2.md` Option 2

**Option 3: Use nginx Header Manipulation (Advanced)**
- Use `proxy_hide_header Set-Cookie` and `add_header Set-Cookie` with variable manipulation
- More complex but can override application-set cookies

---

## Testing Checklist

### Server-Side Tests

- [ ] Capture Set-Cookie headers from `/api/auth/flows/login` endpoint
- [ ] Compare Set-Cookie from `/self-service/` vs `/api/auth/flows/login`
- [ ] Check nginx access logs for Set-Cookie headers
- [ ] Test with regex pattern in nginx config

### Code Review

- [ ] Check Rust backend auth flow handler
- [ ] Search for cookie manipulation in Rust code
- [ ] Check for Axum middleware that modifies cookies
- [ ] Search frontend for `document.cookie` statements

### Browser Tests

- [ ] Fresh incognito mode test
- [ ] Network tab inspection of Set-Cookie headers
- [ ] DevTools Application → Cookies domain check
- [ ] Test with different browsers (Chrome, Firefox, Safari)

---

## Expected Outcomes

### Scenario 1: Rust Backend Modifies Cookies

**Evidence:**
- `/self-service/` returns `Domain=.arack.io` ✅
- `/api/auth/flows/login` returns `Domain=api.arack.io` ❌
- Rust code shows cookie reconstruction

**Solution:** Fix Rust backend OR migrate to auth.arack.io subdomain

**Time:** 3 hours (migration to auth subdomain)

### Scenario 2: nginx Configuration Issue

**Evidence:**
- Both endpoints return same Set-Cookie headers
- Browser still shows wrong domain
- nginx logs show unrewritten cookies

**Solution:** Use regex pattern OR advanced header manipulation

**Time:** 30 minutes

### Scenario 3: Browser/JavaScript Issue

**Evidence:**
- Server sends `Domain=.arack.io`
- Browser receives `Domain=.arack.io`
- But cookies show `Domain=api.arack.io`

**Solution:** Remove client-side cookie setting code

**Time:** 1 hour

---

## Reference Sources

All findings based on official documentation and community troubleshooting:

1. [Module ngx_http_proxy_module](https://nginx.org/en/docs/http/ngx_http_proxy_module.html) - Official nginx documentation
2. [Modifying cookies from upstream](https://clouddocs.f5.com/training/community/nginx/html/class3/module1/module15.html) - F5 nginx training
3. [Using Nginx reverse-proxy for cookies](https://mickeyabhi1999.medium.com/using-nginx-reverse-proxy-to-set-cross-site-cookies-for-your-web-app-7c9e5e502091) - Cookie handling guide
4. [nginx ticket #2331](https://trac.nginx.org/nginx/ticket/2331) - Known issues with proxy_cookie_domain
5. [Ory Kratos Cookie Settings](https://www.ory.sh/docs/kratos/guides/configuring-cookies) - Kratos configuration
6. [Kratos Multi-Domain Cookies](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies) - Cross-subdomain setup

---

## Next Steps

**User Decision Required:**

1. **Run diagnostic tests** (Phase A) to identify root cause
2. **Review Rust backend code** to see how it handles Kratos cookies
3. **Capture browser Network tab** screenshot showing Set-Cookie headers

**OR**

**Skip diagnostics and implement Option 2:**
- Migrate to `auth.arack.io` subdomain (clean architecture)
- Bypasses potential Rust backend cookie issues
- Production best practice per Ory recommendations
- Time: 3 hours

---

## Quick Diagnostic Command

Run this on VPS to quickly check what the Rust backend is sending:

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206 'bash -s' << 'DIAGEOF'
echo "=== Testing Rust Backend Cookie Headers ==="
echo ""

# Get flow ID
FLOW=$(curl -s "https://api.arack.io/api/auth/flows/login" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)
echo "Flow ID: $FLOW"
echo ""

# Test with wrong credentials (will still send cookies)
echo "Response headers from: POST /api/auth/flows/login"
echo "========================================="
curl -i -X POST "https://api.arack.io/api/auth/flows/login" \
  -H "Content-Type: application/json" \
  -d "{\"flow_id\":\"$FLOW\",\"identifier\":\"test@test.com\",\"password\":\"wrong\"}" \
  2>&1 | grep -i "set-cookie"
echo "========================================="
echo ""

echo "If you see Set-Cookie headers above with Domain=api.arack.io,"
echo "then the Rust backend is setting cookies, NOT Kratos."
echo ""
echo "If you see Domain=.arack.io or no Set-Cookie headers,"
echo "then the issue is elsewhere (browser cache, etc.)"
DIAGEOF
```

Would you like me to run the diagnostic test or proceed directly with the auth.arack.io migration?
