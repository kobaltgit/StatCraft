use super::*;
use std::io::Cursor;

fn build_synthetic_header(compression: i32, cases: i32, nominal_vars: i32, bias: f64) -> Vec<u8> {
    let mut buf = vec![0u8; 176];
    buf[0..4].copy_from_slice(b"$FL2");
    // prod name
    let prod = b"@(#) SPSS DATA FILE Synthetic Test Generator";
    buf[4..4 + prod.len()].copy_from_slice(prod);
    // layout code = 2
    buf[64..68].copy_from_slice(&2i32.to_le_bytes());
    // nominal case size
    buf[68..72].copy_from_slice(&nominal_vars.to_le_bytes());
    // compression
    buf[72..76].copy_from_slice(&compression.to_le_bytes());
    // weight index = 0
    buf[76..80].copy_from_slice(&0i32.to_le_bytes());
    // case count
    buf[80..84].copy_from_slice(&cases.to_le_bytes());
    // bias
    buf[84..92].copy_from_slice(&bias.to_le_bytes());
    // date & time
    buf[92..101].copy_from_slice(b"29 Oct 26");
    buf[101..109].copy_from_slice(b"12:00:00");
    buf
}

fn build_var_record(type_code: i32, name: &str, label: Option<&str>, print_fmt: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&2i32.to_le_bytes()); // rec_type
    buf.extend_from_slice(&type_code.to_le_bytes());
    let has_label = if label.is_some() { 1i32 } else { 0i32 };
    buf.extend_from_slice(&has_label.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes()); // missing code 0
    buf.extend_from_slice(&print_fmt.to_le_bytes()); // print format
    buf.extend_from_slice(&print_fmt.to_le_bytes()); // write format

    let mut name_bytes = [b' '; 8];
    let n = name.as_bytes();
    let copy_len = n.len().min(8);
    name_bytes[..copy_len].copy_from_slice(&n[..copy_len]);
    buf.extend_from_slice(&name_bytes);

    if let Some(lbl) = label {
        let lbl_bytes = lbl.as_bytes();
        let len = lbl_bytes.len() as i32;
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(lbl_bytes);
        let pad = (4 - (len % 4)) % 4;
        buf.resize(buf.len() + pad as usize, 0);
    }
    buf
}

#[test]
fn test_synthetic_sav_bytecode_compression() {
    let mut file_bytes = build_synthetic_header(1, 2, 2, 100.0);

    // Variable 1: Numeric VAR00001
    file_bytes.extend(build_var_record(
        0,
        "VAR00001",
        Some("Respondent Age"),
        0x00050800,
    ));
    // Variable 2: String VAR00002 length 8
    file_bytes.extend(build_var_record(
        8,
        "VAR00002",
        Some("Respondent Name"),
        0x00010800,
    ));

    // Record 7, subtype 13: Long names
    let long_names_str = "VAR00001=age\tVAR00002=name\0";
    let ln_bytes = long_names_str.as_bytes();
    file_bytes.extend_from_slice(&7i32.to_le_bytes());
    file_bytes.extend_from_slice(&13i32.to_le_bytes()); // subtype
    file_bytes.extend_from_slice(&1i32.to_le_bytes()); // size = 1
    file_bytes.extend_from_slice(&(ln_bytes.len() as i32).to_le_bytes());
    file_bytes.extend_from_slice(ln_bytes);

    // Record 999: Data
    file_bytes.extend_from_slice(&999i32.to_le_bytes());
    file_bytes.extend_from_slice(&0i32.to_le_bytes()); // filler

    // Case 1: age = 25 (code 125), name = "Alice   " (code 253 + 8 bytes)
    // Case 2: age = SYSMIS (code 255), name = "Bob     " (code 253 + 8 bytes)
    // End: code 252
    let mut opcodes = [0u8; 8];
    opcodes[0] = 125; // 125 - 100 = 25
    opcodes[1] = 253; // literal Alice
    opcodes[2] = 255; // sysmis
    opcodes[3] = 253; // literal Bob
    opcodes[4] = 252; // EOF

    file_bytes.extend_from_slice(&opcodes);
    file_bytes.extend_from_slice(b"Alice   ");
    file_bytes.extend_from_slice(b"Bob     ");

    let mut cursor = Cursor::new(file_bytes);
    let spss_data = load_spss_from_reader(&mut cursor).expect("Failed to load synthetic SPSS");

    assert_eq!(spss_data.variables.len(), 2);
    assert_eq!(spss_data.variables[0].name, "age");
    assert_eq!(spss_data.variables[0].label, "Respondent Age");
    assert_eq!(spss_data.variables[0].var_type, RawDataType::Numeric);

    assert_eq!(spss_data.variables[1].name, "name");
    assert_eq!(spss_data.variables[1].label, "Respondent Name");
    assert_eq!(spss_data.variables[1].var_type, RawDataType::Text);

    assert_eq!(spss_data.raw_table.row_count, 2);
    assert_eq!(
        spss_data.raw_table.columns[0].values[0],
        RawValue::Number(25.0)
    );
    assert_eq!(spss_data.raw_table.columns[0].values[1], RawValue::Null);

    assert_eq!(
        spss_data.raw_table.columns[1].values[0],
        RawValue::Text("Alice".to_string())
    );
    assert_eq!(
        spss_data.raw_table.columns[1].values[1],
        RawValue::Text("Bob".to_string())
    );
}

#[test]
fn test_synthetic_sav_uncompressed() {
    let mut file_bytes = build_synthetic_header(0, 1, 1, 100.0);
    file_bytes.extend(build_var_record(0, "SCORE", None, 0x00050802));

    file_bytes.extend_from_slice(&999i32.to_le_bytes());
    file_bytes.extend_from_slice(&0i32.to_le_bytes());

    // 1 row with score = 95.5
    file_bytes.extend_from_slice(&95.5f64.to_le_bytes());

    let mut cursor = Cursor::new(file_bytes);
    let spss_data = load_spss_from_reader(&mut cursor).expect("Failed to load uncompressed");

    assert_eq!(spss_data.raw_table.row_count, 1);
    assert_eq!(
        spss_data.raw_table.columns[0].values[0],
        RawValue::Number(95.5)
    );
}

#[test]
fn test_malformed_spss_data_returns_error() {
    // 1. Truncated header
    let mut cursor = Cursor::new(vec![0u8; 50]);
    assert!(load_spss_from_reader(&mut cursor).is_err());

    // 2. Invalid magic
    let mut invalid_magic = vec![0u8; 176];
    invalid_magic[0..4].copy_from_slice(b"XXXX");
    let mut cursor = Cursor::new(invalid_magic);
    assert!(matches!(
        load_spss_from_reader(&mut cursor),
        Err(IoError::MalformedSpss(_))
    ));

    // 3. Negative bias
    let bad_bias = build_synthetic_header(1, 1, 1, -50.0);
    let mut cursor = Cursor::new(bad_bias);
    assert!(matches!(
        load_spss_from_reader(&mut cursor),
        Err(IoError::MalformedSpss(_))
    ));

    // 4. Invalid record type
    let mut bad_rec = build_synthetic_header(1, 1, 1, 100.0);
    bad_rec.extend_from_slice(&12345i32.to_le_bytes()); // invalid rec type
    let mut cursor = Cursor::new(bad_rec);
    assert!(matches!(
        load_spss_from_reader(&mut cursor),
        Err(IoError::MalformedSpss(_))
    ));
}
