-- Make kratos_identity_id nullable for Zitadel users
-- Phase 3: Zitadel Migration - Schema Updates

-- Drop the NOT NULL constraint on kratos_identity_id
-- Zitadel users will have zitadel_user_id instead of kratos_identity_id
ALTER TABLE user_preferences
ALTER COLUMN kratos_identity_id DROP NOT NULL;

-- Add comment for documentation
COMMENT ON COLUMN user_preferences.kratos_identity_id IS 'Kratos identity ID. NULL for Zitadel users (who use zitadel_user_id instead).';
