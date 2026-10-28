use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[test]
fn test_all_catalog_keys_match_between_en_and_ru() {
    let en_content = statcraft_i18n::EMBEDDED_EN;
    let ru_content = statcraft_i18n::EMBEDDED_RU;

    let en_keys = extract_ftl_keys(en_content);
    let ru_keys = extract_ftl_keys(ru_content);

    let missing_in_ru: Vec<_> = en_keys.difference(&ru_keys).collect();
    let missing_in_en: Vec<_> = ru_keys.difference(&en_keys).collect();

    assert!(
        missing_in_ru.is_empty(),
        "The following keys exist in English but are missing in Russian: {:?}",
        missing_in_ru
    );
    assert!(
        missing_in_en.is_empty(),
        "The following keys exist in Russian but are missing in English: {:?}",
        missing_in_en
    );
}

#[test]
fn test_zero_raw_cyrillic_in_app_source() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap();

    let app_src = workspace_root
        .join("crates")
        .join("statcraft_app")
        .join("src");
    if !app_src.exists() {
        return;
    }

    let mut violations = vec![];
    scan_for_raw_cyrillic(&app_src, &mut violations);

    assert!(
        violations.is_empty(),
        "Zero-Hardcode Rule Violation: Found raw Cyrillic strings in UI source files:\n{}",
        violations.join("\n")
    );
}

#[test]
fn test_all_used_keys_in_source_exist_in_catalogs() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap();

    let app_src = workspace_root
        .join("crates")
        .join("statcraft_app")
        .join("src");
    if !app_src.exists() {
        return;
    }

    let en_content = statcraft_i18n::EMBEDDED_EN;
    let ru_content = statcraft_i18n::EMBEDDED_RU;
    let en_keys = extract_ftl_keys(en_content);
    let ru_keys = extract_ftl_keys(ru_content);

    let mut used_keys = HashSet::new();
    scan_for_used_i18n_keys(&app_src, &mut used_keys);

    let mut missing_keys = vec![];
    for (file, line_num, key) in used_keys {
        if !en_keys.contains(&key) || !ru_keys.contains(&key) {
            missing_keys.push(format!(
                "{}:{}: Key '{}' not found in catalogs",
                file, line_num, key
            ));
        }
    }

    assert!(
        missing_keys.is_empty(),
        "I18n Key Integrity Violation: Found t!() keys in source that do not exist in catalogs:\n{}",
        missing_keys.join("\n")
    );
}

fn scan_for_used_i18n_keys(dir: &Path, used_keys: &mut HashSet<(String, usize, String)>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_for_used_i18n_keys(&path, used_keys);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("//")
                            || trimmed.starts_with("/*")
                            || trimmed.starts_with('*')
                        {
                            continue;
                        }
                        let mut start = 0;
                        while let Some(pos) = line[start..].find("t!(\"") {
                            let match_idx = start + pos;
                            // Ensure 't' is not part of a larger identifier (e.g., format!("..."))
                            if match_idx > 0 {
                                let prev_char = line[..match_idx].chars().last().unwrap();
                                if prev_char.is_alphanumeric() || prev_char == '_' {
                                    start = match_idx + 4;
                                    continue;
                                }
                            }
                            let key_start = match_idx + 4;
                            if let Some(quote_end) = line[key_start..].find('"') {
                                let key = &line[key_start..key_start + quote_end];
                                used_keys.insert((
                                    path.file_name().unwrap().to_string_lossy().to_string(),
                                    line_idx + 1,
                                    key.to_string(),
                                ));
                                start = key_start + quote_end + 1;
                            } else {
                                break;
                            }
                        }

                        // Scan for title_key: "..." in output models
                        let mut tk_start = 0;
                        while let Some(pos) = line[tk_start..].find("title_key: \"") {
                            let match_idx = tk_start + pos;
                            let key_start = match_idx + 12;
                            if let Some(quote_end) = line[key_start..].find('"') {
                                let key = &line[key_start..key_start + quote_end];
                                used_keys.insert((
                                    path.file_name().unwrap().to_string_lossy().to_string(),
                                    line_idx + 1,
                                    key.to_string(),
                                ));
                                tk_start = key_start + quote_end + 1;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn extract_ftl_keys(content: &str) -> HashSet<String> {
    let mut keys = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            keys.insert(key.trim().to_string());
        }
    }
    keys
}

fn scan_for_raw_cyrillic(dir: &Path, violations: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_for_raw_cyrillic(&path, violations);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_idx, line) in content.lines().enumerate() {
                        let trimmed = line.trim();
                        // Ignore comments
                        if trimmed.starts_with("//")
                            || trimmed.starts_with("/*")
                            || trimmed.starts_with('*')
                        {
                            continue;
                        }
                        for ch in line.chars() {
                            // Cyrillic unicode block
                            if ('\u{0400}'..='\u{04FF}').contains(&ch) {
                                violations.push(format!(
                                    "{}:{}: '{}'",
                                    path.display(),
                                    line_idx + 1,
                                    trimmed
                                ));
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}
