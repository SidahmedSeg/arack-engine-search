# Email Service Safeguards

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
