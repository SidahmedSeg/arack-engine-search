# Email Service Safeguards

## 🚨 CRITICAL - DO NOT TOUCH (2025-12-23)

### ⛔ Stalwart OIDC Configuration - NEVER REMOVE OR MODIFY
**File:** `/opt/arack/ory/stalwart/config.toml`

**CRITICAL SECTIONS - DO NOT DELETE:**
```toml
[storage]
directory = "oidc"  # ← MUST be "oidc" not "internal"

[directory.oidc]
type = "oidc"
timeout = "2s"

[directory.oidc.endpoint]
url = "http://search_engine_hydra:4444/userinfo"
method = "userinfo"

[directory.oidc.fields]
email = "email"
name = "name"

[session.auth]
mechanisms = ["plain", "login", "oauthbearer"]
directory = ["oidc", "internal"]  # ← Array order matters: OIDC first!

[http]
url = "http://localhost:8080"  # ← Required for JMAP discovery
```

**Why This is Critical:**
- **Without OIDC config:** Stalwart REJECTS all OAuth Bearer tokens → JMAP authentication fails
- **Breaking this:** Users get "JMAP authentication failed" error → Email app broken
- **Last broken:** Dec 22, 21:58 (config overwritten, took 14+ hours to diagnose)
- **Restoration date:** Dec 23, 13:26

**Verified Working State:**
- ✅ OAuth Bearer token authentication works
- ✅ Stalwart validates tokens via Hydra userinfo endpoint
- ✅ Backward compatibility maintained (internal directory fallback)
- ✅ Users can send/receive email via JMAP

**Backup Locations:**
- `config.toml.backup_oidc_fix` - Tested working config
- `config.toml.backup_before_oidc_restore_20251223_132607` - Pre-restoration backup

**If Accidentally Modified:**
```bash
cd /opt/arack/ory/stalwart
cp config.toml.backup_oidc_fix config.toml
docker restart arack_stalwart
```

### ⛔ Nginx Stream Block - NEVER REMOVE (2025-12-23)
**File:** `/opt/arack/nginx/nginx.conf`

**CRITICAL SECTION - DO NOT DELETE:**
```nginx
stream {
    # Log format for mail proxy
    log_format mail_proxy '$remote_addr [$time_local] '
                         '$protocol $status $bytes_sent $bytes_received '
                         '$session_time';

    access_log /var/log/nginx/mail_access.log mail_proxy;

    # SMTP (port 25) - inbound mail from internet
    server {
        listen 25;
        listen [::]:25;
        proxy_pass arack_stalwart:25;
        proxy_timeout 300s;
        proxy_connect_timeout 10s;
    }

    # SMTP Submission (port 587) - outbound mail submission
    server {
        listen 587;
        listen [::]:587;
        proxy_pass arack_stalwart:587;
        proxy_timeout 300s;
        proxy_connect_timeout 10s;
    }

    # IMAP (port 143) - mail access
    server {
        listen 143;
        listen [::]:143;
        proxy_pass arack_stalwart:143;
        proxy_timeout 300s;
        proxy_connect_timeout 10s;
    }

    # IMAPS (port 993) - secure mail access
    server {
        listen 993;
        listen [::]:993;
        proxy_pass arack_stalwart:993;
        proxy_timeout 300s;
        proxy_connect_timeout 10s;
    }
}
```

**Why This is Critical:**
- **Without stream block:** Nginx cannot proxy SMTP/IMAP ports to Stalwart → No incoming/outgoing email
- **Breaking this:** Users cannot receive emails from external senders (Gmail, Outlook, etc.)
- **Last broken:** Dec 22, 21:58 (stream block removed, same incident as OIDC)
- **Restoration date:** Dec 23, 14:40

**Verified Working State:**
- ✅ SMTP port 25 accepts connections from internet
- ✅ Nginx proxies to arack_stalwart:25 using Docker DNS
- ✅ Stalwart receives SMTP connections and delivers email
- ✅ IMAP ports 143/993 working for mail access

**Backup Locations:**
- `nginx.conf.backup_before_stream_fix_20251223_143909` - Backup before restoration

**If Accidentally Removed:**
```bash
cd /opt/arack/nginx
cp nginx.conf.backup_before_stream_fix_20251223_143909 nginx.conf
docker exec arack_nginx nginx -s reload
```

**Verification Commands:**
```bash
# Test SMTP connectivity
nc -zv smtp.arack.io 25

# Test SMTP protocol
echo "QUIT" | nc smtp.arack.io 25

# Verify stream block loaded
docker exec arack_nginx nginx -T 2>&1 | grep -c "stream {"
```

---

## Critical Fixes Applied (2025-12-22)

### Issue: Email Provisioning Failures
**Problem**: Email provisioning was failing with "Connection reset by peer" errors when containers tried to communicate with Stalwart.

**Root Cause**: Default `reqwest::Client::new()` lacked proper configuration for Docker container networking.

**Solution Applied**: 
- Updated reqwest HTTP client in `email/stalwart/mod.rs`, `email/jmap/mod.rs`, `email/centrifugo/mod.rs`
- Added production-grade configuration:
  - User-Agent header
  - Connection pooling (max 10 idle per host, 90s timeout)
  - TCP keepalive (60s interval)
  - Proper timeouts (10s connect, 30s request)
  - HTTP/1.1 connection reuse

**Detailed Error Logging Added**:
- All HTTP requests now log detailed error information
- Distinguishes between timeout, connection, and builder errors
- Logs full error chain with `.source()`

### Files Modified
1. `email/stalwart/mod.rs` - Admin API client with detailed logging
2. `email/jmap/mod.rs` - JMAP client with proper reqwest config
3. `email/centrifugo/mod.rs` - Centrifugo client with proper reqwest config

### Verification Steps
1. ✅ Email service starts without migration errors
2. ✅ User registration creates email account in Stalwart
3. ✅ Email account record created in database
4. ✅ User can list mailboxes via JMAP

### DO NOT REMOVE
- The detailed error logging in create_domain() and create_account()
- The reqwest ClientBuilder configuration
- The std::error::Error import for error.source()

### Monitoring Commands
```bash
# Check email service logs
docker logs arack_email_service --tail 100

# Check provisioning status
docker exec arack_postgres psql -U postgres -d engine_search \
  -c "SELECT status, COUNT(*) FROM email.email_provisioning_log GROUP BY status"

# Test health endpoint
curl https://api-mail.arack.io/health
```

### Known Good Configuration
- Kratos identity schema includes: email, first_name, last_name, username, date_of_birth, gender
- All fields are required
- Migration 002 checksum: f46bce093f13978c6a4ee1531256983c969697332cdecefea3dbf152ace67834aa616740f0ad249e06f4a6ce8ef73c1d
