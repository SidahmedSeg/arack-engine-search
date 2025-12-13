-- Create collections table
CREATE TABLE IF NOT EXISTS collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    url_pattern VARCHAR(500),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index on name for faster lookups
CREATE INDEX IF NOT EXISTS idx_collections_name ON collections(name);

-- Create crawl_history table to track crawl jobs
CREATE TABLE IF NOT EXISTS crawl_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    collection_id UUID REFERENCES collections(id) ON DELETE CASCADE,
    urls TEXT[] NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    pages_crawled INTEGER DEFAULT 0,
    pages_indexed INTEGER DEFAULT 0,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    metadata JSONB DEFAULT '{}'::jsonb
);

-- Create indexes for crawl_history
CREATE INDEX IF NOT EXISTS idx_crawl_history_collection_id ON crawl_history(collection_id);
CREATE INDEX IF NOT EXISTS idx_crawl_history_status ON crawl_history(status);
CREATE INDEX IF NOT EXISTS idx_crawl_history_started_at ON crawl_history(started_at DESC);

-- Create crawl_errors table for tracking errors
CREATE TABLE IF NOT EXISTS crawl_errors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    crawl_id UUID REFERENCES crawl_history(id) ON DELETE CASCADE,
    url VARCHAR(2000) NOT NULL,
    error_type VARCHAR(100) NOT NULL,
    error_message TEXT,
    occurred_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create index for crawl_errors
CREATE INDEX IF NOT EXISTS idx_crawl_errors_crawl_id ON crawl_errors(crawl_id);
CREATE INDEX IF NOT EXISTS idx_crawl_errors_url ON crawl_errors(url);
