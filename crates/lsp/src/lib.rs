//! Language server for Ekips Prime Pro.
//!
//! Speaks the Language Server Protocol over stdin/stdout. The VS Code
//! extension starts the `ekips-lsp` binary; other editors can do the same.
//! See `docs/en/lsp.md` for the features and configuration.

pub mod analysis;
pub mod knowledge;
pub mod position;
pub mod server;

pub use server::{Config, run};
