//! Headless Loxone toolkit.
//!
//! Offline byte codecs in [`configuration::archive`] require no networking runtime.
//! The optional `runtime` feature provides an async typed clock operation;
//! `runtime-blocking` adds an explicitly owned driver.
//! No context selection, filesystem, terminal or language-binding API is provided.

pub mod configuration;

#[cfg(feature = "runtime")]
pub mod runtime;
