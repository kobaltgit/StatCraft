use crate::logistic::math::{chisq_p_value, normal_critical_value};
use crate::multinomial_logistic::engine::{fit_multinomial_engine, MultinomialEngineOutput};
use crate::multinomial_logistic::types::{
    GoodnessOfFit, LikelihoodRatioTestRow, ModelFittingInfo, MultinomialClassificationTable,
    MultinomialClassificationTableRow, MultinomialLogisticOptions, ParameterEstimateRow,
    PseudoRSquared,
};

/// Computes overall model fitting criteria (-2LL, Chi-Square, df, p-value, AIC, BIC).
pub fn compute_model_fitting_info(
    engine: &MultinomialEngineOutput,
    n: usize,
    m: usize,
    p: usize,
    include_constant: bool,
) -> ModelFittingInfo {
    let chi_sq = (engine.minus_2ll_intercept_only - engine.minus_2ll).max(0.0);
    let pred_cols = if include_constant {
        p.saturating_sub(1)
    } else {
        p
    };
    let df = m * pred_cols;
    let sig = chisq_p_value(chi_sq, df);
    let k = m * p;
    let n_f64 = (n as f64).max(1.0);
    let aic = engine.minus_2ll + 2.0 * (k as f64);
    let bic = engine.minus_2ll + (k as f64) * n_f64.ln();

    ModelFittingInfo {
        intercept_only_minus_2ll: engine.minus_2ll_intercept_only,
        final_minus_2ll: engine.minus_2ll,
        chi_square: chi_sq,
        df,
        sig,
        aic,
        bic,
    }
}

/// Computes Pearson Chi-Square and Deviance goodness-of-fit statistics across subpopulations.
pub fn compute_goodness_of_fit(
    x: &[Vec<f64>],
    y_indices: &[usize],
    predicted_probs: &[Vec<f64>],
    num_categories: usize,
    p: usize,
) -> GoodnessOfFit {
    let n = x.len();
    if n == 0 {
        return GoodnessOfFit {
            pearson_chi_square: 0.0,
            pearson_df: 0,
            pearson_sig: 1.0,
            deviance_chi_square: 0.0,
            deviance_df: 0,
            deviance_sig: 1.0,
        };
    }

    // Group observations by unique predictor covariate patterns (subpopulations)
    let mut patterns: Vec<(Vec<f64>, Vec<usize>, Vec<f64>)> = Vec::new();

    for (i, row) in x.iter().enumerate() {
        let yi = y_indices[i];
        let mut found = false;
        for (pattern_row, counts, _) in &mut patterns {
            let matches = pattern_row
                .iter()
                .zip(row.iter())
                .all(|(a, b)| (a - b).abs() < 1e-9);
            if matches {
                counts[yi] += 1;
                found = true;
                break;
            }
        }
        if !found {
            let mut counts = vec![0; num_categories];
            counts[yi] = 1;
            patterns.push((row.clone(), counts, predicted_probs[i].clone()));
        }
    }

    let num_subpop = patterns.len();
    let mut pearson = 0.0;
    let mut deviance = 0.0;

    for (_, counts, probs) in &patterns {
        let n_s: usize = counts.iter().sum();
        let n_s_f64 = n_s as f64;
        for j in 0..num_categories {
            let o_j = counts[j] as f64;
            let e_j = (n_s_f64 * probs[j]).max(1e-15);
            pearson += (o_j - e_j).powi(2) / e_j;
            if o_j > 0.0 {
                deviance += 2.0 * o_j * (o_j / e_j).ln();
            }
        }
    }

    let df = num_subpop.saturating_sub(p) * num_categories.saturating_sub(1);
    let pearson_sig = chisq_p_value(pearson, df);
    let deviance_sig = chisq_p_value(deviance, df);

    GoodnessOfFit {
        pearson_chi_square: pearson,
        pearson_df: df,
        pearson_sig,
        deviance_chi_square: deviance,
        deviance_df: df,
        deviance_sig,
    }
}

/// Computes Cox & Snell, Nagelkerke, and McFadden pseudo R-Square measures.
pub fn compute_pseudo_r_squared(
    minus_2ll_null: f64,
    minus_2ll_final: f64,
    n: usize,
) -> PseudoRSquared {
    let n_f64 = (n as f64).max(1.0);
    let chi_sq = (minus_2ll_null - minus_2ll_final).max(0.0);

    let cox_and_snell = (1.0 - (-chi_sq / n_f64).exp()).clamp(0.0, 1.0);
    let max_r2 = 1.0 - (-minus_2ll_null / n_f64).exp();
    let nagelkerke = if max_r2 > 1e-12 {
        (cox_and_snell / max_r2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mcfadden = if minus_2ll_null > 1e-12 {
        (1.0 - (minus_2ll_final / minus_2ll_null)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    PseudoRSquared {
        cox_and_snell,
        nagelkerke,
        mcfadden,
    }
}

/// Computes classification matrix comparing observed outcomes against argmax predicted outcomes.
pub fn compute_classification_table(
    y_indices: &[usize],
    predicted_probs: &[Vec<f64>],
    categories: &[f64],
    labels: &[Option<String>],
) -> MultinomialClassificationTable {
    let num_cat = categories.len();
    let mut matrix = vec![vec![0; num_cat]; num_cat];

    for (i, &obs_idx) in y_indices.iter().enumerate() {
        let probs = &predicted_probs[i];
        let mut best_cat = 0;
        let mut best_p = -1.0;
        for (j, &p) in probs.iter().enumerate() {
            if p > best_p {
                best_p = p;
                best_cat = j;
            }
        }
        if obs_idx < num_cat && best_cat < num_cat {
            matrix[obs_idx][best_cat] += 1;
        }
    }

    let mut rows = Vec::with_capacity(num_cat);
    let mut total_correct = 0;
    let total_cases = y_indices.len();

    for (i, row_counts) in matrix.into_iter().enumerate() {
        let row_total: usize = row_counts.iter().sum();
        let correct = row_counts[i];
        total_correct += correct;
        let pct = if row_total > 0 {
            (correct as f64) / (row_total as f64) * 100.0
        } else {
            0.0
        };
        rows.push(MultinomialClassificationTableRow {
            observed_category: categories[i],
            observed_label: labels.get(i).cloned().flatten(),
            predicted_counts: row_counts,
            percent_correct: pct,
        });
    }

    let overall_percent = if total_cases > 0 {
        (total_correct as f64) / (total_cases as f64) * 100.0
    } else {
        0.0
    };

    MultinomialClassificationTable {
        categories: categories.to_vec(),
        category_labels: labels.to_vec(),
        rows,
        overall_percent,
    }
}

/// Computes the detailed Parameter Estimates table (B, SE, Wald, df, Sig, Exp(B), CI).
#[allow(clippy::too_many_arguments)]
pub fn compute_parameter_estimates(
    engine: &MultinomialEngineOutput,
    categories: &[f64],
    labels: &[Option<String>],
    param_names: &[String],
    m: usize,
    p: usize,
    ref_idx: usize,
    confidence_level: f64,
) -> Vec<ParameterEstimateRow> {
    let mut result = Vec::new();
    let z = normal_critical_value(confidence_level);

    for j in 0..m {
        let cat_val = categories[j];
        let cat_label = labels.get(j).cloned().flatten();
        let offset = j * p;

        for (col, name) in param_names.iter().enumerate() {
            let r = offset + col;
            let b = engine.beta[r];
            let var = engine
                .cov_matrix
                .get(r)
                .and_then(|row| row.get(r))
                .copied()
                .unwrap_or(0.0);
            let se = var.max(0.0).sqrt();
            let wald = if se > 1e-12 { (b / se).powi(2) } else { 0.0 };
            let sig = chisq_p_value(wald, 1);
            let exp_b = b.exp();
            let ci_lower = (b - z * se).exp();
            let ci_upper = (b + z * se).exp();

            result.push(ParameterEstimateRow {
                category_value: cat_val,
                category_label: cat_label.clone(),
                param_name: name.clone(),
                b,
                std_err: se,
                wald,
                df: 1,
                sig,
                exp_b,
                ci_lower,
                ci_upper,
                is_reference_category: false,
            });
        }
    }

    // Reference category row (SPSS prints B=0, Exp(B)=1, Wald=., etc.)
    let ref_val = categories[ref_idx];
    let ref_label = labels.get(ref_idx).cloned().flatten();
    for name in param_names {
        result.push(ParameterEstimateRow {
            category_value: ref_val,
            category_label: ref_label.clone(),
            param_name: name.clone(),
            b: 0.0,
            std_err: 0.0,
            wald: 0.0,
            df: 0,
            sig: 1.0,
            exp_b: 1.0,
            ci_lower: 1.0,
            ci_upper: 1.0,
            is_reference_category: true,
        });
    }

    result
}

/// Computes Likelihood Ratio Tests for each predictor effect by fitting reduced models.
pub fn compute_likelihood_ratio_tests(
    x: &[Vec<f64>],
    y_indices: &[usize],
    category_counts: &[usize],
    m: usize,
    effect_cols: &[(String, Vec<usize>)],
    full_minus_2ll: f64,
    options: &MultinomialLogisticOptions,
) -> Vec<LikelihoodRatioTestRow> {
    let mut lrt_rows = Vec::new();

    for (effect_name, cols_to_omit) in effect_cols {
        // Build reduced X without columns in cols_to_omit
        let mut reduced_x = Vec::with_capacity(x.len());
        for row in x {
            let mut r_row = Vec::new();
            for (col_idx, &val) in row.iter().enumerate() {
                if !cols_to_omit.contains(&col_idx) {
                    r_row.push(val);
                }
            }
            reduced_x.push(r_row);
        }

        let p_reduced = if !reduced_x.is_empty() {
            reduced_x[0].len()
        } else {
            0
        };

        if p_reduced == 0 {
            continue;
        }

        let reduced_opts = MultinomialLogisticOptions {
            max_iterations: 35,
            convergence_tol: 1e-5,
            ..options.clone()
        };

        let minus_2ll_reduced = if let Ok(red_engine) = fit_multinomial_engine(
            &reduced_x,
            y_indices,
            category_counts,
            m,
            p_reduced,
            &reduced_opts,
        ) {
            red_engine.minus_2ll
        } else {
            full_minus_2ll
        };

        let chi_sq = (minus_2ll_reduced - full_minus_2ll).max(0.0);
        let df = m * cols_to_omit.len();
        let sig = chisq_p_value(chi_sq, df);

        lrt_rows.push(LikelihoodRatioTestRow {
            effect_name: effect_name.clone(),
            model_fitting_criteria_minus_2ll: minus_2ll_reduced,
            chi_square: chi_sq,
            df,
            sig,
        });
    }

    lrt_rows
}
