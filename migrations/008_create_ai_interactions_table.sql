-- Migration: Create AI Interactions Table (Phase 5)
-- Purpose: Track AI feature usage and costs for quota management
-- NOTE: This migration only runs if email_accounts table exists (email service only)

DO $$
BEGIN
    -- Check if email.email_accounts table exists
    IF EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'email' AND table_name = 'email_accounts'
    ) THEN
        -- Create table for tracking AI interactions
        CREATE TABLE IF NOT EXISTS email.email_ai_interactions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            account_id UUID NOT NULL REFERENCES email.email_accounts(id) ON DELETE CASCADE,
            feature VARCHAR(50) NOT NULL,  -- 'smart_compose', 'summarize', 'priority'
            tokens_used INTEGER NOT NULL DEFAULT 0,
            cost_usd DECIMAL(10, 6),  -- Cost in USD
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        );

        -- Index for quota checking (user + feature + date)
        CREATE INDEX IF NOT EXISTS idx_ai_interactions_quota
        ON email.email_ai_interactions(account_id, feature, created_at);

        -- Index for cost analytics
        CREATE INDEX IF NOT EXISTS idx_ai_interactions_cost
        ON email.email_ai_interactions(account_id, created_at, cost_usd);

        -- Index for feature usage stats
        CREATE INDEX IF NOT EXISTS idx_ai_interactions_feature
        ON email.email_ai_interactions(feature, created_at);

        -- Add comment
        COMMENT ON TABLE email.email_ai_interactions IS 'Tracks AI feature usage for quota management and cost tracking';
        COMMENT ON COLUMN email.email_ai_interactions.feature IS 'AI feature type: smart_compose, summarize, or priority';
        COMMENT ON COLUMN email.email_ai_interactions.tokens_used IS 'Total OpenAI tokens consumed (input + output)';
        COMMENT ON COLUMN email.email_ai_interactions.cost_usd IS 'Calculated cost in USD based on GPT-4o-mini pricing';
    END IF;
END $$;
