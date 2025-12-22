# Cookie Domain Fix Plan V2 - Root Cause Identified

## Test Results from Browser

**Domain:** arack.io
```
Cookie Name: ory_kratos_session
Cookie Value: ory_st_biylAlYqeuUdVunSE1s5EsVGEfObSxgG (STATE TOKEN!)
Domain: api.arack.io  ← WRONG!
Path: /
SameSite: Lax
```

**Domain:** mail.arack.io
```
NO COOKIES AT ALL
```

---

## Root Cause Analysis

### The Problem

**Cookie domain is `api.arack.io` instead of `.arack.io`**

Despite our Kratos configuration changes, cookies are STILL being set with the wrong domain.

### Why This Is Happening

**1. Frontend Calls api.arack.io for Authentication**

File: `frontend-search/src/lib/api/kratos.ts:10`
```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';
```

All authentication endpoints call `api.arack.io`:
- `https://api.arack.io/api/auth/flows/login`
- `https://api.arack.io/api/auth/flows/registration`
- `https://api.arack.io/api/auth/whoami`

**2. Nginx Proxy Path**

```
Browser (arack.io)
   ↓
api.arack.io (Nginx)
   ↓
location / → search_service:3000 (Rust backend)
   ↓
Kratos:4433 (internal HTTP call)
   ↓
Set-Cookie: Domain=.arack.io (from Kratos config)
   ↓
Rust backend forwards response
   ↓
Nginx forwards to browser
   ↓
BUT: Cookie domain becomes api.arack.io!
```

**3. Missing proxy_cookie_domain in Main Location**

Current nginx config for `api.arack.io`:
```nginx
location /self-service/ {
    proxy_cookie_domain api.arack.io .arack.io;  ← HAS IT
}

location / {  ← MISSING IT!
    proxy_pass http://search_engine_search_service:3000;
    # NO proxy_cookie_domain directive!
}
```

**The `/api/auth/*` endpoints go through `location /`, NOT `location /self-service/`!**

### Why Our Previous Fix Didn't Work

We added `proxy_cookie_domain` to:
- `/self-service/` location (line 143) ✅
- `auth.arack.io /` location (line 202) ✅

BUT we did NOT add it to:
- `api.arack.io /` location ❌ **THIS IS THE PROBLEM!**

---

## Fix Plan - Three Approaches

### 🎯 Option 1: Add Cookie Domain Rewriting to api.arack.io (RECOMMENDED)

**Pros:**
- Quick fix (1-2 minutes)
- No code changes required
- Backwards compatible
- Works immediately

**Cons:**
- Relies on nginx rewriting
- Doesn't fix architectural issue

**Steps:**

1. **Add proxy_cookie_domain to api.arack.io location /**

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Backup nginx config
cp /opt/arack/nginx/sites-enabled/arack.io.conf /opt/arack/nginx/sites-enabled/arack.io.conf.backup.cookie_v2_$(date +%Y%m%d_%H%M%S)

# Edit nginx config
nano /opt/arack/nginx/sites-enabled/arack.io.conf
```

Find the `api.arack.io` server block's `location /` section:
```nginx
location / {
    proxy_pass http://search_engine_search_service:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Origin $http_origin;
}
```

**Add these lines:**
```nginx
location / {
    proxy_pass http://search_engine_search_service:3000;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_set_header Origin $http_origin;

    # ⭐ ADD THESE LINES ⭐
    proxy_cookie_path / /;
    proxy_cookie_domain api.arack.io .arack.io;
}
```

2. **Test nginx configuration:**
```bash
nginx -t
```

3. **Reload nginx:**
```bash
kill -HUP $(cat /var/run/nginx/nginx.pid) || kill -HUP $(ps aux | grep 'nginx: master' | grep -v grep | awk '{print $2}')
```

4. **Test the fix:**
```bash
# Clear cookies and test login flow
curl -c /tmp/test.txt -i 'https://api.arack.io/api/auth/flows/login' 2>&1 | grep -i 'set-cookie'

# Check cookie domain in file
cat /tmp/test.txt
```

**Expected Result:**
```
#HttpOnly_.arack.io	TRUE	/	FALSE	...
```

**Time Estimate:** 5 minutes

---

### ⚡ Option 2: Change Frontend to Use auth.arack.io (CLEAN ARCHITECTURE)

**Pros:**
- Cleaner architecture
- Uses dedicated auth subdomain
- Separates concerns (API vs Auth)
- Future-proof

**Cons:**
- Requires code changes
- Requires frontend rebuild and redeploy
- Takes longer to implement

**Steps:**

1. **Update Frontend Environment Variable**

Local file: `frontend-search/.env`
```env
VITE_API_URL=https://api.arack.io
VITE_AUTH_URL=https://auth.arack.io  ← ADD THIS
```

VPS file: `/opt/arack/frontend-search/.env.production`
```env
VITE_API_URL=https://api.arack.io
VITE_AUTH_URL=https://auth.arack.io  ← ADD THIS
```

2. **Update Kratos API Client**

File: `frontend-search/src/lib/api/kratos.ts:10-11`
```typescript
const API_BASE_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';
const AUTH_BASE_URL = import.meta.env.VITE_AUTH_URL || 'https://auth.arack.io';  ← ADD THIS
```

Then change all auth endpoints:
```typescript
// Line 89 - OLD
const response = await axios.get(`${API_BASE_URL}/api/auth/flows/registration`);

// Line 89 - NEW
const response = await axios.get(`${AUTH_BASE_URL}/self-service/registration/browser`);

// Line 110 - OLD
const response = await axios.post(`${API_BASE_URL}/api/auth/flows/registration`, ...);

// Line 110 - NEW
const response = await axios.post(`${AUTH_BASE_URL}/self-service/registration`, ...);

// Similar changes for login, logout, whoami
```

3. **Rebuild and Deploy Frontend**

```bash
cd "/Users/intelifoxdz/RS Projects/Engine_search/frontend-search"

# Update .env
echo "VITE_AUTH_URL=https://auth.arack.io" >> .env

# Build
npm run build

# Package
tar -czf build.tar.gz build/

# Upload to VPS
scp -i ~/.ssh/id_rsa_arack build.tar.gz root@213.199.59.206:/tmp/

# Deploy on VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206
cd /opt/arack/frontend-search
tar -xzf /tmp/build.tar.gz
pm2 restart frontend-search
```

**Time Estimate:** 20-30 minutes

---

### 🚀 Option 3: Combined Approach (BEST SOLUTION)

**Implement BOTH Option 1 and Option 2**

**Why:**
- Option 1 (nginx fix) = Immediate fix (works now)
- Option 2 (frontend change) = Long-term clean architecture

**Pros:**
- Immediate fix + proper architecture
- Maximum compatibility
- Redundant cookie domain enforcement
- Future-proof

**Cons:**
- More work (both fixes)

**Time Estimate:** 30 minutes total

---

## Recommended Implementation Order

### Phase 1: Immediate Fix (5 minutes)
✅ Implement Option 1 - Add proxy_cookie_domain to nginx

**Reason:** Get mail.arack.io working NOW

### Phase 2: Clean Architecture (Later)
✅ Implement Option 2 - Update frontend to use auth.arack.io

**Reason:** Proper separation of concerns

---

## Implementation Commands (Option 1 - Quick Fix)

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Backup
cp /opt/arack/nginx/sites-enabled/arack.io.conf /opt/arack/nginx/sites-enabled/arack.io.conf.backup.v2

# Find the line number for api.arack.io location /
grep -n 'location / {' /opt/arack/nginx/sites-enabled/arack.io.conf | grep -A 5 -B 5 'api.arack.io'

# Assuming line 170 (adjust based on actual line number)
# Add cookie directives after proxy_set_header Origin line
sed -i '/server_name api.arack.io/,/^}/{ /location \/ {/,/^    \}/ { /proxy_set_header Origin/a\        \n        # Cookie domain rewriting\n        proxy_cookie_path \/ \/;\n        proxy_cookie_domain api.arack.io .arack.io; } }' /opt/arack/nginx/sites-enabled/arack.io.conf

# Test nginx
nginx -t

# If test passes, reload
kill -HUP $(ps aux | grep 'nginx: master' | grep -v grep | awk '{print $2}')

# Verify
curl -c /tmp/cookie_test.txt 'https://api.arack.io/api/auth/flows/login' > /dev/null 2>&1
cat /tmp/cookie_test.txt | grep arack.io
```

**Expected Output:**
```
#HttpOnly_.arack.io	TRUE	/	FALSE	...	csrf_token_...	...
```

If you see `.arack.io` ← SUCCESS!
If you see `api.arack.io` ← Need manual editing

---

## Manual Nginx Edit (If sed fails)

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206
nano /opt/arack/nginx/sites-enabled/arack.io.conf
```

Find this section:
```nginx
server {
    listen 443 ssl http2;
    server_name api.arack.io;

    ...

    location / {
        proxy_pass http://search_engine_search_service:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Origin $http_origin;
    }
}
```

**Change to:**
```nginx
server {
    listen 443 ssl http2;
    server_name api.arack.io;

    ...

    location / {
        proxy_pass http://search_engine_search_service:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Origin $http_origin;

        # Cookie domain rewriting for cross-subdomain auth
        proxy_cookie_path / /;
        proxy_cookie_domain api.arack.io .arack.io;
    }
}
```

Save (Ctrl+O, Enter, Ctrl+X)

```bash
nginx -t && kill -HUP $(ps aux | grep 'nginx: master' | grep -v grep | awk '{print $2}')
```

---

## Testing After Fix

### Test 1: Server-Side Cookie Check
```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206
curl -c /tmp/test.txt -i 'https://api.arack.io/api/auth/flows/login' 2>&1 | grep -i 'set-cookie'
cat /tmp/test.txt
```

**Expected:**
```
#HttpOnly_.arack.io	TRUE	/	FALSE	...
```

### Test 2: Browser Test

**Clear all cookies**, then:

1. Go to `https://arack.io/auth/login`
2. Login with credentials
3. Check DevTools → Cookies → arack.io
4. **Verify:** Domain shows `.arack.io` (not `api.arack.io`)
5. Navigate to `https://mail.arack.io`
6. Check DevTools → Cookies → mail.arack.io
7. **Verify:** Same session cookie appears

### Test 3: API Call from mail.arack.io

Open Console on `https://mail.arack.io`:
```javascript
fetch('https://api-mail.arack.io/api/mail/account/me', {credentials: 'include'})
.then(r => r.json())
.then(console.log)
```

**Expected:** User account data (NOT "No session cookie found")

---

## Rollback Plan

If the fix breaks something:

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Restore backup
cp /opt/arack/nginx/sites-enabled/arack.io.conf.backup.v2 /opt/arack/nginx/sites-enabled/arack.io.conf

# Reload nginx
nginx -t && kill -HUP $(ps aux | grep 'nginx: master' | grep -v grep | awk '{print $2}')
```

---

## Why This Will Work

**Before Fix:**
```
api.arack.io → search_service → kratos
                              ↓ Set-Cookie: Domain=.arack.io
                              ↑ (ignored by nginx)
               ← Cookie sent to browser as Domain=api.arack.io
```

**After Fix:**
```
api.arack.io → search_service → kratos
                              ↓ Set-Cookie: Domain=.arack.io
                              ↑ Nginx rewrites: Domain=.arack.io
               ← Cookie sent to browser as Domain=.arack.io ✅
```

**Result:**
- Browser stores cookie with Domain=`.arack.io`
- Cookie is sent to ALL `*.arack.io` subdomains
- mail.arack.io receives the cookie
- Authentication works across all subdomains

---

## Summary

| Issue | Root Cause | Fix |
|-------|------------|-----|
| Cookie domain = `api.arack.io` | Missing `proxy_cookie_domain` in nginx `location /` | Add directive to rewrite domain |
| Frontend calls api.arack.io | Hardcoded in kratos.ts | Update to use auth.arack.io (optional) |
| mail.arack.io has no cookies | Subdomain-specific cookie | Fix domain to `.arack.io` |

**Primary Fix:** Add `proxy_cookie_domain api.arack.io .arack.io;` to nginx config

**Time:** 5 minutes

**Risk:** LOW (easy rollback, only affects new logins)

---

**Next Steps:**
1. Approve this plan
2. Execute Option 1 (nginx fix) immediately
3. Test in browser
4. Optionally implement Option 2 later for clean architecture
