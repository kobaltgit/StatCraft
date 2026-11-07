use crate::views::OutputViewerState;
use statcraft_algo::{
    BivariateCorrelationsResult, CrosstabOptions, CrosstabResult, DescriptivesResult,
    FactorAnalysisResult, FrequencyResult, IndepTTestResult, LinearRegressionResult, MeansOptions,
    MeansProcedureResult, OneSampleTTestResult, OneWayAnovaResult, PairedTTestResult,
    ReliabilityResult,
};
use statcraft_core::DataSet;
use statcraft_syntax::{CommandBus, SyntaxDispatcher, SyntaxError, SyntaxLexer, SyntaxParser};

#[derive(Debug, Clone, Default)]
pub struct SyntaxBatchOutcome {
    pub count: usize,
    pub output_text: String,
}

/// Executes a batch of SPSS syntax commands, echoing each statement to the
/// Output Viewer log and attaching statistical result tables to the report tree.
pub fn run_syntax_batch(
    script: &str,
    dataset: &mut Option<DataSet>,
    bus: &mut CommandBus,
    output_view: &mut OutputViewerState,
) -> Result<SyntaxBatchOutcome, SyntaxError> {
    let blocks = SyntaxLexer::split_commands(script)?;
    let mut count = 0;
    let initial_proc_count = output_view.procedures.len();

    for block in &blocks {
        if block.is_comment {
            output_view.add_log(&block.raw_text);
            count += 1;
            continue;
        }

        let parsed = SyntaxParser::parse_command(block)?;
        output_view.add_log(format!("> {}", block.raw_text.trim()));

        let res_val = match SyntaxDispatcher::dispatch(&parsed, dataset, bus) {
            Ok(v) => v,
            Err(err) => {
                output_view.add_log(format!("Error: {}", err));
                return Err(err);
            }
        };

        // If the command returned a known statistical result model, render it in OutputViewer
        match parsed.name.as_str() {
            "HELP" => {
                let text = res_val.get("output").and_then(|v| v.as_str()).unwrap_or("");
                output_view.add_entry("Help".to_string(), text.to_string());
            }
            "DESCRIPTIVES" => {
                if let Ok(results) = serde_json::from_value::<Vec<DescriptivesResult>>(res_val) {
                    output_view.add_descriptives(results);
                }
            }
            "FREQUENCIES" => {
                if let Ok(results) = serde_json::from_value::<Vec<FrequencyResult>>(res_val) {
                    output_view.add_frequencies(results);
                }
            }
            "CROSSTABS" => {
                if let Ok(results) = serde_json::from_value::<Vec<CrosstabResult>>(res_val) {
                    output_view.add_crosstabs(results, CrosstabOptions::default());
                }
            }
            "T-TEST" => {
                if let Ok(results) =
                    serde_json::from_value::<Vec<IndepTTestResult>>(res_val.clone())
                {
                    output_view.add_indep_ttest(results, "", "", "", 0.95);
                } else if let Ok(results) =
                    serde_json::from_value::<Vec<PairedTTestResult>>(res_val.clone())
                {
                    output_view.add_paired_ttest(results, &[], 0.95);
                } else if let Ok(results) =
                    serde_json::from_value::<Vec<OneSampleTTestResult>>(res_val)
                {
                    output_view.add_onesample_ttest(results, 0.0, 0.95);
                }
            }
            "ONEWAY" => {
                if let Ok(results) = serde_json::from_value::<Vec<OneWayAnovaResult>>(res_val) {
                    output_view.add_oneway_anova(results, 0.95);
                }
            }
            "CORRELATIONS" => {
                if let Ok(results) = serde_json::from_value::<BivariateCorrelationsResult>(res_val)
                {
                    let raw_vars = parsed.subcommand_tokens("VARIABLES").unwrap_or(&[]);
                    let vars = SyntaxParser::expand_variables(raw_vars, dataset.as_ref())
                        .unwrap_or_default();
                    output_view.add_correlations(results, &vars);
                }
            }
            "REGRESSION" => {
                if let Ok(result) = serde_json::from_value::<LinearRegressionResult>(res_val) {
                    output_view.add_linear_regression(result);
                }
            }
            "RELIABILITY" => {
                if let Ok(result) = serde_json::from_value::<ReliabilityResult>(res_val) {
                    let raw_vars = parsed.subcommand_tokens("VARIABLES").unwrap_or(&[]);
                    let vars = SyntaxParser::expand_variables(raw_vars, dataset.as_ref())
                        .unwrap_or_default();
                    output_view.add_reliability(result, vars);
                }
            }
            "FACTOR" => {
                if let Ok(result) = serde_json::from_value::<FactorAnalysisResult>(res_val) {
                    let raw_vars = parsed.subcommand_tokens("VARIABLES").unwrap_or(&[]);
                    let vars = SyntaxParser::expand_variables(raw_vars, dataset.as_ref())
                        .unwrap_or_default();
                    output_view.add_factor(result, vars);
                }
            }
            "MEANS" => {
                if let Ok(result) = serde_json::from_value::<MeansProcedureResult>(res_val.clone())
                {
                    let opts = MeansOptions::default();
                    output_view.add_means(result, Vec::new(), Vec::new(), &opts);
                } else {
                    output_view.add_generic_result(&parsed.name, &block.raw_text, &res_val);
                }
            }
            _ => {
                if parsed.name != "COMMENT" && parsed.name != "EXECUTE" {
                    output_view.add_generic_result(&parsed.name, &block.raw_text, &res_val);
                }
            }
        }

        count += 1;
    }

    let output_text = if output_view.procedures.len() > initial_proc_count {
        crate::views::output_view::export_procedures_to_txt(
            &output_view.procedures[initial_proc_count..],
        )
    } else {
        String::new()
    };

    Ok(SyntaxBatchOutcome { count, output_text })
}
