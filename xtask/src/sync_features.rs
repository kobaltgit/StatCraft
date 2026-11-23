use std::fs;
use std::path::Path;

struct FeatureSectionMeta {
    number: usize,
    slug: &'static str,
    category: &'static str,
    category_en: &'static str,
    category_ru: &'static str,
}

const SECTION_METAS: &[FeatureSectionMeta] = &[
    FeatureSectionMeta {
        number: 1,
        slug: "features-tabular-editors",
        category: "data-management",
        category_en: "Data Management & IO",
        category_ru: "Управление данными и ввод-вывод",
    },
    FeatureSectionMeta {
        number: 2,
        slug: "features-case-manipulations",
        category: "data-management",
        category_en: "Data Management & IO",
        category_ru: "Управление данными и ввод-вывод",
    },
    FeatureSectionMeta {
        number: 3,
        slug: "features-transformations",
        category: "data-management",
        category_en: "Data Management & IO",
        category_ru: "Управление данными и ввод-вывод",
    },
    FeatureSectionMeta {
        number: 4,
        slug: "features-statistical-procedures",
        category: "procedures",
        category_en: "Statistical Procedures",
        category_ru: "Статистические процедуры",
    },
    FeatureSectionMeta {
        number: 5,
        slug: "features-output-management",
        category: "data-management",
        category_en: "Data Management & IO",
        category_ru: "Управление данными и ввод-вывод",
    },
    FeatureSectionMeta {
        number: 6,
        slug: "features-spss-syntax",
        category: "getting-started",
        category_en: "Getting Started",
        category_ru: "Начало работы",
    },
    FeatureSectionMeta {
        number: 7,
        slug: "features-ai-mcp-bridge",
        category: "agent-lab",
        category_en: "Agent Lab & MCP",
        category_ru: "ИИ-лаборатория и MCP",
    },
    FeatureSectionMeta {
        number: 8,
        slug: "features-design-and-reliability",
        category: "getting-started",
        category_en: "Getting Started",
        category_ru: "Начало работы",
    },
    FeatureSectionMeta {
        number: 9,
        slug: "features-roadmap",
        category: "getting-started",
        category_en: "Getting Started",
        category_ru: "Начало работы",
    },
];

#[derive(Debug, Clone)]
pub struct ParsedSection {
    pub number: usize,
    pub title: String,
    pub description: String,
    pub toc_titles: Vec<String>,
    pub content: String,
}

pub fn parse_features_file(text: &str) -> Vec<ParsedSection> {
    let mut sections = Vec::new();
    let mut current_num = 0;
    let mut current_title = String::new();
    let mut current_lines: Vec<&str> = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            let rest = trimmed.trim_start_matches("## ").trim();
            if let Some(dot_pos) = rest.find('.') {
                let num_str = &rest[..dot_pos];
                if let Ok(num) = num_str.trim().parse::<usize>() {
                    if current_num > 0 {
                        sections.push(build_parsed_section(
                            current_num,
                            &current_title,
                            &current_lines,
                        ));
                        current_lines.clear();
                    }
                    current_num = num;
                    current_title = rest[dot_pos + 1..].trim().to_string();
                    continue;
                }
            }
        }

        if current_num > 0 {
            current_lines.push(line);
        }
    }

    if current_num > 0 {
        sections.push(build_parsed_section(
            current_num,
            &current_title,
            &current_lines,
        ));
    }

    sections
}

fn build_parsed_section(number: usize, title: &str, lines: &[&str]) -> ParsedSection {
    let mut description = String::new();
    let mut toc_titles = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("### ") {
            let h3 = trimmed.trim_start_matches("### ").trim();
            toc_titles.push(h3.to_string());
        } else if trimmed.starts_with("* **") && toc_titles.is_empty() {
            // If section doesn't use H3, extract bold feature names
            if let Some(end_bold) = trimmed[4..].find("**") {
                let bold_name = &trimmed[4..4 + end_bold];
                let clean_name = bold_name.trim_end_matches(':').trim();
                if !clean_name.is_empty() {
                    toc_titles.push(clean_name.to_string());
                }
            }
        }

        if description.is_empty()
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with('*')
            && !trimmed.starts_with('-')
            && !trimmed.starts_with("```")
        {
            description = trimmed.to_string();
        }
    }

    if description.is_empty() {
        description = format!("Comprehensive reference guide for {}.", title);
    }

    let content = lines.join("\n").trim().to_string();

    ParsedSection {
        number,
        title: title.to_string(),
        description,
        toc_titles,
        content,
    }
}

fn slugify(text: &str) -> String {
    let mut s = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() || c == '-' {
            s.push(c.to_ascii_lowercase());
        } else if c == ' ' || c == '_' {
            s.push('-');
        }
    }
    s.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub fn generate_features_typescript(
    en_sections: &[ParsedSection],
    ru_sections: &[ParsedSection],
) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("// Auto-generated by `cargo xtask sync-site` from docs/FEATURES.md and docs/FEATURES_RU.md.\n");
    out.push_str("// DO NOT EDIT MANUALLY. Run `cargo xtask sync-site` to synchronize.\n\n");
    out.push_str("import type { DocSection } from './docs-data';\n\n");
    out.push_str("export const FEATURE_SECTIONS: DocSection[] = [\n");

    for meta in SECTION_METAS {
        let sec_en = en_sections
            .iter()
            .find(|s| s.number == meta.number)
            .ok_or_else(|| format!("Section {} missing in docs/FEATURES.md", meta.number))?;

        let sec_ru = ru_sections
            .iter()
            .find(|s| s.number == meta.number)
            .ok_or_else(|| format!("Section {} missing in docs/FEATURES_RU.md", meta.number))?;

        // Build TOC
        let mut toc_entries = Vec::new();
        let max_toc = sec_en.toc_titles.len().max(sec_ru.toc_titles.len());
        for i in 0..max_toc {
            let title_en = sec_en.toc_titles.get(i).map(|s| s.as_str()).unwrap_or("");
            let title_ru = sec_ru
                .toc_titles
                .get(i)
                .map(|s| s.as_str())
                .unwrap_or(title_en);
            let id = slugify(title_en);
            if !id.is_empty() {
                toc_entries.push(format!(
                    "      {{ id: {}, titleEn: {}, titleRu: {} }}",
                    serde_json::to_string(&id).map_err(|e| e.to_string())?,
                    serde_json::to_string(&title_en).map_err(|e| e.to_string())?,
                    serde_json::to_string(&title_ru).map_err(|e| e.to_string())?,
                ));
            }
        }

        out.push_str("  {\n");
        out.push_str(&format!("    slug: \"{}\",\n", meta.slug));
        out.push_str(&format!("    category: \"{}\",\n", meta.category));
        out.push_str(&format!("    categoryTitleEn: \"{}\",\n", meta.category_en));
        out.push_str(&format!("    categoryTitleRu: \"{}\",\n", meta.category_ru));
        out.push_str(&format!(
            "    titleEn: {},\n",
            serde_json::to_string(&sec_en.title).map_err(|e| e.to_string())?
        ));
        out.push_str(&format!(
            "    titleRu: {},\n",
            serde_json::to_string(&sec_ru.title).map_err(|e| e.to_string())?
        ));
        out.push_str(&format!(
            "    descriptionEn: {},\n",
            serde_json::to_string(&sec_en.description).map_err(|e| e.to_string())?
        ));
        out.push_str(&format!(
            "    descriptionRu: {},\n",
            serde_json::to_string(&sec_ru.description).map_err(|e| e.to_string())?
        ));

        out.push_str("    toc: [\n");
        for t in &toc_entries {
            out.push_str(t);
            out.push_str(",\n");
        }
        out.push_str("    ],\n");

        out.push_str(&format!(
            "    contentEn: {},\n",
            serde_json::to_string(&sec_en.content).map_err(|e| e.to_string())?
        ));
        out.push_str(&format!(
            "    contentRu: {},\n",
            serde_json::to_string(&sec_ru.content).map_err(|e| e.to_string())?
        ));
        out.push_str("  },\n");
    }

    out.push_str("];\n");
    Ok(out)
}

pub fn sync_features(workspace_root: &Path, check_only: bool) -> Result<(), String> {
    let features_en_path = workspace_root.join("docs/FEATURES.md");
    let features_ru_path = workspace_root.join("docs/FEATURES_RU.md");
    let target_ts_path = workspace_root.join("website/src/lib/features-docs.ts");

    if !features_en_path.exists() {
        return Err("docs/FEATURES.md not found".to_string());
    }
    if !features_ru_path.exists() {
        return Err("docs/FEATURES_RU.md not found".to_string());
    }

    let en_text = fs::read_to_string(&features_en_path)
        .map_err(|e| format!("Failed to read docs/FEATURES.md: {}", e))?;
    let ru_text = fs::read_to_string(&features_ru_path)
        .map_err(|e| format!("Failed to read docs/FEATURES_RU.md: {}", e))?;

    let en_sections = parse_features_file(&en_text);
    let ru_sections = parse_features_file(&ru_text);

    if en_sections.len() != 9 {
        return Err(format!(
            "Expected 9 sections in docs/FEATURES.md, found {}",
            en_sections.len()
        ));
    }
    if ru_sections.len() != 9 {
        return Err(format!(
            "Expected 9 sections in docs/FEATURES_RU.md, found {}",
            ru_sections.len()
        ));
    }

    let generated_code = generate_features_typescript(&en_sections, &ru_sections)?;

    if check_only {
        if !target_ts_path.exists() {
            return Err(
                "website/src/lib/features-docs.ts does not exist. Run 'cargo xtask sync-site' to generate it."
                    .to_string(),
            );
        }

        let existing_code = fs::read_to_string(&target_ts_path)
            .map_err(|e| format!("Failed to read website/src/lib/features-docs.ts: {}", e))?;

        let norm_existing = existing_code.replace("\r\n", "\n");
        let norm_generated = generated_code.replace("\r\n", "\n");

        if norm_existing != norm_generated {
            return Err(
                "website/src/lib/features-docs.ts is out of date with docs/FEATURES.md and docs/FEATURES_RU.md! Run 'cargo xtask sync-site' to synchronize."
                    .to_string(),
            );
        }

        println!("[xtask sync-site OK] website/src/lib/features-docs.ts is 100% in sync with docs/FEATURES.md.");
    } else {
        fs::write(&target_ts_path, &generated_code)
            .map_err(|e| format!("Failed to write website/src/lib/features-docs.ts: {}", e))?;

        println!(
            "[xtask sync-site] Generated website/src/lib/features-docs.ts (9 sections synchronized from docs/FEATURES.md)."
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sample_features_file() {
        let sample = r#"
# Feature Guide

## 1. Dual-View Editors

This is a description paragraph.

### 1.1. Data Import
Some text here.

### 1.2. Variable View
More text.

## 2. Case Manipulations

Case management description.

* **Sort Cases (`SORT CASES`)**: Sorting cases.
"#;
        let sections = parse_features_file(sample);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].number, 1);
        assert_eq!(sections[0].title, "Dual-View Editors");
        assert_eq!(sections[0].description, "This is a description paragraph.");
        assert_eq!(
            sections[0].toc_titles,
            vec!["1.1. Data Import", "1.2. Variable View"]
        );

        assert_eq!(sections[1].number, 2);
        assert_eq!(sections[1].title, "Case Manipulations");
        assert_eq!(sections[1].description, "Case management description.");
        assert_eq!(sections[1].toc_titles, vec!["Sort Cases (`SORT CASES`)"]);
    }
}
