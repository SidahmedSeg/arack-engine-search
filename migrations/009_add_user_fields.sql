-- Migration 009: Add username, date_of_birth, and gender fields to user_preferences
-- Phase 8: Simplified Registration

-- Add new columns to user_preferences table
ALTER TABLE user_preferences
  ADD COLUMN IF NOT EXISTS username VARCHAR(50) UNIQUE,
  ADD COLUMN IF NOT EXISTS date_of_birth DATE,
  ADD COLUMN IF NOT EXISTS gender VARCHAR(10) CHECK (gender IN ('male', 'female'));

-- Create case-insensitive unique index for username lookups
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_preferences_username_lower
  ON user_preferences (LOWER(username));

-- Create username availability cache table (performance optimization)
CREATE TABLE IF NOT EXISTS username_availability_cache (
    username VARCHAR(50) PRIMARY KEY,
    available BOOLEAN NOT NULL,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Index for cache expiry checks (5-minute TTL)
CREATE INDEX IF NOT EXISTS idx_username_cache_expires
  ON username_availability_cache (checked_at);

-- PostgreSQL function for fast username availability checks
CREATE OR REPLACE FUNCTION check_username_available(check_username TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN NOT EXISTS (
        SELECT 1 FROM user_preferences
        WHERE LOWER(username) = LOWER(check_username)
    );
END;
$$ LANGUAGE plpgsql;
