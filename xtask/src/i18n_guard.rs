use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct I18nViolation {
    pub file_path: String,
    pub line_number: usize,
    pub rule: &'static str,
    pub message: String,
}

pub fn check_i18n(workspace_root: &Path) -> Result<(), String> {
    let app_src = workspace_root
        .join("crates")
        .join("statcraft_app")
        .join("src");

    let mut violations = Vec::new();
    for subdir in ["dialogs", "views"] {
        let dir = app_src.join(subdir);
        if dir.exists() {
            scan_dir(&dir, &mut violations)?;
        }
    }

    if !violations.is_empty() {
        let mut err_msg = format!(
            "[xtask i18n-guard FAILED] Found {} raw UI string violation(s):\n",
            violations.len()
        );
        for v in &violations {
            err_msg.push_str(&format!(
                "  - {}:{}: [{}] {}\n",
                v.file_path, v.line_number, v.rule, v.message
            ));
        }
        err_msg.push_str(
            "\nAll UI labels, buttons, and table headers must use statcraft_i18n::t!(\"...\").",
        );
        return Err(err_msg);
    }

    println!("[xtask i18n-guard OK] Zero hardcoded UI strings in dialogs and views.");
    Ok(())
}

fn scan_dir(dir: &Path, violations: &mut Vec<I18nViolation>) -> Result<(), String> {
    let entries =
        fs::read_dir(dir).map_err(|e| format!("Failed to read dir {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, violations)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            check_file(&path, violations)?;
        }
    }
    Ok(())
}

fn contains_word(s: &str) -> bool {
    let mut run = 0usize;
    for ch in s.chars() {
        if ch.is_alphabetic() {
            run += 1;
            if run >= 2 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

fn extract_quoted_after<'a>(line: &'a str, marker: &str) -> Option<&'a str> {
    let idx = line.find(marker)?;
    let rest = &line[idx + marker.len()..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

pub fn check_file(file_path: &Path, violations: &mut Vec<I18nViolation>) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let path_str = file_path.to_string_lossy().to_string();

    for (idx, raw_line) in content.lines().enumerate() {
        let trimmed = raw_line.trim();
        if trimmed.starts_with("#[cfg(test)]") {
            break;
        }
        if trimmed.starts_with("//") || trimmed.starts_with("///") {
            continue;
        }

        // Check direct UI widget calls with literal strings
        for marker in [
            "ui.label(\"",
            "ui.strong(\"",
            "ui.heading(\"",
            "ui.button(\"",
            "RichText::new(\"",
            "ReportTable::new(\"",
        ] {
            if let Some(lit) = extract_quoted_after(trimmed, marker) {
                if contains_word(lit) {
                    violations.push(I18nViolation {
                        file_path: path_str.clone(),
                        line_number: idx + 1,
                        rule: "NO_RAW_UI_TEXT",
                        message: format!("Raw string literal in {}: \"{}\"", marker, lit),
                    });
                }
            }
        }

        // Check table column header tuples: ("Header".to_string(), <width>)
        if let Some(to_str_idx) = trimmed.find(".to_string(),") {
            let prefix = &trimmed[..to_str_idx];
            if let Some(open_paren) = prefix.rfind("(\"") {
                let lit = &prefix[open_paren + 2..];
                if !lit.contains('"') && contains_word(lit) {
                    violations.push(I18nViolation {
                        file_path: path_str.clone(),
                        line_number: idx + 1,
                        rule: "NO_RAW_TABLE_HEADER",
                        message: format!("Raw table column header: \"{}\"", lit),
                    });
                }
            }
        }

        // Check ReportTable headers = vec!["...", ...]
        if trimmed.contains(".headers = vec![") && !trimmed.contains("t!(") {
            if let Some(lit) = extract_quoted_after(trimmed, "vec![\"") {
                if contains_word(lit) {
                    violations.push(I18nViolation {
                        file_path: path_str.clone(),
                        line_number: idx + 1,
                        rule: "NO_RAW_REPORT_HEADER",
                        message: format!("Raw ReportTable header literal: \"{}\"", lit),
                    });
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_word() {
        assert!(contains_word("Parameter"));
        assert!(contains_word("Параметр"));
        assert!(!contains_word("..."));
        assert!(!contains_word("0.0"));
        assert!(!contains_word("%"));
    }
}
