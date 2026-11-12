use crate::command::{CommandError, EngineCommand};
use serde_json::{json, Value};
use statcraft_algo::{compute_explore, ExploreMissingHandling, ExploreOptions};
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
                "compute_descriptives": {
                    "type": "boolean",
                    "description": "Compute M-estimators, trimmed mean, IQR, etc.",
                    "default": true
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
                "confidence_level": {
                    "type": "number",
                    "description": "Confidence level for mean CI (e.g. 0.95)",
                    "default": 0.95
                },
                "missing": {
                    "type": "string",
                    "enum": ["listwise", "pairwise"],
                    "description": "Missing value treatment mode",
                    "default": "listwise"
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

        let compute_descriptives = params
            .get("compute_descriptives")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let compute_normality = params
            .get("compute_normality")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let compute_extremes = params
            .get("compute_extremes")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let confidence_level = params
            .get("confidence_level")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.95);

        let missing = match params.get("missing").and_then(|v| v.as_str()) {
            Some(s) if s.eq_ignore_ascii_case("pairwise") => ExploreMissingHandling::Pairwise,
            _ => ExploreMissingHandling::Listwise,
        };

        let opts = ExploreOptions {
            dependent_vars,
            factor_vars,
            compute_descriptives,
            compute_normality,
            compute_extremes,
            confidence_level,
            missing,
        };

        let res =
            compute_explore(ds, &opts).map_err(|e| CommandError::ExecutionFailed(e.to_string()))?;

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_algo::ExploreResult;
    use statcraft_core::DataSet;
    use statcraft_io::{load_csv_from_reader, CsvOptions};

    fn make_test_dataset() -> DataSet {
        let csv = "salary,age,gender\n35000,25,1\n45000,30,2\n55000,35,1\n65000,40,2\n75000,45,1\n";
        let table = load_csv_from_reader(csv.as_bytes(), CsvOptions::default()).unwrap();
        DataSet::from_raw_table(table).unwrap()
    }

    #[test]
    fn test_explore_specs() {
        let cmd = ExploreCommand;
        assert_eq!(cmd.id(), "stat.explore");
        assert_eq!(
            cmd.menu_path(),
            "Analyze > Descriptive Statistics > Explore"
        );
        let specs = cmd.specs();
        assert_eq!(specs["type"], "object");
        assert!(specs["properties"]["dependent_vars"].is_object());
        assert!(specs["properties"]["vars"].is_object());
        assert!(specs["properties"]["factor_vars"].is_object());
    }

    #[test]
    fn test_explore_execute_without_factor() {
        let cmd = ExploreCommand;
        let mut ds = Some(make_test_dataset());

        assert!(cmd.enabled(ds.as_ref()));

        let params = json!({
            "vars": ["salary", "age"],
            "compute_descriptives": true,
            "compute_normality": true,
            "compute_extremes": true,
            "confidence_level": 0.95
        });

        let res_val = cmd
            .execute(&mut ds, params)
            .expect("Execution without factor should succeed");
        let res: ExploreResult = serde_json::from_value(res_val).expect("Valid ExploreResult JSON");

        assert_eq!(res.variables.len(), 2);
        assert_eq!(res.variables[0].variable, "salary");
        assert!(res.variables[0].factor_var.is_none());
        assert_eq!(res.variables[0].groups.len(), 1);
        assert_eq!(res.variables[0].groups[0].case_processing.n_valid, 5);

        let desc = res.variables[0].groups[0].descriptives.as_ref().unwrap();
        assert_eq!(desc.mean, Some(55000.0));
        assert_eq!(desc.median, Some(55000.0));

        let norm = res.variables[0].groups[0].normality.as_ref().unwrap();
        assert!(norm.kolmogorov_smirnov.is_some());
        assert!(norm.shapiro_wilk.is_some());

        let ext = res.variables[0].groups[0].extremes.as_ref().unwrap();
        assert_eq!(ext.highest.len(), 5);
        assert_eq!(ext.lowest.len(), 5);
    }

    #[test]
    fn test_explore_execute_with_factor() {
        let cmd = ExploreCommand;
        let mut ds = Some(make_test_dataset());

        let params = json!({
            "dependent_vars": ["salary"],
            "factor_vars": ["gender"],
            "compute_descriptives": true,
            "compute_normality": false,
            "compute_extremes": false
        });

        let res_val = cmd
            .execute(&mut ds, params)
            .expect("Execution with factor should succeed");
        let res: ExploreResult = serde_json::from_value(res_val).expect("Valid ExploreResult JSON");

        assert_eq!(res.variables.len(), 1);
        assert_eq!(res.variables[0].variable, "salary");
        assert_eq!(res.variables[0].factor_var.as_deref(), Some("gender"));
        assert_eq!(res.variables[0].groups.len(), 2);

        // Group 1: 3 rows (35000, 55000, 75000) -> mean = 55000
        let g1 = &res.variables[0].groups[0];
        assert_eq!(g1.case_processing.n_valid, 3);
        assert_eq!(g1.descriptives.as_ref().unwrap().mean, Some(55000.0));
        assert!(g1.normality.is_none());
        assert!(g1.extremes.is_none());

        // Group 2: 2 rows (45000, 65000) -> mean = 55000
        let g2 = &res.variables[0].groups[1];
        assert_eq!(g2.case_processing.n_valid, 2);
        assert_eq!(g2.descriptives.as_ref().unwrap().mean, Some(55000.0));
    }

    #[test]
    fn test_explore_to_sps() {
        let deps = vec!["salary".to_string(), "age".to_string()];
        let empty_factor: Vec<String> = vec![];
        let sps1 = ExploreCommand::to_sps(&deps, &empty_factor);
        assert_eq!(
            sps1,
            "EXAMINE VARIABLES=salary age\n  /PLOT NPPLOT\n  /STATISTICS DESCRIPTIVES EXTREME."
        );

        let factor = vec!["gender".to_string()];
        let sps2 = ExploreCommand::to_sps(&deps, &factor);
        assert_eq!(
            sps2,
            "EXAMINE VARIABLES=salary age BY gender\n  /PLOT NPPLOT\n  /STATISTICS DESCRIPTIVES EXTREME."
        );
    }
}
