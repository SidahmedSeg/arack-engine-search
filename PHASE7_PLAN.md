# Phase 7: Search Features, Optimization & Analytics

## Overview
Enhance search functionality with advanced features, optimize Meilisearch configuration, and implement analytics to understand user behavior and improve search quality.

## 🎯 Implementation Status

- ✅ **Phase 7.1** - Search Autocomplete (COMPLETED)
- ✅ **Phase 7.2** - Search Suggestions (COMPLETED)
- ✅ **Phase 7.3** - Highlighting & Snippets (COMPLETED)
- ⏰ **Phase 7.4** - Faceted Search (PENDING)
- ⏰ **Phase 7.5** - Meilisearch Optimization (PENDING)
- ⏰ **Phase 7.6** - Search Analytics (PENDING)
- ⏰ **Phase 7.7** - Click Tracking (PENDING)
- ⏰ **Phase 7.8** - Popular Queries Dashboard (PENDING)

---

## Phase 7.1 - Search Autocomplete

### Goals
- Real-time query suggestions as user types
- Fast autocomplete responses
- Cache popular suggestions
- Support prefix matching

### Implementation
- **Files to Create:**
  - `src/search/autocomplete.rs` - Autocomplete logic and caching

- **Features:**
  - Prefix-based query suggestions
  - Limit suggestions (default: 5)
  - Cache frequently searched queries in Redis
  - Debounced requests from frontend
  - Return query + document count

- **API Changes:**
  - `GET /api/search/autocomplete?q=prefix&limit=5`

- **Response Format:**
  ```json
  {
    "suggestions": [
      { "query": "rust programming", "count": 142 },
      { "query": "rust tutorial", "count": 89 }
    ],
    "processing_time_ms": 5
  }
  ```

---

## Phase 7.2 - Search Suggestions (Did You Mean?)

### Goals
- Suggest alternative queries for misspelled searches
- Show "Did you mean?" prompts
- Learn from user corrections

### Implementation
- **Files to Update:**
  - `src/search/mod.rs` - Add suggestion logic

- **Features:**
  - Use Meilisearch typo tolerance
  - Return alternative queries if:
    - Original query returns 0 results
    - Typos detected in query
  - Track which suggestions users click
  - Store common misspellings → corrections mapping

- **Database Changes:**
  ```sql
  CREATE TABLE search_corrections (
    id SERIAL PRIMARY KEY,
    original_query TEXT NOT NULL,
    corrected_query TEXT NOT NULL,
    accepted_count INT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```

- **API Response Enhancement:**
  ```json
  {
    "hits": [...],
    "query": "rustlang",
    "suggestions": ["rust lang", "rust language"],
    "total_hits": 0
  }
  ```

---

## Phase 7.3 - Highlighting & Snippets

### Goals
- Highlight matching terms in search results
- Show relevant text snippets
- Customize highlight tags

### Implementation
- **Files to Update:**
  - `src/search/mod.rs` - Enable highlighting

- **Features:**
  - Enable Meilisearch highlighting
  - Highlight in: title, content, description
  - Configure highlight tags: `<mark>` or custom
  - Return formatted snippets (200 chars around match)
  - Crop long content intelligently

- **Meilisearch Configuration:**
  ```rust
  search
      .with_show_matches_position(true)
      .with_attributes_to_highlight(&["title", "content", "description"])
      .with_highlight_pre_tag("<mark>")
      .with_highlight_post_tag("</mark>")
      .with_crop_length(200)
  ```

- **Response Enhancement:**
  ```json
  {
    "id": "doc123",
    "title": "Rust Programming Guide",
    "_formatted": {
      "title": "<mark>Rust</mark> Programming Guide",
      "content": "Learn <mark>Rust</mark> programming..."
    }
  }
  ```

---

## Phase 7.4 - Faceted Search (Filters)

### Goals
- Add domain filtering
- Filter by content type
- Filter by date ranges
- Filter by word count ranges

### Implementation
- **Files to Update:**
  - `src/search/mod.rs` - Add facet support
  - `src/crawler/mod.rs` - Track domain and content_type

- **Features:**
  - Extract and index domain from URL
  - Track content MIME type
  - Add facetable attributes:
    - `domain`
    - `content_type`
    - `crawled_at` (already sortable)
    - `word_count` (already sortable)
  - Return facet counts
  - Support multiple filter combinations

- **Database Changes:**
  - Add `domain` column to documents
  - Add `content_type` column to documents

- **Meilisearch Configuration:**
  ```rust
  index.set_filterable_attributes(&[
      "domain",
      "content_type",
      "crawled_at",
      "word_count"
  ]).await?;
  ```

- **API Enhancement:**
  - `GET /api/search?q=query&filter=domain:example.com AND word_count >= 100`
  - `GET /api/search/facets?q=query` - Returns facet distributions

- **Response Enhancement:**
  ```json
  {
    "hits": [...],
    "facets": {
      "domain": {
        "example.com": 45,
        "test.org": 23
      },
      "content_type": {
        "text/html": 120,
        "application/pdf": 12
      }
    }
  }
  ```

---

## Phase 7.5 - Meilisearch Optimization

### Goals
- Configure synonyms
- Set up stop words
- Tune typo tolerance
- Optimize ranking rules
- Configure separators and dictionary

### Implementation
- **Files to Update:**
  - `src/search/mod.rs` - Add advanced configuration

- **Features:**

  **A) Synonyms:**
  ```rust
  index.set_synonyms(&[
      ("js", vec!["javascript"]),
      ("ts", vec!["typescript"]),
      ("py", vec!["python"]),
      ("golang", vec!["go", "go lang"]),
      ("rust", vec!["rustlang", "rust-lang"])
  ]).await?;
  ```

  **B) Stop Words:**
  ```rust
  index.set_stop_words(&[
      "the", "a", "an", "is", "are", "was", "were",
      "in", "on", "at", "to", "for", "of", "with"
  ]).await?;
  ```

  **C) Typo Tolerance:**
  ```rust
  index.set_typo_tolerance(&TypoSettings {
      enabled: true,
      min_word_size_for_typos: MinWordSizeForTypos {
          one_typo: 4,
          two_typos: 8,
      },
      disable_on_words: vec![],
      disable_on_attributes: vec![],
  }).await?;
  ```

  **D) Custom Ranking:**
  ```rust
  index.set_ranking_rules(&[
      "words",       // Number of matched query terms
      "typo",        // Fewer typos = higher rank
      "proximity",   // Query terms closer = higher rank
      "attribute",   // Earlier attributes = higher rank
      "sort",        // Custom sort
      "exactness",   // Exact matches = higher rank
      "custom:word_count:desc", // Longer docs = higher rank (optional)
  ]).await?;
  ```

  **E) Search Settings:**
  ```rust
  // Pagination settings
  index.set_pagination(PaginationSettings {
      max_total_hits: 1000,
  }).await?;

  // Dictionary for compound words
  index.set_dictionary(&[
      "web-scraping", "machine-learning", "artificial-intelligence"
  ]).await?;
  ```

- **Configuration File:**
  - Create `search_config.json` for easy tuning
  - Load synonyms, stop words from external files

---

## Phase 7.6 - Search Analytics

### Goals
- Track all search queries
- Record search results quality
- Monitor search performance
- Analyze user search patterns

### Implementation
- **Files to Create:**
  - `src/analytics/mod.rs` - Analytics tracking
  - `src/analytics/search_log.rs` - Search logging

- **Database Schema:**
  ```sql
  CREATE TABLE search_logs (
    id SERIAL PRIMARY KEY,
    query TEXT NOT NULL,
    results_count INT NOT NULL,
    processing_time_ms INT NOT NULL,
    user_ip VARCHAR(45),
    user_agent TEXT,
    clicked_result_id VARCHAR(255),
    clicked_position INT,
    searched_at TIMESTAMPTZ DEFAULT NOW(),
    session_id VARCHAR(255)
  );

  CREATE INDEX idx_search_logs_query ON search_logs(query);
  CREATE INDEX idx_search_logs_searched_at ON search_logs(searched_at);
  CREATE INDEX idx_search_logs_session ON search_logs(session_id);
  ```

- **Features:**
  - Log every search query asynchronously
  - Record:
    - Query text
    - Number of results
    - Processing time
    - User IP (anonymized)
    - Timestamp
    - Session ID
  - Batch inserts to reduce DB load
  - Use Redis buffer for high-throughput logging

- **Metrics to Track:**
  - Queries per hour/day
  - Average results per query
  - Zero-result queries (failures)
  - Average search latency
  - Most common queries
  - Query abandonment rate

---

## Phase 7.7 - Click Tracking

### Goals
- Track which search results users click
- Measure result relevance
- Improve ranking based on clicks
- Track click-through rate (CTR)

### Implementation
- **Files to Create:**
  - `src/analytics/click_tracker.rs` - Click tracking logic

- **Database Schema:**
  ```sql
  CREATE TABLE search_clicks (
    id SERIAL PRIMARY KEY,
    search_log_id INT REFERENCES search_logs(id),
    document_id VARCHAR(255) NOT NULL,
    position INT NOT NULL,
    clicked_at TIMESTAMPTZ DEFAULT NOW(),
    session_id VARCHAR(255)
  );

  CREATE INDEX idx_clicks_document ON search_clicks(document_id);
  CREATE INDEX idx_clicks_search_log ON search_clicks(search_log_id);
  ```

- **Features:**
  - Frontend sends click event: `POST /api/analytics/click`
  - Track:
    - Which result was clicked
    - Position in results (1st, 2nd, 3rd, etc.)
    - Time to click
    - Session information
  - Calculate CTR per query
  - Identify low-CTR queries (poor results)

- **API Endpoints:**
  - `POST /api/analytics/click`
    ```json
    {
      "search_log_id": 12345,
      "document_id": "doc789",
      "position": 2,
      "session_id": "sess-abc123"
    }
    ```

- **Metrics:**
  - Overall CTR
  - CTR by query
  - CTR by position (position bias)
  - Average click position
  - Time to first click

---

## Phase 7.8 - Popular Queries Dashboard

### Goals
- Admin dashboard for search analytics
- View popular queries
- Identify failing queries (zero results)
- Monitor search health

### Implementation
- **Files to Create:**
  - `frontend-admin/src/routes/analytics/+page.svelte` - Analytics UI

- **API Endpoints:**
  - `GET /api/analytics/popular-queries?limit=20&days=7`
  - `GET /api/analytics/zero-result-queries?limit=20&days=7`
  - `GET /api/analytics/search-stats?days=30`
  - `GET /api/analytics/click-stats?days=30`

- **Dashboard Features:**
  - **Popular Queries Widget:**
    - Top 20 queries by frequency
    - Search count + CTR
    - Trend indicators (↑↓)

  - **Zero-Result Queries:**
    - Queries that returned no results
    - Frequency count
    - Suggested actions (add content, fix typos)

  - **Search Health Metrics:**
    - Total searches (today/week/month)
    - Average results per query
    - Average search latency
    - Zero-result rate
    - Average CTR

  - **Search Volume Chart:**
    - Line chart showing searches over time
    - Hourly/daily/weekly views

  - **Click-Through Analysis:**
    - CTR by position (1-10)
    - Most clicked documents
    - Queries with low CTR

- **Visualization:**
  - Use Chart.js or similar for charts
  - Color-coded health indicators
  - Real-time updates (optional)

---

## Implementation Order

### Phase 1: Core Search Features (7.1-7.4)
1. **Phase 7.3** - Highlighting (Quick win, improves UX immediately)
2. **Phase 7.1** - Autocomplete (High value, user-facing)
3. **Phase 7.4** - Faceted Search (Adds filtering capabilities)
4. **Phase 7.2** - Suggestions (Helps with poor queries)

### Phase 2: Optimization (7.5)
5. **Phase 7.5** - Meilisearch Optimization (Foundational improvements)

### Phase 3: Analytics (7.6-7.8)
6. **Phase 7.6** - Search Analytics (Data collection foundation)
7. **Phase 7.7** - Click Tracking (Builds on search logs)
8. **Phase 7.8** - Analytics Dashboard (Visualization)

---

## Success Metrics

- ✅ Autocomplete response time < 50ms
- ✅ Search with highlighting performs well
- ✅ Faceted search supports multiple filters
- ✅ Typo tolerance handles 1-2 character errors
- ✅ Synonyms expand query coverage by 20%+
- ✅ All searches logged to analytics
- ✅ Click tracking CTR > 40%
- ✅ Zero-result queries < 10%
- ✅ Analytics dashboard shows key metrics

---

## Database Migrations

Create `migrations/00X_search_analytics.sql`:

```sql
-- Search logs for analytics
CREATE TABLE IF NOT EXISTS search_logs (
  id SERIAL PRIMARY KEY,
  query TEXT NOT NULL,
  results_count INT NOT NULL,
  processing_time_ms INT NOT NULL,
  user_ip VARCHAR(45),
  user_agent TEXT,
  clicked_result_id VARCHAR(255),
  clicked_position INT,
  searched_at TIMESTAMPTZ DEFAULT NOW(),
  session_id VARCHAR(255)
);

CREATE INDEX IF NOT EXISTS idx_search_logs_query ON search_logs(query);
CREATE INDEX IF NOT EXISTS idx_search_logs_searched_at ON search_logs(searched_at);
CREATE INDEX IF NOT EXISTS idx_search_logs_session ON search_logs(session_id);

-- Click tracking
CREATE TABLE IF NOT EXISTS search_clicks (
  id SERIAL PRIMARY KEY,
  search_log_id INT REFERENCES search_logs(id) ON DELETE CASCADE,
  document_id VARCHAR(255) NOT NULL,
  position INT NOT NULL,
  clicked_at TIMESTAMPTZ DEFAULT NOW(),
  session_id VARCHAR(255)
);

CREATE INDEX IF NOT EXISTS idx_clicks_document ON search_clicks(document_id);
CREATE INDEX IF NOT EXISTS idx_clicks_search_log ON search_clicks(search_log_id);
CREATE INDEX IF NOT EXISTS idx_clicks_session ON search_clicks(session_id);

-- Search corrections (for "Did you mean?")
CREATE TABLE IF NOT EXISTS search_corrections (
  id SERIAL PRIMARY KEY,
  original_query TEXT NOT NULL,
  corrected_query TEXT NOT NULL,
  accepted_count INT DEFAULT 0,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_corrections_original ON search_corrections(original_query);
```

---

## Configuration Changes

Update `.env` and `src/config/mod.rs`:

```bash
# Search Configuration
SEARCH_AUTOCOMPLETE_LIMIT=5
SEARCH_AUTOCOMPLETE_CACHE_TTL=3600
SEARCH_ENABLE_TYPO_TOLERANCE=true
SEARCH_MAX_TYPOS=2
SEARCH_ENABLE_ANALYTICS=true
SEARCH_LOG_USER_IP=false  # Privacy: anonymize IPs
```

---

## Breaking Changes

### API Changes
- Search response now includes `_formatted` field when highlighting enabled
- Autocomplete is a new endpoint
- Analytics endpoints added

### Frontend Changes
- Update SearchEngineAPI client with new methods:
  - `autocomplete(query, limit)`
  - `trackClick(searchLogId, documentId, position, sessionId)`
  - `getPopularQueries(days)`
  - `getAnalytics(days)`

---

## Notes

- **Privacy Considerations:**
  - Anonymize/hash IP addresses
  - Set retention policy for search logs (e.g., 90 days)
  - Add GDPR compliance notes
  - Allow users to opt-out of tracking

- **Performance:**
  - Use Redis for autocomplete caching
  - Batch analytics writes to reduce DB load
  - Index search_logs table properly
  - Consider time-series database for analytics (optional)

- **Meilisearch Limits:**
  - Default pagination: 1000 results max
  - Typo tolerance works best for English
  - Synonyms have a limit (~10K entries)

Ready to implement! Which phase would you like to start with?
