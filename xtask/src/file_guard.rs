use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileCategory {
    RootCoordinator,
    InteractiveScreen,
    StandardFunctionalModule,
    GeneralModule,
}

impl FileCategory {
    pub fn max_lines(&self) -> usize {
        match self {
            FileCategory::RootCoordinator => 1500,
            FileCategory::InteractiveScreen => 1000,
            FileCategory::StandardFunctionalModule => 500,
            FileCategory::GeneralModule => 1000,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FileCategory::RootCoordinator => "Root Coordinator",
            FileCategory::InteractiveScreen => "Interactive Screen",
            FileCategory::StandardFunctionalModule => "Standard Functional Module",
            FileCategory::GeneralModule => "General Module",
        }
    }
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn classify_file(relative_path: &str) -> FileCategory {
    let norm = relative_path.replace('\\', "/");
    let file_name = norm.rsplit('/').next().unwrap_or(&norm);

    if file_name == "app.rs" {
        return FileCategory::RootCoordinator;
    }

    if file_name == "data_view.rs"
        || file_name == "variable_view.rs"
        || file_name == "syntax_editor.rs"
    {
        return FileCategory::InteractiveScreen;
    }

    if norm.contains("/statcraft_algo/")
        || norm.starts_with("crates/statcraft_algo/")
        || norm.contains("/statcraft_syntax/")
        || norm.starts_with("crates/statcraft_syntax/")
        || norm.contains("/dialogs/")
        || norm.contains("/output_view/")
    {
        return FileCategory::StandardFunctionalModule;
    }

    FileCategory::GeneralModule
}

pub struct FileSizeViolation {
    pub path: String,
    pub lines: usize,
    pub max_lines: usize,
    pub category: FileCategory,
}

pub fn scan_rs_files(dir: &Path, file_list: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if path.is_dir() {
            if name_str == "target" || name_str == ".git" || name_str == ".idea" {
                continue;
            }
            scan_rs_files(&path, file_list)?;
        } else if name_str.ends_with(".rs") {
            file_list.push(path);
        }
    }

    Ok(())
}

pub fn check_file_sizes(workspace_root: &Path) -> Result<(), String> {
    let mut rs_files = Vec::new();
    scan_rs_files(workspace_root, &mut rs_files)
        .map_err(|e| format!("Failed to scan Rust files: {}", e))?;

    let mut violations = Vec::new();
    let mut total_scanned = 0;

    for file_path in rs_files {
        let rel_path = file_path
            .strip_prefix(workspace_root)
            .map(normalize_path)
            .unwrap_or_else(|_| normalize_path(&file_path));

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[WARN] Failed to read {}: {}", rel_path, e);
                continue;
            }
        };

        let lines = content.lines().count();
        let category = classify_file(&rel_path);
        let max_lines = category.max_lines();
        total_scanned += 1;

        if lines > max_lines {
            violations.push(FileSizeViolation {
                path: rel_path,
                lines,
                max_lines,
                category,
            });
        }
    }

    if violations.is_empty() {
        println!(
            "[xtask file-guard OK] Scanned {} Rust files. All files are within size limits.",
            total_scanned
        );
        Ok(())
    } else {
        let mut err_msg = format!(
            "Found {} file size violations across {} scanned files:\n\n",
            violations.len(),
            total_scanned
        );

        for v in &violations {
            err_msg.push_str(&format!(
                "[xtask file-guard ERROR] File '{}' has {} lines (exceeds limit of {} lines for {}).\n\
                 КАТЕГОРИЧЕСКИ ЗАПРЕЩЕНО: Урезать функционал, удалять колонки или сжимать код!\n\
                 ОБЯЗАТЕЛЬНОЕ ДЕЙСТВИЕ: Декомпозируйте файл на подпапку с модулями (mod.rs и подмодули).\n\n",
                v.path, v.lines, v.max_lines, v.category.name()
            ));
        }

        Err(err_msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_root_coordinator() {
        assert_eq!(
            classify_file("crates/statcraft_app/src/app.rs"),
            FileCategory::RootCoordinator
        );
        assert_eq!(FileCategory::RootCoordinator.max_lines(), 1500);
    }

    #[test]
    fn test_classify_interactive_screens() {
        assert_eq!(
            classify_file("crates/statcraft_app/src/views/data_view.rs"),
            FileCategory::InteractiveScreen
        );
        assert_eq!(
            classify_file("crates/statcraft_app/src/views/variable_view.rs"),
            FileCategory::InteractiveScreen
        );
        assert_eq!(
            classify_file("crates/statcraft_app/src/views/syntax_editor.rs"),
            FileCategory::InteractiveScreen
        );
        assert_eq!(FileCategory::InteractiveScreen.max_lines(), 1000);
    }

    #[test]
    fn test_classify_standard_functional_modules() {
        assert_eq!(
            classify_file("crates/statcraft_algo/src/descriptives.rs"),
            FileCategory::StandardFunctionalModule
        );
        assert_eq!(
            classify_file("crates/statcraft_syntax/src/commands/dataset_cmds.rs"),
            FileCategory::StandardFunctionalModule
        );
        assert_eq!(
            classify_file("crates/statcraft_app/src/dialogs/frequencies_dialog.rs"),
            FileCategory::StandardFunctionalModule
        );
        assert_eq!(
            classify_file("crates/statcraft_app/src/views/output_view/tables_anova.rs"),
            FileCategory::StandardFunctionalModule
        );
        assert_eq!(FileCategory::StandardFunctionalModule.max_lines(), 500);
    }

    #[test]
    fn test_classify_general_modules() {
        assert_eq!(
            classify_file("crates/statcraft_core/src/dataset.rs"),
            FileCategory::GeneralModule
        );
        assert_eq!(
            classify_file("crates/statcraft_io/src/csv_loader.rs"),
            FileCategory::GeneralModule
        );
        assert_eq!(FileCategory::GeneralModule.max_lines(), 1000);
    }
}
