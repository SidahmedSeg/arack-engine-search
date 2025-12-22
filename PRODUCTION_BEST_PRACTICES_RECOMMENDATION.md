# Production Best Practices - Cookie Domain Architecture

## Research Summary from Official Documentation

Based on official Ory documentation and industry best practices for 2025.

---

## 🎯 Official Ory Kratos Recommendation

### **PRIMARY RECOMMENDATION: Same Domain + Path-Based Routing**

Source: [Advanced base URL, CSRF and session cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies)

**Ory's Official Stance:**

> "It's **not recommended** running them on separate subdomains, such as https://kratos.my-website/ and https://secureapp.my-website/. To allow cookies to work across subdomains, the **best solution** is hosting both systems and routing paths with a Reverse Proxy such as Nginx or Envoy or AWS API Gateway."

**What This Means:**
- ❌ **DON'T:** Use `auth.arack.io`, `api.arack.io`, `mail.arack.io` for authentication
- ✅ **DO:** Use single domain `arack.io` with path-based routing:
  - `arack.io/auth/*` → Kratos
  - `arack.io/api/*` → Your API
  - `arack.io/mail/*` → Mail service

**Why:**
- No cookie domain issues at all
- No CORS complications
- No cross-subdomain security concerns
- Simplest architecture

---

## 🏗️ If You MUST Use Subdomains (Your Current Situation)

Source: [Cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/configuring-cookies)

**When Subdomains Are Acceptable:**
- Serverless platforms (CloudRun, Heroku, Vercel)
- Microservices architecture with separate deployments
- Legacy system constraints

**Proper Subdomain Architecture:**

### 1️⃣ Use Dedicated Auth Subdomain

Source: [Custom Domain Best Practices - Auth0 Community](https://community.auth0.com/t/custom-domain-best-practices-architecture-and-testing-questions/47289)

**✅ CORRECT:**
```
auth.arack.io  → Kratos authentication
api.arack.io   → Your API
mail.arack.io  → Mail frontend
```

**Why:**
- **Branding:** Users see `auth.arack.io` instead of third-party domain
- **Security:** Clear separation of authentication concerns
- **Maintainability:** Authentication services scale independently
- **Trust:** Users confident they're providing credentials to right party

**❌ WRONG (Your Current Setup):**
```
api.arack.io/api/auth/*  → Authentication mixed with API
```

**Why This Is Bad:**
- Violates separation of concerns
- Confuses cookie domain handling
- Makes debugging harder
- Not scalable

### 2️⃣ Cookie Domain Configuration

Source: [Multi-domain cookies | Ory Kratos](https://www.ory.sh/kratos/docs/v0.5/guides/multi-domain-cookies/)

**Official Recommendation:**

```yaml
# Kratos config
cookies:
  domain: arack.io  # NO leading dot in config
  path: /
  same_site: Lax

session:
  cookie:
    domain: arack.io  # NO leading dot in config
    path: /
    same_site: Lax
```

**Modern Browser Behavior:**
- `domain: arack.io` (in config) → Browser stores as `.arack.io` automatically
- `domain: .arack.io` (in config) → Also works, but not necessary

**Critical Security Note:**

> "Be cautious of setting the cookie same_site attribute to Lax in a production environment - this is why Kratos has the --dev flag for local development"

For production:
- Use `SameSite: Strict` if possible
- Only use `Lax` if you need cross-subdomain navigation
- Never use `None` in production

### 3️⃣ Nginx Reverse Proxy Configuration

Source: [Deploy Ory Cloud with NGINX | Bomberbot](https://www.bomberbot.com/proxy/deploy-ory-cloud-with-nginx-for-security-performance-and-flexibility/)

**Is `proxy_cookie_domain` a Best Practice?**

**Answer:** **It's a pragmatic workaround, not ideal.**

From research:
- Used when "application-layer changes aren't feasible"
- Can interfere with session timeout functionality
- Acceptable in production, but application-layer handling is preferred

**Better Approach:**
1. Configure Kratos correctly (cookie domain at application level)
2. Frontend calls correct subdomain (auth.arack.io, not api.arack.io)
3. Nginx only routes, doesn't manipulate cookies

**If You Must Use `proxy_cookie_domain`:**
- Only as temporary fix
- Plan to refactor to proper architecture
- Monitor for cookie-related issues

---

## 📊 Production Architecture Comparison

### Option A: Path-Based Routing (BEST - Ory Recommended)

```
┌─────────────────────────────────────────┐
│         arack.io (Single Domain)        │
├─────────────────────────────────────────┤
│                                         │
│  /auth/*  → Kratos (port 4433)         │
│  /api/*   → Search Service (port 3000)  │
│  /mail/*  → Mail Service (port 3001)    │
│  /*       → Frontend (port 5001)        │
│                                         │
└─────────────────────────────────────────┘

✅ No cookie domain issues
✅ No CORS issues
✅ Simplest configuration
✅ Ory's official recommendation
```

**Nginx Configuration:**
```nginx
server {
    listen 443 ssl http2;
    server_name arack.io;

    location /auth/ {
        proxy_pass http://kratos:4433/;
    }

    location /api/ {
        proxy_pass http://search-service:3000/api/;
    }

    location /mail/ {
        proxy_pass http://mail-frontend:5006/;
    }

    location / {
        proxy_pass http://frontend:5001/;
    }
}
```

**Cookie Domain:** Just use default (arack.io)

---

### Option B: Subdomain Routing (ACCEPTABLE - If Properly Architected)

```
┌─────────────────────────────────────────┐
│          Subdomain Architecture         │
├─────────────────────────────────────────┤
│                                         │
│  auth.arack.io  → Kratos (port 4433)   │
│  api.arack.io   → Search Service       │
│  mail.arack.io  → Mail Frontend        │
│  arack.io       → Main Frontend        │
│                                         │
└─────────────────────────────────────────┘

⚠️  Requires cookie domain: arack.io
⚠️  Requires CORS configuration
⚠️  More complex certificate management
✅  Easier DNS-level routing
✅  Better service isolation
```

**Kratos Configuration:**
```yaml
cookies:
  domain: arack.io

serve:
  public:
    base_url: https://auth.arack.io/
    cors:
      enabled: true
      allowed_origins:
        - https://arack.io
        - https://mail.arack.io
      allow_credentials: true
```

**Frontend Configuration:**
```typescript
// Use auth.arack.io for ALL authentication
const AUTH_BASE_URL = 'https://auth.arack.io';

// Login flow
axios.get(`${AUTH_BASE_URL}/self-service/login/browser`);
```

---

### Option C: Current Setup (PROBLEMATIC - Your Situation)

```
┌─────────────────────────────────────────┐
│          Current Architecture           │
├─────────────────────────────────────────┤
│                                         │
│  api.arack.io/api/auth/* → Kratos      │  ❌ Mixed concerns
│  api.arack.io/self-service/* → Kratos  │  ❌ Confusing paths
│  mail.arack.io → Mail Frontend         │  ❌ Cookie domain mismatch
│                                         │
└─────────────────────────────────────────┘

❌ Authentication mixed with API subdomain
❌ Requires nginx cookie rewriting
❌ Violates separation of concerns
❌ Debugging nightmare
```

---

## 🎯 Recommended Solution for Your Production

### **Phase 1: Immediate Fix (Today)**

**Use nginx `proxy_cookie_domain` as temporary workaround**

**Why:** Gets system working NOW while you plan proper architecture

**Implementation:**
```nginx
# api.arack.io server block
location / {
    proxy_pass http://search-service:3000;
    proxy_cookie_domain api.arack.io .arack.io;
}
```

**Drawback:** Pragmatic workaround, not best practice

**Time:** 5 minutes
**Risk:** Low

---

### **Phase 2: Proper Architecture (Next Week)**

**Migrate to dedicated auth subdomain**

**Steps:**

1. **Update Kratos Configuration**

```yaml
serve:
  public:
    base_url: https://auth.arack.io/
```

2. **Update Frontend to Call auth.arack.io**

```typescript
// frontend-search/src/lib/api/kratos.ts
const AUTH_BASE_URL = 'https://auth.arack.io';

// All auth calls
await axios.get(`${AUTH_BASE_URL}/self-service/login/browser`);
```

3. **Remove nginx Cookie Rewriting**

```nginx
# No more proxy_cookie_domain hacks
# Cookies handled correctly by Kratos
```

4. **Update All Frontends**
- frontend-search
- frontend-email
- frontend-admin

**Benefits:**
- ✅ Follows Ory best practices
- ✅ Clean separation of concerns
- ✅ No nginx cookie manipulation
- ✅ Better security and maintainability

**Time:** 2-3 hours
**Risk:** Medium (requires testing)

---

### **Phase 3: Ideal Architecture (Future - Optional)**

**Migrate to single domain with path-based routing**

**Only if:**
- Major refactor is planned
- Want absolutely simplest architecture
- No subdomain constraints

**Benefits:**
- ✅ Ory's #1 recommendation
- ✅ Zero cookie issues
- ✅ Zero CORS issues
- ✅ Simplest possible setup

**Drawback:**
- Major architectural change
- Requires frontend routing updates
- Certificate changes

---

## 🔒 Production Security Checklist

Source: [Go to production | Ory](https://www.ory.sh/docs/kratos/guides/production)

**Must-Have for Production:**

### 1. Kratos Configuration
- ✅ `cookies.domain: arack.io` (root domain)
- ✅ `same_site: Strict` (or Lax if needed for subdomain navigation)
- ✅ HTTPS only (cookies automatically secure)
- ✅ Change secrets from development defaults
- ✅ Admin API NOT exposed to public internet

### 2. Reverse Proxy (Nginx)
- ✅ SSL/TLS certificates for all subdomains
- ✅ HSTS headers enabled
- ✅ Admin API accessible only from internal network
- ✅ Rate limiting configured
- ✅ Security headers (X-Frame-Options, CSP, etc.)

### 3. Frontend
- ✅ Call auth.arack.io for authentication (not api.arack.io)
- ✅ Use `withCredentials: true` for all auth requests
- ✅ HTTPS only
- ✅ Proper CORS configuration

### 4. Database
- ✅ PostgreSQL with connection pooling
- ✅ Encrypted connections
- ✅ Regular backups

### 5. Monitoring
- ✅ Log authentication failures
- ✅ Monitor cookie-related errors
- ✅ Alert on unusual authentication patterns

---

## 📚 References & Sources

### Official Ory Documentation
- [Advanced base URL, CSRF and session cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/multi-domain-cookies)
- [Cookie settings | Ory](https://www.ory.sh/docs/kratos/guides/configuring-cookies)
- [Go to production | Ory](https://www.ory.sh/docs/kratos/guides/production)
- [Deploy Kratos example | Ory](https://www.ory.com/docs/kratos/guides/deploy-kratos-example)
- [Ory Proxy | Ory](https://www.ory.sh/docs/cli/ory-proxy) - **NOT for production**

### Industry Best Practices
- [Custom Domain Best Practices - Auth0](https://community.auth0.com/t/custom-domain-best-practices-architecture-and-testing-questions/47289)
- [Authentication across subdomains - Indie Hackers](https://www.indiehackers.com/post/authentication-across-sub-domains-239245e7a1)
- [Nginx Cookie Rewriting Guide](https://yukaii.tw/hi-tips/2017-04-13-rewrite-cookie-cross-subdomain-using-nginx/)

### Community Discussions
- [Working with Kratos on different subdomains](https://github.com/ory/kratos/discussions/2235)
- [NextAuth subdomain authentication](https://github.com/nextauthjs/next-auth/issues/2414)

---

## 🎯 Final Recommendation

### **For Immediate Production Fix:**

**Do This Today:** Option 1 - nginx cookie rewriting
- Quick fix (5 minutes)
- Gets system working
- Acceptable workaround

### **For Long-Term Production:**

**Do This Soon:** Option 2 - Dedicated auth subdomain
- Follows Ory best practices
- Clean architecture
- No cookie hacks
- Production-grade solution

### **Optional Future Enhancement:**

**Consider:** Option 3 - Single domain path-based routing
- Ory's #1 recommendation
- Simplest possible architecture
- Only if major refactor planned

---

## ⚡ Action Plan Summary

| Phase | Action | Time | Priority |
|-------|--------|------|----------|
| **NOW** | Add `proxy_cookie_domain` to nginx | 5 min | 🔴 Critical |
| **This Week** | Migrate to `auth.arack.io` subdomain | 3 hours | 🟡 High |
| **Optional** | Consider path-based routing | N/A | 🟢 Low |

**Bottom Line:**
1. Fix it now with nginx workaround
2. Do it right with dedicated auth subdomain
3. Sleep well knowing you follow industry best practices
