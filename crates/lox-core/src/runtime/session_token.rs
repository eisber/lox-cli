//! Provisional, in-memory token ownership for one explicit session subject.
//!
//! A scope match is not authentication, credential eligibility, or a trust
//! decision for either route. Nothing here loads, persists, acquires, refreshes,
//! or sends tokens. Replacing, clearing or dropping a slot releases its owned
//! token normally; neither zeroization nor protection from OS memory capture is
//! guaranteed.

use super::session_endpoints::SessionEndpoints;
use std::fmt;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Exact, nonempty principal and an already admitted pair of routes.
///
/// The principal is not trimmed, case-folded or Unicode-normalized.
pub struct TokenSubject<'a> {
    endpoints: &'a SessionEndpoints,
    principal: &'a str,
}

impl<'a> TokenSubject<'a> {
    pub fn new(
        endpoints: &'a SessionEndpoints,
        principal: &'a str,
    ) -> Result<Self, TokenSlotError> {
        if principal.is_empty() {
            return Err(TokenSlotError::EmptyPrincipal);
        }
        Ok(Self {
            endpoints,
            principal,
        })
    }

    #[cfg(feature = "runtime-websocket")]
    pub(super) fn endpoints(&self) -> &SessionEndpoints {
        self.endpoints
    }
}

impl fmt::Debug for TokenSubject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenSubject").finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenExpiry {
    At(SystemTime),
    /// Unknown expiry never grants token access.
    Unknown,
}

impl TokenExpiry {
    /// Converts caller-provided integer seconds since 2009-01-01 00:00:00 UTC.
    /// No legacy parsing, validity margin, or sentinel interpretation is applied.
    pub fn from_loxone_seconds(seconds: u64) -> Result<Self, TokenSlotError> {
        const LOXONE_EPOCH_UNIX_SECONDS: u64 = 1_230_768_000;
        LOXONE_EPOCH_UNIX_SECONDS
            .checked_add(seconds)
            .and_then(|seconds| UNIX_EPOCH.checked_add(Duration::from_secs(seconds)))
            .map(Self::At)
            .ok_or(TokenSlotError::ExpiryOutOfRange)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenSlotError {
    EmptyPrincipal,
    EmptyToken,
    SubjectMismatch,
    ExpiryOutOfRange,
}

impl fmt::Display for TokenSlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyPrincipal => "token principal must be nonempty",
            Self::EmptyToken => "token must be nonempty",
            Self::SubjectMismatch => "token slot subject mismatch",
            Self::ExpiryOutOfRange => "token expiry is out of range",
        })
    }
}

impl std::error::Error for TokenSlotError {}

#[must_use]
#[derive(Debug)]
pub enum TokenLookup<'a> {
    Match(TokenAccess<'a>),
    Empty,
    SubjectMismatch,
    Expired,
    UnknownExpiry,
}

/// A borrowed token obtained only after a matching, unexpired lookup.
pub struct TokenAccess<'a>(&'a str);

impl TokenAccess<'_> {
    /// Deliberately exposes the opaque token to the caller. Not for logging.
    pub fn expose_token(&self) -> &str {
        self.0
    }
}

impl fmt::Debug for TokenAccess<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenAccess").finish_non_exhaustive()
    }
}

struct TokenRecord {
    token: String,
    expiry: TokenExpiry,
}

/// One optional token record, pinned to its original subject even when empty.
///
/// Scope equality uses the complete canonical HTTP and WebSocket URL strings
/// from admission (including effective ports and exact admitted paths), plus
/// the requested proxy policy and exact principal. Changing the requested
/// policy requires a new slot. Matching that label does not pin actual proxy
/// settings, TLS policy, DNS results, or the peer on either route.
pub struct TokenSlot {
    endpoints: SessionEndpoints,
    principal: String,
    record: Option<TokenRecord>,
}

impl TokenSlot {
    pub fn new(subject: &TokenSubject<'_>) -> Self {
        Self {
            endpoints: subject.endpoints.clone(),
            principal: subject.principal.to_owned(),
            record: None,
        }
    }

    /// Takes ownership of the opaque token without normalizing or returning it.
    /// A mismatch or empty token leaves the existing record intact.
    pub fn replace(
        &mut self,
        subject: &TokenSubject<'_>,
        token: String,
        expiry: TokenExpiry,
    ) -> Result<(), TokenSlotError> {
        self.require_subject(subject)?;
        if token.is_empty() {
            return Err(TokenSlotError::EmptyToken);
        }
        self.record = Some(TokenRecord { token, expiry });
        Ok(())
    }

    /// Uses only the caller's supplied time. At expiry equality access is
    /// denied. This does not evict records or track clock rollback across calls.
    pub fn lookup(&self, subject: &TokenSubject<'_>, now: SystemTime) -> TokenLookup<'_> {
        if self.require_subject(subject).is_err() {
            return TokenLookup::SubjectMismatch;
        }
        match &self.record {
            None => TokenLookup::Empty,
            Some(record) => match record.expiry {
                TokenExpiry::At(expiry) if now < expiry => {
                    TokenLookup::Match(TokenAccess(&record.token))
                }
                TokenExpiry::At(_) => TokenLookup::Expired,
                TokenExpiry::Unknown => TokenLookup::UnknownExpiry,
            },
        }
    }

    /// Removes the record without returning it; the original subject stays pinned.
    pub fn clear(&mut self, subject: &TokenSubject<'_>) -> Result<(), TokenSlotError> {
        self.require_subject(subject)?;
        self.record = None;
        Ok(())
    }

    fn require_subject(&self, subject: &TokenSubject<'_>) -> Result<(), TokenSlotError> {
        if self.endpoints.same_scope(subject.endpoints)
            && self.principal.as_str() == subject.principal
        {
            Ok(())
        } else {
            Err(TokenSlotError::SubjectMismatch)
        }
    }
}

impl fmt::Debug for TokenSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TokenSlot").finish_non_exhaustive()
    }
}
