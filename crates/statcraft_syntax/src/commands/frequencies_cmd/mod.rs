pub mod sps;
#[cfg(test)]
mod tests;

use crate::command::{CommandError, EngineCommand};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_frequencies_full, compute_frequency_chart, CategoryOrder, ChartType, ChartValueMode,
    FrequenciesFormatOptions, FrequenciesStatisticsOptions, FrequencyChartSpec, FrequencyInputItem,
    FrequencyResult, MultipleVarsMode,
};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub struct FrequenciesCommand;

impl EngineCommand for FrequenciesCommand {
    fn id(&self) -> &'static str {
        "stat.frequencies"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > Frequencies"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["vars"],
            "properties": {
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of variable names to analyze"
                },
                "display_tables": {
                    "type": "boolean",
                    "description": "Display frequency tables",
                    "default": true
                },
                "statistics": {
                    "type": "object",
                    "description": "Univariate statistics to compute",
                    "properties": {
                        "mean": { "type": "boolean" },
                        "se_mean": { "type": "boolean" },
                        "median": { "type": "boolean" },
                        "mode": { "type": "boolean" },
                        "sum": { "type": "boolean" },
                        "std_dev": { "type": "boolean" },
                        "variance": { "type": "boolean" },
                        "range": { "type": "boolean" },
                        "min": { "type": "boolean" },
                        "max": { "type": "boolean" },
                        "skewness": { "type": "boolean" },
                        "kurtosis": { "type": "boolean" }
                    }
                },
                "percentiles": {
                    "type": "object",
                    "description": "Percentile computation options",
                    "properties": {
                        "quartiles": { "type": "boolean" },
                        "ntiles": { "type": "integer", "minimum": 2 },
                        "custom": {
                            "type": "array",
                            "items": { "type": "number", "minimum": 0, "maximum": 100 }
                        }
                    }
                },
                "group_midpoints": {
                    "type": "boolean",
                    "description": "Assume grouped data with midpoint estimates",
                    "default": false
                },
                "order": {
                    "type": "string",
                    "enum": ["avalue", "dvalue", "afreq", "dfreq", "analysis", "variable"],
                    "description": "Category ordering (ascending/descending value or frequency)",
                    "default": "avalue"
                },
                "multiple_vars_mode": {
                    "type": "string",
                    "enum": ["compare", "variable"],
                    "description": "Organization of output for multiple variables",
                    "default": "compare"
                },
                "suppress_tables_limit": {
                    "type": ["integer", "null"],
                    "minimum": 1,
                    "description": "Suppress frequency tables if number of categories exceeds this limit"
                },
                "charts": {
                    "type": "object",
                    "description": "Chart options for frequency distributions",
                    "properties": {
                        "chart_type": {
                            "type": "string",
                            "enum": ["none", "bar", "pie", "histogram"],
                            "default": "none"
                        },
                        "values": {
                            "type": "string",
                            "enum": ["frequencies", "percentages"],
                            "default": "frequencies"
                        },
                        "normal_curve": {
                            "type": "boolean",
                            "default": false
                        }
                    }
                }
            }
        })
    }

    fn enabled(&self, dataset: Option<&DataSet>) -> bool {
        match dataset {
            Some(ds) => ds.row_count() > 0 && ds.col_count() > 0,
            None => false,
        }
    }

    fn execute(&self, dataset: &mut Option<DataSet>, params: Value) -> Result<Value, CommandError> {
        let ds = dataset
            .as_ref()
            .ok_or_else(|| CommandError::Disabled("No dataset loaded".to_string()))?;

        let var_names: Vec<String> = params
            .get("vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if var_names.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'vars' must contain at least one variable name".to_string(),
            });
        }

        let stats_options = parse_options(&params);
        let chart_spec = parse_charts_options(&params).map(|opts| opts.to_spec());
        let display_tables = params
            .get("display_tables")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let order_str = params
            .get("order")
            .and_then(|v| v.as_str())
            .unwrap_or("avalue");
        let category_order = CategoryOrder::from_str_loose(order_str);
        let multiple_vars_mode_str = params
            .get("multiple_vars_mode")
            .and_then(|v| v.as_str())
            .unwrap_or("compare");
        let multiple_vars_mode = MultipleVarsMode::from_str_loose(multiple_vars_mode_str);
        let suppress_tables_limit = params
            .get("suppress_tables_limit")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize);

        let format_options = FrequenciesFormatOptions {
            order: category_order,
            multiple_vars_mode,
            suppress_tables_limit,
        };

        let mut results: Vec<FrequencyResult> = Vec::new();
        let active_rows = ds.active_row_indices();

        for var_name in &var_names {
            let col_idx =
                ds.dictionary
                    .find_index(var_name)
                    .ok_or_else(|| CommandError::InvalidParams {
                        id: "stat.frequencies",
                        reason: format!("Variable '{}' not found in dataset", var_name),
                    })?;

            let var_meta = ds
                .dictionary
                .get(col_idx)
                .ok_or_else(|| CommandError::ExecutionFailed("Metadata missing".to_string()))?;

            let mut items: Vec<FrequencyInputItem> = Vec::with_capacity(active_rows.len());

            for &r in &active_rows {
                let cell_val = ds.get_cell(r, col_idx);
                let (value_str, is_missing, num_val) = match cell_val {
                    Some(RawValue::Number(n)) => {
                        let is_miss = var_meta.missing.is_missing(&RawValue::Number(*n));
                        let s = if n.fract() == 0.0 {
                            format!("{:.0}", n)
                        } else {
                            n.to_string()
                        };
                        (s, is_miss, Some(*n))
                    }
                    Some(RawValue::Text(s)) => {
                        let trimmed = s.trim();
                        let is_miss = trimmed.is_empty()
                            || trimmed == "."
                            || trimmed.eq_ignore_ascii_case("NA")
                            || var_meta
                                .missing
                                .is_missing(&RawValue::Text(trimmed.to_string()));
                        let num = trimmed.parse::<f64>().ok();
                        (trimmed.to_string(), is_miss, num)
                    }
                    Some(RawValue::Null) | None => (String::new(), true, None),
                };

                let label = if !is_missing {
                    var_meta.values.get_label(&value_str).map(|s| s.to_string())
                } else {
                    None
                };

                items.push(FrequencyInputItem {
                    value: value_str,
                    label,
                    is_missing,
                    numeric_value: num_val,
                    weight: ds.row_weight(r),
                });
            }

            let mut res = compute_frequencies_full(var_name, &items, stats_options.as_ref());
            res.variable_label = if var_meta.label.trim().is_empty() {
                None
            } else {
                Some(var_meta.label.clone())
            };

            // Apply category ordering
            statcraft_algo::sort_categories(&mut res.categories, format_options.order);

            if let Some(ref spec) = chart_spec {
                let valid_nums: Vec<f64> = items
                    .iter()
                    .filter(|it| !it.is_missing)
                    .filter_map(|it| it.numeric_value)
                    .collect();
                res.chart = compute_frequency_chart(
                    spec,
                    var_name,
                    res.variable_label.as_deref(),
                    &res.categories,
                    &valid_nums,
                );
            }

            // Organization mode for multiple variables
            res.multiple_vars_mode = format_options.multiple_vars_mode;

            // Suppress categories table if number of categories exceeds limit
            if let Some(limit) = format_options.suppress_tables_limit {
                if res.categories.len() > limit {
                    res.table_suppressed = true;
                    res.categories.clear();
                }
            }

            if !display_tables {
                res.categories.clear();
            }

            results.push(res);
        }

        serde_json::to_value(&results).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        // Read-only statistical procedure
        Ok(())
    }
}

fn parse_options(params: &Value) -> Option<FrequenciesStatisticsOptions> {
    let stats_val = params.get("statistics");
    let pcts_val = params.get("percentiles");
    let group_midpoints = params
        .get("group_midpoints")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if stats_val.is_none() && pcts_val.is_none() && !group_midpoints {
        return None;
    }

    let mut opts = FrequenciesStatisticsOptions {
        group_midpoints,
        ..Default::default()
    };

    if let Some(s) = stats_val {
        let b = |k| s.get(k).and_then(|v| v.as_bool()).unwrap_or(false);
        opts.mean = b("mean");
        opts.se_mean = b("se_mean");
        opts.median = b("median");
        opts.mode = b("mode");
        opts.sum = b("sum");
        opts.std_dev = b("std_dev");
        opts.variance = b("variance");
        opts.range = b("range");
        opts.min = b("min");
        opts.max = b("max");
        opts.skewness = b("skewness");
        opts.kurtosis = b("kurtosis");
    }

    if let Some(p) = pcts_val {
        opts.percentiles.quartiles = p
            .get("quartiles")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        opts.percentiles.ntiles = p.get("ntiles").and_then(|v| v.as_u64()).map(|n| n as usize);
        opts.percentiles.custom = p
            .get("custom")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_f64())
                    .filter(|&v| (0.0..=100.0).contains(&v))
                    .collect()
            })
            .unwrap_or_default();
    }

    Some(opts)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChartCommandOptions {
    pub chart_type: String, // "none" | "bar" | "pie" | "histogram"
    pub values: String,     // "frequencies" | "percentages"
    pub normal_curve: bool,
}

impl ChartCommandOptions {
    pub fn to_spec(&self) -> FrequencyChartSpec {
        let chart_type = match self.chart_type.to_ascii_lowercase().as_str() {
            "bar" => ChartType::Bar,
            "pie" => ChartType::Pie,
            "histogram" => ChartType::Histogram,
            _ => ChartType::None,
        };
        let value_mode = match self.values.to_ascii_lowercase().as_str() {
            "percentages" | "percent" => ChartValueMode::Percentages,
            _ => ChartValueMode::Frequencies,
        };
        FrequencyChartSpec {
            chart_type,
            value_mode,
            normal_curve: self.normal_curve,
        }
    }
}

impl Default for ChartCommandOptions {
    fn default() -> Self {
        Self {
            chart_type: "none".to_string(),
            values: "frequencies".to_string(),
            normal_curve: false,
        }
    }
}

pub fn parse_charts_options(params: &Value) -> Option<ChartCommandOptions> {
    params.get("charts").and_then(|c| {
        let chart_type = c
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        if chart_type == "none" {
            return None;
        }
        Some(ChartCommandOptions {
            chart_type: chart_type.to_string(),
            values: c
                .get("values")
                .and_then(|v| v.as_str())
                .unwrap_or("frequencies")
                .to_string(),
            normal_curve: c
                .get("normal_curve")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    })
}

impl FrequenciesCommand {
    pub fn to_sps(vars: &[String]) -> String {
        sps::to_sps(vars)
    }

    pub fn to_sps_syntax(&self, vars: &[String]) -> String {
        Self::to_sps(vars)
    }

    pub fn to_sps_from_params(params: &Value) -> String {
        sps::to_sps_from_params(params)
    }
}
