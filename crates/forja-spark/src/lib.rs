//! forja-spark — Claude CLI process manager for forja desktop.
//!
//! Manages spawning `claude` CLI processes with `--output-format stream-json`
//! and parsing their streaming output for the desktop app's chat UI.

pub mod events;
pub mod parser;
pub mod process;

/// Spark-level error type.
#[derive(Debug, thiserror::Error)]
pub enum SparkError {
    #[error("claude CLI not found — is it installed and in PATH?")]
    ClaudeNotFound,

    #[error("process failed: {0}")]
    Process(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("no active session — send a message first")]
    NoSession,
}

pub type Result<T> = std::result::Result<T, SparkError>;
