-- Create separate schemas for microservices isolation
-- This migration creates the schema structure for the microservices architecture

-- Create schemas
CREATE SCHEMA IF NOT EXISTS search;
CREATE SCHEMA IF NOT EXISTS email;
CREATE SCHEMA IF NOT EXISTS auth;

-- Grant all permissions to postgres user (used by both services in development)
GRANT ALL ON SCHEMA search TO postgres;
GRANT ALL ON SCHEMA email TO postgres;
GRANT ALL ON SCHEMA auth TO postgres;

-- Grant usage permissions on all tables in schemas
GRANT ALL ON ALL TABLES IN SCHEMA search TO postgres;
GRANT ALL ON ALL TABLES IN SCHEMA email TO postgres;
GRANT ALL ON ALL TABLES IN SCHEMA auth TO postgres;

-- Grant usage permissions on all sequences in schemas
GRANT ALL ON ALL SEQUENCES IN SCHEMA search TO postgres;
GRANT ALL ON ALL SEQUENCES IN SCHEMA email TO postgres;
GRANT ALL ON ALL SEQUENCES IN SCHEMA auth TO postgres;

-- Set default privileges for future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA search GRANT ALL ON TABLES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA email GRANT ALL ON TABLES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA auth GRANT ALL ON TABLES TO postgres;

ALTER DEFAULT PRIVILEGES IN SCHEMA search GRANT ALL ON SEQUENCES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA email GRANT ALL ON SEQUENCES TO postgres;
ALTER DEFAULT PRIVILEGES IN SCHEMA auth GRANT ALL ON SEQUENCES TO postgres;

-- Note: Existing tables from migrations 001-004 remain in the public schema
-- They will be migrated to appropriate schemas in future migrations:
-- - collections, pages, crawl_errors → search schema
-- - users, invitations, sessions → auth schema
-- - (email tables will be created directly in email schema)
