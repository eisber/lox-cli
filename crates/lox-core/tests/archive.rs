use lox_core::configuration::archive::{
    ArchiveError, compress_loxcc, decompress_loxcc, extract_and_decompress,
};
use std::error::Error;
use std::io::{Cursor, Write};

fn backup(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        writer.start_file(*name, options).unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

fn raw_blob(payload: &[u8]) -> Vec<u8> {
    let mut blob = 0xaabbcceeu32.to_le_bytes().to_vec();
    blob.extend_from_slice(&[0; 12]);
    blob.extend_from_slice(payload);
    blob
}

#[test]
fn public_codec_preserves_opaque_bytes_and_first_matching_zip_member() {
    let bytes = b"\xef\xbb\xbf<C 1x='001' A=\"a\nb\" A=\"a\nb\"/>\r\n\xff\0";
    let compressed = compress_loxcc(bytes);
    let checksum = u32::from_le_bytes(compressed[12..16].try_into().unwrap());
    assert_eq!(checksum, crc32fast::hash(bytes));
    assert_eq!(decompress_loxcc(&compressed).unwrap(), bytes);
    assert_eq!(extract_and_decompress(&compressed).unwrap(), bytes);

    let second = compress_loxcc(b"not the first member");
    let zip = backup(&[
        ("unrelated", b"skip me"),
        ("nested/prefix-sPs0.LoXcC.backup", &compressed),
        ("sps0.LoxCC", &second),
    ]);
    assert_eq!(extract_and_decompress(&zip).unwrap(), bytes);
}

#[test]
fn public_errors_expose_decode_fields_and_zip_sources() {
    assert!(matches!(
        decompress_loxcc(&[0; 3]),
        Err(ArchiveError::HeaderTooShort { actual: 3 })
    ));
    assert!(matches!(
        decompress_loxcc(&[0; 16]),
        Err(ArchiveError::InvalidMagic { actual: 0 })
    ));
    assert!(matches!(
        decompress_loxcc(&raw_blob(&[0x10, b'A', 0, 0])),
        Err(ArchiveError::ZeroBackReference { position: 2 })
    ));
    assert!(matches!(
        decompress_loxcc(&raw_blob(&[0x10, b'A', 2, 0])),
        Err(ArchiveError::BackReferenceOutOfBounds {
            offset: 2,
            output_len: 1
        })
    ));

    let error = extract_and_decompress(b"not a zip").unwrap_err();
    assert!(matches!(&error, ArchiveError::InvalidZip { .. }));
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<zip::result::ZipError>()
            .is_some()
    );
    assert!(matches!(
        extract_and_decompress(&backup(&[("other", b"ignored")])),
        Err(ArchiveError::MissingConfigEntry)
    ));
}

#[test]
fn public_zip_read_error_retains_member_identity_and_io_source() {
    let mut zip = backup(&[("sps0.LoxCC", &compress_loxcc(b"opaque bytes"))]);
    let start = zip::ZipArchive::new(Cursor::new(&zip))
        .unwrap()
        .by_index(0)
        .unwrap()
        .data_start() as usize;
    zip[start] ^= 1;
    let error = extract_and_decompress(&zip).unwrap_err();
    match &error {
        ArchiveError::ReadZipEntry { index, name, .. } => {
            assert_eq!(*index, 0);
            assert_eq!(name, "sps0.LoxCC");
        }
        other => panic!("expected a member read error, got {other:?}"),
    }
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
}

#[test]
fn compatibility_decoding_is_not_strict_verification() {
    let mut blob = compress_loxcc(b"unchanged bytes");
    blob[4..16].fill(0);
    assert_eq!(decompress_loxcc(&blob).unwrap(), b"unchanged bytes");

    for (payload, expected) in [
        (&[0x50, b'A', b'B'][..], &b"AB"[..]),
        (&[0x10, b'A', 1][..], &b"A"[..]),
        (&[0xf0, 255][..], &b""[..]),
    ] {
        assert_eq!(decompress_loxcc(&raw_blob(payload)).unwrap(), expected);
    }
}
