//! Zitadel OIDC Integration
//!
//! This module provides OAuth2/OIDC integration with Zitadel for authentication.
//! It supports:
//! - Authorization code flow with PKCE
//! - JWT token validation
//! - User info fetching
//! - Webhook handling for user provisioning
//! - Management API for user creation (custom registration)

pub mod client;
pub mod management;
pub mod middleware;
pub mod models;

pub use client::ZitadelClient;
pub use management::{
    CreateUserRequest, CreateUserResponse, UserEmail, UserMetadata, UserPassword, UserProfile,
    ZitadelManagementClient, encode_metadata, to_zitadel_gender,
};
pub use middleware::validate_zitadel_jwt;
pub use models::{
    OAuthState, ZitadelActionsV2Event, ZitadelClaims, ZitadelUserEventPayload, ZitadelUserInfo,
    ZitadelWebhookPayload,
};
