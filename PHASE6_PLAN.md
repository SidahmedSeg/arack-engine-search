# Phase 6: Advanced Crawler Features

## Overview
Enhance the web crawler with production-ready features for respectful, efficient, and intelligent crawling.

## 🎯 Implementation Status

- ✅ **Phase 6.1** - Robots.txt Support (COMPLETED)
- ✅ **Phase 6.2** - Rate Limiting & Politeness (COMPLETED)
- ✅ **Phase 6.3** - User Agent & Headers (COMPLETED)
- ✅ **Phase 6.4** - Content Filtering (COMPLETED)
- ✅ **Phase 6.5** - URL Handling (COMPLETED)
- ✅ **Phase 6.6** - Error Handling & Retry (COMPLETED)
- ⏰ **Phase 6.7** - JavaScript Rendering (OPTIONAL)
- ✅ **Phase 6.8** - Crawl Scheduling (COMPLETED)
- ✅ **Phase 6.9** - Metrics (COMPLETED)
- ✅ **Phase 6.10** - UI (COMPLETED)

---

## Phase 6.1 - Robots.txt & Sitemap Support ✅ COMPLETED

### Goals
- Respect robots.txt directives
- Parse and follow sitemaps
- Implement crawl-delay from robots.txt
- Support sitemap index files

### Implementation
- **Files to Create:**
  - `src/crawler/robots.rs` - Robots.txt parser
  - `src/crawler/sitemap.rs` - Sitemap parser (XML & TXT)

- **Features:**
  - Parse robots.txt on first domain visit
  - Cache robots.txt rules per domain
  - Respect User-agent directives
  - Extract sitemap URLs from robots.txt
  - Parse XML sitemaps (standard & index)
  - Priority-based crawling from sitemap

- **Database Changes:**
  - Add `robots_txt` column to collections table
  - Track sitemap URLs and last crawl time

---

## Phase 6.2 - Rate Limiting & Politeness

### Goals
- Implement per-domain rate limiting
- Respect crawl-delay directives
- Add request throttling
- Domain-specific configuration

### Implementation
- **Files to Create:**
  - `src/crawler/rate_limiter.rs` - Token bucket rate limiter
  - `src/crawler/politeness.rs` - Crawl delay logic

- **Features:**
  - Token bucket algorithm per domain
  - Configurable requests per second
  - Automatic crawl-delay from robots.txt
  - Exponential backoff on errors
  - Concurrent request limits per domain
  - Global bandwidth throttling

- **Configuration:**
  ```env
  CRAWLER_REQUESTS_PER_SECOND=2
  CRAWLER_MIN_DELAY_MS=1000
  CRAWLER_MAX_RETRIES=3
  CRAWLER_TIMEOUT_SECONDS=30
  ```

---

## Phase 6.3 - User Agent & Headers Management

### Goals
- Professional user agent identification
- Configurable HTTP headers
- Contact information
- Browser emulation (optional)

### Implementation
- **Files to Update:**
  - `src/crawler/mod.rs` - Add header configuration

- **Features:**
  - Custom User-Agent with contact info
  - Accept-Language headers
  - Accept-Encoding (gzip, deflate)
  - Referer header management
  - Cookie handling (session-based)

- **Configuration:**
  ```env
  CRAWLER_USER_AGENT="EngineSearchBot/1.0 (+https://example.com/bot)"
  CRAWLER_CONTACT_EMAIL="bot@example.com"
  CRAWLER_ACCEPT_LANGUAGE="en-US,en;q=0.9"
  ```

---

## Phase 6.4 - Content Filtering & Validation ✅ COMPLETED

### Goals
- Filter by content type
- File size limits
- URL pattern filtering
- Duplicate detection

### Implementation
- **Files to Create:**
  - `src/crawler/filters.rs` - Content and URL filters
  - `src/crawler/dedup.rs` - Duplicate detection using bloom filters

- **Features:**
  - Content-Type whitelist (text/html, application/xml, etc.)
  - Max file size enforcement
  - URL blacklist/whitelist patterns
  - Domain whitelist/blacklist
  - Bloom filter for visited URLs
  - Content hash deduplication

- **Database Changes:**
  - Add `url_patterns` column to collections (JSON)
  - Track content hashes for deduplication

---

## Phase 6.5 - Advanced URL Handling

### Goals
- Smart URL normalization
- Query parameter handling
- Fragment removal
- Canonical URL support

### Implementation
- **Files to Create:**
  - `src/crawler/url_processor.rs` - URL normalization & canonicalization

- **Features:**
  - URL normalization (lowercase, trailing slash)
  - Query parameter filtering (remove tracking params)
  - Fragment identifier handling
  - Canonical link extraction
  - Base URL resolution
  - Relative URL resolution improvements

- **Configurable URL Normalization:**
  - Remove UTM parameters
  - Remove session IDs
  - Sort query parameters
  - Lowercase URLs
  - Remove default ports

---

## Phase 6.6 - Error Handling & Retry Logic

### Goals
- Sophisticated retry mechanisms
- Error categorization
- Exponential backoff
- Circuit breaker pattern

### Implementation
- **Files to Create:**
  - `src/crawler/retry.rs` - Retry logic with backoff
  - `src/crawler/circuit_breaker.rs` - Circuit breaker for failing domains

- **Features:**
  - Exponential backoff (base: 1s, max: 60s)
  - Retry on: 408, 429, 500, 502, 503, 504
  - Circuit breaker states: Closed → Open → Half-Open
  - Domain-level circuit breakers
  - Error tracking in database
  - Automatic recovery

- **Database Changes:**
  - Track retry attempts in crawl_errors
  - Add circuit breaker state to domain tracking

---

## Phase 6.7 - JavaScript Rendering (Optional)

### Goals
- Render JavaScript-heavy sites
- Support SPA frameworks
- Dynamic content extraction

### Implementation
- **Dependencies:**
  - Consider: headless_chrome or playwright-rs

- **Features:**
  - Optional JS rendering per URL
  - Wait for network idle
  - Screenshot capture (optional)
  - DOM snapshot after JS execution
  - Configurable render timeout
  - Fallback to static HTML

- **Configuration:**
  ```env
  CRAWLER_ENABLE_JS=false
  CRAWLER_JS_TIMEOUT_MS=10000
  CRAWLER_RENDER_THREADS=2
  ```

⚠️ **Note:** This is resource-intensive. Consider as optional/premium feature.

---

## Phase 6.8 - Crawl Scheduling & Recrawling

### Goals
- Scheduled recrawls
  - Automatic re-indexing of stale content
- Priority-based crawling
- Freshness tracking

### Implementation
- **Files to Create:**
  - `src/crawler/scheduler.rs` - Crawl scheduling logic

- **Features:**
  - Page freshness score based on:
    - Last modified header
    - Content change frequency
    - Page importance (backlinks)
  - Automatic recrawl scheduling
  - Priority queue for important pages
  - TTL-based expiration
  - Incremental crawling

- **Database Changes:**
  - Add `last_crawled_at`, `next_crawl_at` to pages table
  - Add `crawl_frequency` (hourly, daily, weekly, monthly)
  - Add `priority_score` column

---

## Phase 6.9 - Advanced Crawler Metrics

### Goals
- Detailed crawl statistics
- Performance monitoring
- Success/failure tracking
- Domain health metrics

### Implementation
- **Files to Create:**
  - `src/crawler/metrics.rs` - Metrics collection

- **Features:**
  - Requests per second (actual)
  - Average response time
  - Success/failure rates
  - Bandwidth usage
  - Pages per hour
  - Domain-specific metrics
  - Error rate trends

- **Dashboard Endpoints:**
  - `GET /api/crawler/metrics` - Current metrics
  - `GET /api/crawler/domains` - Per-domain stats
  - `GET /api/crawler/health` - Overall health

---

## Phase 6.10 - Crawler Configuration UI

### Goals
- Web-based crawler configuration
- Collection management
- Live crawler monitoring

### Implementation
- **Frontend Changes:**
  - Add crawler settings page
  - Domain whitelist/blacklist UI
  - Rate limit configuration
  - Crawl schedule management
  - Live metrics dashboard

- **Features:**
  - Visual crawl configuration
  - Domain-specific settings
  - Crawl job templates
  - Historical metrics charts
  - Real-time crawler status

---

## Implementation Order

### Quick Wins (Start Here)
1. **Phase 6.2** - Rate Limiting & Politeness (Essential)
2. **Phase 6.3** - User Agent & Headers (Simple & Important)
3. **Phase 6.1** - Robots.txt Support (Ethical requirement)

### Core Features
4. **Phase 6.4** - Content Filtering
5. **Phase 6.5** - URL Handling
6. **Phase 6.6** - Error Handling & Retry

### Advanced Features
7. **Phase 6.8** - Crawl Scheduling
8. **Phase 6.9** - Metrics & Monitoring
9. **Phase 6.1** - Sitemap Support (Complete)

### Optional/Premium
10. **Phase 6.7** - JavaScript Rendering (Resource intensive)
11. **Phase 6.10** - Configuration UI (Nice to have)

---

## Success Metrics

- ✅ Respect robots.txt 100% of the time
- ✅ Configurable rate limiting per domain
- ✅ Professional user agent identification
- ✅ Duplicate detection preventing redundant crawls
- ✅ Automatic retry with exponential backoff
- ✅ Circuit breaker protecting failing domains
- ✅ Detailed metrics for monitoring
- ✅ Crawl scheduling for freshness

---

## Breaking Changes

### API Changes
- Crawl endpoint accepts additional parameters:
  ```json
  {
    "urls": ["https://example.com"],
    "max_depth": 2,
    "respect_robots_txt": true,
    "follow_sitemaps": true,
    "rate_limit": 2,
    "enable_js": false,
    "url_patterns": {
      "include": ["^https://example\\.com/blog/.*"],
      "exclude": [".*\\.pdf$"]
    }
  }
  ```

### Configuration Changes
- New environment variables (see each phase)
- Database schema changes (migrations provided)

---

## Timeline Estimate

- **Phase 6.1**: 2-3 hours (Robots.txt & basic sitemap)
- **Phase 6.2**: 2-3 hours (Rate limiting)
- **Phase 6.3**: 1 hour (Headers)
- **Phase 6.4**: 2 hours (Filtering)
- **Phase 6.5**: 2 hours (URL handling)
- **Phase 6.6**: 3 hours (Retry logic)
- **Phase 6.7**: 4-6 hours (JS rendering - optional)
- **Phase 6.8**: 3 hours (Scheduling)
- **Phase 6.9**: 2 hours (Metrics)
- **Phase 6.10**: 4 hours (UI)

**Total (without JS & UI)**: ~14-16 hours
**Total (complete)**: ~22-28 hours

---

## Dependencies to Add

```toml
# Rate limiting
governor = "0.6"
dashmap = "5.5"

# Robots.txt parsing
robotparser = "0.13"

# Sitemap parsing
quick-xml = "0.31"

# URL normalization
url = "2.5"  # Already have

# Duplicate detection
probabilistic-collections = "0.7"  # Bloom filters

# Retry logic
backoff = "0.4"

# JS Rendering (optional)
# headless_chrome = "1.0"  # Uncomment if implementing 6.7
```

---

## Notes

- Start with essentials (6.1, 6.2, 6.3) for respectful crawling
- Add filtering and retry logic (6.4, 6.6) for robustness
- Consider JS rendering (6.7) only if needed for SPA sites
- UI (6.10) can be built incrementally alongside other phases

Ready to implement! Which phase would you like to start with?
