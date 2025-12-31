//! Arack Shared Library
//!
//! This library contains shared code used by both the search service and email service.
//! It provides common functionality for authentication, database access, configuration,
//! and type definitions.
//!
//! It also includes service-specific modules (search and email) as public modules.

/// Authentication module (axum-login based)
pub mod auth;

/// Database connection and query utilities
pub mod db;

/// Configuration management
pub mod config;

/// Ory Kratos/Hydra integration
pub mod ory;

/// Zitadel OIDC integration (Phase 3)
pub mod zitadel;

/// Shared type definitions
pub mod types;

// Re-export commonly used types for convenience
pub use types::*;

// Service-specific modules (accessible to binaries)
// The modules are at the project root, so we need to specify the path
#[path = "../../search/mod.rs"]
pub mod search;

#[path = "../../email/mod.rs"]
pub mod email;
