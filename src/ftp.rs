//! FTP client wrapper for Loxone Miniserver backup access

use anyhow::{Context, Result};
use suppaftp::FtpStream;

use crate::config::Config;

/// Retry an FTP operation up to 3 times with exponential backoff.
fn ftp_with_retry<T, F>(f: F) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let delays = [200u64, 400, 800];
    let mut last_err = None;
    for (attempt, delay) in std::iter::once(&0u64).chain(delays.iter()).enumerate() {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(*delay));
        }
        match f() {
            Ok(val) => return Ok(val),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.expect("retry loop must execute at least once"))
}

/// Parsed backup filename: sps_<version>_<timestamp>.zip
#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub filename: String,
    pub version: u32,
    pub timestamp: String, // "20260308182256"
    pub size: u64,
}

impl BackupInfo {
    /// Format the timestamp as "YYYY-MM-DD HH:MM:SS"
    pub fn formatted_date(&self) -> String {
        if self.timestamp.len() >= 14 {
            format!(
                "{}-{}-{} {}:{}:{}",
                &self.timestamp[0..4],
                &self.timestamp[4..6],
                &self.timestamp[6..8],
                &self.timestamp[8..10],
                &self.timestamp[10..12],
                &self.timestamp[12..14],
            )
        } else {
            self.timestamp.clone()
        }
    }
}

/// Parse a backup filename like "sps_194_20260308182256.zip"
pub fn parse_backup_name(name: &str, size: u64) -> Option<BackupInfo> {
    let stem = name.strip_suffix(".zip")?;
    let parts: Vec<&str> = stem.splitn(3, '_').collect();
    if parts.len() != 3 || parts[0] != "sps" {
        return None;
    }
    let version = parts[1].parse::<u32>().ok()?;
    let timestamp = parts[2].to_string();
    if timestamp.len() < 14 || !timestamp.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(BackupInfo {
        filename: name.to_string(),
        version,
        timestamp,
        size,
    })
}

/// Connect to the Miniserver FTP and list backup ZIPs in /prog/.
pub fn list_backups(cfg: &Config) -> Result<Vec<BackupInfo>> {
    let host = ftp_host(cfg);
    let listing = ftp_with_retry(|| {
        let mut ftp = FtpStream::connect(format!("{}:21", host))
            .with_context(|| format!("Could not connect to {}:21 — is FTP enabled?", host))?;
        ftp.login(&cfg.user, &cfg.pass)
            .context("FTP login failed — check your admin credentials in lox config")?;
        let listing = ftp.list(Some("/prog")).context("Failed to list /prog")?;
        ftp.quit().ok();
        Ok(listing)
    })?;

    let mut backups = Vec::new();
    for line in &listing {
        // FTP LIST output: parse last token as filename, size from fields
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 4 {
            continue;
        }
        let name = fields.last().unwrap();
        if !name.ends_with(".zip") || !name.starts_with("sps_") {
            continue;
        }
        // Size is typically field index 4 in Unix-style listing
        let size = fields
            .get(4)
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        if let Some(info) = parse_backup_name(name, size) {
            backups.push(info);
        }
    }
    // Sort by timestamp descending (newest first)
    backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(backups)
}

/// List runtime control overlays. A failed/missing directory is not evidence that it is empty.
pub fn list_custom_changes(cfg: &Config) -> Result<Vec<String>> {
    let host = ftp_host(cfg);
    ftp_with_retry(|| {
        let mut ftp = FtpStream::connect(format!("{}:21", host))
            .with_context(|| format!("Could not connect to {host}:21 — is FTP enabled?"))?;
        ftp.login(&cfg.user, &cfg.pass)
            .context("FTP login failed")?;
        let listing = ftp.list(Some("/sys/custom")).context(
            "Cannot list /sys/custom; cannot safely assume there are no runtime changes",
        )?;
        let names = parse_custom_listing(&listing)?;
        ftp.quit().ok();
        Ok(names)
    })
}

fn parse_custom_listing(listing: &[String]) -> Result<Vec<String>> {
    use lox_core::configuration::custom_listing::{ListingError, parse_custom_listing};

    parse_custom_listing(listing).map_err(|error| match error {
        ListingError::Malformed { .. } => anyhow::Error::msg("Malformed /sys/custom FTP listing"),
        ListingError::Unsupported { row } => anyhow::Error::msg(format!(
            "Unsupported runtime overlay listing: {}",
            listing[row]
        )),
    })
}

pub fn download_custom_change(cfg: &Config, filename: &str) -> Result<Vec<u8>> {
    if filename.contains(['/', '\\']) {
        anyhow::bail!("Invalid runtime overlay filename");
    }
    download_path(cfg, &format!("/sys/custom/{filename}"))
}

/// Download a specific backup file from the Miniserver.
pub fn download_backup(cfg: &Config, filename: &str) -> Result<Vec<u8>> {
    let path = format!("/prog/{}", filename);
    download_path(cfg, &path)
}

fn download_path(cfg: &Config, path: &str) -> Result<Vec<u8>> {
    let host = ftp_host(cfg);
    ftp_with_retry(|| {
        let mut ftp = FtpStream::connect(format!("{}:21", host))
            .with_context(|| format!("Could not connect to {}:21 — is FTP enabled?", host))?;
        ftp.login(&cfg.user, &cfg.pass)
            .context("FTP login failed — check your admin credentials")?;
        ftp.transfer_type(suppaftp::types::FileType::Binary)
            .context("Failed to set binary transfer mode")?;
        let cursor = ftp
            .retr_as_buffer(path)
            .with_context(|| format!("Failed to download {}", path))?;
        ftp.quit().ok();
        Ok(cursor.into_inner())
    })
}

/// Upload a backup file to the Miniserver.
pub fn upload_backup(cfg: &Config, filename: &str, data: &[u8]) -> Result<()> {
    let host = ftp_host(cfg);
    let path = format!("/prog/{}", filename);
    ftp_with_retry(|| {
        let mut ftp = FtpStream::connect(format!("{}:21", host))
            .with_context(|| format!("Could not connect to {}:21 — is FTP enabled?", host))?;
        ftp.login(&cfg.user, &cfg.pass)
            .context("FTP login failed — check your admin credentials")?;
        ftp.transfer_type(suppaftp::types::FileType::Binary)
            .context("Failed to set binary transfer mode")?;
        let mut reader = std::io::Cursor::new(data);
        ftp.put_file(&path, &mut reader)
            .with_context(|| format!("Failed to upload {}", path))?;
        ftp.quit().ok();
        Ok(())
    })
}

/// Extract the FTP hostname from config (strip scheme, port, trailing slash).
fn ftp_host(cfg: &Config) -> String {
    cfg.host
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split(':')
        .next()
        .unwrap_or(&cfg.host)
        .trim_end_matches('/')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Frozen original loop: compare the migrated producer adapter independently.
    fn old_custom_listing(listing: Vec<String>) -> Result<Vec<String>> {
        let mut names = Vec::new();
        for line in listing {
            let name = line
                .split_whitespace()
                .last()
                .context("Malformed /sys/custom FTP listing")?;
            if name.to_ascii_lowercase().ends_with(".loxcc") {
                if line.starts_with('d') || line.starts_with('l') || name.contains(['/', '\\']) {
                    anyhow::bail!("Unsupported runtime overlay listing: {line}");
                }
                names.push(name.to_string());
            }
        }
        names.sort();
        Ok(names)
    }

    #[test]
    fn strict_custom_listing_matches_original_values() {
        for rows in [
            vec![],
            vec!["d /a.txt", "l x\\y.txt", "- a.ℓoxcc", "- a.loxccé"],
            vec!["- z.loXcC", "- A.LoxCC", "- z.loXcC"],
            vec![
                "- 12345678-1234-1234-1234567890123456.LoxCC",
                "- not-a-uuid.loxcc",
                "D q.loxcc",
                "L r.loxcc",
                " d s.loxcc",
                " l t.loxcc",
                "nonnumeric permissions\u{2003}u.loxcc",
                "- nul\0.loxcc",
                "- first.loxcc tail",
            ],
        ] {
            let rows: Vec<String> = rows.into_iter().map(str::to_owned).collect();
            assert_eq!(
                parse_custom_listing(&rows).unwrap(),
                old_custom_listing(rows).unwrap()
            );
        }
    }

    #[test]
    fn strict_custom_listing_preserves_error_payloads_and_context() {
        let check = |old: &anyhow::Error, new: &anyhow::Error| {
            assert_eq!(old.to_string(), new.to_string());
            assert_eq!(format!("{old:?}"), format!("{new:?}"));
            assert_eq!(format!("{old:#}"), format!("{new:#}"));
            assert_eq!(
                old.downcast_ref::<&'static str>(),
                new.downcast_ref::<&'static str>()
            );
            assert_eq!(old.downcast_ref::<String>(), new.downcast_ref::<String>());
            assert_eq!(old.source().is_some(), new.source().is_some());
            assert_eq!(
                old.chain().map(ToString::to_string).collect::<Vec<_>>(),
                new.chain().map(ToString::to_string).collect::<Vec<_>>()
            );
            assert!(
                new.downcast_ref::<lox_core::configuration::custom_listing::ListingError>()
                    .is_none()
            );
        };
        for row in [
            "",
            " \t",
            "\u{2003}",
            "d bad.loxcc",
            "l bad.loxcc",
            "- x/y.loxcc",
            "- x\\y.loxcc",
        ] {
            let rows = vec![
                "- valid.LoxCC".to_owned(),
                "- unrelated.txt".to_owned(),
                row.to_owned(),
            ];
            let old = old_custom_listing(rows.clone()).unwrap_err();
            let new = parse_custom_listing(&rows).unwrap_err();
            check(&old, &new);
            if row.split_whitespace().next().is_none() {
                assert_eq!(
                    new.downcast_ref::<&'static str>(),
                    Some(&"Malformed /sys/custom FTP listing")
                );
                assert!(new.downcast_ref::<String>().is_none());
            } else {
                assert_eq!(
                    new.downcast_ref::<String>(),
                    Some(&format!("Unsupported runtime overlay listing: {row}"))
                );
                assert!(new.downcast_ref::<&'static str>().is_none());
            }
            check(
                &old.context(format!("capture {}", 42)),
                &new.context(format!("capture {}", 42)),
            );
        }
    }

    #[test]
    fn test_parse_backup_name_valid() {
        let info = parse_backup_name("sps_194_20260308182256.zip", 252416).unwrap();
        assert_eq!(info.version, 194);
        assert_eq!(info.timestamp, "20260308182256");
        assert_eq!(info.size, 252416);
        assert_eq!(info.formatted_date(), "2026-03-08 18:22:56");
    }

    #[test]
    fn test_parse_backup_name_invalid() {
        assert!(parse_backup_name("random.zip", 0).is_none());
        assert!(parse_backup_name("sps_abc_20260308.zip", 0).is_none());
        assert!(parse_backup_name("sps_194_short.zip", 0).is_none());
        assert!(parse_backup_name("sps_194_20260308182256.tar", 0).is_none());
    }

    #[test]
    fn test_ftp_host_strips_scheme() {
        let cfg = Config {
            host: "https://192.168.1.77".into(),
            ..Default::default()
        };
        assert_eq!(ftp_host(&cfg), "192.168.1.77");
    }

    #[test]
    fn test_ftp_host_strips_port() {
        let cfg = Config {
            host: "https://192.168.1.77:8443".into(),
            ..Default::default()
        };
        assert_eq!(ftp_host(&cfg), "192.168.1.77");
    }

    #[test]
    fn test_formatted_date() {
        let info = BackupInfo {
            filename: "sps_1_20260101120000.zip".into(),
            version: 1,
            timestamp: "20260101120000".into(),
            size: 0,
        };
        assert_eq!(info.formatted_date(), "2026-01-01 12:00:00");
    }
}
