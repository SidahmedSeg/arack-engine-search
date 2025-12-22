# Cross-Subdomain Cookie Sharing Fix Plan

## Problem Statement

**Current Issue:**
- Users authenticate on `arack.io` or `admin.arack.io`
- Session cookies are set ONLY for the specific domain (e.g., `127.0.0.1`)
- When users navigate to `mail.arack.io`, the session cookies are NOT shared
- Result: `{"error":"No session cookie found"}` on mail.arack.io
- Users must re-authenticate on each subdomain separately

**Expected Behavior:**
- User logs in once on `arack.io`
- Session cookie is shared across ALL subdomains: `mail.arack.io`, `admin.arack.io`, etc.
- User remains authenticated across the entire `arack.io` domain

---

## Root Cause Analysis

### Current Kratos Configuration (`ory/kratos/kratos.yml`)

**Lines 31-34:**
```yaml
cookies:
  domain: 127.0.0.1  # ❌ PROBLEM: localhost only, not production domain
  path: /
  same_site: Lax
```

**Issues Identified:**
1. ❌ **Cookie domain is `127.0.0.1`** - Development configuration, NOT production `arack.io`
2. ❌ **No session cookie domain override** - Missing `session.cookie.domain` setting
3. ❌ **CORS only allows localhost** (lines 10-12) - Production domains not whitelisted
4. ❌ **All URLs use localhost** - `base_url`, `allowed_return_urls`, `ui_url` all use `http://127.0.0.1`
5. ❌ **No secure flag** - Missing `secure: true` for HTTPS production cookies

---

## Research Summary

Based on official Ory Kratos documentation and best practices:

### Official Ory Kratos Documentation

**Source:** [Advanced base URL, CSRF and session cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies)

**Key Findings:**
1. **Subdomain Cookie Sharing:**
   - Subdomains can set HTTP Cookies for parent domains
   - Setting `domain: arack.io` allows cookies to work on `mail.arack.io`, `admin.arack.io`, etc.
   - Quote: *"A HTTP Cookie specifying domain=my-domain.com will be allowed to set even if the URL is http://sub.my-domain.com"*

2. **Configuration Structure:**
   ```yaml
   cookies:
     domain: example.com  # Global cookie domain
     path: /
     same_site: Lax

   session:
     cookie:
       domain: example.com  # Session-specific override (optional)
       path: /
       same_site: Lax
   ```

3. **Same-Site Attribute:**
   - Use `Lax` for cross-subdomain authentication
   - `Strict` would block cookies on subdomain redirects
   - Quote from [Cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/configuring-cookies): *"Be cautious of setting the cookie same_site attribute to Lax in a production environment"*

4. **Secure Flag:**
   - Must use HTTPS and set `secure: true` for production
   - Quote: *"Always use HTTPS for cookie sharing and set the Secure flag on cookies to prevent man-in-the-middle attacks"*

### Security Best Practices

**Source:** [How to Access Cross-Domain Cookies: A Comprehensive Guide](https://captaincompliance.com/education/how-to-access-cross-domain-cookies-a-comprehensive-guide/)

**Critical Security Flags:**
1. ✅ **Secure flag** - Only send over HTTPS
2. ✅ **SameSite=Lax** - Prevent CSRF while allowing subdomain access
3. ✅ **HttpOnly** - Prevent JavaScript access (Kratos default)
4. ✅ **Specific domain scope** - Use `arack.io` not `.arack.io` (without leading dot)

### Common Pitfalls

**Source:** [working with kratos on different subdomains · Discussion #2235](https://github.com/ory/kratos/discussions/2235)

**Warnings:**
1. ❌ **Don't use leading dot** - Modern browsers: `arack.io` not `.arack.io`
2. ❌ **Can't share across TLDs** - `arack.io` and `otherdomain.com` CAN'T share cookies
3. ⚠️ **Base URL must match** - Kratos public base URL should use production domain

---

## Solution Design

### Configuration Changes Required

#### 1. Production Kratos Configuration

**File:** `/opt/arack/ory/kratos/kratos.yml` (on VPS)

**Changes:**

```yaml
serve:
  public:
    base_url: https://auth.arack.io/  # Change from http://127.0.0.1:4433/
    cors:
      enabled: true
      allowed_origins:
        # Keep localhost for local development
        - http://127.0.0.1:5001
        - http://localhost:5001
        # Add production domains
        - https://arack.io
        - https://www.arack.io
        - https://mail.arack.io
        - https://admin.arack.io
      allowed_methods:
        - POST
        - GET
        - PUT
        - PATCH
        - DELETE
      allowed_headers:
        - Authorization
        - Content-Type
        - Cookie
        - X-Session-Token
      exposed_headers:
        - Content-Type
        - Set-Cookie
      allow_credentials: true
  admin:
    base_url: http://127.0.0.1:4434/  # Keep internal for admin API

cookies:
  domain: arack.io  # ✅ CRITICAL: Share cookies across all subdomains
  path: /
  same_site: Lax    # ✅ Allow cross-subdomain, prevent CSRF
  secure: true      # ✅ HTTPS only (production)

session:
  cookie:
    domain: arack.io  # ✅ Explicit session cookie domain
    path: /
    same_site: Lax
    secure: true
    persistent: false  # Session cookies (not stored after browser close)

selfservice:
  default_browser_return_url: https://arack.io/
  allowed_return_urls:
    - https://arack.io
    - https://www.arack.io
    - https://mail.arack.io
    - https://admin.arack.io
    - http://127.0.0.1:5001  # Keep for local dev
    - http://localhost:5001

  flows:
    error:
      ui_url: https://arack.io/auth/error

    settings:
      ui_url: https://arack.io/auth/settings
      privileged_session_max_age: 15m

    recovery:
      enabled: true
      ui_url: https://arack.io/auth/recovery
      use: code

    verification:
      enabled: true
      ui_url: https://arack.io/auth/verification
      use: code
      after:
        default_browser_return_url: https://arack.io/auth/verified

    logout:
      after:
        default_browser_return_url: https://arack.io/

    login:
      ui_url: https://arack.io/auth/login
      lifespan: 10m
      after:
        password:
          default_browser_return_url: https://arack.io/auth/callback

    registration:
      lifespan: 10m
      ui_url: https://arack.io/auth/register
      after:
        password:
          hooks:
            - hook: web_hook
              config:
                url: http://search-service:3000/internal/auth/user-created
                method: POST
                body: file:///etc/config/kratos/webhooks/user-created.jsonnet
                response:
                  ignore: false
                  parse: false
            - hook: web_hook
              config:
                url: http://email-service:3001/internal/mail/provision
                method: POST
                body: file:///etc/config/kratos/webhooks/user-created.jsonnet
                response:
                  ignore: false
                  parse: false
            - hook: session
            - hook: show_verification_ui
          default_browser_return_url: https://arack.io/auth/verify-email
```

#### 2. Nginx Configuration for auth.arack.io

**File:** `/opt/arack/nginx/sites-enabled/arack.io.conf`

**Add new server block:**

```nginx
# Ory Kratos Public API (auth.arack.io)
server {
    listen 80;
    server_name auth.arack.io;

    # Redirect to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name auth.arack.io;

    # SSL configuration (use existing Let's Encrypt certs or create new)
    ssl_certificate /etc/letsencrypt/live/arack.io/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/arack.io/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Proxy to Kratos public API
    location / {
        proxy_pass http://search_engine_kratos:4433;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header X-Forwarded-Host $host;

        # CRITICAL: Pass cookies
        proxy_pass_request_headers on;
        proxy_set_header Cookie $http_cookie;
    }
}
```

#### 3. Docker Compose Environment Variables

**File:** `/opt/arack/docker-compose.yml`

**Update Kratos environment:**

```yaml
kratos:
  image: oryd/kratos:v1.1.0
  container_name: search_engine_kratos
  ports:
    - "4433:4433" # Public API
    - "4434:4434" # Admin API
  environment:
    - DSN=postgres://postgres:postgres@postgres:5432/kratos_db?sslmode=disable
    - LOG_LEVEL=debug
    - SERVE_PUBLIC_BASE_URL=https://auth.arack.io/  # ✅ CHANGE THIS
    - SERVE_ADMIN_BASE_URL=http://127.0.0.1:4434/
  volumes:
    - ./ory/kratos:/etc/config/kratos
  command: serve -c /etc/config/kratos/kratos.yml --dev --watch-courier
  restart: unless-stopped
  networks:
    - search_network
  depends_on:
    - kratos-migrate
```

---

## Implementation Steps

### Phase 1: Local Testing (Development Environment)

**Goal:** Test cookie sharing on localhost using different ports to simulate subdomains

**Steps:**

1. **Create test configuration** (`ory/kratos/kratos.test.yml`):
   ```yaml
   cookies:
     domain: 127.0.0.1
     same_site: Lax
     # Note: Can't test subdomain sharing on localhost
   ```

2. **Test on localhost:**
   - Login on `http://127.0.0.1:5001`
   - Try accessing `http://127.0.0.1:5002`
   - **Expected:** Session shared (same domain, different ports work)

3. **Verify cookie attributes in browser DevTools:**
   - Open Application → Cookies
   - Check `ory_kratos_session` cookie:
     - Domain: `127.0.0.1`
     - Path: `/`
     - SameSite: `Lax`
     - Secure: `false` (localhost uses HTTP)

### Phase 2: VPS Production Deployment

**Goal:** Deploy production configuration with cross-subdomain cookie sharing

#### Step 1: Backup Current Configuration

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Backup current kratos config
cd /opt/arack
cp ory/kratos/kratos.yml ory/kratos/kratos.yml.backup.$(date +%Y%m%d_%H%M%S)

# Backup nginx config
cp nginx/sites-enabled/arack.io.conf nginx/sites-enabled/arack.io.conf.backup.$(date +%Y%m%d_%H%M%S)

# Backup docker-compose
cp docker-compose.yml docker-compose.yml.backup.$(date +%Y%m%d_%H%M%S)
```

#### Step 2: Update Kratos Configuration

```bash
# Edit kratos.yml with production settings
nano ory/kratos/kratos.yml

# Apply changes from "Solution Design" section above
# Key changes:
# - cookies.domain: arack.io
# - cookies.secure: true
# - session.cookie.domain: arack.io
# - session.cookie.secure: true
# - All URLs changed from http://127.0.0.1 to https://arack.io
# - CORS allowed_origins includes all subdomains
```

#### Step 3: Configure DNS for auth.arack.io

```bash
# Add A record in DNS provider (Cloudflare/Route53/etc.)
# Type: A
# Name: auth
# Value: 213.199.59.206 (VPS IP)
# TTL: Auto or 300

# Verify DNS propagation
dig auth.arack.io +short
# Expected: 213.199.59.206
```

#### Step 4: Update Nginx Configuration

```bash
# Add auth.arack.io server block
nano nginx/sites-enabled/arack.io.conf

# Add the configuration from "Solution Design" section above

# Test nginx configuration
nginx -t

# Expected output:
# nginx: the configuration file /etc/nginx/nginx.conf syntax is ok
# nginx: configuration file /etc/nginx/nginx.conf test is successful
```

#### Step 5: Generate SSL Certificate for auth.arack.io

```bash
# Using Let's Encrypt certbot
certbot certonly --nginx -d auth.arack.io

# Or expand existing certificate (if using wildcard or SAN)
certbot certonly --nginx -d arack.io -d www.arack.io -d mail.arack.io -d admin.arack.io -d auth.arack.io

# Verify certificate
ls -la /etc/letsencrypt/live/arack.io/
# Should see: fullchain.pem, privkey.pem
```

#### Step 6: Update Docker Compose

```bash
# Edit docker-compose.yml
nano docker-compose.yml

# Update Kratos environment variable:
# - SERVE_PUBLIC_BASE_URL=https://auth.arack.io/
```

#### Step 7: Restart Services

```bash
# Restart Kratos container
docker-compose restart kratos

# Check Kratos logs
docker logs search_engine_kratos --tail 50

# Expected output:
# "Ory Kratos is listening on public endpoint at https://auth.arack.io/"
# No errors about cookie configuration

# Reload nginx
nginx -s reload
```

### Phase 3: Testing & Verification

#### Test 1: Cookie Domain Verification

**Browser DevTools Test:**

1. **Clear all cookies:**
   - Open browser DevTools → Application → Cookies
   - Delete all `arack.io` cookies

2. **Login on main domain:**
   - Navigate to `https://arack.io/auth/login`
   - Login with test user: `flow.tester@arack.io`
   - Check DevTools → Application → Cookies

3. **Verify session cookie attributes:**
   ```
   Name: ory_kratos_session
   Value: [random token]
   Domain: arack.io           ✅ (not 127.0.0.1)
   Path: /
   Expires: Session
   Size: ~200 bytes
   HttpOnly: ✅ Yes
   Secure: ✅ Yes (HTTPS)
   SameSite: Lax             ✅
   Priority: Medium
   ```

4. **Navigate to subdomain:**
   - Go to `https://mail.arack.io`
   - Check DevTools → Application → Cookies
   - **Expected:** Same `ory_kratos_session` cookie is present

5. **Test API authentication:**
   - Open DevTools → Console
   - Run:
     ```javascript
     fetch('https://api-mail.arack.io/api/mail/account/me', {
       credentials: 'include'
     })
     .then(r => r.json())
     .then(console.log)
     ```
   - **Expected:** User account data (not "No session cookie found" error)

#### Test 2: Cross-Subdomain Session Sharing

**Test Scenario:**

1. **Login on arack.io:**
   ```bash
   # Via browser or curl
   curl -X POST https://auth.arack.io/self-service/login/flows \
     -H "Content-Type: application/json" \
     -c cookies.txt
   ```

2. **Access mail.arack.io:**
   ```bash
   curl https://mail.arack.io \
     -b cookies.txt
   ```
   - **Expected:** Authenticated session, no redirect to login

3. **Access admin.arack.io:**
   ```bash
   curl https://admin.arack.io \
     -b cookies.txt
   ```
   - **Expected:** Authenticated session, no redirect to login

#### Test 3: CORS Preflight Verification

**Test from mail.arack.io origin:**

```javascript
// Open https://mail.arack.io
// Run in DevTools Console:

fetch('https://api.arack.io/api/auth/me', {
  method: 'GET',
  credentials: 'include',
  headers: {
    'Content-Type': 'application/json'
  }
})
.then(r => r.json())
.then(console.log)
.catch(console.error)

// Expected: User data (no CORS errors)
```

#### Test 4: Session Persistence

**Multi-Tab Test:**

1. Open 3 browser tabs:
   - Tab 1: `https://arack.io`
   - Tab 2: `https://mail.arack.io`
   - Tab 3: `https://admin.arack.io`

2. Login on Tab 1 (arack.io)

3. Refresh Tab 2 and Tab 3

4. **Expected:**
   - All tabs show authenticated state
   - No login prompts on subdomains
   - Session cookie visible in all tabs (same domain)

#### Test 5: Logout Propagation

**Test Scenario:**

1. Login on `https://arack.io`
2. Verify session on `https://mail.arack.io` (should be authenticated)
3. Logout on `https://arack.io`
4. Refresh `https://mail.arack.io`

**Expected:**
- Session cookie is deleted (domain-wide)
- `https://mail.arack.io` redirects to login
- No authenticated state on any subdomain

---

## Security Considerations

### 1. Cookie Security Flags

| Flag | Value | Purpose |
|------|-------|---------|
| **Secure** | `true` | Only send over HTTPS, prevent MITM attacks |
| **HttpOnly** | `true` (Kratos default) | Prevent JavaScript access, protect against XSS |
| **SameSite** | `Lax` | Allow subdomain cookies, prevent CSRF |
| **Domain** | `arack.io` | Share across all subdomains |
| **Path** | `/` | Cookie available on all paths |

### 2. CORS Configuration

**Critical Settings:**
- `allow_credentials: true` - REQUIRED for cookie sharing
- `allowed_origins` - Explicit whitelist (no wildcards)
- `exposed_headers: Set-Cookie` - Allow frontend to read Set-Cookie header

### 3. Subdomain Security

**Risks:**
- Any compromised subdomain can access session cookies
- Malicious subdomain could steal session tokens

**Mitigations:**
1. ✅ **Use HTTPS on ALL subdomains** (already done)
2. ✅ **Strict CSP headers** on each subdomain
3. ✅ **HSTS with includeSubDomains** flag
4. ✅ **Regular security audits** of all subdomain applications
5. ✅ **Principle of least privilege** for subdomain access

### 4. Session Fixation Prevention

**Kratos Built-in Protections:**
- Session regeneration on login ✅
- Session invalidation on logout ✅
- CSRF tokens on all state-changing operations ✅

### 5. Cookie Theft Prevention

**Protections:**
- HttpOnly flag prevents JavaScript access ✅
- Secure flag prevents HTTP interception ✅
- SameSite=Lax prevents cross-site cookie sending ✅
- Short session lifetime (configurable) ✅

---

## Rollback Plan

If cookie sharing breaks authentication:

### Quick Rollback

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206
cd /opt/arack

# Restore kratos config
cp ory/kratos/kratos.yml.backup.YYYYMMDD_HHMMSS ory/kratos/kratos.yml

# Restore docker-compose
cp docker-compose.yml.backup.YYYYMMDD_HHMMSS docker-compose.yml

# Restart Kratos
docker-compose restart kratos

# Check logs
docker logs search_engine_kratos --tail 50
```

### Partial Rollback

If only specific settings cause issues:

```yaml
# Revert just cookie domain
cookies:
  domain: 127.0.0.1  # Back to localhost (DEV ONLY)
  same_site: Lax

# Keep production URLs
serve:
  public:
    base_url: https://auth.arack.io/
```

---

## Success Criteria

### Must Have (Before marking as complete):

1. ✅ Session cookie domain is `arack.io` (visible in browser DevTools)
2. ✅ Login on `arack.io` creates session accessible on `mail.arack.io`
3. ✅ API calls from `mail.arack.io` to `api-mail.arack.io` include session cookie
4. ✅ No "No session cookie found" errors on mail.arack.io
5. ✅ CORS preflight requests succeed from all subdomains
6. ✅ Logout on one subdomain invalidates session on all subdomains
7. ✅ All cookies have `Secure: true` flag (HTTPS only)
8. ✅ Kratos logs show no cookie configuration errors

### Should Have (Enhanced functionality):

1. ✅ DNS resolves `auth.arack.io` correctly
2. ✅ SSL certificate valid for `auth.arack.io`
3. ✅ Nginx proxies `auth.arack.io` to Kratos public API
4. ✅ Session persists across browser tabs on different subdomains
5. ✅ Frontend applications can check authentication status via `/api/auth/me`

### Nice to Have (Future improvements):

1. ⚪ Session refresh mechanism (extends session before expiry)
2. ⚪ Remember me functionality (persistent cookies)
3. ⚪ Multi-device session management
4. ⚪ Session activity monitoring dashboard

---

## Monitoring & Debugging

### Check Cookie Configuration in Production

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# View current Kratos config
cat /opt/arack/ory/kratos/kratos.yml | grep -A 5 "cookies:"

# Expected output:
# cookies:
#   domain: arack.io
#   path: /
#   same_site: Lax
#   secure: true
```

### Check Kratos Logs for Cookie Issues

```bash
# Real-time logs
docker logs search_engine_kratos --tail 100 -f | grep -i cookie

# Check for errors
docker logs search_engine_kratos --tail 500 | grep -i error

# Check session creation
docker logs search_engine_kratos --tail 500 | grep -i "session created"
```

### Test Cookie from Command Line

```bash
# Login and save cookies
curl -X POST https://auth.arack.io/self-service/login/browser \
  -c cookies.txt \
  -L

# Verify cookie file
cat cookies.txt | grep ory_kratos_session

# Expected format:
# arack.io	TRUE	/	TRUE	0	ory_kratos_session	[token]
#   ^          ^      ^    ^     ^         ^                   ^
#  domain   subdomain path secure expiry  name              value
```

### Browser DevTools Debug Steps

1. **Open Network Tab:**
   - Filter: `auth.arack.io`
   - Look for `Set-Cookie` header in response
   - Verify `Domain=arack.io` attribute

2. **Check Application Tab:**
   - Cookies → arack.io
   - Verify `ory_kratos_session` exists
   - Check Domain, Path, Secure, SameSite values

3. **Console Commands:**
   ```javascript
   // Check if cookie is accessible
   document.cookie.includes('ory_kratos_session')

   // Should return false (HttpOnly flag prevents access)
   // This is CORRECT behavior
   ```

### Common Issues & Fixes

| Issue | Cause | Fix |
|-------|-------|-----|
| Cookie not set on login | Kratos base URL mismatch | Check `SERVE_PUBLIC_BASE_URL` matches domain |
| Cookie domain is `127.0.0.1` | Config not updated | Verify `cookies.domain: arack.io` in kratos.yml |
| CORS error on subdomain | Origin not whitelisted | Add subdomain to `allowed_origins` |
| Cookie not sent to API | `credentials: 'include'` missing | Update fetch calls with `credentials: 'include'` |
| 401 on mail.arack.io | Session not shared | Verify cookie domain is parent domain |
| Secure cookie rejected | Using HTTP | Ensure all subdomains use HTTPS |

---

## Timeline Estimate

| Phase | Tasks | Time |
|-------|-------|------|
| **Phase 1: Local Testing** | Test configuration on localhost | 30 min |
| **Phase 2: VPS Deployment** | Update configs, DNS, SSL, deploy | 1-2 hours |
| **Phase 3: Testing** | Comprehensive testing across subdomains | 30-45 min |
| **Phase 4: Monitoring** | Watch logs, verify production behavior | 24 hours |

**Total:** ~3-4 hours active work + 24 hours monitoring

---

## References & Documentation

### Official Ory Documentation
- [Advanced base URL, CSRF and session cookie settings](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies)
- [Cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/configuring-cookies)
- [working with kratos on different subdomains · Discussion #2235](https://github.com/ory/kratos/discussions/2235)

### Security Best Practices
- [How to Access Cross-Domain Cookies: A Comprehensive Guide](https://captaincompliance.com/education/how-to-access-cross-domain-cookies-a-comprehensive-guide/)
- [Cross-Site Cookies | Descope Documentation](https://docs.descope.com/security-best-practices/crossite-cookies)

### Community Discussions
- [Recommended approach to share session information between applications on subdomains](https://github.com/nextauthjs/next-auth/issues/2414)
- [How to share the cookies between subdomains | ABP.IO](https://abp.io/community/articles/how-to-share-the-cookies-between-subdomains-jfrzggc2)

---

## Next Steps After Implementation

1. **Update Frontend API Clients:**
   - Ensure all `fetch()` calls use `credentials: 'include'`
   - Update API base URLs to use production domains
   - Add error handling for expired sessions

2. **Update Documentation:**
   - Document cookie sharing architecture in README.md
   - Add troubleshooting guide for session issues
   - Update API documentation with authentication requirements

3. **Implement Session Monitoring:**
   - Track session creation/expiration metrics
   - Monitor cross-subdomain session usage
   - Alert on authentication errors

4. **Plan for Future Enhancements:**
   - Session refresh mechanism
   - Remember me functionality
   - Multi-device session management
   - OAuth2/OIDC integration for social login

---

## Summary

**The Fix:**
Change `cookies.domain: 127.0.0.1` → `cookies.domain: arack.io` in Kratos configuration

**Why It Works:**
- Setting domain to parent domain (`arack.io`) shares cookies with all subdomains
- Browser automatically sends cookies to `mail.arack.io`, `admin.arack.io`, etc.
- Single login provides authentication across entire domain

**Key Requirements:**
1. ✅ HTTPS on all subdomains (already in place)
2. ✅ SameSite=Lax to allow cross-subdomain (configured)
3. ✅ Secure=true for production cookies (needs update)
4. ✅ CORS allows all subdomain origins (needs update)
5. ✅ Consistent cookie configuration across environments

**Risk Level:** **LOW**
- Cookie sharing is a standard practice
- Ory Kratos explicitly supports this use case
- Easy rollback if issues occur
- No database changes required
- No code changes required (just configuration)

---

**Status:** 📋 PLAN READY FOR REVIEW

**Next Action:** User approval to proceed with implementation
