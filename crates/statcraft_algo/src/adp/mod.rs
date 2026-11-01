pub mod outliers;
pub mod partition;
pub mod predictive_power;
pub mod screening;
#[cfg(test)]
mod tests;
pub mod transform;
pub mod types;

pub use outliers::*;
pub use partition::*;
pub use predictive_power::*;
pub use screening::*;
pub use transform::*;
pub use types::*;

use statcraft_core::{DataType, Measure, RawValue};

/// Primary entrypoint executing the SPSS Automated Data Preparation (ADP) pipeline.
pub fn compute_adp(
    inputs: &[(String, &[RawValue], DataType, Measure)],
    target: Option<(&str, &[RawValue], DataType, Measure)>,
    weights: Option<&[f64]>,
    options: &AdpOptions,
) -> Result<AdpResult, String> {
    if inputs.is_empty() {
        return Err("No input variables provided for ADP".to_string());
    }

    let n_cases = inputs[0].1.len();
    for (name, vals, _, _) in inputs {
        if vals.len() != n_cases {
            return Err(format!(
                "Input variable '{}' length ({}) does not match first variable ({})",
                name,
                vals.len(),
                n_cases
            ));
        }
    }

    if let Some((tgt_name, tgt_vals, _, _)) = target {
        if tgt_vals.len() != n_cases {
            return Err(format!(
                "Target variable '{}' length ({}) does not match inputs ({})",
                tgt_name,
                tgt_vals.len(),
                n_cases
            ));
        }
    }

    let mut field_actions = Vec::with_capacity(inputs.len());
    let mut transformed_columns = Vec::new();
    let mut screened_out_count = 0usize;
    let mut recommended_count = 0usize;
    let mut continuous_features: Vec<(String, Vec<RawValue>)> = Vec::new();

    // 1. Screening, Outliers, Rescaling, and Category Merging
    for (name, values, dtype, measure) in inputs {
        let is_numeric = matches!(dtype, DataType::Numeric { .. });
        let is_scale = *measure == Measure::Scale || is_numeric;

        // Step 1: Data Quality Screening
        let verdict = screening::evaluate_screening(values, weights, &options.screening);
        if verdict.is_screened_out {
            screened_out_count += 1;
            field_actions.push(AdpFieldAction {
                variable_name: name.clone(),
                is_screened_out: true,
                screening_reason: verdict.reason.clone(),
                action_type: "Excluded".to_string(),
                new_field_name: None,
                details: verdict
                    .reason
                    .unwrap_or_else(|| "Screened out by quality filters".to_string()),
            });
            continue;
        }

        recommended_count += 1;
        let new_col_name = format!("P_{}", name);

        if is_scale && is_numeric {
            // Step 2: Outlier detection & treatment
            let outlier_res = outliers::treat_outliers(values, weights, &options.outliers);

            // Step 3: Supervised binning OR rescaling
            let binning_target = if options.binning.bin_continuous_inputs {
                target.as_ref()
            } else {
                None
            };

            if let Some((_, tgt_vals, _, _)) = binning_target {
                match transform::apply_supervised_binning(
                    &outlier_res.treated_values,
                    tgt_vals,
                    &options.binning,
                ) {
                    Ok((binned_vals, labels)) => {
                        field_actions.push(AdpFieldAction {
                            variable_name: name.clone(),
                            is_screened_out: false,
                            screening_reason: None,
                            action_type: "Binned".to_string(),
                            new_field_name: Some(new_col_name.clone()),
                            details: format!(
                                "Supervised MDLP binning into {} intervals",
                                labels.mapping.len()
                            ),
                        });
                        transformed_columns.push(TransformedColumn {
                            name: new_col_name,
                            original_name: name.clone(),
                            values: binned_vals,
                            labels: Some(labels),
                            is_partition: false,
                        });
                    }
                    Err(_) => {
                        let rescaled = transform::rescale_numeric(
                            &outlier_res.treated_values,
                            options.rescale,
                            weights,
                        );
                        continuous_features.push((name.clone(), rescaled.clone()));
                        field_actions.push(AdpFieldAction {
                            variable_name: name.clone(),
                            is_screened_out: false,
                            screening_reason: None,
                            action_type: "Transformed".to_string(),
                            new_field_name: Some(new_col_name.clone()),
                            details: format!(
                                "Outliers treated: {}, Rescaling applied",
                                outlier_res.outlier_count
                            ),
                        });
                        transformed_columns.push(TransformedColumn {
                            name: new_col_name,
                            original_name: name.clone(),
                            values: rescaled,
                            labels: None,
                            is_partition: false,
                        });
                    }
                }
            } else {
                let rescaled = transform::rescale_numeric(
                    &outlier_res.treated_values,
                    options.rescale,
                    weights,
                );
                continuous_features.push((name.clone(), rescaled.clone()));

                let details = if outlier_res.outlier_count > 0 {
                    format!(
                        "Treated {} outliers; Rescaled ({:?})",
                        outlier_res.outlier_count, options.rescale
                    )
                } else {
                    format!("Rescaled ({:?})", options.rescale)
                };

                field_actions.push(AdpFieldAction {
                    variable_name: name.clone(),
                    is_screened_out: false,
                    screening_reason: None,
                    action_type: "Transformed".to_string(),
                    new_field_name: Some(new_col_name.clone()),
                    details,
                });
                transformed_columns.push(TransformedColumn {
                    name: new_col_name,
                    original_name: name.clone(),
                    values: rescaled,
                    labels: None,
                    is_partition: false,
                });
            }
        } else {
            // Categorical field: merge sparse categories
            let cat_res = transform::merge_sparse_categories(values, &options.categorical, weights);
            let details = if cat_res.merged_count > 0 {
                format!("Merged {} rare cases into 'Other'", cat_res.merged_count)
            } else {
                "Categories preserved".to_string()
            };

            field_actions.push(AdpFieldAction {
                variable_name: name.clone(),
                is_screened_out: false,
                screening_reason: None,
                action_type: "Merged".to_string(),
                new_field_name: Some(new_col_name.clone()),
                details,
            });
            transformed_columns.push(TransformedColumn {
                name: new_col_name,
                original_name: name.clone(),
                values: cat_res.values,
                labels: if cat_res.value_labels.is_empty() {
                    None
                } else {
                    Some(cat_res.value_labels)
                },
                is_partition: false,
            });
        }
    }

    // 2. Predictive Power ranking
    let mut predictive_powers = Vec::new();
    if let Some((_, tgt_vals, _, tgt_measure)) = target {
        for (name, values, _, measure) in inputs {
            let pwr = predictive_power::compute_predictive_power(
                name,
                values,
                *measure,
                tgt_vals,
                tgt_measure,
                weights,
            );
            predictive_powers.push(pwr);
        }
        predictive_powers.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    // 3. Interactions construction (if enabled)
    if options.interactions.create_interactions && continuous_features.len() >= 2 {
        let max_pairs = options.interactions.max_pairs.min(10);
        let mut pairs_created = 0usize;

        for i in 0..continuous_features.len() {
            for j in (i + 1)..continuous_features.len() {
                if pairs_created >= max_pairs {
                    break;
                }
                let (ref name1, ref vals1) = continuous_features[i];
                let (ref name2, ref vals2) = continuous_features[j];
                let inter_name = format!("P_{}_X_{}", name1, name2);
                let inter_vals = transform::compute_interaction_product(vals1, vals2);

                field_actions.push(AdpFieldAction {
                    variable_name: format!("{} * {}", name1, name2),
                    is_screened_out: false,
                    screening_reason: None,
                    action_type: "Constructed Interaction".to_string(),
                    new_field_name: Some(inter_name.clone()),
                    details: "Pairwise product interaction term".to_string(),
                });

                transformed_columns.push(TransformedColumn {
                    name: inter_name,
                    original_name: format!("{}_{}", name1, name2),
                    values: inter_vals,
                    labels: None,
                    is_partition: false,
                });
                pairs_created += 1;
            }
            if pairs_created >= max_pairs {
                break;
            }
        }
    }

    // 4. Sample Partitioning
    let (partition_counts, partition_col) = if options.partition.enabled {
        let part_res = partition::compute_partition(n_cases, &options.partition);
        let p_col = TransformedColumn {
            name: "_Partition_".to_string(),
            original_name: "_Partition_".to_string(),
            values: part_res.values,
            labels: Some(part_res.labels),
            is_partition: true,
        };
        (Some(part_res.counts), Some(p_col))
    } else {
        (None, None)
    };

    if let Some(p_col) = partition_col {
        transformed_columns.push(p_col);
    }

    Ok(AdpResult {
        target_variable: target.map(|(t, _, _, _)| t.to_string()),
        objective: options.objective,
        total_inputs: inputs.len(),
        screened_out_count,
        recommended_count,
        field_actions,
        predictive_powers,
        partition_counts,
        transformed_columns,
    })
}
