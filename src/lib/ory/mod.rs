// Authentication & User Features Module
//
// This module provides integration with our custom SSO system (account-service)
// and user feature management (preferences, saved searches, history)
//
// Architecture:
// - AccountServiceClient validates JWTs using JWKS from account.arack.io
// - Uses RS256 algorithm with cached public keys
// - Session cookie: arack_session on .arack.io domain
// - See CUSTOM_SSO_SYSTEM.md for full documentation

mod account_service;
mod models;
mod repository;
pub mod middleware;

// Re-export public types
pub use account_service::AccountServiceClient;
pub use models::*;
pub use repository::OryUserRepository;
pub use middleware::{require_ory_auth, optional_ory_auth, OrySession};
