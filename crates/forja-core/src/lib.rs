//! forja-core — Shared business logic for the forja skills marketplace.
//!
//! This crate contains all platform-independent logic: error types, path resolution,
//! data models, catalog scanning, symlink management, linting, analytics, and templates.
//! It has no CLI or UI dependencies — both the CLI and desktop app import from here.

pub mod analytics;
pub mod error;
pub mod frontmatter;
pub mod lint;
pub mod models;
pub mod paths;
pub mod registry;
pub mod scanner;
pub mod settings;
pub mod symlink;
pub mod templates;
