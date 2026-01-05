# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## 🚨 RED LINES - NEVER CROSS THESE

### ⛔ CRITICAL: Custom SSO System
**📖 Full Documentation:** See `CUSTOM_SSO_SYSTEM.md` for complete details.

**System Overview:**
- **Account Service:** `account.arack.io:3002` (Go)
- **Cookie:** `arack_session` on `.arack.io` domain
- **Tokens:** RS256 JWT (access + refresh)
- **Session:** Redis with 30-day TTL

---

### ⛔ CRITICAL: SSO Cookie Configuration
**File:** `account-service/internal/config/config.go`

**ABSOLUTE RED LINES - DO NOT CHANGE:**
```go
COOKIE_NAME=arack_session      // All apps depend on this exact name
COOKIE_DOMAIN=.arack.io        // Required for cross-subdomain SSO
```

**Why:** Changing cookie name or domain breaks SSO across all subdomains.
- Impact: Users cannot login to mail.arack.io, admin.arack.io
- All frontends check for `arack_session` cookie

---

### ⛔ CRITICAL: JWT Issuer/Audience
**File:** `account-service/internal/config/config.go`

**NEVER CHANGE:**
```go
JWT_ISSUER=https://account.arack.io
JWT_AUDIENCE=https://arack.io
```

**Why:** Stalwart validates these claims for email authentication.

---

### ⛔ CRITICAL: RSA Keys
**Location:** `/app/keys/private.pem`, `/app/keys/public.pem` (Docker volume)

**Why:** These keys sign ALL tokens. Losing them invalidates all sessions.
**Recovery:** If lost, all users must re-login.

---

### ⛔ CRITICAL: Stalwart OIDC Configuration
**File:** `/opt/arack/ory/stalwart/config.toml` (VPS Production)

**ABSOLUTE RED LINES - DO NOT MODIFY:**

```toml
[storage]
directory = "oidc"  # MUST be "oidc", NOT "internal"

[directory.oidc]
type = "oidc"
userinfo-url = "https://account.arack.io/userinfo"  # Custom SSO endpoint
```

**Why:** Without these, email OAuth authentication COMPLETELY BREAKS
- Symptom: "JMAP authentication failed. Your OAuth token may be invalid."
- Impact: Email app unusable, users cannot access mailboxes

**Recovery if broken:**
```bash
ssh root@213.199.59.206
cd /opt/arack/ory/stalwart
cp config.toml.backup_oidc_fix config.toml
docker restart arack_stalwart
```

**Safe operations:**
- ✅ Reading the config
- ✅ Adding new sections (if documented)
- ✅ Modifying logging/storage paths
- ❌ Removing OIDC sections
- ❌ Changing directory from "oidc" to "internal"
- ❌ Changing userinfo-url to anything other than account.arack.io

---

### ⛔ CRITICAL: JMAP Authentication Method
**Files:** `email/api/mod.rs` - `get_jmap_session()` function

**RED LINE:** NEVER change from OAuth Bearer tokens to Basic Auth
```rust
// ✅ CORRECT - OAuth Bearer tokens
let auth = JmapAuth::Bearer(access_token);

// ❌ WRONG - Basic Auth (security regression)
let auth = JmapAuth::Basic { username, password };
```

**Why:**
- Basic Auth = shared password for all users (security vulnerability)
- OAuth = individual tokens, auto-refresh, revocable (industry standard)

**If JMAP auth fails:** Problem is in Stalwart OIDC config, NOT in this code

---

### ⛔ CRITICAL: Reqwest HTTP Client Configuration
**Files:** `email/stalwart/mod.rs`, `email/jmap/mod.rs`, `email/centrifugo/mod.rs`

**RED LINES - DO NOT REMOVE:**
```rust
use std::error::Error;  // ← Required for .source()

let client = Client::builder()
    .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .tcp_keepalive(Duration::from_secs(60))
    .connect_timeout(Duration::from_secs(10))
    .timeout(Duration::from_secs(30))
    .http1_title_case_headers()
    .build()
    .expect("Failed to build HTTP client");
```

**Why:** Docker container networking requires this configuration
- Without it: "Connection reset by peer" errors
- Impact: Email provisioning fails, no accounts created

---

### ⛔ CRITICAL: Nginx Stream Block for Email Delivery
**File:** `/opt/arack/nginx/nginx.conf` (VPS Production)

**ABSOLUTE RED LINE - DO NOT REMOVE:**

The stream block at the end of nginx.conf MUST contain SMTP/IMAP proxy configuration:

```nginx
stream {
    # Log format for mail proxy
    log_format mail_proxy '$remote_addr [$time_local] '
                         '$protocol $status $bytes_sent $bytes_received '
                         '$session_time';

    access_log /var/log/nginx/mail_access.log mail_proxy;

    # SMTP (port 25) - inbound mail
    server {
        listen 25;
        listen [::]:25;
        proxy_pass arack_stalwart:25;
        proxy_timeout 300s;
        proxy_connect_timeout 10s;
    }

    # SMTP Submission (port 587), IMAP (143), IMAPS (993)
    # ... (all 4 server blocks required)
}
```

**Why:** Without stream block, ALL email delivery COMPLETELY BREAKS
- Symptom: No incoming emails from Gmail/external senders
- Impact: SMTP port 25 not proxied to Stalwart → emails fail silently
- Last incident: Dec 22-23, 2025 (removed same time as OIDC config)
- Restoration: Dec 23, 14:40
- Verified working: Dec 23, 14:45 (multiple test emails received)

**Recovery if broken:**
```bash
ssh root@213.199.59.206
cd /opt/arack/nginx
cp nginx.conf.backup_before_stream_fix_20251223_143909 nginx.conf
docker exec arack_nginx nginx -s reload
```

**Verification commands:**
```bash
# Test SMTP connectivity
nc -zv smtp.arack.io 25

# Verify stream block loaded
docker exec arack_nginx nginx -T 2>&1 | grep -c "stream {"
```

**Full details:** See `EMAIL_SERVICE_SAFEGUARDS.md` and `EMAIL_DELIVERY_FIX_COMPLETE.md`

---

### ⛔ CRITICAL: Frontend hooks.server.ts Cookie Check
**File:** `frontend-email/src/hooks.server.ts`

**MUST match account-service cookie name:**
```typescript
const sessionCookie = cookies.get('arack_session');  // MUST be 'arack_session'
```

**Why:** Mismatch causes authentication failures in email app.

---

### ⛔ CRITICAL: Account Service OIDC Endpoints
**File:** `account-service/internal/handler/oauth.go`

**These endpoints MUST exist (Stalwart depends on them):**

| Endpoint | Purpose |
|----------|---------|
| `/.well-known/jwks.json` | Public keys for JWT validation |
| `/.well-known/openid-configuration` | OIDC discovery document |
| `/userinfo` | User info for OAuth clients |

**Why:** Stalwart and email authentication depend on these exact paths.

---

## Project Overview

This is a full-stack web search engine built with Rust (backend) and SvelteKit (frontends). The system crawls websites, indexes content using Meilisearch, and provides search functionality through multiple interfaces.

## Architecture

### Backend (Rust)
**Entry Point:** `src/main.rs`

The backend is a production-grade web crawler and search API with the following components:

**Core Modules:**
- `api/` - Axum REST API server with CORS support and authentication
- `auth/` - **NEW (Phase 8.1)** - axum-login based authentication system
- `analytics/` - Search analytics and click tracking
- `crawler/` - Production web crawler with 10 submodules (details below)
- `search/` - Meilisearch client integration with autocomplete
- `db/` - PostgreSQL database layer with SQLx
- `redis/` - Redis-based job queue and caching
- `scheduler/` - Tokio cron scheduler for background jobs
- `worker/` - Background worker pool for async crawling
- `config/` - Environment-based configuration
- `types.rs` - Shared type definitions

**Crawler Architecture (`src/crawler/`):**
The crawler is the most complex module with production-ready features:
- `mod.rs` - Main crawler orchestration and URL processing
- `robots.rs` - Robots.txt parsing and compliance
- `rate_limiter.rs` - Token bucket rate limiting per domain
- `politeness.rs` - Crawl delay enforcement (from robots.txt or config)
- `headers.rs` - HTTP header management (User-Agent, Accept-Language, etc.)
- `filters.rs` - Content-Type and file size filtering
- `url_processor.rs` - URL normalization and canonicalization
- `retry.rs` - Exponential backoff retry logic
- `circuit_breaker.rs` - Domain-level circuit breaker pattern (Closed/Open/HalfOpen states)
- `scheduler.rs` - Priority-based crawl scheduling with freshness tracking

**Authentication Architecture (`src/auth/`):**
The authentication system (Phase 8.1) uses axum-login with session-based auth:
- `mod.rs` - Backend trait implementation for axum-login
- `models.rs` - User, Credentials, RegisterRequest types
- `password.rs` - Argon2id password hashing
- `repository.rs` - User CRUD operations in PostgreSQL
- Session storage: PostgreSQL-backed via tower-sessions-sqlx-store
- Session expiry: 7 days on inactivity
- Password security: Argon2id (OWASP recommended)

**Database Layer:**
- Uses SQLx for type-safe SQL queries
- Migrations in `migrations/` directory (run automatically on startup)
- Connection pooling with PgPool
- Stores: collections, pages, crawl errors, **users, invitations, sessions** (Phase 8)

**Job Processing Flow:**
1. HTTP request → API endpoint (`POST /api/crawl`)
2. Job enqueued to Redis (`redis/queue.rs`)
3. Background worker picks up job (`worker/mod.rs`)
4. Crawler processes URLs with rate limiting and circuit breakers
5. Content indexed to Meilisearch
6. Job status updated in Redis

### Frontend Architecture

**Three Separate SvelteKit Apps:**

1. **frontend-admin/** (Port 5000)
   - Admin dashboard for crawl management
   - Routes:
     - `/crawl` - Start new crawl jobs
     - `/crawler-metrics` - Real-time crawler monitoring dashboard
   - Uses shared API client from `shared/api-client/`

2. **frontend-search/** (Port 5001)
   - Public search interface
   - Clean, minimal search UI

3. **frontend/** (Legacy)
   - Original frontend, being deprecated

**Shared Code:**
- `shared/` - TypeScript monorepo package
  - `api-client/index.ts` - SearchEngineAPI class (axios-based)
  - `types/index.ts` - Shared TypeScript types
  - Path alias: `$shared` → `../shared` (via vite.config.ts)

### Frontend UI Components

**Custom Component System (frontend-search):**

The search frontend uses a **manual component system** inspired by shadcn-svelte patterns. This approach was chosen because:
- All modern Svelte UI libraries (shadcn-svelte, Skeleton UI, Flowbite) require Tailwind v4
- Project uses Tailwind v3 (Tailwind v4 requires Vite 5-6, but we use Vite 7)
- Downgrading Vite would break existing functionality
- Manual components provide exact control over design (compact, modern, minimal Google-like aesthetic)

**📖 Complete Guide:** See [`frontend-search/CUSTOM_COMPONENTS_GUIDE.md`](frontend-search/CUSTOM_COMPONENTS_GUIDE.md) for:
- Component creation patterns (simple, form, compound, interactive)
- Design system specifications (colors, spacing, typography, focus states)
- Step-by-step guides with complete code examples
- Troubleshooting and best practices
- Migration examples from old to new components

**Existing Components** (`frontend-search/src/lib/components/ui/`):
- **Button** - Multiple variants (default, destructive, outline, secondary, ghost, link), sizes (default, sm, lg, icon)
- **Input** - Form input with label, error display, two-way binding via `$bindable`
- **Label** - Accessible form labels with required indicator
- **Card** - Compound component (Card.Root, Card.Header, Card.Title, Card.Description, Card.Content)
- **OTP Input** - 6-digit code input with auto-advance, paste support, keyboard navigation

**Key Utilities:**
- `cn()` function (`src/lib/utils.ts`) - Merges Tailwind classes with conflict resolution (clsx + tailwind-merge)
- `components.json` - Component configuration (style: default, baseColor: neutral)

**Design Philosophy:**
- **Compact spacing** - h-9 inputs/buttons (vs h-10/h-11 in typical libraries)
- **Minimal aesthetic** - Clean, Google-like interface
- **Svelte 5 patterns** - Uses runes (`$state`, `$bindable`, `$props`, `$effect`), snippets for children
- **TypeScript props** - Strongly typed interfaces for all components
- **Accessibility** - ARIA labels, focus states, keyboard navigation

**When to create new components:** See the guide's "Creating New Components" section for step-by-step instructions.

## Development Commands

### Backend

```bash
# Run in development mode (with logging)
cargo run

# Run in release mode (optimized)
cargo run --release

# Run tests (76 tests across crawler modules)
cargo test

# Run specific test
cargo test test_name

# Check code without running
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

### Frontend

```bash
# Admin dashboard (port 5000)
cd frontend-admin && npm run dev

# Public search (port 5001)
cd frontend-search && npm run dev

# Build for production
npm run build

# Run production build
npm run preview

# Type checking
npm run check
```

### Infrastructure

The application requires:
- **PostgreSQL** (default: localhost:5434)
- **Redis** (default: localhost:6379)
- **Meilisearch** (default: localhost:7700)

## Configuration

Create `.env` file in root (see `.env.example`):

```bash
# Backend runs on 127.0.0.1:3000
SERVER_HOST=127.0.0.1
SERVER_PORT=3000

# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5434/engine_search
REDIS_URL=redis://127.0.0.1:6379

# Meilisearch
MEILISEARCH_URL=http://127.0.0.1:7700
MEILISEARCH_KEY=masterKey

# Crawler behavior
CRAWLER_MAX_DEPTH=3
CRAWLER_MAX_CONCURRENT=10
CRAWLER_REQUESTS_PER_SECOND=2
CRAWLER_MIN_DELAY_MS=1000
CRAWLER_USER_AGENT=EngineSearchBot/1.0
CRAWLER_ACCEPT_LANGUAGE=en-US,en;q=0.9
```

Frontend apps have separate `.env` files for API URLs.

## API Endpoints

**Authentication (Phase 8.1):**
- `POST /api/auth/register` - User self-registration (email, password, first_name, last_name)
- `POST /api/auth/login` - Email/password login (creates session)
- `POST /api/auth/logout` - Logout (destroys session)
- `GET /api/auth/me` - Get current user info (requires auth)

**Search:**
- `GET /api/search?q=query&limit=10&offset=0` - Search indexed content
- `GET /api/search/autocomplete?q=prefix&limit=5` - Autocomplete suggestions (Phase 7.1)

**Analytics (Phase 7.6-7.8):**
- `GET /api/analytics/summary?days=7` - Search analytics summary
- `POST /api/analytics/click` - Track result clicks

**Crawling:**
- `POST /api/crawl` - Start crawl job (returns job_id)
- `GET /api/jobs/:job_id` - Get job status

**Crawler Metrics (Phase 6.10):**
- `GET /api/crawler/metrics` - Comprehensive crawler stats
- `GET /api/crawler/domains` - Per-domain circuit breaker states
- `GET /api/crawler/scheduler` - Scheduled crawl tasks

**Index Management:**
- `GET /api/stats` - Meilisearch statistics
- `DELETE /api/index` - Clear entire index

**Health:**
- `GET /health` - Health check

## Important Implementation Details

### Crawler Rate Limiting
The crawler uses a sophisticated multi-layer approach:
1. **Per-domain rate limiting** (token bucket algorithm via Governor)
2. **Politeness delays** (respects robots.txt `Crawl-delay`)
3. **Circuit breakers** (protects failing domains with 3 states)
4. **Retry logic** (exponential backoff for 408, 429, 500, 502, 503, 504)

### Async Architecture
- Uses Tokio runtime with `tokio::spawn` for background workers
- Workers process jobs concurrently from Redis queue
- SQLx connection pool shared across workers
- Arc<Crawler> clones distributed to workers for thread-safe access

### Frontend API Integration
Frontends use a shared API client (`SearchEngineAPI` class) that:
- Handles success/error responses uniformly
- Returns typed data via `ApiResponse<T>` wrapper
- Throws errors on API failures for clean error handling

### Database Migrations

**How Migrations Work in This Project:**

Migrations run **automatically** on every server startup via `db::run_migrations()` in `main.rs`. This means:
1. No manual migration commands needed
2. Safe to restart server - already-applied migrations are skipped
3. New migrations are applied in sequential order (by filename)

**Migration Files Location:** `migrations/*.sql`

**Current Migrations:**
- `001_create_collections.sql` - Collections, crawl history, errors
- `002_create_auth_tables.sql` - Users, invitations, sessions (Phase 8)

**To Add a New Migration:**
1. Create a new file in `migrations/` with sequential numbering:
   - Example: `003_add_user_preferences.sql`
2. Write SQL with `CREATE TABLE IF NOT EXISTS` for safety
3. Add indexes after table creation
4. **No rollback needed** - SQLx migration tracking prevents re-running
5. Restart backend - migration runs automatically on startup

**Migration Tracking:**
- SQLx creates `_sqlx_migrations` table automatically
- Tracks which migrations have been applied (by checksum)
- Prevents duplicate runs even if server restarted

**Example Migration:**
```sql
-- migrations/003_add_feature.sql
CREATE TABLE IF NOT EXISTS new_feature (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_new_feature_name ON new_feature(name);
```

**Testing Migrations:**
- Start the backend: `cargo run`
- Check logs for `Database migrations completed successfully`
- Migrations are idempotent (safe to run multiple times due to `IF NOT EXISTS`)

**Manual Migration Check (Optional):**
```bash
# Check which migrations have been applied
psql -d engine_search -c "SELECT * FROM _sqlx_migrations ORDER BY version;"
```

## Testing

The crawler has comprehensive test coverage (76 tests):
- Unit tests in each module (at bottom of `.rs` files)
- Tests use `#[tokio::test]` for async tests
- Circuit breaker tests verify state transitions
- Retry tests use `Arc<Mutex<T>>` pattern for shared state in closures

## Phase Implementation Guide

This project follows a phase-based development approach documented in `PHASE*_PLAN.md` files:

- **Phase 6** (Advanced Crawler Features) - ✅ **COMPLETED**
  - 6.1-6.6: Core features (robots.txt, rate limiting, headers, filtering, URL handling, error handling)
  - 6.8-6.9: Scheduling and metrics
  - 6.10: Crawler metrics UI dashboard
  - 6.7: JavaScript rendering (OPTIONAL - not implemented)

- **Phase 7** (Advanced Search Features) - ✅ **COMPLETED**
  - 7.1: Search autocomplete with debouncing and keyboard navigation
  - 7.2: Search suggestions (zero-result query suggestions)
  - 7.4: Faceted search (domain-based filtering)
  - 7.5: Meilisearch optimization (synonyms, stop words, typo tolerance)
  - 7.6-7.7: Search analytics and click tracking
  - 7.8: Analytics dashboard UI

- **Phase 8** (Authentication & Security) - 🚧 **IN PROGRESS**
  - 8.1: User registration & login with axum-login - ✅ **COMPLETED**
  - 8.3: Admin invite system - 🔄 **NEXT**
  - 8.4: Admin user management
  - 8.5: Frontend login/register UI
  - 8.6: Frontend admin user management UI
  - 8.7-8.8: Security enhancements and testing

**When completing a phase:** Update the corresponding `PHASE*_PLAN.md` file to mark features as ✅ COMPLETED.

## Code Conventions

### Rust
- Use `anyhow::Result<T>` for error handling
- Prefer `tracing::info!` over `println!`
- Clone-on-write pattern with `Arc<T>` for shared state
- Async functions return `impl Future` or explicit `Result<T>`

### Frontend
- SvelteKit 5 with Svelte Runes (`$state`, `$derived`, `$effect`)
- TypeScript strict mode enabled
- Tailwind CSS for styling
- Lucide Svelte for icons

### API Client Usage
```typescript
import { api } from '$lib/stores/api';

const results = await api.search({ q: 'query', limit: 10, offset: 0 });
const metrics = await api.getCrawlerMetrics();
```

## Common Gotchas

1. **Crawler in API state is not used for crawling** - It's only for compatibility. Actual crawling happens in background workers.

2. **Circuit breaker states persist in memory** - They're not stored in database, reset on restart.

3. **Frontend path aliases** - `$shared` is configured in `vite.config.ts`, not `tsconfig.json`.

4. **Postgres port** - Default is 5434 (not standard 5432) to avoid conflicts.

5. **Arc<Mutex<T>> in async tests** - Required pattern for captured variables in async closures that outlive the function.
- always refere to Claude.md