use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub struct DesignViolation {
    pub file_path: String,
    pub line_number: usize,
    pub rule: &'static str,
    pub message: String,
}

pub fn check_dialog_design(workspace_root: &Path) -> Result<(), String> {
    let dialogs_dir = workspace_root
        .join("crates")
        .join("statcraft_app")
        .join("src")
        .join("dialogs");

    if !dialogs_dir.exists() {
        return Err(format!(
            "Dialogs directory not found: {}",
            dialogs_dir.display()
        ));
    }

    let mut violations = Vec::new();
    scan_dialogs_dir(&dialogs_dir, &mut violations)?;

    if !violations.is_empty() {
        let mut err_msg = format!(
            "[xtask design-guard FAILED] Found {} dialog design violation(s):\n",
            violations.len()
        );
        for v in &violations {
            err_msg.push_str(&format!(
                "  - {}:{}: [{}] {}\n",
                v.file_path, v.line_number, v.rule, v.message
            ));
        }
        err_msg.push_str(
            "\nAll dialogs must use DialogKit (DialogShell, DialogActionBar, SubdialogActionBar).",
        );
        return Err(err_msg);
    }

    println!("[xtask design-guard OK] All dialogs strictly comply with the StatCraft DialogKit design standard.");
    Ok(())
}

fn scan_dialogs_dir(dir: &Path, violations: &mut Vec<DesignViolation>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dialogs dir {}: {}", dir.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip common/ directory where DialogKit primitives are implemented
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name == "common" {
                continue;
            }
            scan_dialogs_dir(&path, violations)?;
        } else if path.is_file() {
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name.ends_with(".rs") && file_name != "mod.rs" {
                check_dialog_file(&path, violations)?;
            }
        }
    }

    Ok(())
}

pub fn check_dialog_file(
    file_path: &Path,
    violations: &mut Vec<DesignViolation>,
) -> Result<(), String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let norm_path = file_path.to_string_lossy().replace('\\', "/");
    let relative_path = if let Some(idx) = norm_path.find("crates/") {
        &norm_path[idx..]
    } else {
        &norm_path
    };

    let forbidden_button_keys = [
        "dialog-btn-ok",
        "dialog-btn-paste",
        "dialog-btn-reset",
        "dialog-btn-cancel",
        "dialog-btn-continue",
    ];

    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Skip comment lines
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }

        // Check 1: Raw action buttons instead of DialogActionBar / SubdialogActionBar
        for key in &forbidden_button_keys {
            let contains_key = trimmed.contains(&format!("t!(\"{}\")", key))
                || trimmed.contains(&format!("t!('{}')", key));
            let contains_btn = trimmed.contains("Button::new")
                || trimmed.contains("ui.button")
                || trimmed.contains("ui.add");

            if contains_key && contains_btn {
                violations.push(DesignViolation {
                    file_path: relative_path.to_string(),
                    line_number: line_num,
                    rule: "FORBIDDEN_RAW_ACTION_BUTTON",
                    message: format!(
                        "Directly creating action button with '{}' is forbidden. Use DialogActionBar or SubdialogActionBar.",
                        key
                    ),
                });
            }
        }

        // Check 2: Direct Window creation instead of DialogShell
        if trimmed.contains("Window::new(") && !trimmed.contains("DialogShell") {
            violations.push(DesignViolation {
                file_path: relative_path.to_string(),
                line_number: line_num,
                rule: "FORBIDDEN_RAW_WINDOW",
                message: "Directly calling Window::new() is forbidden. Wrap the dialog in DialogShell::standard(), subdialog() or properties().".to_string(),
            });
        }

        // Check 3: Arbitrary fixed_size(vec2(...)) in dialog implementation
        if trimmed.contains(".fixed_size(vec2(") {
            violations.push(DesignViolation {
                file_path: relative_path.to_string(),
                line_number: line_num,
                rule: "FORBIDDEN_ARBITRARY_WINDOW_SIZE",
                message: "Hardcoding fixed_size(vec2(...)) is forbidden. Use DialogSize::Standard, Subdialog or Properties via DialogShell.".to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catches_raw_action_button() {
        let code = r#"
            if ui.button(t!("dialog-btn-ok")).clicked() {
                action = Run;
            }
        "#;
        let temp_dir = std::env::temp_dir();
        let file = temp_dir.join("test_bad_dialog.rs");
        fs::write(&file, code).unwrap();

        let mut violations = Vec::new();
        check_dialog_file(&file, &mut violations).unwrap();
        assert!(violations
            .iter()
            .any(|v| v.rule == "FORBIDDEN_RAW_ACTION_BUTTON"));

        let _ = fs::remove_file(file);
    }

    #[test]
    fn test_catches_raw_window() {
        let code = r#"
            Window::new(t!("title"))
                .fixed_size(vec2(500.0, 400.0))
                .show(ctx, |ui| {});
        "#;
        let temp_dir = std::env::temp_dir();
        let file = temp_dir.join("test_bad_window.rs");
        fs::write(&file, code).unwrap();

        let mut violations = Vec::new();
        check_dialog_file(&file, &mut violations).unwrap();
        assert!(violations.iter().any(|v| v.rule == "FORBIDDEN_RAW_WINDOW"));
        assert!(violations
            .iter()
            .any(|v| v.rule == "FORBIDDEN_ARBITRARY_WINDOW_SIZE"));

        let _ = fs::remove_file(file);
    }
}
