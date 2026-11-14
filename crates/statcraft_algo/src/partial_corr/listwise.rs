use crate::anova_oneway::AlgoError;
use crate::correlations::{compute_bivariate_correlations, BivariateCorrelationsOptions};
use crate::correlations_types::{CorrelationType, MissingHandling, VarDescriptives, VariableInput};
use crate::partial_corr::math::{calc_pearson_slice, calculate_sig, matrix_multiply};
use crate::partial_corr::types::{PartialCorrCell, PartialCorrOptions, PartialCorrResult};
use crate::regression::invert_matrix;

/// Computes partial correlations under Listwise deletion (canonical SPSS mode).
pub fn compute_partial_listwise(
    variables: &[VariableInput<'_>],
    controls: &[VariableInput<'_>],
    var_names: &[String],
    control_names: &[String],
    options: &PartialCorrOptions,
) -> Result<PartialCorrResult, AlgoError> {
    let p = variables.len();
    let k = controls.len();
    let n_rows = variables[0].1.len();

    // 1. Build listwise mask: row is valid if all variables and controls are finite
    let mut valid_mask = vec![true; n_rows];
    for (_, vals) in variables.iter().chain(controls.iter()) {
        for (i, val) in vals.iter().enumerate() {
            if val.is_none_or(|v| !v.is_finite()) {
                valid_mask[i] = false;
            }
        }
    }

    // Filtered vectors of valid values
    let mut all_data: Vec<Vec<f64>> = Vec::with_capacity(p + k);
    for (_, vals) in variables.iter().chain(controls.iter()) {
        let col: Vec<f64> = vals
            .iter()
            .enumerate()
            .filter(|(idx, _)| valid_mask[*idx])
            .filter_map(|(_, &v)| v)
            .collect();
        all_data.push(col);
    }

    let valid_n = if let Some(first) = all_data.first() {
        first.len()
    } else {
        0
    };

    if valid_n <= k + 2 {
        return Err(AlgoError::InsufficientData(format!(
            "Valid sample size ({valid_n}) must be greater than number of controls ({k}) + 2"
        )));
    }

    // 2. Compute correlation matrix of all p + k variables
    let total_vars = p + k;
    let mut corr_matrix = vec![vec![0.0; total_vars]; total_vars];

    for i in 0..total_vars {
        corr_matrix[i][i] = 1.0;
        for j in (i + 1)..total_vars {
            let r = calc_pearson_slice(&all_data[i], &all_data[j])?;
            corr_matrix[i][j] = r;
            corr_matrix[j][i] = r;
        }
    }

    // 3. Extract blocks R_XX (p x p), R_XZ (p x k), R_ZX (k x p), R_ZZ (k x k)
    let mut r_xx = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in 0..p {
            r_xx[i][j] = corr_matrix[i][j];
        }
    }

    let mut r_xz = vec![vec![0.0; k]; p];
    for i in 0..p {
        for j in 0..k {
            r_xz[i][j] = corr_matrix[i][p + j];
        }
    }

    let mut r_zx = vec![vec![0.0; p]; k];
    for i in 0..k {
        for j in 0..p {
            r_zx[i][j] = corr_matrix[p + i][j];
        }
    }

    let mut r_zz = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in 0..k {
            r_zz[i][j] = corr_matrix[p + i][p + j];
        }
    }

    // Invert R_ZZ
    let inv_r_zz = invert_matrix(&r_zz).map_err(|_| {
        AlgoError::ComputationError(
            "Singular matrix: controlling variables are perfectly collinear".to_string(),
        )
    })?;

    // Schur complement: S = R_XX - (R_XZ * inv_R_ZZ * R_ZX)
    let m = matrix_multiply(&r_xz, &inv_r_zz); // p x k
    let t = matrix_multiply(&m, &r_zx); // p x p

    let mut s = vec![vec![0.0; p]; p];
    for i in 0..p {
        for j in 0..p {
            s[i][j] = r_xx[i][j] - t[i][j];
        }
    }

    // Symmetrize S to ensure numerical stability and exact symmetry
    #[allow(clippy::needless_range_loop)]
    for i in 0..p {
        for j in (i + 1)..p {
            let avg = (s[i][j] + s[j][i]) * 0.5;
            s[i][j] = avg;
            s[j][i] = avg;
        }
    }

    let df = valid_n - k - 2;

    // 4. Construct cells of partial correlation matrix
    let mut cells = vec![vec![PartialCorrCell::empty(); p]; p];
    for i in 0..p {
        for j in 0..p {
            if i == j {
                cells[i][j] = PartialCorrCell {
                    r: Some(1.0),
                    p_val: None,
                    df: 0,
                    n: valid_n,
                };
            } else {
                let denom = (s[i][i] * s[j][j]).sqrt();
                if denom > 1e-14 && denom.is_finite() {
                    let r = (s[i][j] / denom).clamp(-1.0, 1.0);
                    let (p_val, cell_df) = calculate_sig(r, df, options.significance);
                    cells[i][j] = PartialCorrCell {
                        r: Some(r),
                        p_val,
                        df: cell_df,
                        n: valid_n,
                    };
                } else {
                    cells[i][j] = PartialCorrCell {
                        r: None,
                        p_val: None,
                        df,
                        n: valid_n,
                    };
                }
            }
        }
    }

    // 5. Descriptives if requested
    let descriptives = if options.descriptives {
        let mut desc = Vec::with_capacity(total_vars);
        for (idx, (name, _)) in variables.iter().chain(controls.iter()).enumerate() {
            let col = &all_data[idx];
            let n = col.len();
            if n > 0 {
                let mean = col.iter().sum::<f64>() / (n as f64);
                let var = if n > 1 {
                    col.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / ((n - 1) as f64)
                } else {
                    0.0
                };
                desc.push(VarDescriptives {
                    var_name: (*name).to_string(),
                    mean: Some(mean),
                    std_dev: Some(var.sqrt()),
                    n,
                });
            }
        }
        Some(desc)
    } else {
        None
    };

    // 6. Zero-order correlations if requested
    let zero_order = if options.zero_order_correlations {
        let biv_opts = BivariateCorrelationsOptions {
            pearson: true,
            spearman: false,
            kendall: false,
            significance: options.significance,
            flag_significant: false,
            missing: MissingHandling::Listwise,
            descriptives: false,
            cross_prod_cov: false,
        };
        let biv_res = compute_bivariate_correlations(variables, &biv_opts)?;
        biv_res
            .matrices
            .into_iter()
            .find(|m| m.corr_type == CorrelationType::Pearson)
    } else {
        None
    };

    Ok(PartialCorrResult {
        variables: var_names.to_vec(),
        controls: control_names.to_vec(),
        cells,
        descriptives,
        zero_order,
    })
}
