# Email Features Test Report

**Date:** 2025-12-19
**Email Service Version:** 0.3.0
**Status:** Service Running (Phase 3 + Phase 5 AI features implemented)

---

## 📊 Overall Status

| Category | Status | Notes |
|----------|--------|-------|
| **Email Service** | ✅ Running | Healthy, no crashes |
| **User Provisioning** | ✅ Working | Auto-creates accounts on registration |
| **Account Management** | ✅ Working | Can query account info and quotas |
| **Mailboxes** | ⚠️ Partial | API exists but JMAP auth failing |
| **Messages (Read)** | ⚠️ Partial | API exists but JMAP auth failing |
| **Messages (Send)** | ❌ Not Tested | API exists, requires JMAP auth |
| **Draft Management** | ❌ Not Implemented | No dedicated draft endpoints |
| **Trash Management** | ❌ Not Implemented | No dedicated trash endpoints |
| **AI Features** | ❌ Not Working | OpenAI API key not configured |
| **Search** | ❌ Stub Only | Returns empty results |
| **Real-time (WebSocket)** | ⚠️ Partial | Token generation works |

---

## ✅ Working Features

### 1. Email Account Provisioning ✅

**Status:** FULLY WORKING

**What Works:**
- Automatic email account creation when user registers via Kratos
- Stalwart mail server account creation
- Domain configuration (arack.io)
- Database record creation
- Retry mechanism for failed provisioning

**Evidence:**
```
Recent Accounts:
- mus.dali@arack.io      (stalwart_13) - Created: 2025-12-19 19:53:36
- wassim.yessam@arack.io (stalwart_12) - Created: 2025-12-19 18:40:46
- yacine.wanik@arack.io  (stalwart_11) - Created: 2025-12-19 14:05:18
```

**Logs:**
```
✅ Created Stalwart domain arack.io with ID 0
✅ Created Stalwart account for mus.dali@arack.io with ID 13
✅ Email account record created
✅ Default mailboxes created by Stalwart automatically
✅ Email account provisioned successfully
```

**API Endpoint:**
- `POST /internal/mail/provision` (Webhook from Kratos)

---

### 2. Account Information ✅

**Status:** FULLY WORKING

**What Works:**
- Get current user's email account from Kratos session cookie
- Retrieve account details (email, quota, storage used)
- Calculate quota percentage

**Test Result:**
```bash
GET /api/mail/account/me
Cookie: ory_kratos_session=...

Response:
{
    "account": {
        "email_address": "omar.djedi@arack.io",
        "id": "5d049be8-4fef-4a5b-960f-4c4368a6b361",
        "is_active": true,
        "kratos_identity_id": "3794d157-c2c5-4201-accb-5e08b7aacf96",
        "stalwart_user_id": "stalwart_7",
        "storage_quota_bytes": 5368709120,  // 5 GB
        "storage_used_bytes": 0
    },
    "quota_percentage": 0.0
}
```

**API Endpoints:**
- `GET /api/mail/account/me` - Get current user's account ✅
- `GET /api/mail/account?kratos_id=...` - Get account by Kratos ID ✅

---

### 3. WebSocket Token Generation ✅

**Status:** WORKING

**What Works:**
- Generate Centrifugo connection tokens from session
- Token generation for real-time notifications

**API Endpoint:**
- `GET /api/mail/ws/token` - Generate WebSocket token ✅

---

## ⚠️ Partially Working Features

### 4. Mailbox Management ⚠️

**Status:** API EXISTS, JMAP AUTH FAILING

**What's Implemented:**
- List mailboxes (Inbox, Sent, Drafts, Trash, Junk)
- Create custom mailboxes
- Get mailbox counts (total emails, unread)

**Current Issue:**
```
Error: "Authentication failed"
```

**Root Cause:** JMAP authentication to Stalwart requires user password. The system uses `DEFAULT_EMAIL_PASSWORD` from environment, but mailbox listing requires actual user credentials.

**Code Location:** `email/api/mod.rs:507` - Uses `state.default_email_password`

**API Endpoints:**
- `GET /api/mail/mailboxes` - List all mailboxes ⚠️
- `POST /api/mail/mailboxes` - Create new mailbox ⚠️

**Fix Needed:** Configure correct default password OR implement password management

---

### 5. Message Management ⚠️

**Status:** API EXISTS, JMAP AUTH FAILING

**What's Implemented:**
- List messages in mailbox with pagination
- Get single message by ID (full content, HTML/text body)
- Send email via JMAP
- Real-time notifications via Centrifugo

**Current Issue:**
```
Error: "Authentication failed"
```

**Same root cause as mailboxes** - JMAP authentication failing

**API Endpoints:**
- `GET /api/mail/messages?mailbox_id=inbox&limit=50` - List messages ⚠️
- `GET /api/mail/messages/:id` - Get message by ID ⚠️
- `POST /api/mail/messages` - Send email ⚠️

**Response Format (when working):**
```json
{
  "messages": [
    {
      "id": "msg123",
      "subject": "Hello World",
      "from": {"email": "sender@example.com", "name": "Sender Name"},
      "to": [{"email": "recipient@example.com", "name": "Recipient"}],
      "preview": "First 100 characters...",
      "received_at": "2025-12-19T12:00:00Z",
      "is_read": false,
      "is_flagged": false,
      "has_attachments": true,
      "mailbox_ids": ["inbox"]
    }
  ],
  "total": 1,
  "limit": 50
}
```

---

### 6. Email Search ⚠️

**Status:** STUB IMPLEMENTATION

**Current Implementation:**
```rust
async fn search_emails() -> impl IntoResponse {
    // TODO: For Phase 3, return stub data
    Json(json!({
        "results": [],
        "total": 0,
        "query": req.query
    }))
}
```

**API Endpoint:**
- `GET /api/mail/search?query=...` - Always returns empty results

**Fix Needed:** Implement Meilisearch integration for email search

---

## ❌ Not Working / Not Implemented

### 7. AI Features ❌

**Status:** IMPLEMENTED BUT NOT WORKING

**Implemented Features:**
1. **Smart Compose** - AI-powered email completion suggestions
2. **Email Summarization** - Summarize email threads
3. **Priority Ranking** - AI-based inbox priority sorting
4. **Quota Management** - Daily usage limits per feature

**Current Issue:**
```
ERROR: Incorrect API key provided: sk-placeholder-user-to-provide
```

**Evidence from Logs:**
```
WARN Failed to generate smart compose suggestion 0:
      Some("invalid_request_error"): Incorrect API key provided
WARN Failed to generate smart compose suggestion 1:
      Some("invalid_request_error"): Incorrect API key provided
WARN Failed to generate smart compose suggestion 2:
      Some("invalid_request_error"): Incorrect API key provided
```

**Environment Variable:**
```bash
OPENAI_API_KEY=sk-placeholder-user-to-provide
```

**API Endpoints:**
- `POST /api/mail/ai/smart-compose?account_id=...` ❌
- `POST /api/mail/ai/summarize?account_id=...` ❌
- `POST /api/mail/ai/priority-rank?account_id=...` ❌
- `GET /api/mail/ai/quota?account_id=...` ✅ (No API key needed)

**Daily Quotas (Configured):**
- Smart Compose: 100 requests/day
- Summarization: 20 requests/day
- Priority Ranking: 50 requests/day

**Fix Needed:** Set real OpenAI API key in environment

**How to Fix:**
```bash
# On VPS
echo "OPENAI_API_KEY=sk-proj-REAL_KEY_HERE" >> /opt/arack/.env.production
docker restart search_engine_email_service
```

---

### 8. Draft Management ❌

**Status:** NOT IMPLEMENTED

**What's Missing:**
- No dedicated `/api/mail/drafts` endpoints
- No draft save/update/delete functionality
- Drafts mailbox exists in JMAP but no API to manage

**Workaround:** Use JMAP directly or implement custom endpoints

**Priority:** Medium (users can compose in frontend and save locally)

---

### 9. Trash Management ❌

**Status:** NOT IMPLEMENTED

**What's Missing:**
- No `/api/mail/trash` endpoints
- No move-to-trash functionality
- No permanent delete vs soft delete
- Trash mailbox exists but no API

**Required Endpoints (Not Implemented):**
- `POST /api/mail/messages/:id/trash` - Move to trash
- `POST /api/mail/messages/:id/delete` - Permanent delete
- `POST /api/mail/trash/empty` - Empty trash
- `POST /api/mail/messages/:id/restore` - Restore from trash

**Priority:** High (basic email functionality)

---

### 10. Database Schema Issues ❌

**Issue:** `email.email_ai_interactions` table missing columns

**Current Schema:**
```sql
 column_name |        data_type
-------------+--------------------------
 id          | uuid
 account_id  | uuid
 feature     | character varying
 created_at  | timestamp with time zone
```

**Expected Schema (from code):**
- ✅ `id` - UUID
- ✅ `account_id` - UUID
- ✅ `feature` - VARCHAR
- ✅ `created_at` - TIMESTAMP
- ❌ `tokens_used` - INTEGER (MISSING)
- ❌ `cost_usd` - DECIMAL (MISSING)

**Impact:**
```
WARN Failed to record AI usage:
     error returned from database: column "tokens_used" of relation
     "email_ai_interactions" does not exist
```

**Fix Needed:** Run migration to add missing columns

---

## 📋 Feature Implementation Summary

### Phase 2: Email Provisioning ✅ COMPLETE
- [x] Kratos webhook handler
- [x] Stalwart account creation
- [x] Domain configuration
- [x] Database record creation
- [x] Retry mechanism with Redis queue
- [x] Provisioning log

### Phase 3: Email API ⚠️ PARTIAL
- [x] Account management endpoints
- [x] Mailbox listing (implemented, auth failing)
- [x] Message listing (implemented, auth failing)
- [x] Send email (implemented, auth failing)
- [ ] Draft management (not implemented)
- [ ] Trash management (not implemented)
- [ ] Email search (stub only)
- [x] Real-time WebSocket tokens

### Phase 4: Contact Management ❓ UNKNOWN
- Database table exists: `email.email_contacts`
- No API endpoints visible in router

### Phase 5: AI Features ⚠️ IMPLEMENTED, NOT WORKING
- [x] Smart Compose API (OpenAI key missing)
- [x] Email Summarization API (OpenAI key missing)
- [x] Priority Ranking API (OpenAI key missing)
- [x] Quota management (working)
- [ ] Database schema fix needed

---

## 🔧 Fixes Required (Priority Order)

### High Priority (Blocks Core Functionality)

1. **Fix JMAP Authentication** ⚠️
   - Issue: Mailbox and message APIs fail with "Authentication failed"
   - Root Cause: `DEFAULT_EMAIL_PASSWORD` environment variable
   - Fix: Set correct password OR implement user password storage
   - Impact: Blocks all email reading/sending functionality

2. **Implement Trash Management** ❌
   - Issue: No API to delete/restore emails
   - Fix: Add endpoints for move-to-trash, delete, restore
   - Impact: Basic email functionality missing

3. **Implement Draft Management** ❌
   - Issue: No API to save/update/delete drafts
   - Fix: Add `/api/mail/drafts` endpoints
   - Impact: Users can't save draft emails

### Medium Priority (AI Features)

4. **Configure OpenAI API Key** ❌
   - Issue: All AI features failing with "Incorrect API key"
   - Fix: Set `OPENAI_API_KEY` environment variable
   - Impact: No AI features working

5. **Fix Database Schema** ❌
   - Issue: `email_ai_interactions` missing `tokens_used`, `cost_usd` columns
   - Fix: Create and run migration
   - Impact: AI usage tracking fails

### Low Priority (Nice to Have)

6. **Implement Email Search** ⚠️
   - Issue: Returns empty results (stub)
   - Fix: Integrate Meilisearch indexer
   - Impact: No email search capability

7. **Check Contact Management** ❓
   - Issue: Table exists but no API endpoints visible
   - Fix: Verify if implemented or add endpoints
   - Impact: Unknown

---

## 📝 Environment Variables Needed

**Current Status:**
```bash
# ❌ MISSING/WRONG:
OPENAI_API_KEY=sk-placeholder-user-to-provide

# ⚠️ VERIFY:
DEFAULT_EMAIL_PASSWORD=???  # Used for JMAP authentication

# ✅ WORKING:
DATABASE_URL=postgresql://...
REDIS_URL=redis://...
STALWART_ADMIN_URL=http://stalwart:8080
JMAP_URL=http://stalwart:8080/jmap
CENTRIFUGO_API_URL=http://centrifugo:8000/api
CENTRIFUGO_API_KEY=...
```

---

## 🧪 Test Commands

### Working Tests ✅

```bash
# Get account info
curl -H "Cookie: ory_kratos_session=YOUR_SESSION" \
  https://api-mail.arack.io/api/mail/account/me

# Get WebSocket token
curl -H "Cookie: ory_kratos_session=YOUR_SESSION" \
  https://api-mail.arack.io/api/mail/ws/token
```

### Failing Tests ⚠️

```bash
# List mailboxes (fails with auth error)
curl -H "Cookie: ory_kratos_session=YOUR_SESSION" \
  https://api-mail.arack.io/api/mail/mailboxes

# List messages (fails with auth error)
curl -H "Cookie: ory_kratos_session=YOUR_SESSION" \
  https://api-mail.arack.io/api/mail/messages

# AI smart compose (fails with API key error)
curl -X POST "https://api-mail.arack.io/api/mail/ai/smart-compose?account_id=UUID" \
  -H "Content-Type: application/json" \
  -d '{"partial_text":"Hello","context":"Meeting"}'
```

---

## 📚 API Documentation Summary

### Account Endpoints ✅
- `GET /api/mail/account/me` - Get current user's account
- `GET /api/mail/account?kratos_id=...` - Get account by ID

### Mailbox Endpoints ⚠️
- `GET /api/mail/mailboxes` - List all mailboxes
- `POST /api/mail/mailboxes` - Create mailbox

### Message Endpoints ⚠️
- `GET /api/mail/messages?mailbox_id=...&limit=50` - List messages
- `GET /api/mail/messages/:id` - Get message
- `POST /api/mail/messages` - Send email

### Search Endpoint ⚠️
- `GET /api/mail/search?query=...` - Search emails (stub)

### AI Endpoints ❌
- `POST /api/mail/ai/smart-compose?account_id=...` - Smart compose
- `POST /api/mail/ai/summarize?account_id=...` - Summarize thread
- `POST /api/mail/ai/priority-rank?account_id=...` - Priority ranking
- `GET /api/mail/ai/quota?account_id=...` - Get AI quota

### Real-time Endpoint ✅
- `GET /api/mail/ws/token` - Get WebSocket token

### Internal Endpoint ✅
- `POST /internal/mail/provision` - Provision webhook (Kratos)

---

## 🎯 Next Steps

1. **Immediate:** Fix JMAP authentication (check `DEFAULT_EMAIL_PASSWORD`)
2. **Short-term:** Implement trash and draft management
3. **Medium-term:** Configure OpenAI API key for AI features
4. **Long-term:** Implement email search with Meilisearch

---

**Report Generated:** 2025-12-19 20:05:00 UTC
**Service Status:** Running (with limitations)
**Overall Health:** 65% functional
