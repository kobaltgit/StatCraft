pub mod listwise;
pub mod math;
pub mod pairwise;
pub mod types;

#[cfg(test)]
mod tests;

pub use types::*;

use crate::anova_oneway::AlgoError;
use crate::correlations_types::{MissingHandling, VariableInput};

/// Computes partial correlation coefficients between analyzed variables controlling for one or more variables.
pub fn compute_partial_correlations(
    variables: &[VariableInput<'_>],
    controls: &[VariableInput<'_>],
    options: &PartialCorrOptions,
) -> Result<PartialCorrResult, AlgoError> {
    if variables.len() < 2 {
        return Err(AlgoError::InvalidInput(
            "At least two analyzed variables are required for partial correlations".to_string(),
        ));
    }
    if controls.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one control variable is required for partial correlations".to_string(),
        ));
    }

    let n_rows = variables[0].1.len();

    // Verify lengths
    for (name, vals) in variables.iter().chain(controls.iter()) {
        if vals.len() != n_rows {
            return Err(AlgoError::InvalidInput(format!(
                "Variable '{name}' length ({}) does not match expected length ({n_rows})",
                vals.len()
            )));
        }
    }

    let var_names: Vec<String> = variables
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();
    let control_names: Vec<String> = controls
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect();

    // Check for duplicate variable names between variables and controls
    for v in &var_names {
        if control_names.contains(v) {
            return Err(AlgoError::InvalidInput(format!(
                "Variable '{v}' cannot appear as both an analyzed and a control variable"
            )));
        }
    }

    match options.missing {
        MissingHandling::Listwise => listwise::compute_partial_listwise(
            variables,
            controls,
            &var_names,
            &control_names,
            options,
        ),
        MissingHandling::Pairwise => pairwise::compute_partial_pairwise(
            variables,
            controls,
            &var_names,
            &control_names,
            options,
        ),
    }
}
