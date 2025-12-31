-- Migration 004: Optimize Autocomplete with Query Log Indexes
--
-- Purpose: Add B-tree indexes for fast ILIKE prefix matching on search_history
-- This enables production-grade autocomplete using query logs instead of page titles
--
-- Performance Impact:
-- - Without index: Full table scan (slow at 1M+ rows)
-- - With index: Index-only scan (sub-millisecond latency)

-- Optimize autocomplete queries with case-insensitive prefix matching
-- text_pattern_ops enables efficient LIKE/ILIKE operations
CREATE INDEX IF NOT EXISTS idx_search_history_query_lower
ON search_history (LOWER(query) text_pattern_ops);

-- Composite index for scoring (query + recency)
-- Used for calculating popularity and recency scores
CREATE INDEX IF NOT EXISTS idx_search_history_query_created
ON search_history (query, created_at DESC)
WHERE query != '';

-- Note: Existing GIN index (to_tsvector) is for full-text search, not prefix matching
-- The text_pattern_ops operator class is specifically optimized for LIKE/ILIKE operations
