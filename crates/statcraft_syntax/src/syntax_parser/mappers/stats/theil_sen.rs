use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::ParsedCommand;
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_theil_sen(
    cmd: &ParsedCommand,
    _dataset: Option<&DataSet>,
) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    let dep_var = cmd
        .subcommand_tokens("DEPENDENT")
        .and_then(|t| t.first())
        .map(|s| s.to_string())
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "DEPENDENT".to_string(),
            reason: "Dependent variable is required".to_string(),
        })?;

    let indep_var = cmd
        .subcommand_tokens("INDEPENDENT")
        .and_then(|t| t.first())
        .map(|s| s.to_string())
        .ok_or_else(|| SyntaxError::InvalidSubcommand {
            line: cmd.line_num,
            command: cmd.name.clone(),
            subcommand: "INDEPENDENT".to_string(),
            reason: "Independent variable is required".to_string(),
        })?;

    let ci = cmd
        .subcommand_tokens("CI")
        .and_then(|t| t.first())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.95);

    let mann_kendall = !cmd.has_token("STATISTICS", "NONE");

    let has_plot = cmd.has_subcommand("PLOT");
    let include_scatter_plot = !has_plot || cmd.has_token("PLOT", "SCATTER");
    let include_residual_plot = cmd.has_token("PLOT", "RESIDUAL");

    let save_predicted = cmd.has_token("SAVE", "PRE");
    let save_residuals = cmd.has_token("SAVE", "RES");

    Ok(Some((
        "stat.regression.theil_sen",
        json!({
            "dependent_var": dep_var,
            "independent_var": indep_var,
            "ci": ci,
            "mann_kendall": mann_kendall,
            "include_scatter_plot": include_scatter_plot,
            "include_residual_plot": include_residual_plot,
            "save_predicted": save_predicted,
            "save_residuals": save_residuals,
        }),
    )))
}
