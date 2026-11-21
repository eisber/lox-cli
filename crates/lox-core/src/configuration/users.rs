//! Read-only raw XML user inventory, not authentication or permission evaluation.

use super::xml::attr_value;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfo {
    pub name: String,
    pub uuid: String,
    pub nfc: bool,
    pub description: String,
}

#[derive(Debug)]
pub enum UserInventoryError {
    Xml { source: quick_xml::Error },
}

impl fmt::Display for UserInventoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml { source } => source.fmt(f),
        }
    }
}

impl std::error::Error for UserInventoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml { source } => Some(source),
        }
    }
}

pub type Result<T> = std::result::Result<T, UserInventoryError>;

/// Inventory exact `C` elements with raw `Type="User"` at any depth.
///
/// Results retain encounter order and duplicate identities. Attributes retain
/// raw UTF-8/entity spellings; the first matching attribute wins, unreadable
/// values default independently, and attribute iterator errors are ignored.
/// Missing `Title`, `U` and `Desc` become empty. Every literal underscore in
/// `Desc` becomes a space. NFC means only that raw `NFCArr` is nonempty.
///
/// Input is borrowed, never repaired, normalized or DOM-loaded. Empty/text-only
/// input and incomplete EOF can succeed; reader errors return no partial list.
/// This neither validates users nor interprets credentials or permissions.
pub fn parse_users(xml: &[u8]) -> Result<Vec<UserInfo>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut users = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e))
                if e.name().as_ref() == b"C"
                    && attr_value(e, b"Type").as_deref() == Some("User") =>
            {
                users.push(UserInfo {
                    name: attr_value(e, b"Title").unwrap_or_default(),
                    uuid: attr_value(e, b"U").unwrap_or_default(),
                    nfc: !attr_value(e, b"NFCArr").unwrap_or_default().is_empty(),
                    description: attr_value(e, b"Desc").unwrap_or_default().replace('_', " "),
                });
            }
            Ok(Event::Eof) => break,
            Err(source) => return Err(UserInventoryError::Xml { source }),
            _ => {}
        }
        buf.clear();
    }
    Ok(users)
}
