//! Search Service Module
//!
//! This module contains all search-related functionality including:
//! - Web crawler
//! - Analytics
//! - Search engine integration
//! - API routes
//! - Background workers
//! - Job scheduling

pub mod api;
pub mod analytics;
pub mod crawler;
pub mod redis;
pub mod scheduler;
pub mod search;
pub mod worker;
pub mod qdrant;
