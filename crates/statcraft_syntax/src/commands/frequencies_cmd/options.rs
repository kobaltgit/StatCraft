use crate::command::CommandError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use statcraft_algo::{ChartType, ChartValueMode, FrequenciesStatisticsOptions, FrequencyChartSpec};
use statcraft_core::DataSet;
use statcraft_io::RawValue;

pub(super) fn parse_options(params: &Value) -> Option<FrequenciesStatisticsOptions> {
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

pub(super) fn build_strata_keys(
    ds: &DataSet,
    active_rows: &[usize],
    strata_vars: &[String],
) -> Result<Vec<String>, CommandError> {
    let mut col_indices = Vec::with_capacity(strata_vars.len());
    for sv in strata_vars {
        let idx = ds
            .dictionary
            .find_index(sv)
            .ok_or_else(|| CommandError::InvalidParams {
                id: "stat.frequencies",
                reason: format!("Strata variable '{}' not found in dataset", sv),
            })?;
        col_indices.push(idx);
    }

    let mut keys = Vec::with_capacity(active_rows.len());
    for &r in active_rows {
        let mut parts = Vec::with_capacity(col_indices.len());
        for &c in &col_indices {
            let s = match ds.get_cell(r, c) {
                Some(RawValue::Number(n)) => {
                    if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        n.to_string()
                    }
                }
                Some(RawValue::Text(t)) => t.trim().to_string(),
                Some(RawValue::Null) | None => String::new(),
            };
            parts.push(s);
        }
        keys.push(parts.join("\x1F"));
    }
    Ok(keys)
}
