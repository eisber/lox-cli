//! Headless Loxone toolkit.
//!
//! Offline byte codecs in [`configuration::archive`] and in-memory [`control`]
//! resolution require no networking runtime.
//! [`color`] provides typed offline encoders and explicit legacy color observers.
//! [`numeric_state`] decodes complete numeric value-state table payloads, not frames.
//! [`text_state`] decodes complete UTF-8 text-state table payloads, including icon IDs.
//! [`daytimer_state`] observes complete runtime Daytimer-State payloads.
//! [`weather_state`] observes complete Weather-State payloads without domain interpretation.
//! [`message_header`] observes separate application headers without transport or reassembly.
//! Optional `configuration-xml` adds typed room inventory, summaries and summary deltas.
//! Optional `configuration-editor` adds a legacy-compatible set-param byte transaction.
//! Optional `statistics-xml` parses statistics indexes and monthly XML without I/O.
//! Optional `firmware-metadata` observes update XML and assesses numeric versions offline.
//! Optional `auth-hash` provides explicit existing APP/CONFIG hash profiles.
//! Optional `structure-json` projects typed LoxApp3 control/autopilot catalogs and metadata.
//! Optional `legacy-wsx` encodes the existing deterministic legacy handshake.
//! Optional `legacy-token-aes` encodes the existing CLI's legacy AES command bytes.
//! The optional `runtime` feature provides an async typed clock operation;
//! `runtime-blocking` adds an explicitly owned driver.
//! No context selection, filesystem, terminal or language-binding API is provided.

pub mod backup_catalog;
pub mod color;
pub mod configuration;
pub mod control;
pub mod daytimer_state;
pub mod message_header;
pub mod numeric_state;
pub mod text_state;
pub mod weather_state;

#[cfg(feature = "firmware-metadata")]
pub mod firmware_metadata;

#[cfg(feature = "structure-json")]
pub mod control_catalog;

#[cfg(feature = "structure-json")]
pub mod structure_metadata;

#[cfg(feature = "structure-json")]
pub mod autopilot_catalog;

#[cfg(feature = "statistics-xml")]
pub mod statistics;

#[cfg(feature = "auth-hash")]
pub mod authentication;

#[cfg(feature = "legacy-wsx")]
pub mod legacy_wsx;

#[cfg(feature = "legacy-token-aes")]
pub mod legacy_token_aes;

#[cfg(feature = "runtime")]
pub mod runtime;
/// Unix timestamp of 2009-01-01 00:00:00 UTC, the Loxone timestamp epoch.
pub const LOXONE_EPOCH_SECS: u64 = 1_230_768_000;
