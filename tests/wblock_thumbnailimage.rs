use std::io::Cursor;

use acadrust::{CadDocument, DxfReader, DxfVersion, DxfWriter, Vector3};

fn document() -> CadDocument {
    let mut document = CadDocument::with_version(DxfVersion::AC1018);
    let line = acadrust::entities::Line::from_points(Vector3::ZERO, Vector3::new(1.0, 1.0, 0.0));
    document
        .add_entity(acadrust::entities::EntityType::Line(line))
        .unwrap();
    document
}

/// A deterministic payload long enough to span several 127-byte chunks.
fn payload() -> Vec<u8> {
    (0..300).map(|i| (i % 256) as u8).collect()
}

/// Isolate the THUMBNAILIMAGE section body (between its name and ENDSEC).
fn thumbnail_section(dxf: &str) -> Option<&str> {
    let start = dxf.find("THUMBNAILIMAGE")?;
    let rest = &dxf[start..];
    let end = rest.find("ENDSEC")?;
    Some(&rest[..end])
}

/// Decode the hex carried by the section's 310 chunks back into bytes.
fn decode_chunks(section: &str) -> Vec<u8> {
    let mut hex = String::new();
    let mut lines = section.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "310" {
            if let Some(value) = lines.next() {
                hex.push_str(value.trim());
            }
        }
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn preview_writes_a_thumbnailimage_section() {
    let source = document();
    let bytes = DxfWriter::new(&source)
        .with_preview(payload())
        .write_to_vec()
        .unwrap();
    let dxf = String::from_utf8(bytes).unwrap();
    let section = thumbnail_section(&dxf).expect("a THUMBNAILIMAGE section");
    assert!(
        section.contains("300"),
        "the byte count is the payload length"
    );
    assert_eq!(decode_chunks(section), payload());
}

#[test]
fn preview_chunks_are_127_bytes_of_uppercase_hex() {
    let source = document();
    let bytes = DxfWriter::new(&source)
        .with_preview(payload())
        .write_to_vec()
        .unwrap();
    let dxf = String::from_utf8(bytes).unwrap();
    let section = thumbnail_section(&dxf).expect("a THUMBNAILIMAGE section");
    let mut chunk_lengths = Vec::new();
    let mut lines = section.lines();
    while let Some(line) = lines.next() {
        if line.trim() == "310" {
            if let Some(value) = lines.next() {
                let value = value.trim();
                assert!(
                    value
                        .chars()
                        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()),
                    "chunks are uppercase hex: {value}"
                );
                chunk_lengths.push(value.len());
            }
        }
    }
    // 300 bytes = 127 + 127 + 46 -> two full 254-char chunks and a 92-char tail.
    assert_eq!(chunk_lengths, vec![254, 254, 92]);
}

#[test]
fn no_preview_section_by_default() {
    let source = document();
    let bytes = DxfWriter::new(&source).write_to_vec().unwrap();
    let dxf = String::from_utf8(bytes).unwrap();
    assert!(!dxf.contains("THUMBNAILIMAGE"));
}

#[test]
fn binary_output_carries_no_preview() {
    let source = document();
    let bytes = DxfWriter::new_binary(&source)
        .with_preview(payload())
        .write_to_vec()
        .unwrap();
    assert!(!bytes.windows(14).any(|w| w == b"THUMBNAILIMAGE"));
}

#[test]
fn preview_survives_a_read_write_cycle() {
    let source = document();
    let bytes = DxfWriter::new(&source)
        .with_preview(payload())
        .write_to_vec()
        .unwrap();
    // The reader skips the section, but the file must still parse cleanly.
    let restored = DxfReader::from_reader(Cursor::new(bytes))
        .unwrap()
        .read()
        .unwrap();
    assert_eq!(restored.entities().count(), 1);
}
