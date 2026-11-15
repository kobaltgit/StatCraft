use crate::anova_oneway::AlgoError;
use crate::factor::jacobi_eigen_symmetric;

#[derive(Debug, Clone)]
pub struct FactorLevelInfo {
    pub name: String,
    pub levels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermType {
    Intercept,
    Factor(String),
    Interaction(Vec<String>),
    Covariate(String),
}

#[derive(Debug, Clone)]
pub struct ModelTerm {
    pub name: String,
    pub term_type: TermType,
    pub column_indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DesignMatrix {
    pub n_rows: usize,
    pub terms: Vec<ModelTerm>,
    pub matrix: Vec<Vec<f64>>, // n_rows x n_cols
}

/// Solves symmetric positive semidefinite pseudo-inverse using Jacobi eigenvalue decomposition.
/// Computes (X^T X)^+ and returns (beta, rank).
#[allow(clippy::needless_range_loop)]
pub fn solve_ols_svd(x: &[Vec<f64>], y: &[f64]) -> Result<(f64, usize, Vec<f64>), AlgoError> {
    let n = x.len();
    if n == 0 || y.len() != n {
        return Err(AlgoError::InvalidInput(
            "Invalid dimensions in solve_ols".to_string(),
        ));
    }
    let p = if n > 0 { x[0].len() } else { 0 };
    if p == 0 {
        // Model with 0 predictors: SS_res = sum(y^2), rank = 0
        let ss_res: f64 = y.iter().map(|v| v * v).sum();
        return Ok((ss_res, 0, Vec::new()));
    }

    // Compute X^T X (p x p) and X^T y (p x 1)
    let mut xtx = vec![vec![0.0; p]; p];
    let mut xty = vec![0.0; p];

    for i in 0..n {
        let yi = y[i];
        let row = &x[i];
        for j in 0..p {
            xty[j] += row[j] * yi;
            for k in j..p {
                xtx[j][k] += row[j] * row[k];
            }
        }
    }
    for j in 0..p {
        for k in 0..j {
            xtx[j][k] = xtx[k][j];
        }
    }

    // Jacobi eigenvalue decomposition of symmetric matrix xtx
    let (evals, evecs) = jacobi_eigen_symmetric(&xtx);

    let max_eval = evals.first().copied().unwrap_or(0.0).abs().max(1e-12);
    let threshold = 1e-10 * max_eval;

    let mut rank = 0;
    // Construct pseudoinverse pinv = V * Lambda^+ * V^T
    let mut pinv = vec![vec![0.0; p]; p];
    for idx in 0..p {
        let val = evals[idx];
        if val > threshold {
            rank += 1;
            let inv_val = 1.0 / val;
            for r in 0..p {
                for c in 0..p {
                    pinv[r][c] += evecs[r][idx] * evecs[c][idx] * inv_val;
                }
            }
        }
    }

    // beta = pinv * xty
    let mut beta = vec![0.0; p];
    for r in 0..p {
        let mut sum = 0.0;
        for c in 0..p {
            sum += pinv[r][c] * xty[c];
        }
        beta[r] = sum;
    }

    // Compute residuals e = y - X * beta and SS_res = sum(e_i^2)
    let mut ss_res = 0.0;
    for i in 0..n {
        let mut y_hat = 0.0;
        for j in 0..p {
            y_hat += x[i][j] * beta[j];
        }
        let res = y[i] - y_hat;
        ss_res += res * res;
    }

    Ok((ss_res, rank, beta))
}

/// Builds deviation contrast columns for a categorical factor.
/// Levels must be sorted. Level k (last) gets -1.0 for all contrast columns.
pub fn build_factor_deviation_contrasts(values: &[String], levels: &[String]) -> Vec<Vec<f64>> {
    let k = levels.len();
    if k <= 1 {
        return vec![Vec::new(); values.len()];
    }
    let n_contrasts = k - 1;
    let mut cols = Vec::with_capacity(values.len());

    for val in values {
        let mut row = vec![0.0; n_contrasts];
        if let Some(pos) = levels.iter().position(|l| l == val) {
            if pos < n_contrasts {
                row[pos] = 1.0;
            } else {
                // Last level has -1.0 across all contrasts
                for item in &mut row {
                    *item = -1.0;
                }
            }
        }
        cols.push(row);
    }
    cols
}

/// Builds design matrix for specified factors, interactions, and covariates.
pub fn build_design_matrix(
    n_rows: usize,
    intercept: bool,
    factors: &[(String, Vec<String>, Vec<String>)], // (name, raw_values, sorted_levels)
    interactions: &[Vec<String>],
    covariates: &[(String, Vec<f64>)], // (name, values)
) -> Result<DesignMatrix, AlgoError> {
    let mut terms = Vec::new();
    let mut columns: Vec<Vec<f64>> = Vec::new(); // n_cols x n_rows

    // 1. Intercept
    if intercept {
        let start_col = columns.len();
        columns.push(vec![1.0; n_rows]);
        terms.push(ModelTerm {
            name: "Intercept".to_string(),
            term_type: TermType::Intercept,
            column_indices: vec![start_col],
        });
    }

    // 2. Main factors (Deviation contrasts)
    let mut factor_contrasts = Vec::new();
    for (name, values, levels) in factors {
        let contr = build_factor_deviation_contrasts(values, levels);
        let n_contrasts = if levels.len() > 1 {
            levels.len() - 1
        } else {
            0
        };
        let mut term_indices = Vec::new();

        for c_idx in 0..n_contrasts {
            let col: Vec<f64> = contr.iter().map(|r| r[c_idx]).collect();
            let col_idx = columns.len();
            columns.push(col);
            term_indices.push(col_idx);
        }

        terms.push(ModelTerm {
            name: name.clone(),
            term_type: TermType::Factor(name.clone()),
            column_indices: term_indices,
        });

        factor_contrasts.push((name.clone(), contr));
    }

    // 3. Interactions (Kronecker products of factor deviation contrasts)
    for inter in interactions {
        if inter.len() < 2 {
            continue;
        }
        // Collect contrast matrices for interacting factors
        let mut inter_cols = vec![vec![1.0]; n_rows];
        for f_name in inter {
            let matching = factor_contrasts.iter().find(|(name, _)| name == f_name);
            if let Some((_, c_matrix)) = matching {
                let n_k = if !c_matrix.is_empty() {
                    c_matrix[0].len()
                } else {
                    0
                };
                if n_k == 0 {
                    continue;
                }
                let mut new_inter = Vec::with_capacity(n_rows);
                for row_idx in 0..n_rows {
                    let mut expanded = Vec::new();
                    for prev_val in &inter_cols[row_idx] {
                        for curr_val in &c_matrix[row_idx] {
                            expanded.push(prev_val * curr_val);
                        }
                    }
                    new_inter.push(expanded);
                }
                inter_cols = new_inter;
            }
        }

        let n_sub_cols = if !inter_cols.is_empty() {
            inter_cols[0].len()
        } else {
            0
        };
        let mut term_indices = Vec::new();
        for sc in 0..n_sub_cols {
            let col: Vec<f64> = inter_cols.iter().map(|r| r[sc]).collect();
            let col_idx = columns.len();
            columns.push(col);
            term_indices.push(col_idx);
        }

        terms.push(ModelTerm {
            name: inter.join(" * "),
            term_type: TermType::Interaction(inter.clone()),
            column_indices: term_indices,
        });
    }

    // 4. Covariates
    for (name, values) in covariates {
        if values.len() != n_rows {
            return Err(AlgoError::InvalidInput(format!(
                "Covariate {} length mismatch",
                name
            )));
        }
        let col_idx = columns.len();
        columns.push(values.clone());
        terms.push(ModelTerm {
            name: name.clone(),
            term_type: TermType::Covariate(name.clone()),
            column_indices: vec![col_idx],
        });
    }

    // Transpose columns to rows (n_rows x n_cols)
    let n_cols = columns.len();
    let mut matrix = vec![vec![0.0; n_cols]; n_rows];
    for r in 0..n_rows {
        for c in 0..n_cols {
            matrix[r][c] = columns[c][r];
        }
    }

    Ok(DesignMatrix {
        n_rows,
        terms,
        matrix,
    })
}

/// Extracts a submatrix of the design matrix excluding the specified term indices.
pub fn extract_submatrix(full: &DesignMatrix, exclude_col_indices: &[usize]) -> Vec<Vec<f64>> {
    let mut keep_cols = Vec::new();
    let total_cols = if full.n_rows > 0 {
        full.matrix[0].len()
    } else {
        0
    };
    for c in 0..total_cols {
        if !exclude_col_indices.contains(&c) {
            keep_cols.push(c);
        }
    }

    let mut sub = Vec::with_capacity(full.n_rows);
    for full_row in full.matrix.iter().take(full.n_rows) {
        let mut row = Vec::with_capacity(keep_cols.len());
        for &c in &keep_cols {
            row.push(full_row[c]);
        }
        sub.push(row);
    }
    sub
}
