//! Framework-neutral Ameya application core.
//!
//! This crate owns persistence, domain models, AI providers, search/RAG,
//! simulation, jobs, and logic. Windowing and command transport belong to the
//! NanaUI host and are deliberately not coupled to this crate.

pub mod ai;
pub mod db;
pub mod domain;
pub mod logic;
pub mod services;
pub mod vector;

pub mod test_support;
