-- Add Zitadel user ID support to email tables
-- Phase 3: Zitadel Migration - Email Service Support

-- Make kratos_identity_id nullable in email_accounts (for Zitadel users)
ALTER TABLE email.email_accounts
ALTER COLUMN kratos_identity_id DROP NOT NULL;

-- Add zitadel_user_id column to email_accounts
ALTER TABLE email.email_accounts
ADD COLUMN IF NOT EXISTS zitadel_user_id TEXT;

-- Create unique index on zitadel_user_id (partial index to exclude NULLs)
CREATE UNIQUE INDEX IF NOT EXISTS idx_email_accounts_zitadel_user_id
ON email.email_accounts(zitadel_user_id)
WHERE zitadel_user_id IS NOT NULL;

-- Add comments for documentation
COMMENT ON COLUMN email.email_accounts.zitadel_user_id IS 'Zitadel user ID (numeric string). Used for Zitadel authentication.';
COMMENT ON COLUMN email.email_accounts.kratos_identity_id IS 'Kratos identity ID. NULL for Zitadel users (who use zitadel_user_id instead).';

-- Make kratos_identity_id nullable in email_provisioning_log
ALTER TABLE email.email_provisioning_log
ALTER COLUMN kratos_identity_id DROP NOT NULL;

-- Add zitadel_user_id column to email_provisioning_log
ALTER TABLE email.email_provisioning_log
ADD COLUMN IF NOT EXISTS zitadel_user_id TEXT;

-- Create index on zitadel_user_id for provisioning log
CREATE INDEX IF NOT EXISTS idx_provisioning_log_zitadel_user_id
ON email.email_provisioning_log(zitadel_user_id)
WHERE zitadel_user_id IS NOT NULL;

-- Add comments
COMMENT ON COLUMN email.email_provisioning_log.zitadel_user_id IS 'Zitadel user ID (numeric string). Used for Zitadel provisioning logs.';
COMMENT ON COLUMN email.email_provisioning_log.kratos_identity_id IS 'Kratos identity ID. NULL for Zitadel users.';
