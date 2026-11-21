//! In-memory control resolution, independent of transport, cache and context.
//!
//! Prepare the query before loading a catalog: identifiers and exact aliases
//! resolve without one. Catalog order is preserved, including in ambiguity
//! errors; the caller owns catalog acquisition, sorting and diagnostics.

use std::collections::HashMap;
use std::fmt;

/// A control from a caller-supplied catalog. IDs are opaque catalog values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    pub name: String,
    pub uuid: String,
    pub typ: String,
    pub room: Option<String>,
    pub cat: Option<String>,
    pub is_favorite: bool,
    pub is_secured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedControl {
    pub uuid: String,
}

/// The first resolution stage, which never needs a catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreparedControlQuery<'a> {
    Resolved(ResolvedControl),
    Catalog(ControlQuery<'a>),
}

/// A query requiring a catalog, after identifier and alias bypasses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlQuery<'a> {
    original: &'a str,
    name: &'a str,
    room: Option<&'a str>,
}

impl<'a> ControlQuery<'a> {
    /// Preserve CLI compatibility: identifier first, then an exact case-sensitive
    /// alias of the raw query, then the rightmost bracket room qualifier.
    /// Alias values are returned verbatim, not validated or recursively resolved.
    pub fn prepare(
        query: &'a str,
        room_filter: Option<&'a str>,
        aliases: &HashMap<String, String>,
    ) -> PreparedControlQuery<'a> {
        if is_uuid(query) {
            return PreparedControlQuery::Resolved(ResolvedControl { uuid: query.into() });
        }
        if let Some(uuid) = aliases.get(query) {
            return PreparedControlQuery::Resolved(ResolvedControl { uuid: uuid.clone() });
        }
        let (name, room) = if let Some(idx) = query.rfind('[')
            && query.ends_with(']')
        {
            (query[..idx].trim(), Some(&query[idx + 1..query.len() - 1]))
        } else {
            (query, None)
        };
        PreparedControlQuery::Catalog(Self {
            original: query,
            name,
            room: room.or(room_filter),
        })
    }

    /// Unicode-lowercased substring matching, with no exact-name preference.
    /// Missing rooms match the empty string; neither empty queries nor literal
    /// room whitespace are rejected or normalized.
    pub fn resolve(&self, controls: &[Control]) -> Result<ResolvedControl, ResolutionError> {
        let matches: Vec<&Control> = controls
            .iter()
            .filter(|control| matches_text(&control.name, self.name))
            .filter(|control| {
                self.room
                    .is_none_or(|room| matches_text(control.room.as_deref().unwrap_or(""), room))
            })
            .collect();
        match matches.as_slice() {
            [] => Err(ResolutionError::NotFound {
                query: self.original.into(),
            }),
            [control] => Ok(ResolvedControl {
                uuid: control.uuid.clone(),
            }),
            _ => Err(ResolutionError::Ambiguous {
                query: self.original.into(),
                candidates: matches.into_iter().cloned().collect(),
            }),
        }
    }
}

/// Structured selection failures; callers choose their own error presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionError {
    NotFound {
        query: String,
    },
    Ambiguous {
        query: String,
        candidates: Vec<Control>,
    },
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound { query } => write!(f, "control query {query:?} matched no controls"),
            Self::Ambiguous { query, candidates } => write!(
                f,
                "control query {query:?} matched {} controls",
                candidates.len()
            ),
        }
    }
}

impl std::error::Error for ResolutionError {}

/// Compatibility substring comparison; not Unicode case folding or ranking.
pub fn matches_text(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

/// Legacy identifier detection, deliberately not a strict UUID grammar.
/// This is distinct from URL-path validation and imposes no upper length limit.
pub fn is_uuid(value: &str) -> bool {
    value.len() > 20
        && value.contains('-')
        && value.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentifierError {
    InvalidLength,
    InvalidCharacters,
    NonHex,
}

impl fmt::Display for IdentifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidLength => "identifier length must be between 1 and 50 bytes",
            Self::InvalidCharacters => "identifier contains unsafe path characters",
            Self::NonHex => "identifier contains non-hex characters",
        })
    }
}

impl std::error::Error for IdentifierError {}

/// Validate the legacy command-path identifier contract without tightening it
/// to the detection grammar: short hex/dash strings remain accepted.
pub fn validate_uuid(value: &str) -> Result<(), IdentifierError> {
    if value.is_empty() || value.len() > 50 {
        return Err(IdentifierError::InvalidLength);
    }
    if value.contains("..") || value.contains('/') || value.contains('\\') || value.contains('%') {
        return Err(IdentifierError::InvalidCharacters);
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        return Err(IdentifierError::NonHex);
    }
    Ok(())
}
