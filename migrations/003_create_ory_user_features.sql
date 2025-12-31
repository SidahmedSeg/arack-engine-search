-- Phase 8.6: Ory User Features Tables
-- This migration creates tables for user features in the search engine
-- These tables link to Ory Kratos identities via kratos_identity_id

-- ===== USER PREFERENCES TABLE =====
-- Anchor table for user settings (theme, results per page, analytics opt-out)
-- Links to Ory Kratos identity via UUID
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID UNIQUE NOT NULL, -- Links to Ory Kratos identity
    theme VARCHAR(20) NOT NULL DEFAULT 'light', -- 'light' or 'dark'
    results_per_page INTEGER NOT NULL DEFAULT 20,
    analytics_opt_out BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_user_preferences_kratos_id ON user_preferences(kratos_identity_id);

-- ===== SAVED SEARCHES TABLE =====
-- Users can save search queries with custom names
CREATE TABLE IF NOT EXISTS saved_searches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    query TEXT NOT NULL,
    filters JSONB, -- Store search filters as JSON (sort, order, word count, etc.)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_saved_searches_user FOREIGN KEY (kratos_identity_id)
        REFERENCES user_preferences(kratos_identity_id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_saved_searches_kratos_id ON saved_searches(kratos_identity_id);
CREATE INDEX IF NOT EXISTS idx_saved_searches_created_at ON saved_searches(created_at DESC);

-- ===== SEARCH HISTORY TABLE =====
-- Track user search queries and clicks for analytics
CREATE TABLE IF NOT EXISTS search_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID NOT NULL,
    query TEXT NOT NULL,
    filters JSONB, -- Store filters used in the search
    result_count INTEGER,
    clicked_url TEXT, -- URL that was clicked (if any)
    clicked_position INTEGER, -- Position in results (0-indexed)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_search_history_user FOREIGN KEY (kratos_identity_id)
        REFERENCES user_preferences(kratos_identity_id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_search_history_kratos_id ON search_history(kratos_identity_id);
CREATE INDEX IF NOT EXISTS idx_search_history_created_at ON search_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_query ON search_history USING gin(to_tsvector('english', query));

-- ===== TRIGGERS FOR UPDATED_AT =====
-- Automatically update updated_at timestamp on record modification

CREATE TRIGGER update_user_preferences_updated_at
    BEFORE UPDATE ON user_preferences
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_saved_searches_updated_at
    BEFORE UPDATE ON saved_searches
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ===== NOTES =====
-- 1. Ory Kratos creates its own tables (identities, identity_credentials, etc.)
-- 2. We reference kratos_identity_id but don't create a foreign key constraint
--    to Kratos tables because they're managed separately by Kratos migrations
-- 3. The CASCADE on DELETE ensures cleanup when users are deleted from Kratos
-- 4. user_preferences serves as the anchor table - other tables reference it
