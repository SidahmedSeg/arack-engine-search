-- Email Service Tables (Phase 2: Automatic Provisioning)
--
-- These tables support the email microservice functionality

-- Email Accounts - Maps Kratos identities to email accounts
CREATE TABLE IF NOT EXISTS email_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID UNIQUE NOT NULL,
    email_address VARCHAR(255) UNIQUE NOT NULL,
    stalwart_user_id VARCHAR(255) UNIQUE NOT NULL,
    storage_quota_bytes BIGINT DEFAULT 5368709120, -- 5GB default
    storage_used_bytes BIGINT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_accounts_kratos_id ON email_accounts(kratos_identity_id);
CREATE INDEX IF NOT EXISTS idx_email_accounts_email ON email_accounts(email_address);
CREATE INDEX IF NOT EXISTS idx_email_accounts_stalwart_id ON email_accounts(stalwart_user_id);

-- Email Provisioning Log - Audit trail for account provisioning
CREATE TABLE IF NOT EXISTS email_provisioning_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kratos_identity_id UUID NOT NULL,
    action VARCHAR(50) NOT NULL, -- 'create', 'update', 'delete'
    status VARCHAR(50) NOT NULL, -- 'pending', 'success', 'failed'
    error_message TEXT,
    attempt_count INTEGER DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX IF NOT EXISTS idx_provisioning_log_kratos_id ON email_provisioning_log(kratos_identity_id);
CREATE INDEX IF NOT EXISTS idx_provisioning_log_status ON email_provisioning_log(status);
CREATE INDEX IF NOT EXISTS idx_provisioning_log_created_at ON email_provisioning_log(created_at DESC);

-- Email Metadata - Cached email metadata for quick searches
CREATE TABLE IF NOT EXISTS email_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
    jmap_id VARCHAR(255) NOT NULL,
    mailbox_id VARCHAR(255) NOT NULL,
    subject TEXT,
    from_address VARCHAR(255),
    to_addresses TEXT[], -- Array of recipient emails
    received_at TIMESTAMP WITH TIME ZONE,
    snippet TEXT, -- First 200 chars of body
    has_attachments BOOLEAN DEFAULT FALSE,
    is_read BOOLEAN DEFAULT FALSE,
    is_starred BOOLEAN DEFAULT FALSE,
    is_deleted BOOLEAN DEFAULT FALSE,
    indexed_in_meilisearch BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, jmap_id)
);

CREATE INDEX IF NOT EXISTS idx_email_metadata_account ON email_metadata(account_id);
CREATE INDEX IF NOT EXISTS idx_email_metadata_mailbox ON email_metadata(mailbox_id);
CREATE INDEX IF NOT EXISTS idx_email_metadata_received_at ON email_metadata(received_at DESC);
CREATE INDEX IF NOT EXISTS idx_email_metadata_indexed ON email_metadata(indexed_in_meilisearch) WHERE indexed_in_meilisearch = FALSE;

-- Email Contacts - Auto-extracted from sent/received emails
CREATE TABLE IF NOT EXISTS email_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
    email_address VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    first_seen_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    message_count INTEGER DEFAULT 1,
    is_frequent BOOLEAN DEFAULT FALSE, -- Top 50 contacts
    UNIQUE(account_id, email_address)
);

CREATE INDEX IF NOT EXISTS idx_email_contacts_account ON email_contacts(account_id);
CREATE INDEX IF NOT EXISTS idx_email_contacts_email ON email_contacts(email_address);
CREATE INDEX IF NOT EXISTS idx_email_contacts_frequent ON email_contacts(is_frequent) WHERE is_frequent = TRUE;

-- Email Labels - User-defined labels
CREATE TABLE IF NOT EXISTS email_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7), -- Hex color code
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(account_id, name)
);

CREATE INDEX IF NOT EXISTS idx_email_labels_account ON email_labels(account_id);

-- Email AI Interactions - Track AI feature usage
CREATE TABLE IF NOT EXISTS email_ai_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES email_accounts(id) ON DELETE CASCADE,
    feature VARCHAR(50) NOT NULL, -- 'smart_compose', 'summarize', 'priority_rank'
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_email_ai_account ON email_ai_interactions(account_id);
CREATE INDEX IF NOT EXISTS idx_email_ai_feature ON email_ai_interactions(feature);
CREATE INDEX IF NOT EXISTS idx_email_ai_created_at ON email_ai_interactions(created_at DESC);
