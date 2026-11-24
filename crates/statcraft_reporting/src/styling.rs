use crate::model::ReportTable;
use serde::{Deserialize, Serialize};

/// Target table selection for conditional formatting rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableTarget {
    /// Apply to all eligible tables regardless of subtype.
    AllEligible,
    /// Apply only to tables matching a specific subtype (e.g. "Frequencies", "Descriptives").
    Subtype(String),
}

/// Target value selection within table dimensions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetValue {
    /// Apply to all data cells in the selected dimension.
    AllDataCells,
    /// Apply to cells matching a specific header/row label.
    Label(String),
}

/// Table dimension target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionTarget {
    Columns,
    Rows,
    Both,
}

/// Predicate condition evaluated on cell content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CellCondition {
    /// Matches all cells.
    All,
    /// Cell numeric value equals target.
    Equal(f64),
    /// Cell numeric value does not equal target.
    NotEqual(f64),
    /// Cell numeric value is strictly less than target.
    LessThan(f64),
    /// Cell numeric value is less than or equal to target.
    LessOrEqual(f64),
    /// Cell numeric value is strictly greater than target.
    GreaterThan(f64),
    /// Cell numeric value is greater than or equal to target.
    GreaterOrEqual(f64),
    /// Cell numeric value is within range [low, high] inclusive.
    Between(f64, f64),
    /// Cell numeric value is strictly outside range (v < low || v > high).
    NotBetween(f64, f64),
    /// Top N numeric values within the dimension.
    TopN(usize),
    /// Bottom N numeric values within the dimension.
    BottomN(usize),
    /// System missing value (empty, ".", "SYSMIS", "NaN").
    SysMis,
}

/// Target scope for applying formatting (cells only, entire column, or entire row).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CellTargetScope {
    #[default]
    CellsOnly,
    EntireColumn,
    EntireRow,
}

/// Formatting style applied to matching cells.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellStyleFormat {
    pub bold: bool,
    pub italic: bool,
    #[serde(default)]
    pub underline: bool,
    pub text_color: Option<[u8; 3]>,
    pub bg_color: Option<[u8; 3]>,
    #[serde(default)]
    pub format_mask_idx: usize,
    #[serde(default)]
    pub decimals: Option<usize>,
    #[serde(default)]
    pub target_scope: CellTargetScope,
    #[serde(default)]
    pub replace_value: Option<String>,
    pub reset_defaults: bool,
}

/// A complete styling rule with target, condition, and format.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableStyleRule {
    pub table_target: TableTarget,
    pub value_target: TargetValue,
    pub dimension: DimensionTarget,
    pub condition: CellCondition,
    pub format: CellStyleFormat,
}

/// Evaluation engine for applying table conditional formatting.
pub struct ConditionalFormattingEngine;

impl ConditionalFormattingEngine {
    /// Applies an ordered sequence of rules to a report table.
    /// Returns the total number of cell modifications made.
    pub fn apply_rules(
        table: &mut ReportTable,
        table_subtype: &str,
        rules: &[TableStyleRule],
    ) -> usize {
        let mut match_count = 0;
        for rule in rules {
            if !Self::matches_table(&rule.table_target, table_subtype) {
                continue;
            }
            match_count += Self::apply_single_rule(table, rule);
        }
        match_count
    }

    /// Convenience method to apply rules without a specific subtype.
    pub fn apply(table: &mut ReportTable, rules: &[TableStyleRule]) -> usize {
        Self::apply_rules(table, "", rules)
    }

    /// Checks if a rule targets the given table subtype.
    pub fn matches_table(target: &TableTarget, subtype: &str) -> bool {
        match target {
            TableTarget::AllEligible => true,
            TableTarget::Subtype(name) => name.eq_ignore_ascii_case(subtype),
        }
    }

    /// Applies a single rule across the table cells.
    pub fn apply_single_rule(table: &mut ReportTable, rule: &TableStyleRule) -> usize {
        let num_rows = table.rows.len();
        let num_cols = table.headers.len();
        let mut modified = 0;

        let targets = match rule.condition {
            CellCondition::TopN(n) => Self::find_top_or_bottom_n(table, rule, n, true),
            CellCondition::BottomN(n) => Self::find_top_or_bottom_n(table, rule, n, false),
            _ => {
                let mut matched = Vec::new();
                for r in 0..num_rows {
                    for c in 0..num_cols {
                        if !Self::matches_dimension(table, rule, r, c) {
                            continue;
                        }
                        let cell_str = match table.rows.get(r).and_then(|row| row.get(c)) {
                            Some(s) => s.as_str(),
                            None => continue,
                        };
                        if Self::eval_condition(&rule.condition, cell_str) {
                            matched.push((r, c));
                        }
                    }
                }
                matched
            }
        };

        for (r, c) in targets {
            match rule.format.target_scope {
                CellTargetScope::CellsOnly => {
                    Self::set_cell_format(table, r, c, &rule.format);
                    modified += 1;
                }
                CellTargetScope::EntireColumn => {
                    for row_idx in 0..num_rows {
                        Self::set_cell_format(table, row_idx, c, &rule.format);
                        modified += 1;
                    }
                }
                CellTargetScope::EntireRow => {
                    for col_idx in 0..num_cols {
                        Self::set_cell_format(table, r, col_idx, &rule.format);
                        modified += 1;
                    }
                }
            }
        }

        modified
    }

    fn set_cell_format(table: &mut ReportTable, r: usize, c: usize, format: &CellStyleFormat) {
        if let Some(ref rep) = format.replace_value {
            if !format.reset_defaults {
                if let Some(row) = table.rows.get_mut(r) {
                    if let Some(cell) = row.get_mut(c) {
                        *cell = rep.clone();
                    }
                }
            }
        }
        if format.reset_defaults {
            table.cell_styles.remove(&(r, c));
        } else {
            table.cell_styles.insert((r, c), format.clone());
        }
    }

    /// Checks whether row `r` and column `c` match the dimension and value targeting.
    pub fn matches_dimension(
        table: &ReportTable,
        rule: &TableStyleRule,
        r: usize,
        c: usize,
    ) -> bool {
        match &rule.value_target {
            TargetValue::AllDataCells => true,
            TargetValue::Label(target_label) => match rule.dimension {
                DimensionTarget::Columns => table
                    .headers
                    .get(c)
                    .is_some_and(|h| label_matches(h, target_label)),
                DimensionTarget::Rows => table
                    .rows
                    .get(r)
                    .and_then(|row| row.first())
                    .is_some_and(|lbl| label_matches(lbl, target_label)),
                DimensionTarget::Both => {
                    let col_match = table
                        .headers
                        .get(c)
                        .is_some_and(|h| label_matches(h, target_label));
                    let row_match = table
                        .rows
                        .get(r)
                        .and_then(|row| row.first())
                        .is_some_and(|lbl| label_matches(lbl, target_label));
                    col_match || row_match
                }
            },
        }
    }

    /// Evaluates point-wise condition against a cell string.
    pub fn eval_condition(cond: &CellCondition, val_str: &str) -> bool {
        match cond {
            CellCondition::All => true,
            CellCondition::SysMis => {
                let t = val_str.trim();
                t == "."
                    || t.is_empty()
                    || t.eq_ignore_ascii_case("sysmis")
                    || t.eq_ignore_ascii_case("nan")
            }
            CellCondition::Equal(target) => {
                parse_num(val_str).is_some_and(|v| (v - target).abs() < 1e-9)
            }
            CellCondition::NotEqual(target) => {
                parse_num(val_str).is_some_and(|v| (v - target).abs() >= 1e-9)
            }
            CellCondition::LessThan(target) => parse_num(val_str).is_some_and(|v| v < *target),
            CellCondition::LessOrEqual(target) => parse_num(val_str).is_some_and(|v| v <= *target),
            CellCondition::GreaterThan(target) => parse_num(val_str).is_some_and(|v| v > *target),
            CellCondition::GreaterOrEqual(target) => {
                parse_num(val_str).is_some_and(|v| v >= *target)
            }
            CellCondition::Between(low, high) => {
                parse_num(val_str).is_some_and(|v| v >= *low && v <= *high)
            }
            CellCondition::NotBetween(low, high) => {
                parse_num(val_str).is_some_and(|v| v < *low || v > *high)
            }
            CellCondition::TopN(_) | CellCondition::BottomN(_) => false,
        }
    }

    fn find_top_or_bottom_n(
        table: &ReportTable,
        rule: &TableStyleRule,
        n: usize,
        is_top: bool,
    ) -> Vec<(usize, usize)> {
        if n == 0 {
            return Vec::new();
        }

        let num_rows = table.rows.len();
        let num_cols = table.headers.len();
        let mut results = Vec::new();

        match rule.dimension {
            DimensionTarget::Columns => {
                for c in 0..num_cols {
                    let mut col_values = Vec::new();
                    for r in 0..num_rows {
                        if !Self::matches_dimension(table, rule, r, c) {
                            continue;
                        }
                        if let Some(cell_str) = table.rows.get(r).and_then(|row| row.get(c)) {
                            if let Some(val) = parse_num(cell_str) {
                                col_values.push((r, val));
                            }
                        }
                    }
                    sort_and_select_n(&mut col_values, n, is_top);
                    for (r, _) in col_values {
                        results.push((r, c));
                    }
                }
            }
            DimensionTarget::Rows => {
                for r in 0..num_rows {
                    let mut row_values = Vec::new();
                    for c in 0..num_cols {
                        if !Self::matches_dimension(table, rule, r, c) {
                            continue;
                        }
                        if let Some(cell_str) = table.rows.get(r).and_then(|row| row.get(c)) {
                            if let Some(val) = parse_num(cell_str) {
                                row_values.push((c, val));
                            }
                        }
                    }
                    sort_and_select_n(&mut row_values, n, is_top);
                    for (c, _) in row_values {
                        results.push((r, c));
                    }
                }
            }
            DimensionTarget::Both => {
                let mut all_values = Vec::new();
                for r in 0..num_rows {
                    for c in 0..num_cols {
                        if !Self::matches_dimension(table, rule, r, c) {
                            continue;
                        }
                        if let Some(cell_str) = table.rows.get(r).and_then(|row| row.get(c)) {
                            if let Some(val) = parse_num(cell_str) {
                                all_values.push(((r, c), val));
                            }
                        }
                    }
                }
                if is_top {
                    all_values
                        .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                } else {
                    all_values
                        .sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
                }
                all_values.truncate(n);
                for ((r, c), _) in all_values {
                    results.push((r, c));
                }
            }
        }

        results
    }
}

fn sort_and_select_n(values: &mut Vec<(usize, f64)>, n: usize, is_top: bool) {
    if is_top {
        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        values.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    }
    values.truncate(n);
}

fn label_matches(cell_label: &str, target_label: &str) -> bool {
    let cell_trimmed = cell_label.trim();
    let target_trimmed = target_label.trim();
    if cell_trimmed.eq_ignore_ascii_case(target_trimmed) {
        return true;
    }
    // Check if target matches any word token in cell_label (e.g. "Percent" in "Valid Percent")
    cell_trimmed
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| !word.is_empty() && word.eq_ignore_ascii_case(target_trimmed))
}

/// Safely parses a numeric value from a table cell, ignoring %, thousands commas, and handling decimal commas.
pub fn parse_num(s: &str) -> Option<f64> {
    let trimmed = s.trim().trim_end_matches('%').trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Ok(v) = trimmed.parse::<f64>() {
        return Some(v);
    }
    // If contains both comma and dot, comma is thousands separator: "1,234.56" -> "1234.56"
    if trimmed.contains(',') && trimmed.contains('.') {
        let without_commas = trimmed.replace(',', "");
        return without_commas.parse::<f64>().ok();
    }
    // If contains comma and no dot:
    if trimmed.contains(',') {
        if trimmed.matches(',').count() > 1 {
            return trimmed.replace(',', "").parse::<f64>().ok();
        }
        let comma_to_dot = trimmed.replace(',', ".");
        if let Ok(v) = comma_to_dot.parse::<f64>() {
            return Some(v);
        }
    }
    None
}
