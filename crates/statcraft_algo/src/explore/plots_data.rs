use super::percentiles::compute_tukey_hinges;
use crate::rank_cases::inv_normal_cdf;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploreOutputDisplay {
    #[default]
    Both,
    Statistics,
    Plots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExploreBoxplotDisplay {
    #[default]
    FactorLevelsTogether,
    DependentsTogether,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoxplotOutlierPoint {
    pub case_number: usize,
    pub value: f64,
    #[serde(default)]
    pub case_label: Option<String>,
    pub is_extreme: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreBoxplotData {
    pub n: usize,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub whisker_low: f64,
    pub whisker_high: f64,
    pub outliers: Vec<BoxplotOutlierPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StemLeafRow {
    pub frequency: usize,
    pub stem: String,
    pub leaves: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StemLeafPlotData {
    pub n: usize,
    pub stem_width: f64,
    pub leaf_unit: f64,
    pub rows: Vec<StemLeafRow>,
    pub extreme_low_count: usize,
    pub extreme_high_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreHistogramBin {
    pub left: f64,
    pub right: f64,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreHistogramData {
    pub n: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub bins: Vec<ExploreHistogramBin>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreQqPoint {
    pub observed: f64,
    pub expected_normal: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreQqPlotData {
    pub n: usize,
    pub mean: f64,
    pub std_dev: f64,
    pub points: Vec<ExploreQqPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExploreGroupPlotData {
    #[serde(default)]
    pub boxplot: Option<ExploreBoxplotData>,
    #[serde(default)]
    pub stem_leaf: Option<StemLeafPlotData>,
    #[serde(default)]
    pub histogram: Option<ExploreHistogramData>,
    #[serde(default)]
    pub qq_plot: Option<ExploreQqPlotData>,
}

pub fn compute_group_plot_data(
    indexed_vals: &[(usize, f64, Option<String>)],
    want_boxplot: bool,
    want_stem_leaf: bool,
    want_histogram: bool,
    want_qq: bool,
) -> Option<ExploreGroupPlotData> {
    if indexed_vals.is_empty() || (!want_boxplot && !want_stem_leaf && !want_histogram && !want_qq)
    {
        return None;
    }

    let mut sorted = indexed_vals.to_vec();
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let vals: Vec<f64> = sorted.iter().map(|&(_, v, _)| v).collect();

    let boxplot = if want_boxplot {
        compute_boxplot_data(&sorted, &vals)
    } else {
        None
    };
    let stem_leaf = if want_stem_leaf {
        compute_stem_leaf_data(&vals)
    } else {
        None
    };
    let histogram = if want_histogram {
        compute_histogram_data(&vals)
    } else {
        None
    };
    let qq_plot = if want_qq {
        compute_qq_plot_data(&vals)
    } else {
        None
    };

    Some(ExploreGroupPlotData {
        boxplot,
        stem_leaf,
        histogram,
        qq_plot,
    })
}

fn compute_boxplot_data(
    sorted_cases: &[(usize, f64, Option<String>)],
    sorted_vals: &[f64],
) -> Option<ExploreBoxplotData> {
    let (q1, median, q3) = compute_tukey_hinges(sorted_vals)?;
    let iqr = (q3 - q1).max(0.0);
    let inner_lo = q1 - 1.5 * iqr;
    let inner_hi = q3 + 1.5 * iqr;
    let outer_lo = q1 - 3.0 * iqr;
    let outer_hi = q3 + 3.0 * iqr;

    let mut whisker_low = sorted_vals[0];
    let mut whisker_high = sorted_vals[sorted_vals.len() - 1];
    let mut found_in_range = false;
    let mut outliers = Vec::new();

    for (case_num, val, label) in sorted_cases {
        let v = *val;
        if v < inner_lo || v > inner_hi {
            let is_extreme = v < outer_lo || v > outer_hi;
            outliers.push(BoxplotOutlierPoint {
                case_number: *case_num,
                value: v,
                case_label: label.clone(),
                is_extreme,
            });
        } else if !found_in_range {
            whisker_low = v;
            whisker_high = v;
            found_in_range = true;
        } else {
            if v < whisker_low {
                whisker_low = v;
            }
            if v > whisker_high {
                whisker_high = v;
            }
        }
    }

    if !found_in_range {
        whisker_low = q1;
        whisker_high = q3;
    }

    Some(ExploreBoxplotData {
        n: sorted_vals.len(),
        q1,
        median,
        q3,
        whisker_low,
        whisker_high,
        outliers,
    })
}

fn compute_stem_leaf_data(sorted_vals: &[f64]) -> Option<StemLeafPlotData> {
    let n = sorted_vals.len();
    if n == 0 {
        return None;
    }
    let min_v = sorted_vals[0];
    let max_v = sorted_vals[n - 1];
    let span = (max_v - min_v).abs();

    let stem_width = if span <= 1e-12 {
        1.0
    } else {
        let raw_step = span / 8.0;
        let exp = raw_step.log10().floor();
        let pow10 = 10.0_f64.powf(exp);
        let frac = raw_step / pow10;
        if frac <= 1.5 {
            pow10
        } else if frac <= 3.5 {
            2.0 * pow10
        } else if frac <= 7.5 {
            5.0 * pow10
        } else {
            10.0 * pow10
        }
    };
    let leaf_unit = stem_width / 10.0;

    let mut map: BTreeMap<i64, Vec<u8>> = BTreeMap::new();
    for &v in sorted_vals {
        let scaled = (v / stem_width).floor();
        let stem_idx = scaled as i64;
        let rem = (v - (stem_idx as f64) * stem_width) / leaf_unit;
        let leaf_digit = (rem.floor() as i64).clamp(0, 9) as u8;
        map.entry(stem_idx).or_default().push(leaf_digit);
    }

    let min_stem = map.keys().next().copied().unwrap_or(0);
    let max_stem = map.keys().next_back().copied().unwrap_or(0);
    let mut rows = Vec::new();
    for s in min_stem..=max_stem {
        let leaves_vec = map.get(&s).cloned().unwrap_or_default();
        let leaves: String = leaves_vec
            .iter()
            .take(60)
            .map(|d| char::from(b'0' + *d))
            .collect();
        rows.push(StemLeafRow {
            frequency: leaves_vec.len(),
            stem: format!("{}", s),
            leaves,
        });
    }

    Some(StemLeafPlotData {
        n,
        stem_width,
        leaf_unit,
        rows,
        extreme_low_count: 0,
        extreme_high_count: 0,
    })
}

fn compute_histogram_data(sorted_vals: &[f64]) -> Option<ExploreHistogramData> {
    let n = sorted_vals.len();
    if n == 0 {
        return None;
    }
    let min_v = sorted_vals[0];
    let max_v = sorted_vals[n - 1];
    let mean = sorted_vals.iter().sum::<f64>() / n as f64;
    let var = if n > 1 {
        sorted_vals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64
    } else {
        0.0
    };
    let std_dev = var.sqrt();

    let k = ((1.0 + (n as f64).log2()).ceil() as usize).clamp(4, 24);
    let span = (max_v - min_v).max(1e-9);
    let bin_w = span / k as f64;
    let mut counts = vec![0usize; k];
    for &v in sorted_vals {
        let mut idx = ((v - min_v) / bin_w).floor() as usize;
        if idx >= k {
            idx = k - 1;
        }
        counts[idx] += 1;
    }
    let bins = (0..k)
        .map(|i| ExploreHistogramBin {
            left: min_v + i as f64 * bin_w,
            right: min_v + (i + 1) as f64 * bin_w,
            count: counts[i],
        })
        .collect();

    Some(ExploreHistogramData {
        n,
        mean,
        std_dev,
        bins,
    })
}

fn compute_qq_plot_data(sorted_vals: &[f64]) -> Option<ExploreQqPlotData> {
    let n = sorted_vals.len();
    if n < 2 {
        return None;
    }
    let mean = sorted_vals.iter().sum::<f64>() / n as f64;
    let var = sorted_vals.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let std_dev = var.sqrt();

    let n_f = n as f64;
    let step = (n / 400).max(1);
    let mut points = Vec::with_capacity(n.min(400));
    for (i, &obs) in sorted_vals.iter().enumerate() {
        if i % step != 0 && i + 1 != n {
            continue;
        }
        // Blom's formula: (rank - 3/8) / (n + 1/4)
        let p = ((i as f64 + 1.0) - 0.375) / (n_f + 0.25);
        let z = inv_normal_cdf(p.clamp(1e-6, 1.0 - 1e-6));
        points.push(ExploreQqPoint {
            observed: obs,
            expected_normal: z,
        });
    }

    Some(ExploreQqPlotData {
        n,
        mean,
        std_dev,
        points,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_group_plot_data_all() {
        let cases: Vec<(usize, f64, Option<String>)> = vec![
            (1, 10.0, None),
            (2, 12.0, None),
            (3, 14.0, None),
            (4, 15.0, None),
            (5, 16.0, None),
            (6, 18.0, None),
            (7, 50.0, Some("Outlier".to_string())),
        ];
        let plot = compute_group_plot_data(&cases, true, true, true, true).expect("plot data");
        let bp = plot.boxplot.expect("boxplot");
        assert_eq!(bp.n, 7);
        assert_eq!(bp.outliers.len(), 1);
        assert_eq!(bp.outliers[0].case_number, 7);
        assert!(plot.stem_leaf.is_some());
        assert!(plot.histogram.is_some());
        let qq = plot.qq_plot.expect("qq");
        assert_eq!(qq.points.len(), 7);
    }
}
