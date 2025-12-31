//! Email Service Module
//!
//! This module contains all email-related functionality including:
//! - Stalwart admin API (Phase 2) ✅
//! - Email provisioning (Phase 2) ✅
//! - JMAP client integration (Phase 3) ✅
//! - Meilisearch search (Phase 3) ✅
//! - Centrifugo real-time messaging (Phase 3) ✅
//! - OAuth 2.0 authentication (Phase 8 - OIDC) ✅
//! - Contact management (Phase 6) - stub
//! - AI features (Phase 7) - stub

pub mod ai;
pub mod api;
pub mod centrifugo;
pub mod contacts;
pub mod jmap;
pub mod oauth;
pub mod provisioning;
pub mod search;
pub mod stalwart;
pub mod types;
