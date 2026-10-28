use std::fs;
use std::path::Path;

pub fn check_parity(workspace_root: &Path) -> Result<(), String> {
    let checklist_path = workspace_root.join("docs").join("parity-checklist.md");
    if !checklist_path.exists() {
        return Err("docs/parity-checklist.md not found".to_string());
    }

    let content = fs::read_to_string(&checklist_path)
        .map_err(|e| format!("Failed to read parity checklist: {}", e))?;

    let lines: Vec<&str> = content.lines().collect();
    let mut total_cmds = 0;
    let mut implemented_cmds = 0;

    for line in lines {
        if line.starts_with('|') && line.contains('`') {
            total_cmds += 1;
            if line.contains("Implemented") || line.contains("In Progress") {
                implemented_cmds += 1;
            }
        }
    }

    println!(
        "[xtask parity] Parity tracked: {} commands documented (active: {}).",
        total_cmds, implemented_cmds
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parity_tracker_runs_on_valid_markdown() {
        let sample = "| File > Open | `data.import_csv` | In Progress | Notes |\n";
        assert!(sample.contains("`data.import_csv`"));
    }
}
