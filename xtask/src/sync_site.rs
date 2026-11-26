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

pub fn check_line_for_hardcoded_block_counts(line: &str) -> Option<String> {
    let lower = line.to_lowercase();

    // 1. Check for <digit>-block or <digit>-блок (e.g. "20-block", "23-block", "20-блок")
    for keyword in &["-block", "-блок"] {
        let mut start = 0;
        while let Some(pos) = lower[start..].find(keyword) {
            let abs_pos = start + pos;
            if abs_pos > 0 {
                let prev_char = lower[..abs_pos].chars().last();
                if let Some(c) = prev_char {
                    if c.is_ascii_digit() {
                        return Some(format!("Hardcoded '{}{}' pattern", c, keyword));
                    }
                }
            }
            start = abs_pos + keyword.len();
        }
    }

    // 2. Check for "оракул <digits> блок" or "oracle <digits> block"
    for phrase in &["оракул", "oracle"] {
        if let Some(pos) = lower.find(phrase) {
            let rest = &lower[pos + phrase.len()..];
            let trimmed = rest.trim_start();
            let digits_len = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
            if digits_len > 0 {
                let after_digits = trimmed[digits_len..].trim_start();
                if after_digits.starts_with("блок") || after_digits.starts_with("block") {
                    return Some(format!("Hardcoded '{} <N> block' phrase", phrase));
                }
            }
        }
    }

    // 3. Check for "<digits> блоков доказательного" or "всех <digits> проверенных блоков"
    // or "все <digits> ключевых блоков" or "all <digits> mathematical" or "all <digits> verified"
    for prefix in &["всех", "все", "all"] {
        if let Some(pos) = lower.find(prefix) {
            let rest = &lower[pos + prefix.len()..];
            let trimmed = rest.trim_start();
            let digits_len = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
            if digits_len > 0 {
                let after_digits = trimmed[digits_len..].trim_start();
                if after_digits.starts_with("проверенных блоков")
                    || after_digits.starts_with("ключевых блоков")
                    || after_digits.starts_with("mathematical")
                    || after_digits.starts_with("verified")
                    || after_digits.starts_with("блоков")
                    || after_digits.starts_with("blocks")
                {
                    return Some(format!("Hardcoded '{} <N> blocks' phrase", prefix));
                }
            }
        }
    }

    // 4. Check for "<digits> блоков доказательного"
    if lower.contains("блоков доказательного") {
        if let Some(pos) = lower.find("блоков доказательного") {
            let before = lower[..pos].trim_end();
            if let Some(c) = before.chars().last() {
                if c.is_ascii_digit() {
                    return Some("Hardcoded '<N> блоков доказательного' phrase".to_string());
                }
            }
        }
    }

    None
}

fn collect_source_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if !dir.is_dir() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_source_files(&path, files);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if ext == "ts" || ext == "tsx" {
                    files.push(path);
                }
            }
        }
    }
}

pub fn check_no_hardcoded_block_counts(workspace_root: &Path) -> Result<(), String> {
    let components_dir = workspace_root.join("website/src/components");
    let docs_data_path = workspace_root.join("website/src/lib/docs-data.ts");

    let mut files_to_check = Vec::new();
    collect_source_files(&components_dir, &mut files_to_check);
    if docs_data_path.exists() {
        files_to_check.push(docs_data_path);
    }

    for file_path in files_to_check {
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;
        for (line_no, line) in content.lines().enumerate() {
            if let Some(violation) = check_line_for_hardcoded_block_counts(line) {
                let rel_path = file_path.strip_prefix(workspace_root).unwrap_or(&file_path);
                return Err(format!(
                    "Hardcoded audit block count detected in {}:{} - {} (line: '{}'). Use dynamic count (e.g. AUDIT_BLOCKS.length) instead.",
                    rel_path.display(),
                    line_no + 1,
                    violation,
                    line.trim()
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

    // 6. Check that website components and docs have no hardcoded block counts
    check_no_hardcoded_block_counts(workspace_root)?;

    // 7. Synchronize / validate features documentation generated from docs/FEATURES.md
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

    #[test]
    fn test_check_line_for_hardcoded_block_counts_detects_violations() {
        assert!(check_line_for_hardcoded_block_counts("Оракул 20 блоков").is_some());
        assert!(check_line_for_hardcoded_block_counts("02 / 20-BLOCK ORACLE").is_some());
        assert!(check_line_for_hardcoded_block_counts(
            "titleEn: \"20-Block Verification Protocol\""
        )
        .is_some());
        assert!(check_line_for_hardcoded_block_counts(
            "titleRu: \"20 блоков доказательного аудита\""
        )
        .is_some());
        assert!(check_line_for_hardcoded_block_counts("All 20 mathematical blocks").is_some());
        assert!(check_line_for_hardcoded_block_counts("Все 20 ключевых блоков").is_some());
        assert!(check_line_for_hardcoded_block_counts("Всех 23 проверенных блоков").is_some());
    }

    #[test]
    fn test_check_line_for_hardcoded_block_counts_allows_dynamic() {
        assert!(check_line_for_hardcoded_block_counts(
            "label: `Оракул ${AUDIT_BLOCKS.length} блоков`"
        )
        .is_none());
        assert!(check_line_for_hardcoded_block_counts(
            "label: `02 / ${AUDIT_BLOCKS.length}-BLOCK ORACLE`"
        )
        .is_none());
        assert!(check_line_for_hardcoded_block_counts(
            "titleEn: `${AUDIT_BLOCKS.length}-Block Verification Protocol`"
        )
        .is_none());
        assert!(check_line_for_hardcoded_block_counts(
            "All ${AUDIT_BLOCKS.length} mathematical..."
        )
        .is_none());
        assert!(
            check_line_for_hardcoded_block_counts("* **Block 1–4:** Basic Statistics").is_none()
        );
        assert!(
            check_line_for_hardcoded_block_counts("* **Block 16–20:** Advanced Regression")
                .is_none()
        );
        assert!(
            check_line_for_hardcoded_block_counts("<div className=\"block text-xs\">").is_none()
        );
    }
}
