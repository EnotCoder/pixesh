//! Application layer: document model, application state, and UI.

pub(crate) mod config;
pub(crate) mod document;
pub(crate) mod effects;
pub(crate) mod state;
pub(crate) mod ui;

pub(crate) use document::Document;
pub(crate) use state::PixeshApp;
