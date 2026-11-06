use std::fs::{self, File};
use std::io::Cursor;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Instant;

use statcraft_io::{
    load_csv_from_path, load_csv_from_reader, load_excel_from_path, load_spss_from_path,
    load_spss_from_reader, load_stc_container, load_stc_container_from_path, save_stc_container,
    CsvOptions, RawColumn, RawDataType, RawTable, RawValue, StcContainer,
};

pub struct CorpusStats {
    pub total_scanned: usize,
    pub valid_passed: usize,
    pub safe_errors: usize,
    pub fuzzed_passed: usize,
    pub panics: usize,
}

pub fn run_test_corpus(workspace_root: &Path, custom_dir: Option<&Path>) -> Result<(), String> {
    println!("=======================================================");
    println!("  StatCraft Real-World Corpora & Codecs Audit Runner  ");
    println!("  Clean-Room Standards & Never Crash Verification     ");
    println!("=======================================================\n");

    let corpus_dir = match custom_dir {
        Some(d) => d.to_path_buf(),
        None => workspace_root.join("fixtures").join("corpus"),
    };

    create_fixtures_if_needed(&corpus_dir)?;

    let mut stats = CorpusStats {
        total_scanned: 0,
        valid_passed: 0,
        safe_errors: 0,
        fuzzed_passed: 0,
        panics: 0,
    };

    // 1. Scan and test corpus files from directory
    scan_and_test_directory(&corpus_dir, &mut stats)?;

    // Also check test_data/ if it exists
    let test_data_dir = workspace_root.join("test_data");
    if test_data_dir.exists() && test_data_dir != corpus_dir {
        println!("\n[INFO] Scanning supplementary dataset files in 'test_data/'...");
        scan_and_test_directory(&test_data_dir, &mut stats)?;
    }

    // 2. Run in-memory Malformed Input Fuzzing Suite
    println!("\n=== Running Malformed Input Fuzzing Suite (Never Crash Invariant) ===");
    run_malformed_fuzzing_suite(&mut stats)?;

    // 3. Print Summary
    println!("\n=======================================================");
    println!("  CORPUS AUDIT SUMMARY");
    println!("  Total Scanned Files : {}", stats.total_scanned);
    println!("  Valid Files OK      : {}", stats.valid_passed);
    println!("  Corrupt Safe Errors : {}", stats.safe_errors);
    println!("  Fuzzed Probes OK    : {}", stats.fuzzed_passed);
    println!("  Panics Detected     : {}", stats.panics);
    println!("=======================================================\n");

    if stats.panics > 0 {
        Err(format!(
            "CRITICAL: {} panic(s) detected during corpus execution! Never Crash invariant violated.",
            stats.panics
        ))
    } else {
        println!(">>> [SUCCESS] All corpora and codecs passed with ZERO panics!");
        Ok(())
    }
}

fn scan_and_test_directory(dir: &Path, stats: &mut CorpusStats) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory '{}': {}", dir.display(), e))?
        .filter_map(|e| e.ok().map(|ent| ent.path()))
        .filter(|p| p.is_file())
        .collect();
    entries.sort();

    println!(
        "{:<35} | {:<8} | {:<16} | {:<12} | {:<8}",
        "File", "Format", "Status", "Dim (R x C)", "Time"
    );
    println!(
        "{:-<35}-+-{:-<8}-+-{:-<16}-+-{:-<12}-+-{:-<8}",
        "", "", "", "", ""
    );

    for path in entries {
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();

        stats.total_scanned += 1;
        let start = Instant::now();

        let path_clone = path.clone();
        let ext_clone = ext.clone();

        let unwind_res = panic::catch_unwind(AssertUnwindSafe(|| {
            test_single_file(&path_clone, &ext_clone)
        }));

        let elapsed_ms = start.elapsed().as_millis();

        match unwind_res {
            Ok(Ok((rows, cols))) => {
                stats.valid_passed += 1;
                println!(
                    "{:<35} | {:<8} | {:<16} | {:<12} | {:>5} ms",
                    truncate_str(&file_name, 35),
                    ext,
                    "[PASS] Loaded OK",
                    format!("{} x {}", rows, cols),
                    elapsed_ms
                );
            }
            Ok(Err(err_msg)) => {
                stats.safe_errors += 1;
                let is_expected_corrupt = file_name.contains("corrupt");
                let status = if is_expected_corrupt {
                    "[PASS] Safe Err"
                } else {
                    "[WARN] Err (Safe)"
                };
                println!(
                    "{:<35} | {:<8} | {:<16} | {:<12} | {:>5} ms",
                    truncate_str(&file_name, 35),
                    ext,
                    status,
                    truncate_str(&err_msg, 12),
                    elapsed_ms
                );
            }
            Err(_) => {
                stats.panics += 1;
                eprintln!(
                    "{:<35} | {:<8} | {:<16} | {:<12} | {:>5} ms",
                    truncate_str(&file_name, 35),
                    ext,
                    "[FAIL] PANIC!",
                    "-",
                    elapsed_ms
                );
            }
        }
    }

    Ok(())
}

fn test_single_file(path: &Path, ext: &str) -> Result<(usize, usize), String> {
    match ext {
        "sav" | "zsav" => {
            let data = load_spss_from_path(path).map_err(|e| format!("{:?}", e))?;
            Ok((data.raw_table.row_count, data.raw_table.col_count()))
        }
        "csv" | "tsv" => {
            let table =
                load_csv_from_path(path, CsvOptions::default()).map_err(|e| format!("{:?}", e))?;
            Ok((table.row_count, table.col_count()))
        }
        "xlsx" | "xls" | "ods" => {
            let table = load_excel_from_path(path, None).map_err(|e| format!("{:?}", e))?;
            Ok((table.row_count, table.col_count()))
        }
        "stc" => {
            let container = load_stc_container_from_path(path).map_err(|e| format!("{:?}", e))?;
            Ok((container.table.row_count, container.table.col_count()))
        }
        _ => Err(format!("Unsupported format: {}", ext)),
    }
}

fn run_malformed_fuzzing_suite(stats: &mut CorpusStats) -> Result<(), String> {
    let fuzzed_payloads: Vec<(&str, Vec<u8>)> = vec![
        ("Zero bytes", vec![]),
        ("Single null byte", vec![0u8]),
        ("Truncated SPSS magic", b"$FL".to_vec()),
        (
            "Random garbage 16B",
            (0..16).map(|i| (i * 37) as u8).collect(),
        ),
        ("Truncated SPSS header 50B", vec![0u8; 50]),
        ("SPSS bad magic 176B", {
            let mut v = vec![0u8; 176];
            v[0..4].copy_from_slice(b"BADM");
            v
        }),
        ("SPSS negative bias 176B", {
            let mut v = vec![0u8; 176];
            v[0..4].copy_from_slice(b"$FL2");
            v[64..68].copy_from_slice(&2i32.to_le_bytes());
            v[68..72].copy_from_slice(&1i32.to_le_bytes());
            v[84..92].copy_from_slice(&(-100.0f64).to_le_bytes());
            v
        }),
        ("Corrupt GZIP header 4B", vec![0x1f, 0x8b, 0xff, 0xff]),
        (
            "Truncated JSON STC",
            b"{\"format_version\":1,\"table\":".to_vec(),
        ),
        (
            "CSV unclosed quotes",
            b"col1,col2\n\"unclosed quote,val".to_vec(),
        ),
        (
            "CSV null characters",
            vec![b'a', 0, b'b', b',', b'c', 0, b'd'],
        ),
    ];

    for (name, payload) in fuzzed_payloads {
        let name_str = name.to_string();
        let payload_clone = payload.clone();

        let unwind_res = panic::catch_unwind(AssertUnwindSafe(move || {
            // Test 1: load_spss_from_reader
            let mut c1 = Cursor::new(&payload_clone);
            let _ = load_spss_from_reader(&mut c1);

            // Test 2: load_csv_from_reader
            let c2 = Cursor::new(&payload_clone);
            let _ = load_csv_from_reader(c2, CsvOptions::default());

            // Test 3: load_stc_container
            let c3 = Cursor::new(&payload_clone);
            let _ = load_stc_container(c3);
        }));

        match unwind_res {
            Ok(()) => {
                stats.fuzzed_passed += 1;
                println!("  [FUZZ OK] {:<30} -> Safe typed Err (0 panics)", name_str);
            }
            Err(_) => {
                stats.panics += 1;
                eprintln!("  [FUZZ CRITICAL PANIC] {:<30} -> Panicked!", name_str);
            }
        }
    }

    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}

fn create_fixtures_if_needed(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir)
        .map_err(|e| format!("Cannot create fixtures dir '{}': {}", dir.display(), e))?;

    // 1. valid_survey.csv
    let csv_path = dir.join("valid_survey.csv");
    if !csv_path.exists() {
        let content = "id,age,salary,gender\n1,25,50000.0,Female\n2,34,65000.5,Male\n3,42,NA,Female\n4,28,48000.0,Male\n";
        fs::write(&csv_path, content).map_err(|e| e.to_string())?;
    }

    // 2. multiline_bom.csv
    let bom_path = dir.join("multiline_bom.csv");
    if !bom_path.exists() {
        let content = "\u{FEFF}id,feedback,status\n1,\"Great service!\nVery responsive\",Active\n2,\"He said \"\"super\"\"\",Pending\n";
        fs::write(&bom_path, content).map_err(|e| e.to_string())?;
    }

    // 3. semicolon_european.csv
    let semi_path = dir.join("semicolon_european.csv");
    if !semi_path.exists() {
        let content = "id;price;rating\n101;19,99;4.8\n102;49,50;4.5\n103;120,00;5.0\n";
        fs::write(&semi_path, content).map_err(|e| e.to_string())?;
    }

    // 4. pipe_delimited.csv
    let pipe_path = dir.join("pipe_delimited.csv");
    if !pipe_path.exists() {
        let content = "item_id|sku|stock\n1|SKU-001|150\n2|SKU-002|45\n";
        fs::write(&pipe_path, content).map_err(|e| e.to_string())?;
    }

    // 5. valid_survey.stc
    let stc_path = dir.join("valid_survey.stc");
    if !stc_path.exists() {
        let table = RawTable {
            columns: vec![
                RawColumn {
                    name: "id".to_string(),
                    data_type: RawDataType::Numeric,
                    values: vec![RawValue::Number(1.0), RawValue::Number(2.0)],
                },
                RawColumn {
                    name: "group".to_string(),
                    data_type: RawDataType::Text,
                    values: vec![
                        RawValue::Text("Control".to_string()),
                        RawValue::Text("Treatment".to_string()),
                    ],
                },
            ],
            row_count: 2,
        };
        let container =
            StcContainer::new("SurveyProject", r#"{"vars":[]}"#, table, None, None, None);
        let f = File::create(&stc_path).map_err(|e| e.to_string())?;
        save_stc_container(f, &container).map_err(|e| format!("{:?}", e))?;
    }

    // 6. corrupt_truncated.sav
    let trunc_path = dir.join("corrupt_truncated.sav");
    if !trunc_path.exists() {
        let mut buf = vec![0u8; 40];
        buf[0..4].copy_from_slice(b"$FL2");
        fs::write(&trunc_path, buf).map_err(|e| e.to_string())?;
    }

    // 7. corrupt_bad_magic.sav
    let bad_magic_path = dir.join("corrupt_bad_magic.sav");
    if !bad_magic_path.exists() {
        let mut buf = vec![0u8; 176];
        buf[0..4].copy_from_slice(b"NOPE");
        fs::write(&bad_magic_path, buf).map_err(|e| e.to_string())?;
    }

    // 8. synthetic_clean.sav
    let clean_sav_path = dir.join("synthetic_clean.sav");
    if !clean_sav_path.exists() {
        let mut buf = vec![0u8; 176];
        buf[0..4].copy_from_slice(b"$FL2");
        buf[64..68].copy_from_slice(&2i32.to_le_bytes()); // layout
        buf[68..72].copy_from_slice(&1i32.to_le_bytes()); // nominal vars
        buf[72..76].copy_from_slice(&0i32.to_le_bytes()); // uncompressed
        buf[80..84].copy_from_slice(&1i32.to_le_bytes()); // 1 case
        buf[84..92].copy_from_slice(&100.0f64.to_le_bytes()); // bias
                                                              // Record 2: VAR00001
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes()); // numeric
        buf.extend_from_slice(&0i32.to_le_bytes()); // no label
        buf.extend_from_slice(&0i32.to_le_bytes()); // missing
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(b"VAR00001");
        // Record 999
        buf.extend_from_slice(&999i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        // Value: 42.0
        buf.extend_from_slice(&42.0f64.to_le_bytes());
        fs::write(&clean_sav_path, buf).map_err(|e| e.to_string())?;
    }

    // 9. synthetic_compressed.sav
    let comp_sav_path = dir.join("synthetic_compressed.sav");
    if !comp_sav_path.exists() {
        let mut buf = vec![0u8; 176];
        buf[0..4].copy_from_slice(b"$FL2");
        buf[64..68].copy_from_slice(&2i32.to_le_bytes());
        buf[68..72].copy_from_slice(&1i32.to_le_bytes());
        buf[72..76].copy_from_slice(&1i32.to_le_bytes()); // compression = 1
        buf[80..84].copy_from_slice(&3i32.to_le_bytes()); // 3 cases
        buf[84..92].copy_from_slice(&100.0f64.to_le_bytes()); // bias = 100
                                                              // Record 2
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(b"VAR00001");
        // Record 999
        buf.extend_from_slice(&999i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        // Opcodes: 125 (25.0), 254 (0.0), 255 (Null), 252 (EOF)
        let mut opcodes = [0u8; 8];
        opcodes[0] = 125;
        opcodes[1] = 254;
        opcodes[2] = 255;
        opcodes[3] = 252;
        buf.extend_from_slice(&opcodes);
        fs::write(&comp_sav_path, buf).map_err(|e| e.to_string())?;
    }

    // 10. synthetic_cyrillic.sav
    let cyr_sav_path = dir.join("synthetic_cyrillic.sav");
    if !cyr_sav_path.exists() {
        let mut buf = vec![0u8; 176];
        buf[0..4].copy_from_slice(b"$FL2");
        buf[64..68].copy_from_slice(&2i32.to_le_bytes());
        buf[68..72].copy_from_slice(&1i32.to_le_bytes());
        buf[72..76].copy_from_slice(&1i32.to_le_bytes());
        buf[80..84].copy_from_slice(&1i32.to_le_bytes());
        buf[84..92].copy_from_slice(&100.0f64.to_le_bytes());

        // Variable with Russian CP1251 label "Возраст"
        let cp1251_label = [0xC2, 0xEE, 0xE7, 0xF0, 0xE0, 0xF1, 0xF2];
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&1i32.to_le_bytes()); // has label
        buf.extend_from_slice(&0i32.to_le_bytes());
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(&0x00050800u32.to_le_bytes());
        buf.extend_from_slice(b"AGE     ");
        buf.extend_from_slice(&(cp1251_label.len() as i32).to_le_bytes());
        buf.extend_from_slice(&cp1251_label);
        let pad = (4 - (cp1251_label.len() % 4)) % 4;
        buf.resize(buf.len() + pad, 0);

        // Record 7 Subtype 3 (Codepage 1251)
        buf.extend_from_slice(&7i32.to_le_bytes());
        buf.extend_from_slice(&3i32.to_le_bytes());
        buf.extend_from_slice(&4i32.to_le_bytes());
        buf.extend_from_slice(&8i32.to_le_bytes());
        for v in [21i32, 0, 0, 1, 1, 1, 2, 1251] {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        // Record 7 Subtype 11 (Display params: Ordinal, width 10, center)
        buf.extend_from_slice(&7i32.to_le_bytes());
        buf.extend_from_slice(&11i32.to_le_bytes());
        buf.extend_from_slice(&4i32.to_le_bytes());
        buf.extend_from_slice(&3i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());
        buf.extend_from_slice(&10i32.to_le_bytes());
        buf.extend_from_slice(&2i32.to_le_bytes());

        // Record 999
        buf.extend_from_slice(&999i32.to_le_bytes());
        buf.extend_from_slice(&0i32.to_le_bytes());
        let mut opcodes = [0u8; 8];
        opcodes[0] = 130; // 30.0
        opcodes[1] = 252; // EOF
        buf.extend_from_slice(&opcodes);
        fs::write(&cyr_sav_path, buf).map_err(|e| e.to_string())?;
    }

    Ok(())
}
