-- Migration 010: Create OAuth Tokens Table (Phase 8 - OIDC)
-- This table stores OAuth 2.0 access and refresh tokens for email service authentication

CREATE TABLE IF NOT EXISTS email.email_oauth_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID NOT NULL UNIQUE,
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    scope TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Foreign key to ensure user exists
    CONSTRAINT fk_oauth_tokens_email_accounts
        FOREIGN KEY (kratos_identity_id)
        REFERENCES email.email_accounts(kratos_identity_id)
        ON DELETE CASCADE
);

-- Index for fast token lookup by user
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_kratos_id
    ON email.email_oauth_tokens(kratos_identity_id);

-- Index for expired token cleanup (future optimization)
CREATE INDEX IF NOT EXISTS idx_oauth_tokens_expires_at
    ON email.email_oauth_tokens(expires_at);

COMMENT ON TABLE email.email_oauth_tokens IS 'OAuth 2.0 tokens for email service authentication via Ory Hydra';
COMMENT ON COLUMN email.email_oauth_tokens.access_token IS 'Short-lived access token for JMAP Bearer authentication';
COMMENT ON COLUMN email.email_oauth_tokens.refresh_token IS 'Long-lived refresh token for obtaining new access tokens';
COMMENT ON COLUMN email.email_oauth_tokens.expires_at IS 'Expiration timestamp for access token';
COMMENT ON COLUMN email.email_oauth_tokens.scope IS 'OAuth scopes granted (e.g., "openid email profile")';
