# Stalwart Authentication Options - Complete Analysis

**Date:** 2025-12-19
**Goal:** Find better solution than master user for unified authentication

---

## 🔍 Research Summary

I researched all available Stalwart authentication backends to find alternatives to the master user approach.

**Authentication Backends Supported by Stalwart:**
1. ✅ Internal Directory
2. ✅ LDAP
3. ✅ SQL (PostgreSQL, MySQL, SQLite)
4. ✅ OpenID Connect (OIDC)
5. ❌ HTTP/Webhook (NOT available)

---

## 📋 Option-by-Option Analysis

### **Option 1: SQL Authentication Backend**

**Source:** [Stalwart SQL Backend Documentation](https://stalw.art/docs/auth/backend/sql/)

**How it works:**
- Stalwart queries PostgreSQL database for authentication
- Verifies passwords against hashed values in SQL table

**Configuration Example:**
```sql
CREATE TABLE accounts (
  name TEXT PRIMARY KEY,
  secret TEXT,  -- Password hash
  type TEXT NOT NULL,
  active BOOLEAN DEFAULT true
);
```

**Required SQL Queries:**
```toml
[store.postgres.query]
name = "SELECT name, type, secret FROM accounts WHERE name = $1 AND active = true"
```

**Why This WON'T Work:** ❌

**Critical Issue: Password Hash Incompatibility**

Stalwart expects passwords hashed with:
- SHA512: `$6$salt$hash...`
- SHA256: `$5$salt$hash...`
- SHA1/MD5 (legacy)

**Example from Stalwart docs:**
```bash
# Generate Stalwart-compatible password
openssl passwd -6 "MyPassword123"
# Output: $6$MM1wz7Y8.L8O4eN0$ti3/072t3T5SJ6xryK45RvpW38dW2hSH86cBcV0XHtgnBYCCAFjqibS84OsdxfAITd6.VkKfhfUhlfVczdkFx1
```

Kratos stores passwords hashed with:
- Argon2id (modern, secure)
- bcrypt (older systems)

**Hash Format Comparison:**
```
Kratos (Argon2id):
  $argon2id$v=19$m=32768,t=4,p=1$base64salt$base64hash

Stalwart (SHA512):
  $6$salt$hash
```

**These are INCOMPATIBLE!** ❌

Even if we point Stalwart to Kratos's database, it can't verify passwords.

**Verdict:** ❌ **NOT VIABLE**

---

### **Option 2: OpenID Connect (OIDC) Backend**

**Source:** [Stalwart OIDC Backend Documentation](https://stalw.art/docs/auth/backend/oidc/)

**How it works:**
- Stalwart delegates authentication to OIDC provider
- Email clients use OAUTHBEARER SASL mechanism
- Stalwart validates access tokens with OIDC provider

**Configuration Example:**
```toml
[directory."oidc"]
type = "oidc"
endpoint.url = "https://auth.arack.io/userinfo"
endpoint.method = "userinfo"
fields.email = "email"
fields.username = "preferred_username"
```

**Requirements:**
1. ✅ OIDC provider (need to add **Ory Hydra**)
2. ❌ Email clients must support OAUTHBEARER
3. ❌ JMAP clients must provide access tokens

**Critical Issues:**

#### Issue 1: Kratos is NOT an OIDC Provider ❌

**From research:** [Ory Documentation](https://www.ory.com/docs)

> "Ory Kratos provides identity, credentials, and user-facing flows. However, **Kratos itself is not an OpenID Connect provider**."

**To be an OIDC provider, you need:**
- ✅ Ory Kratos (user management)
- ✅ **Ory Hydra** (OIDC server)

**This means adding another service to your stack!**

#### Issue 2: Our JMAP Client Uses Basic Auth ❌

**Current code:**
```rust
let auth = JmapAuth::Basic {
    username: "yacine.wanik",
    password: "password",
};
```

**OIDC expects:**
```rust
let auth = JmapAuth::Bearer(access_token);
```

**Would require rewriting the entire authentication flow!**

#### Issue 3: OAUTHBEARER is for Email Clients ❌

**From Stalwart docs:**
> "As a mail server, Stalwart expects clients to provide access tokens via the **OAUTHBEARER SASL mechanism**"

**OAUTHBEARER is for:**
- Thunderbird
- Outlook
- Apple Mail
- Other desktop email clients

**Our use case:**
- Backend Email API accessing JMAP
- Not a desktop email client
- Doesn't use SASL mechanisms

**Verdict:** ⚠️ **POSSIBLE BUT COMPLEX**

**Would require:**
1. Install and configure **Ory Hydra**
2. Integrate Kratos with Hydra
3. Rewrite Email API to use OAuth flow
4. Get access tokens from Hydra
5. Pass tokens to JMAP client
6. Handle token refresh
7. Manage token expiration

**Time estimate:** 2-3 days of work
**Added complexity:** HIGH
**Added services:** 1 (Hydra)

---

### **Option 3: LDAP Authentication Backend**

**Source:** [Stalwart LDAP Backend Documentation](https://stalw.art/docs/auth/backend/ldap/)

**How it works:**
- Stalwart queries LDAP server for authentication
- LDAP server validates passwords

**Configuration Example:**
```toml
[directory."ldap"]
type = "ldap"
url = "ldap://ldap.arack.io:389"
base-dn = "dc=arack,dc=io"
```

**Requirements:**
1. ❌ LDAP server (don't have one)
2. ❌ Kratos → LDAP bridge
3. ❌ LDAP schema mapping

**Verdict:** ❌ **NOT VIABLE**

**Would require:**
1. Install OpenLDAP or similar
2. Build Kratos → LDAP sync mechanism
3. Keep LDAP synchronized with Kratos
4. Another service to maintain

**This is MORE complex than master user!**

---

### **Option 4: HTTP/Webhook Authentication Backend**

**What I searched for:**
- HTTP authentication backend
- REST API password verification
- Custom webhook authentication

**Result:** ❌ **DOES NOT EXIST**

**From research:** [Stalwart Webhooks Documentation](https://stalw.art/docs/api/webhooks/)

Stalwart has webhooks for **outgoing events** (notifications):
- User login events
- Email delivery events
- Authentication failures

**But NO webhook for incoming authentication** (password verification)

**Verdict:** ❌ **NOT AVAILABLE**

---

### **Option 5: Remote SMTP/IMAP Backend**

**Source:** [Stalwart Remote Backend Documentation](https://stalw.art/docs/auth/backend/remote/)

**How it works:**
- Stalwart validates credentials against remote SMTP/IMAP server
- Used for migration scenarios

**Why This WON'T Work:** ❌

This is for authenticating against **another mail server**, not Kratos.

**Verdict:** ❌ **NOT APPLICABLE**

---

## 📊 **Comparison Table**

| Option | Complexity | Services Added | Kratos Compatible | Time to Implement |
|--------|------------|----------------|-------------------|-------------------|
| **Master User** | 🟢 Low | 0 | ✅ Yes | 1.5 hours |
| **SQL Backend** | 🟢 Low | 0 | ❌ No (hash incompatible) | N/A |
| **OIDC + Hydra** | 🔴 High | 1 (Hydra) | ✅ Yes | 2-3 days |
| **LDAP** | 🔴 Very High | 1 (LDAP server) | ⚠️ Needs bridge | 3-4 days |
| **HTTP/Webhook** | N/A | N/A | N/A | N/A (doesn't exist) |

---

## 🎯 **Final Recommendation**

After exhaustive research of all Stalwart authentication backends, **the master user approach is still the best solution** for your use case.

### Why Master User is Best:

✅ **Works with existing infrastructure**
- No new services needed
- No changes to Kratos
- Uses what you already have

✅ **Simple implementation**
- ~1.5 hours of work
- Well-documented pattern
- Clear rollback path

✅ **Secure**
- Admin credentials server-side only
- Session validation before every request
- Industry-standard proxy pattern

✅ **Maintainable**
- No password sync
- No additional services
- Easy to debug

### Why Alternatives Are Worse:

❌ **SQL Backend:** Password hash formats incompatible

❌ **OIDC:** Requires adding Ory Hydra + rewriting auth flow

❌ **LDAP:** Requires new LDAP server + sync mechanism

❌ **HTTP/Webhook:** Doesn't exist in Stalwart

---

## 🔄 **Alternative: Add Ory Hydra (If You Want OIDC)**

**IF** you want to go the OIDC route anyway:

### Architecture:
```
User → Kratos (login) → Hydra (OIDC tokens) → Email API → Stalwart (OIDC auth)
```

### Implementation Steps:

1. **Install Ory Hydra:**
```yaml
# docker-compose.yml
hydra:
  image: oryd/hydra:latest
  command: serve all --dev
  environment:
    - DSN=postgres://...
    - URLS_SELF_ISSUER=https://auth.arack.io
```

2. **Configure Kratos-Hydra Integration:**
   - Set up Hydra as OAuth provider
   - Configure Kratos login to use Hydra

3. **Update Email API:**
```rust
// Get OAuth token from Hydra
let token = hydra_client.get_token_for_user(kratos_id).await?;

// Use token for JMAP
let auth = JmapAuth::Bearer(token);
jmap_client.get_mailboxes(&auth).await
```

4. **Configure Stalwart OIDC:**
```toml
[directory."oidc"]
type = "oidc"
endpoint.url = "https://auth.arack.io/userinfo"
```

**Time:** 2-3 days
**Complexity:** High
**Benefit:** Standards-compliant OAuth flow

**Is it worth it?** Only if:
- You plan to add many more services
- You need OAuth for other integrations
- You have time for proper implementation

**For now:** Stick with master user, migrate to OIDC later if needed.

---

## ✅ **Conclusion**

**Research completed. Findings:**

1. ✅ Checked all Stalwart authentication backends
2. ✅ Explored OIDC, SQL, LDAP, HTTP options
3. ✅ Verified Kratos OIDC capabilities (needs Hydra)
4. ✅ Compared complexity vs benefits

**Result:** Master user is the simplest, most practical solution.

**Recommendation:** Proceed with [JMAP_AUTH_FIX_PLAN_V2.md](./JMAP_AUTH_FIX_PLAN_V2.md)

---

## 📚 **References**

**Stalwart Documentation:**
- [SQL Backend](https://stalw.art/docs/auth/backend/sql/)
- [OIDC Backend](https://stalw.art/docs/auth/backend/oidc/)
- [LDAP Backend](https://stalw.art/docs/auth/backend/ldap/)
- [Authentication Overview](https://stalw.art/docs/auth/authentication/overview/)
- [Webhooks](https://stalw.art/docs/api/webhooks/)

**Ory Documentation:**
- [Kratos vs Hydra](https://github.com/ory/hydra)
- [Kratos Social Sign-In](https://www.ory.com/docs/kratos/social-signin/generic)
- [Hydra OIDC Provider](https://github.com/ory/hydra)

**Industry Standards:**
- [OIDC Specification](https://openid.net/connect/)
- [OAuth 2.0 RFC](https://tools.ietf.org/html/rfc6749)

---

**Status:** ✅ Research Complete
**Recommendation:** Use Master User (JMAP_AUTH_FIX_PLAN_V2.md)
**Alternative:** Add Ory Hydra for OIDC (2-3 days work)
