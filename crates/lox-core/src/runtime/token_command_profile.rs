//! Provisional token command recommendations, not firmware or transport qualification.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenFormat {
    Legacy,
    Jwt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenOperation {
    Acquire,
    /// `None` means the existing token's format is not known.
    Refresh {
        existing_format: Option<TokenFormat>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenCommand {
    GetToken,
    GetJwt,
    RefreshToken,
    RefreshJwt,
}

impl TokenCommand {
    /// A fixed command name only, not a path, payload, or request.
    pub fn label(self) -> &'static str {
        match self {
            Self::GetToken => "gettoken",
            Self::GetJwt => "getjwt",
            Self::RefreshToken => "refreshtoken",
            Self::RefreshJwt => "refreshjwt",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenCommandError {
    MissingFirmware,
    FirmwareTooOld {
        major: u32,
        minor: u32,
    },
    UnknownExistingFormat,
    RefreshFormatMismatch {
        recommended: TokenFormat,
        existing: TokenFormat,
    },
}

impl fmt::Display for TokenCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingFirmware => f.write_str("known firmware major and minor are required"),
            Self::FirmwareTooOld { major, minor } => {
                write!(
                    f,
                    "firmware {major}.{minor} is below the token policy minimum of 9.0"
                )
            }
            Self::UnknownExistingFormat => f.write_str("refresh requires a known token format"),
            Self::RefreshFormatMismatch {
                recommended,
                existing,
            } => write!(
                f,
                "existing {existing:?} format does not match the recommended {recommended:?} refresh format"
            ),
        }
    }
}

impl std::error::Error for TokenCommandError {}

/// Recommends a fixed command from a caller-supplied known `(major, minor)` pair.
///
/// Pass `None` if either component is unavailable. An unauthenticated bootstrap
/// hint does not establish a known version for this policy. Refresh never selects
/// a different format, including the vendor's legacy-to-JWT conversion route.
/// A recommendation provides no channel, encryption, or argument policy.
pub fn select_token_command(
    firmware: Option<(u32, u32)>,
    operation: TokenOperation,
) -> Result<TokenCommand, TokenCommandError> {
    let (major, minor) = firmware.ok_or(TokenCommandError::MissingFirmware)?;
    let recommended = match (major, minor) {
        (0..=8, _) => return Err(TokenCommandError::FirmwareTooOld { major, minor }),
        (9, _) | (10, 0..=1) => TokenFormat::Legacy,
        _ => TokenFormat::Jwt,
    };

    match operation {
        TokenOperation::Acquire => Ok(match recommended {
            TokenFormat::Legacy => TokenCommand::GetToken,
            TokenFormat::Jwt => TokenCommand::GetJwt,
        }),
        TokenOperation::Refresh { existing_format } => {
            let existing = existing_format.ok_or(TokenCommandError::UnknownExistingFormat)?;
            if existing != recommended {
                return Err(TokenCommandError::RefreshFormatMismatch {
                    recommended,
                    existing,
                });
            }
            Ok(match recommended {
                TokenFormat::Legacy => TokenCommand::RefreshToken,
                TokenFormat::Jwt => TokenCommand::RefreshJwt,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquisition_switches_to_jwt_at_10_2() {
        for firmware in [(9, 0), (9, 99), (10, 0), (10, 1)] {
            let command = select_token_command(Some(firmware), TokenOperation::Acquire).unwrap();
            assert_eq!(command, TokenCommand::GetToken);
            assert_eq!(command.label(), "gettoken");
        }
        for firmware in [(10, 2), (10, 10), (11, 0), (11, 2), (16, 0)] {
            let command = select_token_command(Some(firmware), TokenOperation::Acquire).unwrap();
            assert_eq!(command, TokenCommand::GetJwt);
            assert_eq!(command.label(), "getjwt");
        }
    }

    #[test]
    fn missing_or_old_firmware_never_selects_a_command() {
        for operation in [
            TokenOperation::Acquire,
            TokenOperation::Refresh {
                existing_format: Some(TokenFormat::Legacy),
            },
        ] {
            assert_eq!(
                select_token_command(None, operation),
                Err(TokenCommandError::MissingFirmware)
            );
            assert_eq!(
                select_token_command(Some((8, 99)), operation),
                Err(TokenCommandError::FirmwareTooOld {
                    major: 8,
                    minor: 99
                })
            );
        }
    }

    #[test]
    fn refresh_preserves_the_known_format() {
        for (firmware, format, expected, label) in [
            (
                (9, 0),
                TokenFormat::Legacy,
                TokenCommand::RefreshToken,
                "refreshtoken",
            ),
            (
                (10, 1),
                TokenFormat::Legacy,
                TokenCommand::RefreshToken,
                "refreshtoken",
            ),
            (
                (10, 2),
                TokenFormat::Jwt,
                TokenCommand::RefreshJwt,
                "refreshjwt",
            ),
            (
                (11, 2),
                TokenFormat::Jwt,
                TokenCommand::RefreshJwt,
                "refreshjwt",
            ),
        ] {
            let command = select_token_command(
                Some(firmware),
                TokenOperation::Refresh {
                    existing_format: Some(format),
                },
            )
            .unwrap();
            assert_eq!(command, expected);
            assert_eq!(command.label(), label);
        }
    }

    #[test]
    fn refresh_refuses_unknown_format_and_both_mismatch_directions() {
        for (firmware, recommended, existing) in [
            ((10, 1), TokenFormat::Legacy, TokenFormat::Jwt),
            ((10, 2), TokenFormat::Jwt, TokenFormat::Legacy),
        ] {
            assert_eq!(
                select_token_command(
                    Some(firmware),
                    TokenOperation::Refresh {
                        existing_format: None
                    },
                ),
                Err(TokenCommandError::UnknownExistingFormat)
            );
            assert_eq!(
                select_token_command(
                    Some(firmware),
                    TokenOperation::Refresh {
                        existing_format: Some(existing)
                    },
                ),
                Err(TokenCommandError::RefreshFormatMismatch {
                    recommended,
                    existing
                })
            );
        }
    }
}
