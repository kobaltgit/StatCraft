use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::ParsedCommand;
use serde_json::{json, Value};
use statcraft_core::DataSet;

pub fn map_deming_regression(
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

    let lambda = cmd
        .subcommand_tokens("LAMBDA")
        .and_then(|t| t.first())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);

    let ci = cmd
        .subcommand_tokens("CI")
        .and_then(|t| t.first())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.95);

    let has_plot = cmd.has_subcommand("PLOT");
    let plot_none = cmd.has_token("PLOT", "NONE");
    let include_scatter_plot = !plot_none && (!has_plot || cmd.has_token("PLOT", "SCATTER"));
    let include_identity_line = !plot_none && (!has_plot || cmd.has_token("PLOT", "IDENTITY"));
    let include_residual_plot = !plot_none && cmd.has_token("PLOT", "RESIDUAL");

    let save_predicted = cmd.has_token("SAVE", "PRE");
    let save_residuals = cmd.has_token("SAVE", "RES");

    Ok(Some((
        "stat.regression.orthogonal",
        json!({
            "dependent_var": dep_var,
            "independent_var": indep_var,
            "lambda": lambda,
            "ci": ci,
            "include_scatter_plot": include_scatter_plot,
            "include_identity_line": include_identity_line,
            "include_residual_plot": include_residual_plot,
            "save_predicted": save_predicted,
            "save_residuals": save_residuals,
        }),
    )))
}
