# Search Engine API Documentation

## Base URL
```
http://127.0.0.1:3000
```

## Authentication
Currently, no authentication is required (development mode).

---

## Endpoints

### 1. Health Check
Check if the API server is running.

**Endpoint:** `GET /health`

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-12-09T20:44:31.600333+00:00"
}
```

**Example:**
```bash
curl http://127.0.0.1:3000/health
```

---

### 2. Start Crawl
Initiate a web crawl for the specified URLs.

**Endpoint:** `POST /api/crawl`

**Request Body:**
```json
{
  "urls": ["https://example.com"],
  "max_depth": 1
}
```

**Parameters:**
- `urls` (required): Array of URLs to crawl
- `max_depth` (optional, default: 3): Maximum crawl depth

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Crawl completed successfully",
    "documents_indexed": 1,
    "urls": ["https://example.com"]
  }
}
```

**Example:**
```bash
curl -X POST http://127.0.0.1:3000/api/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "max_depth": 1
  }'
```

---

### 3. Search Documents
Search indexed documents with advanced filtering and pagination.

**Endpoint:** `GET /api/search`

**Query Parameters:**
- `q` (required): Search query string
- `limit` (optional, default: 20): Number of results to return
- `offset` (optional, default: 0): Number of results to skip (pagination)
- `sort_by` (optional): Field to sort by (`crawled_at`, `word_count`)
- `sort_order` (optional, default: "asc"): Sort order (`asc`, `desc`)
- `min_word_count` (optional): Minimum word count filter
- `max_word_count` (optional): Maximum word count filter
- `from_date` (optional): Filter documents crawled after this date (ISO 8601 format)
- `to_date` (optional): Filter documents crawled before this date (ISO 8601 format)

**Response:**
```json
{
  "success": true,
  "data": {
    "hits": [
      {
        "id": "f02c504a-727f-468e-8bce-739c9dfef1e4",
        "url": "https://example.com",
        "title": "Example Domain",
        "content": "Example Domain This domain is for use in documentation...",
        "word_count": 19,
        "crawled_at": "2025-12-09T16:59:31.861793+00:00"
      }
    ],
    "processing_time_ms": 3,
    "query": "example",
    "total_hits": 1
  }
}
```

#### Basic Search
```bash
curl "http://127.0.0.1:3000/api/search?q=example&limit=10"
```

#### Pagination
```bash
# Get first page (results 0-9)
curl "http://127.0.0.1:3000/api/search?q=example&limit=10&offset=0"

# Get second page (results 10-19)
curl "http://127.0.0.1:3000/api/search?q=example&limit=10&offset=10"
```

#### Filter by Word Count
```bash
# Find documents with 10-50 words
curl "http://127.0.0.1:3000/api/search?q=example&min_word_count=10&max_word_count=50"
```

#### Sort Results
```bash
# Sort by word count (descending)
curl "http://127.0.0.1:3000/api/search?q=example&sort_by=word_count&sort_order=desc"

# Sort by crawl date (most recent first)
curl "http://127.0.0.1:3000/api/search?q=example&sort_by=crawled_at&sort_order=desc"
```

#### Filter by Date
```bash
# Find documents crawled after a specific date
curl "http://127.0.0.1:3000/api/search?q=example&from_date=2025-12-01T00:00:00Z"

# Find documents crawled in a date range
curl "http://127.0.0.1:3000/api/search?q=example&from_date=2025-12-01T00:00:00Z&to_date=2025-12-31T23:59:59Z"
```

#### Combined Filters
```bash
# Search with multiple filters
curl "http://127.0.0.1:3000/api/search?q=example&limit=10&offset=0&min_word_count=10&sort_by=word_count&sort_order=desc"
```

---

### 4. Get Index Statistics
Retrieve statistics about the search index.

**Endpoint:** `GET /api/stats`

**Response:**
```json
{
  "success": true,
  "data": {
    "numberOfDocuments": 47,
    "isIndexing": false,
    "fieldDistribution": {
      "id": 47,
      "url": 47,
      "title": 47,
      "content": 47,
      "word_count": 47,
      "crawled_at": 47
    }
  }
}
```

**Example:**
```bash
curl http://127.0.0.1:3000/api/stats
```

---

### 5. Clear Index
Delete all documents from the search index.

**Endpoint:** `DELETE /api/index`

**Response:**
```json
{
  "success": true,
  "data": {
    "message": "Index cleared successfully"
  }
}
```

**Example:**
```bash
curl -X DELETE http://127.0.0.1:3000/api/index
```

---

## Response Format

All API responses follow this standard format:

### Success Response
```json
{
  "success": true,
  "data": {
    // Response data here
  }
}
```

### Error Response
```json
{
  "success": false,
  "error": "Error message describing what went wrong"
}
```

---

## HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 400 | Bad Request (invalid parameters) |
| 500 | Internal Server Error |

---

## Search Features

### 1. Typo Tolerance
Meilisearch automatically handles typos in search queries.

```bash
# Will still find "Example Domain"
curl "http://127.0.0.1:3000/api/search?q=exampl"
```

### 2. Ranking
Results are ranked by:
1. **Words**: Number of query words found
2. **Typo**: Fewer typos rank higher
3. **Proximity**: Closer words rank higher
4. **Attribute**: Title matches rank higher than content matches
5. **Sort**: Custom sort order (if specified)
6. **Exactness**: Exact matches rank higher

### 3. Searchable Fields (in priority order)
1. `title` (highest priority)
2. `description`
3. `keywords`
4. `content`
5. `url` (lowest priority)

---

## Performance

- **Average search time**: 3-5ms
- **Crawl time**: Varies by website size
- **Index time**: < 100ms per document

---

## Limits

- **Default page size**: 20 results
- **Maximum page size**: 1000 results
- **Maximum offset**: 10,000
- **Maximum content length**: 10,000 characters per document

---

## Examples

### Complete Workflow

```bash
# 1. Check health
curl http://127.0.0.1:3000/health

# 2. Start a crawl
curl -X POST http://127.0.0.1:3000/api/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["https://example.com"], "max_depth": 1}'

# 3. Wait for crawl to complete (check response)

# 4. Search the indexed content
curl "http://127.0.0.1:3000/api/search?q=example&limit=10"

# 5. Get statistics
curl http://127.0.0.1:3000/api/stats

# 6. Clear index (if needed)
curl -X DELETE http://127.0.0.1:3000/api/index
```

### Advanced Search Examples

```bash
# Find long documents about "rust" sorted by length
curl "http://127.0.0.1:3000/api/search?q=rust&min_word_count=100&sort_by=word_count&sort_order=desc&limit=5"

# Recent documents about "tutorial"
curl "http://127.0.0.1:3000/api/search?q=tutorial&from_date=2025-12-01T00:00:00Z&sort_by=crawled_at&sort_order=desc"

# Paginated search with filters
curl "http://127.0.0.1:3000/api/search?q=programming&limit=20&offset=20&min_word_count=50"
```

---

## Error Handling

### Invalid URL in Crawl Request
```bash
curl -X POST http://127.0.0.1:3000/api/crawl \
  -H "Content-Type: application/json" \
  -d '{"urls": ["not-a-valid-url"]}'
```

Response:
```json
{
  "success": false,
  "error": "Crawl failed: invalid URL"
}
```

### Empty Search Query
```bash
curl "http://127.0.0.1:3000/api/search?q="
```

Returns all documents (up to limit).

---

## CORS

CORS is currently configured in permissive mode for development. All origins are allowed.

---

## Rate Limiting

Not currently implemented. Will be added in future phases.

---

## Notes

- The search engine uses Meilisearch under the hood for fast, typo-tolerant search
- Documents are indexed asynchronously during crawling
- The index is persistent and survives server restarts
- Dates must be in ISO 8601 format (e.g., `2025-12-09T16:59:31.861793+00:00`)
