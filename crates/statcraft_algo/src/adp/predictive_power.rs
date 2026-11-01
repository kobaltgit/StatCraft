use super::types::AdpPredictivePower;
use crate::anova_oneway::f_distribution_sf;
use crate::ttest_onesample::student_t_sf_2tailed;
use statcraft_core::{Measure, RawValue};
use std::collections::HashMap;

/// Calculates predictive power (association score and p-value) between a predictor and the target.
pub fn compute_predictive_power(
    var_name: &str,
    feature_vals: &[RawValue],
    feature_measure: Measure,
    target_vals: &[RawValue],
    target_measure: Measure,
    weights: Option<&[f64]>,
) -> AdpPredictivePower {
    let n = feature_vals.len().min(target_vals.len());
    if n < 3 {
        return AdpPredictivePower {
            variable_name: var_name.to_string(),
            method: "N/A".to_string(),
            score: 0.0,
            p_value: None,
        };
    }

    let is_target_continuous = target_measure == Measure::Scale;
    let is_feature_continuous = feature_measure == Measure::Scale;

    if is_target_continuous && is_feature_continuous {
        // Continuous vs Continuous -> Pearson correlation r
        compute_pearson_power(var_name, feature_vals, target_vals, weights, n)
    } else if is_target_continuous && !is_feature_continuous {
        // Categorical feature vs Continuous target -> ANOVA eta^2
        compute_anova_power(var_name, target_vals, feature_vals, weights, n)
    } else if !is_target_continuous && is_feature_continuous {
        // Continuous feature vs Categorical target -> ANOVA eta^2
        compute_anova_power(var_name, feature_vals, target_vals, weights, n)
    } else {
        // Categorical vs Categorical -> Grouped ANOVA on recoded target or Pearson on codes
        compute_categorical_power(var_name, feature_vals, target_vals, weights, n)
    }
}

fn compute_pearson_power(
    var_name: &str,
    x_vals: &[RawValue],
    y_vals: &[RawValue],
    weights: Option<&[f64]>,
    n: usize,
) -> AdpPredictivePower {
    let mut pairs = Vec::with_capacity(n);
    let mut sum_w = 0.0;
    let mut sum_wx = 0.0;
    let mut sum_wy = 0.0;

    for i in 0..n {
        if let (RawValue::Number(x), RawValue::Number(y)) = (&x_vals[i], &y_vals[i]) {
            if x.is_finite() && y.is_finite() {
                let w = weights.and_then(|ws| ws.get(i)).copied().unwrap_or(1.0);
                if w > 0.0 {
                    pairs.push((*x, *y, w));
                    sum_w += w;
                    sum_wx += w * *x;
                    sum_wy += w * *y;
                }
            }
        }
    }

    let valid_n = pairs.len();
    if valid_n < 3 || sum_w <= 0.0 {
        return AdpPredictivePower {
            variable_name: var_name.to_string(),
            method: "Pearson r".to_string(),
            score: 0.0,
            p_value: None,
        };
    }

    let mean_x = sum_wx / sum_w;
    let mean_y = sum_wy / sum_w;

    let mut cov_xy = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for (x, y, w) in pairs {
        let dx = x - mean_x;
        let dy = y - mean_y;
        cov_xy += w * dx * dy;
        var_x += w * dx * dx;
        var_y += w * dy * dy;
    }

    let denom = (var_x * var_y).sqrt();
    let r = if denom > 1e-12 {
        (cov_xy / denom).clamp(-1.0, 1.0)
    } else {
        0.0
    };

    let df = (valid_n - 2) as f64;
    let r_sq = r * r;
    let p_value = if r_sq < 1.0 - 1e-12 && df > 0.0 {
        let t = r * (df / (1.0 - r_sq)).sqrt();
        Some(student_t_sf_2tailed(t.abs(), df))
    } else if r_sq >= 1.0 - 1e-12 {
        Some(0.0)
    } else {
        None
    };

    AdpPredictivePower {
        variable_name: var_name.to_string(),
        method: "Pearson r".to_string(),
        score: r.abs(),
        p_value,
    }
}

fn compute_anova_power(
    var_name: &str,
    dep_vals: &[RawValue],
    group_vals: &[RawValue],
    weights: Option<&[f64]>,
    n: usize,
) -> AdpPredictivePower {
    let mut groups: HashMap<String, Vec<(f64, f64)>> = HashMap::new(); // group -> [(dep_val, weight)]
    let mut total_w = 0.0;
    let mut grand_sum = 0.0;
    let mut valid_n = 0usize;

    for i in 0..n {
        if let RawValue::Number(y) = &dep_vals[i] {
            if y.is_finite() {
                let grp_key = match &group_vals[i] {
                    RawValue::Text(s) => {
                        let t = s.trim();
                        if t.is_empty() {
                            None
                        } else {
                            Some(t.to_string())
                        }
                    }
                    RawValue::Number(num) if num.is_finite() => {
                        if num.fract() == 0.0 {
                            Some(format!("{:.0}", num))
                        } else {
                            Some(format!("{:.4}", num))
                        }
                    }
                    _ => None,
                };

                if let Some(key) = grp_key {
                    let w = weights.and_then(|ws| ws.get(i)).copied().unwrap_or(1.0);
                    if w > 0.0 {
                        groups.entry(key).or_default().push((*y, w));
                        total_w += w;
                        grand_sum += w * *y;
                        valid_n += 1;
                    }
                }
            }
        }
    }

    let k = groups.len();
    if k < 2 || valid_n <= k || total_w <= 0.0 {
        return AdpPredictivePower {
            variable_name: var_name.to_string(),
            method: "ANOVA eta^2".to_string(),
            score: 0.0,
            p_value: None,
        };
    }

    let grand_mean = grand_sum / total_w;
    let mut ss_between = 0.0;
    let mut ss_within = 0.0;

    for items in groups.values() {
        let mut grp_w = 0.0;
        let mut grp_sum = 0.0;
        for (y, w) in items {
            grp_w += *w;
            grp_sum += *w * *y;
        }
        if grp_w > 0.0 {
            let grp_mean = grp_sum / grp_w;
            ss_between += grp_w * (grp_mean - grand_mean).powi(2);
            for (y, w) in items {
                ss_within += *w * (*y - grp_mean).powi(2);
            }
        }
    }

    let ss_total = ss_between + ss_within;
    let eta_sq = if ss_total > 1e-12 {
        (ss_between / ss_total).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let df1 = (k - 1) as f64;
    let df2 = (valid_n - k) as f64;

    let p_value = if df1 > 0.0 && df2 > 0.0 {
        if ss_within > 1e-12 {
            let ms_between = ss_between / df1;
            let ms_within = ss_within / df2;
            let f_stat = (ms_between / ms_within).max(0.0);
            Some(f_distribution_sf(f_stat, df1, df2))
        } else if ss_between > 1e-12 {
            Some(0.0)
        } else {
            None
        }
    } else {
        None
    };

    AdpPredictivePower {
        variable_name: var_name.to_string(),
        method: "ANOVA eta^2".to_string(),
        score: eta_sq,
        p_value,
    }
}

fn compute_categorical_power(
    var_name: &str,
    feature_vals: &[RawValue],
    target_vals: &[RawValue],
    weights: Option<&[f64]>,
    n: usize,
) -> AdpPredictivePower {
    // Treat target codes as numbers for group ANOVA
    let mut target_codes: HashMap<String, f64> = HashMap::new();
    let mut current_code = 1.0;

    let numeric_target: Vec<RawValue> = target_vals
        .iter()
        .map(|v| match v {
            RawValue::Text(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    RawValue::Null
                } else {
                    let code = *target_codes.entry(trimmed.to_string()).or_insert_with(|| {
                        let c = current_code;
                        current_code += 1.0;
                        c
                    });
                    RawValue::Number(code)
                }
            }
            RawValue::Number(num) if num.is_finite() => RawValue::Number(*num),
            _ => RawValue::Null,
        })
        .collect();

    compute_anova_power(var_name, &numeric_target, feature_vals, weights, n)
}
