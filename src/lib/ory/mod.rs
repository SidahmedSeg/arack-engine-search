// Phase 8.6: Ory Integration Module
// This module provides integration with Ory Kratos for authentication
// and user feature management (preferences, saved searches, history)

mod kratos;
mod models;
mod repository;
pub mod middleware;

// Re-export public types
pub use kratos::KratosClient;
pub use models::*;
pub use repository::OryUserRepository;
pub use middleware::{require_ory_auth, optional_ory_auth, OrySession};
