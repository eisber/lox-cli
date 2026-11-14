use super::*;
use std::io::Write;

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
    let data = vec![0xee, 0xcc, 0xbb, 0xaa];
    let err = decompress_loxcc(&data).unwrap_err();
    assert!(err.to_string().contains("too short"));
}

#[test]
fn test_compress_small() {
    let xml = b"<Config>Hello</Config>";
    let blob = compress_loxcc(xml);

    let magic = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]);
    assert_eq!(magic, LOXCC_MAGIC);

    let crc = u32::from_le_bytes([blob[12], blob[13], blob[14], blob[15]]);
    let mut hasher = Crc32Hasher::new();
    hasher.update(xml);
    assert_eq!(crc, hasher.finalize());

    let result = decompress_loxcc(&blob).unwrap();
    assert_eq!(result, xml);
}

#[test]
fn test_compress_large() {
    let xml = b"<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ControlList Version=\"267\"/>";
    let blob = compress_loxcc(xml);
    let result = decompress_loxcc(&blob).unwrap();
    assert_eq!(result, xml);
}

#[test]
fn test_compress_very_large() {
    let xml: Vec<u8> = std::iter::repeat_n(b'X', 1000).collect();
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
    let mut compressed = vec![0x32];
    compressed.extend_from_slice(b"ABC");
    compressed.extend_from_slice(&3u16.to_le_bytes());

    let mut blob = Vec::new();
    blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
    blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    blob.extend_from_slice(&9u32.to_le_bytes());
    blob.extend_from_slice(&0u32.to_le_bytes());
    blob.extend_from_slice(&compressed);

    let result = decompress_loxcc(&blob).unwrap();
    assert_eq!(result, b"ABCABCABC");
}

#[test]
fn test_extract_missing_loxcc() {
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut zip_writer = zip::ZipWriter::new(cursor);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip_writer.start_file("other.txt", options).unwrap();
    zip_writer.write_all(b"hello").unwrap();
    let cursor = zip_writer.finish().unwrap();
    let zip_data = cursor.into_inner();

    let err = extract_and_decompress(&zip_data).unwrap_err();
    assert!(err.to_string().contains("sps0.LoxCC"));
}

#[test]
fn test_extract_raw_container() {
    let xml = b"<?xml version=\"1.0\"?><Resources><Block Name=\"Example\"/></Resources>";
    assert_eq!(extract_and_decompress(&make_loxcc(xml)).unwrap(), xml);
}

#[test]
fn test_extract_truncated_raw_header() {
    let err = extract_and_decompress(&LOXCC_MAGIC.to_le_bytes()).unwrap_err();
    assert!(err.to_string().contains("too short"));
}

#[test]
fn test_compress_with_bom_prefix() {
    let mut data = Vec::new();
    data.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    data.extend_from_slice(b"<?xml version=\"1.0\" encoding=\"utf-8\"?><Config/>");
    let blob = compress_loxcc(&data);
    let result = decompress_loxcc(&blob).unwrap();
    assert_eq!(result, data);
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
    let xml: Vec<u8> = (0..65536).map(|i| b"ABCDEFGHIJ"[i % 10]).collect();
    let blob = compress_loxcc(&xml);
    let result = decompress_loxcc(&blob).unwrap();
    assert_eq!(result, xml);
}

#[test]
fn test_crc32_verification() {
    let xml = b"<Config>test data</Config>";
    let blob = compress_loxcc(xml);
    let stored_crc = u32::from_le_bytes([blob[12], blob[13], blob[14], blob[15]]);

    let mut hasher = Crc32Hasher::new();
    hasher.update(xml);
    let expected_crc = hasher.finalize();

    assert_eq!(stored_crc, expected_crc);
    assert_ne!(stored_crc, 0);
}

#[test]
fn test_header_sizes_correct() {
    let xml = b"Hello World XML";
    let blob = compress_loxcc(xml);
    let compressed_size = u32::from_le_bytes([blob[4], blob[5], blob[6], blob[7]]) as usize;
    let uncompressed_size = u32::from_le_bytes([blob[8], blob[9], blob[10], blob[11]]) as usize;

    assert_eq!(uncompressed_size, xml.len());
    assert_eq!(compressed_size, blob.len() - 16);
}

#[test]
fn test_compress_exactly_15_bytes() {
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
    let mut compressed = vec![0x31];
    compressed.extend_from_slice(b"ABC");
    compressed.extend_from_slice(&100u16.to_le_bytes());

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
    let mut compressed = vec![0x31];
    compressed.extend_from_slice(b"ABC");
    compressed.extend_from_slice(&0u16.to_le_bytes());

    let mut blob = Vec::new();
    blob.extend_from_slice(&LOXCC_MAGIC.to_le_bytes());
    blob.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    blob.extend_from_slice(&100u32.to_le_bytes());
    blob.extend_from_slice(&0u32.to_le_bytes());
    blob.extend_from_slice(&compressed);

    let err = decompress_loxcc(&blob).unwrap_err();
    assert!(err.to_string().contains("zero back-reference"));
}
