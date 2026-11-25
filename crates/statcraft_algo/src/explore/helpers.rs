use super::descriptives::compute_explore_descriptives;
use super::extremes::compute_extreme_values_with_labels;
use super::m_estimators::compute_m_estimators;
use super::normality_ks::compute_lilliefors_ks;
use super::normality_sw::compute_shapiro_wilk;
use super::percentiles::compute_explore_percentiles;
use super::plots_data::{compute_group_plot_data, ExploreBoxplotDisplay, ExploreOutputDisplay};
use super::{ExploreCaseProcessing, ExploreGroupResult, ExploreOptions, NormalityTestsResult};
use statcraft_core::{DataSet, RawValue};

pub(super) fn extract_numeric_value(dataset: &DataSet, row: usize, col_idx: usize) -> Option<f64> {
    let raw = dataset.get_cell(row, col_idx)?;
    if let Some(meta) = dataset.dictionary.get(col_idx) {
        if meta.missing.is_missing(raw) {
            return None;
        }
    }
    match raw {
        RawValue::Number(n) if n.is_finite() => Some(*n),
        RawValue::Text(s) => s.trim().parse::<f64>().ok().filter(|x| x.is_finite()),
        _ => None,
    }
}

pub(super) fn extract_factor_value(
    dataset: &DataSet,
    row: usize,
    col_idx: usize,
) -> Option<String> {
    let raw = dataset.get_cell(row, col_idx)?;
    if let Some(meta) = dataset.dictionary.get(col_idx) {
        if meta.missing.is_missing(raw) {
            return None;
        }
    }
    match raw {
        RawValue::Text(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed == "." || trimmed == "NA" {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        RawValue::Number(n) if n.is_finite() => Some(format!("{n}")),
        _ => None,
    }
}

pub(super) fn extract_label_value(dataset: &DataSet, row: usize, col_idx: usize) -> Option<String> {
    let raw = dataset.get_cell(row, col_idx)?;
    if let Some(meta) = dataset.dictionary.get(col_idx) {
        if meta.missing.is_missing(raw) {
            return None;
        }
    }
    match raw {
        RawValue::Text(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        RawValue::Number(n) if n.is_finite() => Some(format!("{n}")),
        _ => None,
    }
}

pub(super) fn build_group_result(
    var_name: &str,
    factor_level: Option<String>,
    indexed_vals: &[(usize, f64, Option<String>)],
    n_missing: usize,
    options: &ExploreOptions,
) -> ExploreGroupResult {
    let n_valid = indexed_vals.len();
    let n_total = n_valid + n_missing;
    let (pct_valid, pct_missing, pct_total) = if n_total > 0 {
        (
            (n_valid as f64 / n_total as f64) * 100.0,
            (n_missing as f64 / n_total as f64) * 100.0,
            100.0,
        )
    } else {
        (0.0, 0.0, 0.0)
    };

    let case_processing = ExploreCaseProcessing {
        variable: var_name.to_string(),
        factor_level: factor_level.clone(),
        n_valid,
        pct_valid,
        n_missing,
        pct_missing,
        n_total,
        pct_total,
    };

    let vals: Vec<f64> = indexed_vals.iter().map(|&(_, v, _)| v).collect();

    let descriptives = if options.compute_descriptives {
        Some(compute_explore_descriptives(
            &vals,
            n_missing,
            options.confidence_level,
        ))
    } else {
        None
    };

    let m_estimators = if options.compute_m_estimators {
        Some(compute_m_estimators(&vals))
    } else {
        None
    };

    let percentiles = if options.compute_percentiles {
        Some(compute_explore_percentiles(&vals))
    } else {
        None
    };

    let normality = if options.compute_normality {
        let ks = compute_lilliefors_ks(&vals).ok();
        let sw = compute_shapiro_wilk(&vals).ok();
        Some(NormalityTestsResult {
            variable: var_name.to_string(),
            factor_level: factor_level.clone(),
            kolmogorov_smirnov: ks,
            shapiro_wilk: sw,
        })
    } else {
        None
    };

    let extremes = if options.compute_extremes {
        Some(compute_extreme_values_with_labels(
            var_name,
            factor_level.clone(),
            indexed_vals,
        ))
    } else {
        None
    };

    let include_plots = options.display_mode != ExploreOutputDisplay::Statistics;
    let plot_data = if include_plots {
        compute_group_plot_data(
            indexed_vals,
            options.boxplot_mode != ExploreBoxplotDisplay::None,
            options.plot_stem_leaf,
            options.plot_histogram,
            options.compute_normality,
        )
    } else {
        None
    };

    ExploreGroupResult {
        factor_level,
        case_processing,
        descriptives,
        m_estimators,
        percentiles,
        normality,
        extremes,
        plot_data,
    }
}
