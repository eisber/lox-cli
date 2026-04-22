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
        let listing = ftp.list(Some("/prog")).unwrap_or_default();
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

/// Download a specific backup file from the Miniserver.
pub fn download_backup(cfg: &Config, filename: &str) -> Result<Vec<u8>> {
    let host = ftp_host(cfg);
    let path = format!("/prog/{}", filename);
    ftp_with_retry(|| {
        let mut ftp = FtpStream::connect(format!("{}:21", host))
            .with_context(|| format!("Could not connect to {}:21 — is FTP enabled?", host))?;
        ftp.login(&cfg.user, &cfg.pass)
            .context("FTP login failed — check your admin credentials")?;
        ftp.transfer_type(suppaftp::types::FileType::Binary)
            .context("Failed to set binary transfer mode")?;
        let cursor = ftp
            .retr_as_buffer(&path)
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
}
