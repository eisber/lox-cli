//! Binary-private archive deployment policy and CLI codec error adapters.
//!
//! The byte codec lives in `lox_core::configuration::archive`; these helpers
//! retain the CLI's sidecar trust and snapshot/deployment distinctions.
//!
//! ## Config load priority on Miniserver
//!
//! 1. `/prog/Emergency.LoxCC` (crash recovery only)
//! 2. `/prog/sps_new.zip` or `.LoxCC`
//! 3. `/prog/sps_<vers>_<timestamp>.zip` or `.LoxCC` (latest wins)
//! 4. `/prog/sps.zip`, `/prog/sps_old.zip`
//! 5. `/prog/Default.Loxone` or `/prog/DefaultGo.Loxone`
//!
//! ## Password fields
//!
//! Config XML fields with `t="15"` are encrypted by the Miniserver firmware.
//! Fields with `t="11"` contain plaintext strings. The Miniserver accepts
//! `t="11"` for password fields (e.g. `mqtt_auth_pwd`), using the value
//! directly — no encryption needed.
//!
//! ## References
//!
//! - <https://github.com/sarnau/Inside-The-Loxone-Miniserver>
//! - CRC32 discovery and `t="11"` plaintext trick from eisber/lox-cli contributors

use anyhow::{Context, Result, bail};
use lox_core::configuration::archive::{self, ArchiveError};
use std::collections::BTreeMap;
use std::io::{Cursor, Read, Write};

pub use archive::compress_loxcc;

mod sidecar_guard;

/// Preserve the existing anyhow chain rendered by the CLI and its error envelope.
pub(crate) fn archive_error(error: ArchiveError) -> anyhow::Error {
    match error {
        ArchiveError::InvalidZip { source } => {
            anyhow::Error::new(source).context("Invalid ZIP archive")
        }
        ArchiveError::ZipEntry { source, .. } => source.into(),
        ArchiveError::ReadZipEntry { source, .. } => source.into(),
        other => other.into(),
    }
}

pub fn decompress_loxcc(data: &[u8]) -> Result<Vec<u8>> {
    archive::decompress_loxcc(data).map_err(archive_error)
}

pub fn extract_and_decompress(data: &[u8]) -> Result<Vec<u8>> {
    archive::extract_and_decompress(data).map_err(archive_error)
}

/// Repack a backup ZIP, replacing `sps0.LoxCC` with recompressed XML.
///
/// Rejects edits that need native sidecar compilation. Unrelated native
/// sidecars are copied without recompressing or rewriting their contents.
pub fn repack_zip(src_zip: &[u8], xml: &[u8]) -> Result<Vec<u8>> {
    validate_repack(src_zip, xml)?;
    repack_snapshot_zip(src_zip, xml)
}

/// Check an XML replacement against the original, compiled backup.
pub(crate) fn validate_repack(src_zip: &[u8], xml: &[u8]) -> Result<()> {
    let mut archive = zip::ZipArchive::new(Cursor::new(src_zip)).context("Invalid source ZIP")?;
    let mut source_xml = None;
    let mut app = None;
    let mut permissions = None;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let slot = match file.name().to_ascii_lowercase().as_str() {
            "sps0.loxcc" => &mut source_xml,
            "loxapp3.json" => &mut app,
            "permissions.bin" => &mut permissions,
            _ => continue,
        };
        if slot.is_some() {
            bail!("Ambiguous deployment archive: duplicate {}", file.name());
        }
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        *slot = Some(data);
    }
    let source_xml = source_xml.context("Source ZIP does not contain a root sps0.LoxCC")?;
    let source_xml = decompress_loxcc(&source_xml)?;
    sidecar_guard::validate(&source_xml, xml, app.as_deref(), permissions.as_deref())
}

/// Validate a supplied ZIP as well as the trusted, compiled live baseline.
///
/// XML-neutral raw uploads must not bypass sidecar validation or substitute
/// unverified catalogue, permission, or physical display payloads.
pub(crate) fn validate_archive_deployment(
    source_zip: &[u8],
    candidate_zip: &[u8],
    xml: &[u8],
) -> Result<()> {
    fn sidecars(data: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
        let mut archive =
            zip::ZipArchive::new(Cursor::new(data)).context("Invalid deployment ZIP")?;
        let mut result = BTreeMap::new();
        for index in 0..archive.len() {
            let mut file = archive.by_index(index)?;
            let name = file.name().replace('\\', "/").to_ascii_lowercase();
            if file.is_dir()
                || !(matches!(name.as_str(), "loxapp3.json" | "permissions.bin")
                    || name.starts_with("tpdisp/"))
            {
                continue;
            }
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            if result.insert(name.clone(), bytes).is_some() {
                bail!("Ambiguous deployment archive: duplicate sidecar {name}");
            }
        }
        Ok(result)
    }

    validate_repack(source_zip, xml)?;
    validate_repack(candidate_zip, xml)?;
    if sidecars(source_zip)? != sidecars(candidate_zip)? {
        bail!(
            "Candidate ZIP replaces or omits native LoxAPP3.json, permissions.bin or TPDISP sidecars. \
             lox cannot verify replacement sidecars and will not upload an incomplete deployment. \
             Use an unchanged complete native backup for sidecar-neutral edits, \
             or save and deploy with Loxone Config."
        );
    }
    Ok(())
}

/// Reconstruct a read-only download snapshot after merging runtime overlays.
///
/// This deliberately does not compile sidecars and MUST NOT be used as a
/// deployment artifact or as the compiled baseline for `validate_repack`.
pub(crate) fn repack_snapshot_zip(src_zip: &[u8], xml: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(src_zip);
    let mut archive = zip::ZipArchive::new(cursor).context("Invalid source ZIP")?;
    let sps_indices: Vec<_> = archive
        .file_names()
        .enumerate()
        .filter_map(|(index, name)| name.eq_ignore_ascii_case("sps0.LoxCC").then_some(index))
        .collect();
    let index = match sps_indices.as_slice() {
        [index] => *index,
        [] => bail!("Source ZIP does not contain a root sps0.LoxCC"),
        _ => bail!("Ambiguous source ZIP: duplicate sps0.LoxCC"),
    };
    let mut original = Vec::new();
    archive.by_index(index)?.read_to_end(&mut original)?;
    if decompress_loxcc(&original)? == xml {
        return Ok(src_zip.to_vec());
    }
    let new_loxcc = compress_loxcc(xml);

    let buf = Vec::new();
    let out = Cursor::new(buf);
    let mut writer = zip::ZipWriter::new(out);
    writer.set_raw_comment(archive.comment().into());
    let mut replaced = false;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name().to_string();
        if name.eq_ignore_ascii_case("sps0.loxcc") {
            if replaced {
                bail!("Ambiguous source ZIP: duplicate sps0.LoxCC");
            }
            replaced = true;
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(file.compression())
                .last_modified_time(file.last_modified().unwrap_or_default())
                .unix_permissions(file.unix_mode().unwrap_or(0o644));
            writer.start_file(&name, options)?;
            writer.write_all(&new_loxcc)?;
        } else if file.is_dir() {
            let options = zip::write::SimpleFileOptions::default()
                .last_modified_time(file.last_modified().unwrap_or_default())
                .unix_permissions(file.unix_mode().unwrap_or(0o755));
            writer.add_directory(&name, options)?;
        } else {
            writer.raw_copy_file(file)?;
        }
    }
    if !replaced {
        bail!("Source ZIP does not contain a root sps0.LoxCC");
    }

    let out = writer.finish()?;
    Ok(out.into_inner())
}

pub(crate) fn require_deployable_archive(data: &[u8]) -> Result<()> {
    if !data.starts_with(b"PK") {
        bail!(
            "Cannot deploy raw LoxCC/XML: cannot validate matching LoxAPP3.json/permissions.bin sidecars. Use Loxone Config, or a supported config ZIP workflow."
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn archive_error_adapter_preserves_cli_chains() {
        let source = zip::result::ZipError::FileNotFound;
        let expected = format!(
            "{:#}",
            anyhow::Error::new(source).context("Invalid ZIP archive")
        );
        let actual = archive_error(ArchiveError::InvalidZip {
            source: zip::result::ZipError::FileNotFound,
        });
        assert_eq!(format!("{actual:#}"), expected);

        let actual = archive_error(ArchiveError::ZipEntry {
            index: 4,
            source: zip::result::ZipError::FileNotFound,
        });
        let expected = anyhow::Error::new(zip::result::ZipError::FileNotFound);
        assert_eq!(format!("{actual:#}"), format!("{expected:#}"));

        let actual = archive_error(ArchiveError::ReadZipEntry {
            index: 4,
            name: "sps0.LoxCC".to_owned(),
            source: std::io::Error::other("entry read failed"),
        });
        let expected = anyhow::Error::new(std::io::Error::other("entry read failed"));
        assert_eq!(format!("{actual:#}"), format!("{expected:#}"));
    }

    #[test]
    fn test_repack_zip_roundtrip() {
        let xml = b"<?xml version=\"1.0\"?><ControlList Version=\"1\"/>";

        // Build a source ZIP with sps0.LoxCC + another file
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip_writer.start_file("sps0.LoxCC", options).unwrap();
        zip_writer.write_all(&compress_loxcc(xml)).unwrap();

        zip_writer.start_file("LoxAPP3.json", options).unwrap();
        zip_writer.write_all(b"{\"test\": true}").unwrap();

        let cursor = zip_writer.finish().unwrap();
        let src_zip = cursor.into_inner();

        // Repack with modified XML
        let new_xml = b"<?xml version=\"1.0\"?><ControlList Version=\"2\"/>";
        let repacked = repack_zip(&src_zip, new_xml).unwrap();

        // Verify the repacked ZIP contains the new XML
        let result = extract_and_decompress(&repacked).unwrap();
        assert_eq!(result, new_xml.as_slice());

        // Verify other entries are preserved
        let cursor = Cursor::new(&repacked);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut json_file = archive.by_name("LoxAPP3.json").unwrap();
        let mut json_data = Vec::new();
        json_file.read_to_end(&mut json_data).unwrap();
        assert_eq!(json_data, b"{\"test\": true}");
    }
}
