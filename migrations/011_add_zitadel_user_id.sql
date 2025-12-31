-- Add Zitadel user ID column to user_preferences
-- Phase 3: Zitadel Migration

-- Add zitadel_user_id column (nullable for backward compatibility)
ALTER TABLE user_preferences
ADD COLUMN IF NOT EXISTS zitadel_user_id TEXT;

-- Create unique index on zitadel_user_id (partial index to exclude NULLs)
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_prefs_zitadel_user_id
ON user_preferences(zitadel_user_id)
WHERE zitadel_user_id IS NOT NULL;

-- Add comment for documentation
COMMENT ON COLUMN user_preferences.zitadel_user_id IS 'Zitadel user ID (sub claim from JWT). Used for Zitadel authentication.';
