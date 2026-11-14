use crate::anova_oneway::AlgoError;
use crate::correlations_types::VariableInput;
use crate::partial_corr::math::{calc_pearson_slice, calculate_sig, matrix_multiply};
use crate::partial_corr::types::{PartialCorrCell, PartialCorrOptions, PartialCorrResult};
use crate::regression::invert_matrix;

/// Computes partial correlations under Pairwise deletion.
pub fn compute_partial_pairwise(
    variables: &[VariableInput<'_>],
    controls: &[VariableInput<'_>],
    var_names: &[String],
    control_names: &[String],
    options: &PartialCorrOptions,
) -> Result<PartialCorrResult, AlgoError> {
    let p = variables.len();
    let k = controls.len();
    let n_rows = variables[0].1.len();

    let mut cells = vec![vec![PartialCorrCell::empty(); p]; p];

    // For each pair (i, j), filter observations where X_i, X_j and all controls are valid
    for i in 0..p {
        cells[i][i] = PartialCorrCell {
            r: Some(1.0),
            p_val: None,
            df: 0,
            n: variables[i]
                .1
                .iter()
                .filter(|v| v.is_some_and(|x| x.is_finite()))
                .count(),
        };

        for j in (i + 1)..p {
            let mut pair_mask = vec![true; n_rows];
            for (row, mask) in pair_mask.iter_mut().enumerate().take(n_rows) {
                if variables[i].1[row].is_none_or(|v| !v.is_finite())
                    || variables[j].1[row].is_none_or(|v| !v.is_finite())
                {
                    *mask = false;
                }
            }
            for ctrl in controls {
                for (row, mask) in pair_mask.iter_mut().enumerate().take(n_rows) {
                    if ctrl.1[row].is_none_or(|v| !v.is_finite()) {
                        *mask = false;
                    }
                }
            }

            let pair_data_i: Vec<f64> = variables[i]
                .1
                .iter()
                .enumerate()
                .filter(|(idx, _)| pair_mask[*idx])
                .filter_map(|(_, &v)| v)
                .collect();
            let pair_data_j: Vec<f64> = variables[j]
                .1
                .iter()
                .enumerate()
                .filter(|(idx, _)| pair_mask[*idx])
                .filter_map(|(_, &v)| v)
                .collect();

            let pair_n = pair_data_i.len();
            if pair_n <= k + 2 {
                cells[i][j] = PartialCorrCell {
                    r: None,
                    p_val: None,
                    df: 0,
                    n: pair_n,
                };
                cells[j][i] = cells[i][j].clone();
                continue;
            }

            // Collect control data for this valid pair subset
            let mut ctrl_data: Vec<Vec<f64>> = Vec::with_capacity(k);
            for ctrl in controls {
                let c_col: Vec<f64> = ctrl
                    .1
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| pair_mask[*idx])
                    .filter_map(|(_, &v)| v)
                    .collect();
                ctrl_data.push(c_col);
            }

            // Build correlation matrix for (X_i, X_j, Z_1..Z_k)
            let sub_p = 2;
            let sub_total = sub_p + k;
            let mut sub_all = Vec::with_capacity(sub_total);
            sub_all.push(pair_data_i);
            sub_all.push(pair_data_j);
            sub_all.extend(ctrl_data);

            let mut sub_corr = vec![vec![0.0; sub_total]; sub_total];
            let mut valid_corr = true;
            for r_idx in 0..sub_total {
                sub_corr[r_idx][r_idx] = 1.0;
                for c_idx in (r_idx + 1)..sub_total {
                    match calc_pearson_slice(&sub_all[r_idx], &sub_all[c_idx]) {
                        Ok(r) => {
                            sub_corr[r_idx][c_idx] = r;
                            sub_corr[c_idx][r_idx] = r;
                        }
                        Err(_) => {
                            valid_corr = false;
                            break;
                        }
                    }
                }
                if !valid_corr {
                    break;
                }
            }

            if !valid_corr {
                cells[i][j] = PartialCorrCell {
                    r: None,
                    p_val: None,
                    df: pair_n.saturating_sub(k + 2),
                    n: pair_n,
                };
                cells[j][i] = cells[i][j].clone();
                continue;
            }

            let mut sub_r_xx = vec![vec![0.0; 2]; 2];
            for a in 0..2 {
                for b in 0..2 {
                    sub_r_xx[a][b] = sub_corr[a][b];
                }
            }

            let mut sub_r_xz = vec![vec![0.0; k]; 2];
            for a in 0..2 {
                for b in 0..k {
                    sub_r_xz[a][b] = sub_corr[a][2 + b];
                }
            }

            let mut sub_r_zx = vec![vec![0.0; 2]; k];
            for a in 0..k {
                for b in 0..2 {
                    sub_r_zx[a][b] = sub_corr[2 + a][b];
                }
            }

            let mut sub_r_zz = vec![vec![0.0; k]; k];
            for a in 0..k {
                for b in 0..k {
                    sub_r_zz[a][b] = sub_corr[2 + a][2 + b];
                }
            }

            let inv_sub_zz = match invert_matrix(&sub_r_zz) {
                Ok(inv) => inv,
                Err(_) => {
                    cells[i][j] = PartialCorrCell {
                        r: None,
                        p_val: None,
                        df: pair_n.saturating_sub(k + 2),
                        n: pair_n,
                    };
                    cells[j][i] = cells[i][j].clone();
                    continue;
                }
            };

            let m = matrix_multiply(&sub_r_xz, &inv_sub_zz);
            let t = matrix_multiply(&m, &sub_r_zx);
            let mut s = vec![vec![0.0; 2]; 2];
            for a in 0..2 {
                for b in 0..2 {
                    s[a][b] = sub_r_xx[a][b] - t[a][b];
                }
            }

            let denom = (s[0][0] * s[1][1]).sqrt();
            let df = pair_n - k - 2;
            if denom > 1e-14 && denom.is_finite() {
                let r = (s[0][1] / denom).clamp(-1.0, 1.0);
                let (p_val, cell_df) = calculate_sig(r, df, options.significance);
                cells[i][j] = PartialCorrCell {
                    r: Some(r),
                    p_val,
                    df: cell_df,
                    n: pair_n,
                };
            } else {
                cells[i][j] = PartialCorrCell {
                    r: None,
                    p_val: None,
                    df,
                    n: pair_n,
                };
            }
            cells[j][i] = cells[i][j].clone();
        }
    }

    Ok(PartialCorrResult {
        variables: var_names.to_vec(),
        controls: control_names.to_vec(),
        cells,
        descriptives: None,
        zero_order: None,
    })
}
