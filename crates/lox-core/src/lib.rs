//! Headless Loxone toolkit.
//!
//! Offline byte codecs in [`configuration::archive`] and in-memory [`control`]
//! resolution require no networking runtime.
//! Optional `configuration-xml` adds a typed byte-input room inventory.
//! Optional `auth-hash` provides explicit existing APP/CONFIG hash profiles.
//! The optional `runtime` feature provides an async typed clock operation;
//! `runtime-blocking` adds an explicitly owned driver.
//! No context selection, filesystem, terminal or language-binding API is provided.

pub mod configuration;
pub mod control;

#[cfg(feature = "auth-hash")]
pub mod authentication;

#[cfg(feature = "runtime")]
pub mod runtime;
