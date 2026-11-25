use crate::bootstrap::{bootstrap_param_schema, extract_strata_keys, parse_bootstrap_options};
use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{
    compute_explore, compute_explore_bootstrap, ExploreBoxplotDisplay, ExploreMissingHandling,
    ExploreOptions, ExploreOutputDisplay, SpreadLevelMode,
};
use statcraft_core::DataSet;

pub struct ExploreCommand;

impl EngineCommand for ExploreCommand {
    fn id(&self) -> &'static str {
        "stat.explore"
    }

    fn menu_path(&self) -> &'static str {
        "Analyze > Descriptive Statistics > Explore"
    }

    fn default_shortcut(&self) -> &'static str {
        ""
    }

    fn specs(&self) -> Value {
        json!({
            "type": "object",
            "required": ["dependent_vars"],
            "properties": {
                "dependent_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "description": "List of quantitative dependent variable names to analyze"
                },
                "vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "description": "Alias for dependent_vars"
                },
                "factor_vars": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional list of categorical grouping factor variables"
                },
                "label_var": {
                    "type": "string",
                    "description": "Optional case label variable (ID) for extreme values"
                },
                "compute_descriptives": {
                    "type": "boolean",
                    "description": "Compute descriptive statistics (mean, CI, trimmed mean, IQR, etc.)",
                    "default": true
                },
                "compute_m_estimators": {
                    "type": "boolean",
                    "description": "Compute Huber, Tukey, Hampel, and Andrews M-estimators",
                    "default": false
                },
                "compute_percentiles": {
                    "type": "boolean",
                    "description": "Compute HAVERAGE percentiles (5..95) and Tukey's Hinges",
                    "default": false
                },
                "compute_normality": {
                    "type": "boolean",
                    "description": "Compute Kolmogorov-Smirnov (Lilliefors) and Shapiro-Wilk tests",
                    "default": true
                },
                "compute_extremes": {
                    "type": "boolean",
                    "description": "Compute top 5 highest and bottom 5 lowest extreme values",
                    "default": true
                },
                "spread_level_mode": {
                    "type": "string",
                    "enum": ["none", "power_estimation", "natural_log", "power"],
                    "description": "Spread-vs-Level and Levene homogeneity test mode",
                    "default": "none"
                },
                "spread_level_power": {
                    "type": "number",
                    "description": "Explicit transformation power when spread_level_mode is 'power'"
                },
                "confidence_level": {
                    "type": "number",
                    "description": "Confidence level for mean CI (e.g. 0.95)",
                    "default": 0.95
                },
                "missing": {
                    "type": "string",
                    "enum": ["listwise", "pairwise", "report"],
                    "description": "Missing value treatment mode",
                    "default": "listwise"
                },
                "display_mode": {
                    "type": "string",
                    "enum": ["both", "statistics", "plots"],
                    "description": "Output display mode (both, statistics, or plots)",
                    "default": "both"
                },
                "boxplot_mode": {
                    "type": "string",
                    "enum": ["factor_levels_together", "dependents_together", "none"],
                    "description": "Boxplot grouping mode",
                    "default": "factor_levels_together"
                },
                "plot_stem_leaf": {
                    "type": "boolean",
                    "description": "Include Stem-and-Leaf plot",
                    "default": true
                },
                "plot_histogram": {
                    "type": "boolean",
                    "description": "Include Histogram plot",
                    "default": false
                },
                "bootstrap": bootstrap_param_schema()
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

        let dependent_vars: Vec<String> = params
            .get("dependent_vars")
            .or_else(|| params.get("vars"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        if dependent_vars.is_empty() {
            return Err(CommandError::InvalidParams {
                id: self.id(),
                reason: "Parameter 'dependent_vars' (or 'vars') must contain at least one variable"
                    .to_string(),
            });
        }

        let factor_vars: Vec<String> = params
            .get("factor_vars")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let label_var = params
            .get("label_var")
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let compute_descriptives = params
            .get("compute_descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let compute_m_estimators = params
            .get("compute_m_estimators")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let compute_percentiles = params
            .get("compute_percentiles")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let compute_normality = params
            .get("compute_normality")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let compute_extremes = params
            .get("compute_extremes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let spread_level_power = params
            .get("spread_level_power")
            .and_then(|v| v.as_f64())
            .filter(|p| p.is_finite());

        let spread_level_mode = match params.get("spread_level_mode").and_then(|v| v.as_str()) {
            Some(s) if s.eq_ignore_ascii_case("power_estimation") => {
                SpreadLevelMode::PowerEstimation
            }
            Some(s) if s.eq_ignore_ascii_case("natural_log") || s.eq_ignore_ascii_case("ln") => {
                SpreadLevelMode::NaturalLog
            }
            Some(s) if s.eq_ignore_ascii_case("power") => match spread_level_power {
                Some(p) if p.abs() < 1e-12 => SpreadLevelMode::NaturalLog,
                Some(p) => SpreadLevelMode::Power(p),
                None => SpreadLevelMode::PowerEstimation,
            },
            _ => SpreadLevelMode::None,
        };

        let confidence_level = params
            .get("confidence_level")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.95);

        let missing = match params.get("missing").and_then(|v| v.as_str()) {
            Some(s) if s.eq_ignore_ascii_case("pairwise") => ExploreMissingHandling::Pairwise,
            Some(s) if s.eq_ignore_ascii_case("report") => ExploreMissingHandling::Report,
            _ => ExploreMissingHandling::Listwise,
        };

        let display_mode = match params.get("display_mode").and_then(|v| v.as_str()) {
            Some(s) if s.eq_ignore_ascii_case("statistics") => ExploreOutputDisplay::Statistics,
            Some(s) if s.eq_ignore_ascii_case("plots") => ExploreOutputDisplay::Plots,
            _ => ExploreOutputDisplay::Both,
        };

        let boxplot_mode = match params.get("boxplot_mode").and_then(|v| v.as_str()) {
            Some(s) if s.eq_ignore_ascii_case("dependents_together") => {
                ExploreBoxplotDisplay::DependentsTogether
            }
            Some(s) if s.eq_ignore_ascii_case("none") => ExploreBoxplotDisplay::None,
            _ => ExploreBoxplotDisplay::FactorLevelsTogether,
        };

        let plot_stem_leaf = params
            .get("plot_stem_leaf")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let plot_histogram = params
            .get("plot_histogram")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let boot_options = parse_bootstrap_options(&params, self.id())?;

        let opts = ExploreOptions {
            dependent_vars,
            factor_vars,
            label_var,
            compute_descriptives,
            compute_m_estimators,
            compute_percentiles,
            compute_normality,
            compute_extremes,
            spread_level_mode,
            display_mode,
            boxplot_mode,
            plot_stem_leaf,
            plot_histogram,
            confidence_level,
            missing,
        };

        let mut res =
            compute_explore(ds, &opts).map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

        if let Some(ref boot) = boot_options {
            let active_rows = ds.active_row_indices();
            let strata_keys = extract_strata_keys(ds, &active_rows, Some(boot), self.id())?;
            let boot_res = compute_explore_bootstrap(ds, &opts, strata_keys.as_deref(), boot)
                .map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;
            res.bootstrap = Some(boot_res);
        }

        serde_json::to_value(&res).map_err(|e| CommandError::ExecutionFailed(e.to_string()))
    }

    fn undo(&self, _dataset: &mut Option<DataSet>) -> Result<(), CommandError> {
        Ok(())
    }

    fn syntax_names(&self) -> &'static [&'static str] {
        &["EXAMINE", "EXPLORE"]
    }
}

impl ExploreCommand {
    pub fn to_sps(dependent_vars: &[String], factor_vars: &[String]) -> String {
        let vars_part = if factor_vars.is_empty() {
            dependent_vars.join(" ")
        } else {
            format!("{} BY {}", dependent_vars.join(" "), factor_vars.join(" "))
        };
        format!(
            "EXAMINE VARIABLES={}\n  /PLOT NPPLOT\n  /STATISTICS DESCRIPTIVES EXTREME.",
            vars_part
        )
    }

    pub fn to_sps_full(options: &ExploreOptions) -> String {
        let vars_part = if options.factor_vars.is_empty() {
            options.dependent_vars.join(" ")
        } else {
            format!(
                "{} BY {}",
                options.dependent_vars.join(" "),
                options.factor_vars.join(" ")
            )
        };

        let mut lines = vec![format!("EXAMINE VARIABLES={}", vars_part)];

        if let Some(ref label) = options.label_var {
            let trimmed = label.trim();
            if !trimmed.is_empty() {
                lines.push(format!("  /ID={}", trimmed));
            }
        }

        let mut plot_tokens = Vec::new();
        if options.compute_normality {
            plot_tokens.push("NPPLOT".to_string());
        }
        match options.spread_level_mode {
            SpreadLevelMode::None => {}
            SpreadLevelMode::PowerEstimation => plot_tokens.push("SPREADLEVEL".to_string()),
            SpreadLevelMode::NaturalLog => plot_tokens.push("SPREADLEVEL(0)".to_string()),
            SpreadLevelMode::Power(p) => plot_tokens.push(format!("SPREADLEVEL({p})")),
        }
        if plot_tokens.is_empty() {
            lines.push("  /PLOT NONE".to_string());
        } else {
            lines.push(format!("  /PLOT {}", plot_tokens.join(" ")));
        }

        let mut stat_tokens = Vec::new();
        if options.compute_descriptives {
            stat_tokens.push("DESCRIPTIVES");
        }
        if options.compute_extremes {
            stat_tokens.push("EXTREME");
        }
        if options.compute_m_estimators {
            stat_tokens.push("MESTIMATORS");
        }
        if options.compute_percentiles {
            stat_tokens.push("PERCENTILES");
        }
        if stat_tokens.is_empty() {
            lines.push("  /STATISTICS NONE".to_string());
        } else {
            lines.push(format!("  /STATISTICS {}", stat_tokens.join(" ")));
        }

        let ci_pct = options.confidence_level * 100.0;
        if (ci_pct - ci_pct.round()).abs() < 1e-6 {
            lines.push(format!("  /CINTERVAL {}", ci_pct.round() as i64));
        } else {
            lines.push(format!("  /CINTERVAL {ci_pct}"));
        }

        let missing_kw = match options.missing {
            ExploreMissingHandling::Listwise => "LISTWISE",
            ExploreMissingHandling::Pairwise => "PAIRWISE",
            ExploreMissingHandling::Report => "REPORT",
        };
        lines.push(format!("  /MISSING {}.", missing_kw));

        lines.join("\n")
    }
}

#[cfg(test)]
#[path = "explore_cmd_tests.rs"]
mod tests;
