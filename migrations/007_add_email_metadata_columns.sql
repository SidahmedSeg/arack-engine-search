-- Migration 007: Add missing columns to email_metadata for Phase 3
--
-- Adds columns needed for Meilisearch indexing and JMAP integration
-- NOTE: This migration only runs if email_metadata table exists (email service only)

DO $$
BEGIN
    -- Check if email_metadata table exists
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'email_metadata') THEN
        -- Add from_name column
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS from_name VARCHAR(255);

        -- Add cc_addresses column
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS cc_addresses TEXT[] DEFAULT '{}';

        -- Add body_preview column (renamed from snippet for consistency)
        -- If snippet exists, it will be manually migrated; otherwise just create body_preview
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS body_preview TEXT;

        -- Add indexed_at timestamp
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS indexed_at TIMESTAMP WITH TIME ZONE;

        -- Add keywords array (JMAP keywords like $seen, $flagged, etc.)
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS keywords TEXT[] DEFAULT '{}';

        -- Add mailbox_ids array (email can be in multiple mailboxes)
        ALTER TABLE email_metadata
        ADD COLUMN IF NOT EXISTS mailbox_ids TEXT[] DEFAULT '{}';

        -- Keep mailbox_id for backwards compatibility (primary mailbox)
        -- We can remove this later once fully migrated to mailbox_ids

        -- Create index on keywords for filtering
        CREATE INDEX IF NOT EXISTS idx_email_metadata_keywords ON email_metadata USING GIN(keywords);

        -- Create index on mailbox_ids for filtering
        CREATE INDEX IF NOT EXISTS idx_email_metadata_mailbox_ids ON email_metadata USING GIN(mailbox_ids);
    END IF;
END $$;
