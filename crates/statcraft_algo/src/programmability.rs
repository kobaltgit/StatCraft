use crate::expr_eval::{CompiledExpr, EvalError};
use serde::{Deserialize, Serialize};
use statcraft_core::{DataSet, DataType, Measure, RawValue};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ProgrammabilityError {
    #[error("Script cannot be empty")]
    EmptyScript,
    #[error("Parse error at line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Execution error: {0}")]
    ExecutionError(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProgrammabilityRule {
    pub target_var: String,
    pub condition: Option<CompiledExpr>,
    pub expr: CompiledExpr,
    pub else_expr: Option<CompiledExpr>,
    pub raw_line: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProgrammabilityScript {
    pub rules: Vec<ProgrammabilityRule>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProgrammabilityResult {
    pub created_vars: Vec<String>,
    pub modified_vars: Vec<String>,
    pub rows_affected: usize,
    pub statements_count: usize,
    pub logs: Vec<String>,
}

impl ProgrammabilityScript {
    pub fn parse(input: &str) -> Result<Self, ProgrammabilityError> {
        let mut rules = Vec::new();
        let lines: Vec<&str> = input.lines().collect();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed.starts_with("//")
                || trimmed.starts_with("--")
            {
                continue;
            }

            let rule = parse_rule_line(trimmed, line_num)?;
            rules.push(rule);
        }

        if rules.is_empty() {
            return Err(ProgrammabilityError::EmptyScript);
        }

        Ok(Self { rules })
    }
}

fn parse_rule_line(
    line: &str,
    line_num: usize,
) -> Result<ProgrammabilityRule, ProgrammabilityError> {
    let upper = line.to_uppercase();
    if upper.starts_with("IF ") || upper.starts_with("IF(") {
        parse_conditional_rule(line, line_num)
    } else {
        parse_assignment_rule(line, line_num)
    }
}

fn parse_assignment_rule(
    line: &str,
    line_num: usize,
) -> Result<ProgrammabilityRule, ProgrammabilityError> {
    let eq_idx = line
        .find('=')
        .ok_or_else(|| ProgrammabilityError::ParseError {
            line: line_num,
            message: "Missing '=' in assignment statement".to_string(),
        })?;

    let target_var = line[..eq_idx].trim().to_string();
    validate_target_var(&target_var, line_num)?;

    let expr_str = line[eq_idx + 1..].trim();
    if expr_str.is_empty() {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: "Missing expression after '='".to_string(),
        });
    }

    let expr = CompiledExpr::parse(expr_str).map_err(|e| map_eval_err(e, line_num))?;

    Ok(ProgrammabilityRule {
        target_var,
        condition: None,
        expr,
        else_expr: None,
        raw_line: line.to_string(),
    })
}

fn parse_conditional_rule(
    line: &str,
    line_num: usize,
) -> Result<ProgrammabilityRule, ProgrammabilityError> {
    let rest = if line.len() > 2 && line[..3].eq_ignore_ascii_case("IF ") {
        &line[3..]
    } else {
        &line[2..]
    };

    let (cond_part, action_part) = if let Some(idx) = find_keyword_case_insensitive(rest, "THEN") {
        (&rest[..idx], &rest[idx + 4..])
    } else if let Some(idx) = rest.find("=>") {
        (&rest[..idx], &rest[idx + 2..])
    } else if let Some(idx) = rest.find(':') {
        (&rest[..idx], &rest[idx + 1..])
    } else {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: "Conditional IF statement requires 'THEN', ':', or '=>'".to_string(),
        });
    };

    let cond_str = cond_part.trim();
    if cond_str.is_empty() {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: "Empty condition in IF statement".to_string(),
        });
    }
    let condition = CompiledExpr::parse(cond_str).map_err(|e| map_eval_err(e, line_num))?;

    let (then_part, else_part_opt) =
        if let Some(idx) = find_keyword_case_insensitive(action_part, "ELSE") {
            let else_clause = action_part[idx + 4..]
                .trim()
                .strip_prefix(':')
                .unwrap_or(&action_part[idx + 4..]);
            (&action_part[..idx], Some(else_clause.trim()))
        } else {
            (action_part, None)
        };

    let then_str = then_part.trim();
    let eq_idx = then_str
        .find('=')
        .ok_or_else(|| ProgrammabilityError::ParseError {
            line: line_num,
            message: "Missing '=' in conditional action".to_string(),
        })?;

    let target_var = then_str[..eq_idx].trim().to_string();
    validate_target_var(&target_var, line_num)?;

    let expr_str = then_str[eq_idx + 1..].trim();
    if expr_str.is_empty() {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: "Missing expression in conditional action".to_string(),
        });
    }
    let expr = CompiledExpr::parse(expr_str).map_err(|e| map_eval_err(e, line_num))?;

    let else_expr = if let Some(else_raw) = else_part_opt {
        let else_trimmed = else_raw.trim();
        if else_trimmed.is_empty() {
            None
        } else {
            let actual_else_expr = if let Some(e_eq) = else_trimmed.find('=') {
                else_trimmed[e_eq + 1..].trim()
            } else {
                else_trimmed
            };
            Some(CompiledExpr::parse(actual_else_expr).map_err(|e| map_eval_err(e, line_num))?)
        }
    } else {
        None
    };

    Ok(ProgrammabilityRule {
        target_var,
        condition: Some(condition),
        expr,
        else_expr,
        raw_line: line.to_string(),
    })
}

fn find_keyword_case_insensitive(s: &str, kw: &str) -> Option<usize> {
    let kw_len = kw.len();
    if s.len() < kw_len {
        return None;
    }
    let bytes = s.as_bytes();
    for i in 0..=(s.len() - kw_len) {
        if s[i..i + kw_len].eq_ignore_ascii_case(kw) {
            let before_ok = i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b')';
            let after_ok = i + kw_len == s.len()
                || bytes[i + kw_len].is_ascii_whitespace()
                || bytes[i + kw_len] == b':';
            if before_ok && after_ok {
                return Some(i);
            }
        }
    }
    None
}

fn validate_target_var(var: &str, line_num: usize) -> Result<(), ProgrammabilityError> {
    if var.is_empty() {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: "Variable name cannot be empty".to_string(),
        });
    }
    let first = var.chars().next().unwrap_or(' ');
    if !first.is_ascii_alphabetic() && first != '_' && first != '@' && first != '$' {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: format!("Variable name '{}' must begin with a letter or '_'", var),
        });
    }
    if !var
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(ProgrammabilityError::ParseError {
            line: line_num,
            message: format!("Variable name '{}' contains invalid characters", var),
        });
    }
    Ok(())
}

fn map_eval_err(err: EvalError, line_num: usize) -> ProgrammabilityError {
    ProgrammabilityError::ParseError {
        line: line_num,
        message: err.to_string(),
    }
}

pub fn execute_programmability(
    dataset: &mut DataSet,
    script: &ProgrammabilityScript,
) -> Result<ProgrammabilityResult, ProgrammabilityError> {
    let row_count = dataset.row_count();
    let mut created_set = HashSet::new();
    let mut modified_set = HashSet::new();
    let mut total_rows_affected = 0;
    let mut logs = Vec::new();

    for rule in &script.rules {
        let existing_idx = dataset.dictionary.find_index(&rule.target_var);
        let col_idx = match existing_idx {
            Some(idx) => {
                if !created_set.contains(&rule.target_var) {
                    modified_set.insert(rule.target_var.clone());
                }
                idx
            }
            None => {
                let new_idx = dataset
                    .add_variable(
                        &rule.target_var,
                        DataType::Numeric { decimals: 2 },
                        Measure::Scale,
                    )
                    .map_err(|e| ProgrammabilityError::ExecutionError(e.to_string()))?;
                created_set.insert(rule.target_var.clone());
                new_idx
            }
        };

        let mut affected_in_rule = 0;

        for row_idx in 0..row_count {
            let lookup = |var_name: &str| -> Option<f64> {
                let idx = dataset.dictionary.find_index(var_name)?;
                match dataset.get_cell(row_idx, idx)? {
                    RawValue::Number(n) => Some(*n),
                    RawValue::Text(s) => s.parse::<f64>().ok(),
                    RawValue::Null => Some(f64::NAN),
                }
            };

            let target_value = if let Some(cond) = &rule.condition {
                let cond_res = cond.eval(&lookup).unwrap_or(f64::NAN);
                let cond_true = !cond_res.is_nan() && cond_res != 0.0;
                if cond_true {
                    let v = rule.expr.eval(&lookup).unwrap_or(f64::NAN);
                    Some(v)
                } else if let Some(else_expr) = &rule.else_expr {
                    let v = else_expr.eval(&lookup).unwrap_or(f64::NAN);
                    Some(v)
                } else {
                    None
                }
            } else {
                let v = rule.expr.eval(&lookup).unwrap_or(f64::NAN);
                Some(v)
            };

            if let Some(val) = target_value {
                let cell_val = if val.is_nan() {
                    RawValue::Null
                } else {
                    RawValue::Number(val)
                };
                if let Some(col) = dataset.table.columns.get_mut(col_idx) {
                    if let Some(slot) = col.values.get_mut(row_idx) {
                        *slot = cell_val;
                        affected_in_rule += 1;
                    }
                }
            }
        }

        total_rows_affected += affected_in_rule;
        logs.push(format!(
            "Executed: {} (rows affected: {})",
            rule.raw_line, affected_in_rule
        ));
    }

    Ok(ProgrammabilityResult {
        created_vars: created_set.into_iter().collect(),
        modified_vars: modified_set.into_iter().collect(),
        rows_affected: total_rows_affected,
        statements_count: script.rules.len(),
        logs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_dataset() -> DataSet {
        let mut ds = DataSet::empty();
        ds.table.row_count = 3;
        let c0 = ds
            .add_variable("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .unwrap();
        let c1 = ds
            .add_variable("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
            .unwrap();

        ds.table.columns[c0].values = vec![
            RawValue::Number(10000.0),
            RawValue::Number(20000.0),
            RawValue::Number(50000.0),
        ];
        ds.table.columns[c1].values = vec![
            RawValue::Number(12.0),
            RawValue::Number(14.0),
            RawValue::Number(16.0),
        ];
        ds
    }

    #[test]
    fn test_parse_simple_assignment() {
        let script = ProgrammabilityScript::parse("bonus = salary * 0.1").unwrap();
        assert_eq!(script.rules.len(), 1);
        assert_eq!(script.rules[0].target_var, "bonus");
        assert!(script.rules[0].condition.is_none());
    }

    #[test]
    fn test_parse_conditional_then_else() {
        let script = ProgrammabilityScript::parse(
            "IF educ >= 16 THEN high_educ = 1 ELSE 0\n# comment\nscaled = salary / 1000",
        )
        .unwrap();
        assert_eq!(script.rules.len(), 2);
        assert_eq!(script.rules[0].target_var, "high_educ");
        assert!(script.rules[0].condition.is_some());
        assert!(script.rules[0].else_expr.is_some());
        assert_eq!(script.rules[1].target_var, "scaled");
    }

    #[test]
    fn test_execute_programmability_transform() {
        let mut ds = sample_dataset();
        let script = ProgrammabilityScript::parse(
            "ratio = salary / educ\nIF educ >= 16 THEN high_flag = 1 ELSE high_flag = 0",
        )
        .unwrap();

        let res = execute_programmability(&mut ds, &script).unwrap();
        assert_eq!(res.statements_count, 2);
        assert_eq!(res.created_vars.len(), 2);

        let ratio_idx = ds.dictionary.find_index("ratio").unwrap();
        assert_eq!(
            ds.get_cell(0, ratio_idx),
            Some(&RawValue::Number(10000.0 / 12.0))
        );

        let flag_idx = ds.dictionary.find_index("high_flag").unwrap();
        assert_eq!(ds.get_cell(0, flag_idx), Some(&RawValue::Number(0.0)));
        assert_eq!(ds.get_cell(1, flag_idx), Some(&RawValue::Number(0.0)));
        assert_eq!(ds.get_cell(2, flag_idx), Some(&RawValue::Number(1.0)));
    }

    #[test]
    fn test_empty_script_fails() {
        let res = ProgrammabilityScript::parse("# only comments\n   \n");
        assert_eq!(res, Err(ProgrammabilityError::EmptyScript));
    }
}
