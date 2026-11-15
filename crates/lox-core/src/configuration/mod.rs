//! Offline configuration operations.

pub mod archive;

#[cfg(feature = "configuration-xml")]
pub mod controls;

#[cfg(feature = "configuration-xml")]
pub mod devices;

#[cfg(feature = "configuration-xml")]
pub mod diff;

#[cfg(feature = "configuration-xml")]
pub mod rooms;

#[cfg(feature = "configuration-xml")]
pub mod summary;

#[cfg(feature = "configuration-xml")]
pub mod users;

#[cfg(feature = "configuration-xml")]
mod xml;
