use acadrust::entities::{EntityType, Line};
use acadrust::{CadDocument, DxfVersion, DxfWriter, Vector3};

fn document() -> CadDocument {
    let mut document = CadDocument::with_version(DxfVersion::AC1018);
    let line = Line::from_points(
        Vector3::new(0.0, 0.0, 0.0),
        Vector3::new(1.23456789, 2.34567891, 0.0),
    );
    document.add_entity(EntityType::Line(line)).unwrap();
    document
}

fn write(places: Option<u8>) -> String {
    let doc = document();
    let mut writer = DxfWriter::new(&doc);
    if let Some(p) = places {
        writer = writer.with_decimal_places(p);
    }
    String::from_utf8(writer.write_to_vec().unwrap()).unwrap()
}

#[test]
fn decimal_places_rounds_the_written_coordinates() {
    let dxf = write(Some(2));
    assert!(dxf.contains("\r\n1.23\r\n"), "endpoint X rounds to 2 places");
    assert!(dxf.contains("\r\n2.35\r\n"), "endpoint Y rounds to 2 places");
    assert!(
        !dxf.contains("1.23456789"),
        "the full-precision value must be gone"
    );
}

#[test]
fn default_keeps_full_precision() {
    let dxf = write(None);
    assert!(
        dxf.contains("1.23456789"),
        "the default keeps the shortest round-trip value"
    );
}

#[test]
fn decimal_places_zero_writes_integer_coordinates() {
    let dxf = write(Some(0));
    assert!(dxf.contains(" 11\r\n1\r\n"), "endpoint X rounds to an integer");
    assert!(dxf.contains(" 21\r\n2\r\n"), "endpoint Y rounds to an integer");
}