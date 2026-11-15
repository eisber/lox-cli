//! Headless Loxone toolkit.
//!
//! Offline byte codecs in [`configuration::archive`] and in-memory [`control`]
//! resolution require no networking runtime.
//! Optional `configuration-xml` adds a typed byte-input room inventory.
//! Optional `statistics-xml` decodes monthly statistics XML without I/O.
//! The optional `runtime` feature provides an async typed clock operation;
//! `runtime-blocking` adds an explicitly owned driver.
//! No context selection, filesystem, terminal or language-binding API is provided.

pub mod configuration;
pub mod control;

#[cfg(feature = "statistics-xml")]
pub mod statistics;

#[cfg(feature = "runtime")]
pub mod runtime;
