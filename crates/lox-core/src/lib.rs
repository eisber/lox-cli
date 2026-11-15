//! Headless Loxone toolkit.
//!
//! Offline byte codecs in [`configuration::archive`] and in-memory [`control`]
//! resolution require no networking runtime.
//! [`color`] provides typed offline encoders and explicit legacy color observers.
//! [`numeric_state`] decodes complete numeric value-state table payloads, not frames.
//! Optional `configuration-xml` adds typed room inventory, summaries and summary deltas.
//! Optional `statistics-xml` decodes monthly statistics XML without I/O.
//! Optional `firmware-metadata` observes update XML and assesses numeric versions offline.
//! Optional `auth-hash` provides explicit existing APP/CONFIG hash profiles.
//! Optional `structure-json` projects typed top-level LoxApp3 control catalogs.
//! Optional `legacy-wsx` encodes the existing deterministic legacy handshake.
//! The optional `runtime` feature provides an async typed clock operation;
//! `runtime-blocking` adds an explicitly owned driver.
//! No context selection, filesystem, terminal or language-binding API is provided.

pub mod color;
pub mod configuration;
pub mod control;
pub mod numeric_state;

#[cfg(feature = "firmware-metadata")]
pub mod firmware_metadata;

#[cfg(feature = "structure-json")]
pub mod control_catalog;

#[cfg(feature = "statistics-xml")]
pub mod statistics;

#[cfg(feature = "auth-hash")]
pub mod authentication;

#[cfg(feature = "legacy-wsx")]
pub mod legacy_wsx;

#[cfg(feature = "runtime")]
pub mod runtime;
