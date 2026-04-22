//! LoxCC compression and decompression — Loxone Miniserver config archives
//!
//! Backup ZIPs contain `sps0.LoxCC`, a custom LZ-compressed binary.
//! This module can decompress it to XML and compress XML back to LoxCC.
//!
//! ## Binary format
//!
//! ```text
//!   [0..4]   u32_le  magic (0xaabbccee)
//!   [4..8]   u32_le  compressed payload size
//!   [8..12]  u32_le  uncompressed size
//!   [12..16] u32_le  CRC32 of uncompressed data
//!   [16..]   compressed data (LZ4-style tokens)
//! ```
//!
//! The CRC32 field (offset 12) is **required** for the Miniserver to trust
//! encrypted config fields (e.g. `t="15"` password hashes). A zero CRC32
//! causes the Miniserver to load the structure but ignore sensitive fields.
//!
//! ## Compression tokens
//!
//! Each token byte encodes: `high nibble = literal count`, `low nibble =
//! match_length - 4`. If a nibble is 15, additional bytes follow (each 0–255,
//! stop at <255). After the literal bytes, a 2-byte LE back-reference offset
//! follows. The last block in the stream may omit the back-reference.
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
use crc32fast::Hasher as Crc32Hasher;
use std::io::{Cursor, Read, Write};

const LOXCC_MAGIC: u32 = 0xaabbccee;

/// Decompress a `sps0.LoxCC` binary blob into XML bytes.
///
/// See module docs for the binary format specification.
pub fn decompress_loxcc(data: &[u8]) -> Result<Vec<u8>> {
    if data.len() < 16 {
        bail!("LoxCC data too short ({} bytes)", data.len());
    }
    let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    if magic != LOXCC_MAGIC {
        bail!(
            "Not a valid LoxCC file (magic: 0x{:08x}, expected 0x{:08x})",
            magic,
            LOXCC_MAGIC
        );
    }
    let _comp_size = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
    let uncomp_hint = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;

    let compressed = &data[16..];
    // Cap allocation to prevent OOM from malicious headers (max 64MB)
    let capped_hint = uncomp_hint.clamp(1024, 64 * 1024 * 1024);
    let mut output = Vec::with_capacity(capped_hint);
    let mut pos = 0;

    while pos < compressed.len() {
        let token = compressed[pos];
        pos += 1;

        // High nibble: literal count
        let mut lit_len = (token >> 4) as usize;
        if lit_len == 15 {
            loop {
                if pos >= compressed.len() {
                    break;
                }
                let extra = compressed[pos] as usize;
                pos += 1;
                lit_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        // Copy literals
        if pos + lit_len > compressed.len() {
            // Allow partial final literal block (end of stream)
            let remaining = compressed.len() - pos;
            output.extend_from_slice(&compressed[pos..pos + remaining]);
            break;
        }
        output.extend_from_slice(&compressed[pos..pos + lit_len]);
        pos += lit_len;

        // End of compressed data — no match follows the last literal
        if pos >= compressed.len() {
            break;
        }

        // Back-reference offset (u16_le)
        if pos + 2 > compressed.len() {
            break;
        }
        let offset = u16::from_le_bytes([compressed[pos], compressed[pos + 1]]) as usize;
        pos += 2;

        if offset == 0 {
            bail!("LoxCC: zero back-reference offset at byte {}", pos - 2);
        }

        // Low nibble: match length (min 4)
        let mut match_len = (token & 0x0F) as usize + 4;
        if (token & 0x0F) == 15 {
            loop {
                if pos >= compressed.len() {
                    break;
                }
                let extra = compressed[pos] as usize;
                pos += 1;
                match_len += extra;
                if extra != 255 {
                    break;
                }
            }
        }

        // Copy from output buffer (byte-by-byte for overlapping runs)
        if offset > output.len() {
            bail!(
                "LoxCC: back-reference offset {} exceeds output size {}",
                offset,
                output.len()
            );
        }
        let start = output.len() - offset;
        for i in 0..match_len {
            let byte = output[start + (i % offset)];
            output.push(byte);
        }
    }

    Ok(output)
}

/// Extract `sps0.LoxCC` from a backup ZIP, then decompress to XML.
pub fn extract_and_decompress(zip_data: &[u8]) -> Result<Vec<u8>> {
    let cursor = Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor).context("Invalid ZIP archive")?;

    // Look for sps0.LoxCC (case-insensitive)
    let mut loxcc_data = None;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if file.name().to_lowercase().contains("sps0.loxcc") {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            loxcc_data = Some(buf);
            break;
        }
    }

    let data = loxcc_data.context("Backup ZIP does not contain sps0.LoxCC")?;
    decompress_loxcc(&data)
}

/// Compress raw XML bytes into LoxCC format.
///
/// Uses a literals-only encoding (no back-references) which produces larger
/// but perfectly valid LoxCC output. The Miniserver accepts this format and
/// loads it identically to the optimized compression used by Loxone Config.
///
/// The CRC32 of the uncompressed data is written into the header at offset 12,
/// which is required for the Miniserver to honour encrypted fields.
pub fn compress_loxcc(data: &[u8]) -> Vec<u8> {
    // Use proper LZ4 block compression for good compression ratio
    let compressed = lz4_flex::block::compress(data);

    // CRC32 of uncompressed data
    let mut hasher = Crc32Hasher::new();
    hasher.update(data);
    let checksum = hasher.finalize();

    // Build header: magic + payload_size + uncompressed_size + crc32
    let mut blob = Vec::with_capacity(16 + compressed.len());
    blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
    blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    blob.extend_from_slice(&(data.len() as u32).to_le_bytes());
    blob.extend_from_slice(&checksum.to_le_bytes());
    blob.extend_from_slice(&compressed);
    blob
}

/// Repack a backup ZIP, replacing `sps0.LoxCC` with recompressed XML.
///
/// All other entries (LoxAPP3.json, permissions.bin, etc.) are preserved
/// from the source ZIP.
pub fn repack_zip(src_zip: &[u8], xml: &[u8]) -> Result<Vec<u8>> {
    let new_loxcc = compress_loxcc(xml);
    let cursor = Cursor::new(src_zip);
    let mut archive = zip::ZipArchive::new(cursor).context("Invalid source ZIP")?;

    let buf = Vec::new();
    let out = Cursor::new(buf);
    let mut writer = zip::ZipWriter::new(out);

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        writer.start_file(&name, options)?;
        if name.to_lowercase() == "sps0.loxcc" {
            writer.write_all(&new_loxcc)?;
        } else {
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            writer.write_all(&data)?;
        }
    }

    let out = writer.finish()?;
    Ok(out.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Helper: build a LoxCC blob from raw XML using compress_loxcc.
    fn make_loxcc(xml: &[u8]) -> Vec<u8> {
        compress_loxcc(xml)
    }

    #[test]
    fn test_bad_magic() {
        let data = vec![0x00; 100];
        let err = decompress_loxcc(&data).unwrap_err();
        assert!(err.to_string().contains("Not a valid LoxCC"));
    }

    #[test]
    fn test_too_short() {
        let data = vec![0xee, 0xcc, 0xbb, 0xaa]; // just magic, no header
        let err = decompress_loxcc(&data).unwrap_err();
        assert!(err.to_string().contains("too short"));
    }

    #[test]
    fn test_compress_small() {
        let xml = b"<Config>Hello</Config>";
        let blob = compress_loxcc(xml);

        // Verify header
        let magic = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]);
        assert_eq!(magic, LOXCC_MAGIC);

        // Verify CRC32 is set (not zero)
        let crc = u32::from_le_bytes([blob[12], blob[13], blob[14], blob[15]]);
        let mut hasher = Crc32Hasher::new();
        hasher.update(xml);
        assert_eq!(crc, hasher.finalize());

        // Verify roundtrip
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn test_compress_large() {
        // Test with data > 15 bytes (triggers variable-length literal encoding)
        let xml = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"267\"/>";
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn test_compress_very_large() {
        // Test with data > 255+15 bytes (triggers multi-byte length encoding)
        let xml: Vec<u8> = std::iter::repeat(b'X').take(1000).collect();
        let blob = compress_loxcc(&xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn test_roundtrip_realistic_xml() {
        let xml = br#"<?xml version="1.0" encoding="utf-8"?>
<ControlList Version="267" LxAV="84">
  <C Type="Document" Title="TestProject">
    <C Type="Place" U="aaaa-bbbb" Title="Room1"/>
    <C Type="Plugin" gid="Mqtt" Title="MQTT">
      <mqtt_broker_address t="11" v="192.168.1.100"/>
      <mqtt_auth_pwd t="11" v="testpassword"/>
    </C>
  </C>
</ControlList>"#;
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml.as_slice());
    }

    #[test]
    fn test_decompress_with_backreference() {
        // Encode "ABCABCABC" using a literal "ABC" + back-reference
        let mut compressed = Vec::new();

        // Token: 3 literals, match_len - 4 = 2 (i.e., match_len = 6)
        compressed.push(0x32); // high=3 (literals), low=2 (match_extra)
        compressed.extend_from_slice(b"ABC"); // 3 literal bytes
        compressed.extend_from_slice(&3u16.to_le_bytes()); // offset = 3

        let mut blob = Vec::new();
        blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
        blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        blob.extend_from_slice(&9u32.to_le_bytes()); // uncompressed hint
        blob.extend_from_slice(&0u32.to_le_bytes()); // CRC32 (not checked by decompress)
        blob.extend_from_slice(&compressed);

        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, b"ABCABCABC"); // 3 literal + 6 from back-ref
    }

    #[test]
    fn test_extract_missing_loxcc() {
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut zip_writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        zip_writer.start_file("other.txt", options).unwrap();
        zip_writer.write_all(b"hello").unwrap();
        let cursor = zip_writer.finish().unwrap();
        let zip_data = cursor.into_inner();

        let err = extract_and_decompress(&zip_data).unwrap_err();
        assert!(err.to_string().contains("sps0.LoxCC"));
    }

    #[test]
    fn test_repack_zip_roundtrip() {
        let xml = b"<?xml version=\"1.0\"?><LoxoneProject/>";

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
        let new_xml = b"<?xml version=\"1.0\"?><LoxoneProject modified=\"true\"/>";
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

    // ── Additional LoxCC edge case tests ─────────────────────────────────

    #[test]
    fn test_compress_with_bom_prefix() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0xEF, 0xBB, 0xBF]); // UTF-8 BOM
        data.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><Config/>");
        let blob = compress_loxcc(&data);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, data);
        // Verify BOM is preserved
        assert_eq!(&result[..3], &[0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn test_compress_empty_xml() {
        let xml = b"";
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml.as_slice());
    }

    #[test]
    fn test_compress_single_byte() {
        let xml = b"X";
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml.as_slice());
    }

    #[test]
    fn test_compress_64kb_data() {
        // Test multi-byte length encoding with >64KB data
        let xml: Vec<u8> = (0..65536).map(|i| b"ABCDEFGHIJ"[i % 10]).collect();
        let blob = compress_loxcc(&xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn test_crc32_verification() {
        let xml = b"<Config>test data</Config>";
        let blob = compress_loxcc(xml);

        // Read back the CRC32 from the header
        let stored_crc = u32::from_le_bytes([blob[12], blob[13], blob[14], blob[15]]);

        // Compute expected CRC32
        let mut hasher = Crc32Hasher::new();
        hasher.update(xml);
        let expected_crc = hasher.finalize();

        assert_eq!(stored_crc, expected_crc);
        assert_ne!(stored_crc, 0); // CRC32 should never be zero for non-empty data
    }

    #[test]
    fn test_header_sizes_correct() {
        let xml = b"Hello World XML";
        let blob = compress_loxcc(xml);

        let compressed_size = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]) as usize;
        let uncompressed_size = u32::from_le_bytes([blob[8], blob[9], blob[10], blob[11]]) as usize;

        assert_eq!(uncompressed_size, xml.len());
        assert_eq!(compressed_size, blob.len() - 16); // payload = total - header
    }

    #[test]
    fn test_compress_exactly_15_bytes() {
        // Edge case: exactly 15 bytes triggers the boundary between nibble and extended length
        let xml = b"123456789012345";
        assert_eq!(xml.len(), 15);
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml.as_slice());
    }

    #[test]
    fn test_compress_exactly_14_bytes() {
        let xml = b"12345678901234";
        assert_eq!(xml.len(), 14);
        let blob = compress_loxcc(xml);
        let result = decompress_loxcc(&blob).unwrap();
        assert_eq!(result, xml.as_slice());
    }

    #[test]
    fn test_decompress_bad_offset() {
        // Construct a blob with a back-reference pointing beyond output
        let mut compressed = Vec::new();
        compressed.push(0x31); // 3 literals, match_len - 4 = 1 (match_len=5)
        compressed.extend_from_slice(b"ABC");
        compressed.extend_from_slice(&100u16.to_le_bytes()); // offset=100 > output(3)

        let mut blob = Vec::new();
        blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
        blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        blob.extend_from_slice(&100u32.to_le_bytes());
        blob.extend_from_slice(&0u32.to_le_bytes());
        blob.extend_from_slice(&compressed);

        let err = decompress_loxcc(&blob).unwrap_err();
        assert!(err.to_string().contains("back-reference offset"));
    }

    #[test]
    fn test_decompress_zero_offset() {
        let mut compressed = Vec::new();
        compressed.push(0x31); // 3 literals, match_len - 4 = 1
        compressed.extend_from_slice(b"ABC");
        compressed.extend_from_slice(&0u16.to_le_bytes()); // offset=0

        let mut blob = Vec::new();
        blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
        blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
        blob.extend_from_slice(&100u32.to_le_bytes());
        blob.extend_from_slice(&0u32.to_le_bytes());
        blob.extend_from_slice(&compressed);

        let err = decompress_loxcc(&blob).unwrap_err();
        assert!(err.to_string().contains("zero back-reference"));
    }
}
