//! Offline configuration operations.

pub mod archive;

#[cfg(feature = "configuration-xml")]
pub mod rooms;

#[cfg(feature = "configuration-xml")]
pub mod summary;

#[cfg(feature = "configuration-xml")]
mod xml;
