// Public surface for tests and binaries.
pub use finance_core::money;
pub use finance_core::tax;

pub mod clauses;
pub mod cli;
pub mod commands;
pub mod config;
pub mod db;
pub mod error;
pub mod kinds;
pub mod legal;
pub mod output;
pub mod render;
pub mod typst_assets;
