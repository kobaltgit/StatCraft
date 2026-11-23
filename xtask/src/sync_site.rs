use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuditBlockSummary {
    pub number: usize,
    pub title: String,
    pub procedures: Vec<String>,
    pub files: Vec<String>,
    pub status: String,
    pub precision: String,
}

pub fn parse_markdown_matrix(content: &str) -> Result<Vec<AuditBlockSummary>, String> {
    let mut blocks = Vec::new();
    let mut in_matrix = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## 2. Summary Audit Progress Matrix") {
            in_matrix = true;
            continue;
        }
        if in_matrix && trimmed.starts_with("## ") {
            break;
        }
        if in_matrix && trimmed.starts_with("| **Block") {
            let cols: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
            if cols.len() < 6 {
                continue;
            }
            // col 1: **Block N**
            let num_str = cols[1]
                .trim_start_matches('*')
                .trim_end_matches('*')
                .replace("Block", "");
            let number: usize = num_str
                .trim()
                .parse()
                .map_err(|e| format!("Failed to parse block number from '{}': {}", cols[1], e))?;

            let title_raw = cols[2];
            // Extract procedures enclosed in backticks
            let procedures: Vec<String> = title_raw
                .split('`')
                .enumerate()
                .filter(|(idx, _)| idx % 2 == 1)
                .map(|(_, s)| s.to_string())
                .collect();

            // Extract implementation files enclosed in backticks
            let files_raw = cols[3];
            let files: Vec<String> = files_raw
                .split('`')
                .enumerate()
                .filter(|(idx, _)| idx % 2 == 1)
                .map(|(_, s)| s.to_string())
                .collect();

            let status = cols[4].to_string();
            let precision = cols[5].to_string();

            blocks.push(AuditBlockSummary {
                number,
                title: title_raw.to_string(),
                procedures,
                files,
                status,
                precision,
            });
        }
    }

    if blocks.is_empty() {
        return Err("No audit blocks found in docs/SPSS_ALGORITHMS_AUDIT.md".to_string());
    }

    Ok(blocks)
}

pub fn parse_typescript_blocks(content: &str) -> Result<Vec<(usize, Vec<String>)>, String> {
    let mut ts_blocks = Vec::new();
    let mut current_number: Option<usize> = None;
    let mut current_procs = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("number:") {
            if let Some(num_str) = trimmed.strip_prefix("number:") {
                let clean = num_str.trim().trim_end_matches(',');
                if let Ok(n) = clean.parse::<usize>() {
                    if let Some(prev) = current_number {
                        ts_blocks.push((prev, current_procs.clone()));
                        current_procs.clear();
                    }
                    current_number = Some(n);
                }
            }
        } else if trimmed.starts_with("procedures:") {
            if let Some(start) = trimmed.find('[') {
                if let Some(end) = trimmed.find(']') {
                    let inside = &trimmed[start + 1..end];
                    for item in inside.split(',') {
                        let clean = item.trim().trim_matches('"').trim_matches('\'').trim();
                        if !clean.is_empty() {
                            current_procs.push(clean.to_string());
                        }
                    }
                }
            }
        }
    }

    if let Some(prev) = current_number {
        ts_blocks.push((prev, current_procs));
    }

    if ts_blocks.is_empty() {
        return Err("No audit blocks found in website/src/lib/audit-data.ts".to_string());
    }

    Ok(ts_blocks)
}

fn find_file_recursive(dir: &Path, target: &str) -> bool {
    if !dir.is_dir() {
        return false;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == target {
                    return true;
                }
            }
            if path.is_dir() && find_file_recursive(&path, target) {
                return true;
            }
        }
    }
    false
}

pub fn verify_files_exist(
    workspace_root: &Path,
    blocks: &[AuditBlockSummary],
) -> Result<(), String> {
    let crates_dir = workspace_root.join("crates");

    for block in blocks {
        for file in &block.files {
            let clean_file = file.trim_end_matches('/');
            let found = crates_dir.join(clean_file).exists()
                || find_file_recursive(&crates_dir, clean_file);

            if !found {
                return Err(format!(
                    "Block {}: implementation target '{}' not found in workspace crates",
                    block.number, file
                ));
            }
        }
    }

    Ok(())
}

pub fn sync_site(workspace_root: &Path, check_only: bool) -> Result<(), String> {
    let audit_md_path = workspace_root.join("docs/SPSS_ALGORITHMS_AUDIT.md");
    let audit_ts_path = workspace_root.join("website/src/lib/audit-data.ts");
    let docs_ts_path = workspace_root.join("website/src/lib/docs-data.ts");

    if !audit_md_path.exists() {
        return Err("docs/SPSS_ALGORITHMS_AUDIT.md not found".to_string());
    }
    if !audit_ts_path.exists() {
        return Err("website/src/lib/audit-data.ts not found".to_string());
    }
    if !docs_ts_path.exists() {
        return Err("website/src/lib/docs-data.ts not found".to_string());
    }

    let md_content = fs::read_to_string(&audit_md_path)
        .map_err(|e| format!("Failed to read audit markdown: {}", e))?;
    let ts_content = fs::read_to_string(&audit_ts_path)
        .map_err(|e| format!("Failed to read website audit-data.ts: {}", e))?;

    // 1. Parse markdown matrix
    let md_blocks = parse_markdown_matrix(&md_content)?;
    let ts_blocks = parse_typescript_blocks(&ts_content)?;

    // 2. Validate block count match
    if md_blocks.len() != ts_blocks.len() {
        return Err(format!(
            "Audit matrix mismatch! docs/SPSS_ALGORITHMS_AUDIT.md defines {} blocks, but website/src/lib/audit-data.ts contains {} blocks. Run sync-site to resolve.",
            md_blocks.len(),
            ts_blocks.len()
        ));
    }

    // 3. Validate block numbers and procedures
    for (md, (ts_num, ts_procs)) in md_blocks.iter().zip(ts_blocks.iter()) {
        if md.number != *ts_num {
            return Err(format!(
                "Block index sequence mismatch: Markdown Block {} vs TypeScript number {}",
                md.number, ts_num
            ));
        }

        for p in &md.procedures {
            let matched = ts_procs
                .iter()
                .any(|ts| ts == p || ts.starts_with(p) || ts.contains(p) || p.contains(ts));
            if !matched {
                return Err(format!(
                    "Block {} procedure mismatch: procedure '{}' in docs/SPSS_ALGORITHMS_AUDIT.md is missing from website/src/lib/audit-data.ts (found: {:?})",
                    md.number, p, ts_procs
                ));
            }
        }
    }

    // 4. Verify physical file existence in crates
    verify_files_exist(workspace_root, &md_blocks)?;

    // 5. Verify documentation sections
    let docs_content = fs::read_to_string(&docs_ts_path)
        .map_err(|e| format!("Failed to read docs-data.ts: {}", e))?;

    let doc_slugs = [
        "experiment-manifest",
        "independent-t-test",
        "open-math-audit",
        "mcp-agent-bridge",
        "safe-rust-architecture",
        "spss-sav-codecs",
        "oneway-anova-procedure",
        "linear-regression-procedure",
        "frequencies-and-charts",
    ];

    for slug in &doc_slugs {
        if !docs_content.contains(&format!("slug: \"{}\"", slug)) {
            return Err(format!(
                "Critical documentation section slug '{}' missing from website/src/lib/docs-data.ts",
                slug
            ));
        }
    }

    // 6. Synchronize / validate features documentation generated from docs/FEATURES.md
    crate::sync_features::sync_features(workspace_root, check_only)?;

    if check_only {
        println!(
            "[xtask sync-site OK] All {} audit blocks and {} documentation sections are verified and in sync.",
            md_blocks.len(),
            doc_slugs.len()
        );
    } else {
        println!(
            "[xtask sync-site] SUCCESS: Verified {} audit blocks with 100% file existence and {} docs pages in website.",
            md_blocks.len(),
            doc_slugs.len()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_markdown_matrix() {
        let md = r#"
## 2. Summary Audit Progress Matrix

| Block | Procedure Group | Implementation Files | Status | Oracle Precision |
|---|---|---|---|---|
| **Block 1** | Descriptives & Frequencies (`DESCRIPTIVES`, `FREQUENCIES`) | `descriptives.rs` | **100% Verified** | $\le 10^{-6}$ |
"#;
        let blocks = parse_markdown_matrix(md).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].number, 1);
        assert_eq!(blocks[0].procedures, vec!["DESCRIPTIVES", "FREQUENCIES"]);
        assert_eq!(blocks[0].files, vec!["descriptives.rs"]);
    }

    #[test]
    fn test_parse_sample_typescript_blocks() {
        let ts = r#"
export const AUDIT_BLOCKS: AuditBlock[] = [
  {
    id: "block-1",
    number: 1,
    titleEn: "Descriptives",
    procedures: ["DESCRIPTIVES", "FREQUENCIES"],
  },
];
"#;
        let blocks = parse_typescript_blocks(ts).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, 1);
        assert_eq!(blocks[0].1, vec!["DESCRIPTIVES", "FREQUENCIES"]);
    }
}
