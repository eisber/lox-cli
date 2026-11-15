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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "Retained head compatibility helper")
)]
pub fn parse_backup_name(name: &str, size: u64) -> Option<BackupInfo> {
    lox_core::backup_catalog::parse_backup_name(name, size).map(Into::into)
}

impl From<lox_core::backup_catalog::BackupInfo> for BackupInfo {
    fn from(info: lox_core::backup_catalog::BackupInfo) -> Self {
        Self {
            filename: info.filename,
            version: info.version,
            timestamp: info.timestamp,
            size: info.size,
        }
    }
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

    Ok(lox_core::backup_catalog::parse_backup_listing(&listing)
        .into_iter()
        .map(Into::into)
        .collect())
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
        ftp.quit().ok();
        Ok(names)
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

    #[test]
    fn shared_catalog_adapter_retains_head_date_and_output_fields() {
        let rows = ["drwx x x x +00042 sps_+001_20261399259999999.zip".to_string()];
        let info: BackupInfo = lox_core::backup_catalog::parse_backup_listing(&rows)
            .pop()
            .unwrap()
            .into();
        assert_eq!(info.filename, "sps_+001_20261399259999999.zip");
        assert_eq!(info.version, 1);
        assert_eq!(info.timestamp, "20261399259999999");
        assert_eq!(info.size, 42);
        assert_eq!(info.formatted_date(), "2026-13-99 25:99:99");
        let parsed = parse_backup_name(&info.filename, info.size).unwrap();
        assert_eq!(parsed.timestamp, info.timestamp);
        assert_eq!(parsed.formatted_date(), info.formatted_date());
    }
}
