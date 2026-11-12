use super::tables::fmt_opt;
use statcraft_algo::ExploreResult;

pub fn explore_to_tsv(res: &ExploreResult) -> String {
    let mut out = String::new();
    out.push_str("Explore / EXAMINE Summary\n");

    // Case Processing
    out.push_str("Case Processing Summary\n");
    out.push_str("Variable\tValid N\tValid %\tMissing N\tMissing %\tTotal N\tTotal %\n");
    for var in &res.variables {
        for g in &var.groups {
            let name = match &g.factor_level {
                Some(f) => format!("{} ({})", var.variable, f),
                None => var.variable.clone(),
            };
            let cp = &g.case_processing;
            out.push_str(&format!(
                "{}\t{}\t{:.1}%\t{}\t{:.1}%\t{}\t{:.1}%\n",
                name,
                cp.n_valid,
                cp.pct_valid,
                cp.n_missing,
                cp.pct_missing,
                cp.n_total,
                cp.pct_total
            ));
        }
    }
    out.push('\n');

    // Descriptives
    out.push_str("Descriptives\n");
    out.push_str("Variable\tStatistic\tValue\tStd. Error\n");
    for var in &res.variables {
        for g in &var.groups {
            if let Some(d) = &g.descriptives {
                let name = match &g.factor_level {
                    Some(f) => format!("{} ({})", var.variable, f),
                    None => var.variable.clone(),
                };
                let rows = [
                    ("Mean", fmt_opt(d.mean), fmt_opt(d.mean_se)),
                    (
                        "95% CI Lower Bound",
                        fmt_opt(d.mean_ci_lower),
                        String::new(),
                    ),
                    (
                        "95% CI Upper Bound",
                        fmt_opt(d.mean_ci_upper),
                        String::new(),
                    ),
                    (
                        "5% Trimmed Mean",
                        fmt_opt(d.trimmed_mean_5pct),
                        String::new(),
                    ),
                    ("Median", fmt_opt(d.median), String::new()),
                    ("Variance", fmt_opt(d.variance), String::new()),
                    ("Std. Deviation", fmt_opt(d.std_dev), String::new()),
                    ("Minimum", fmt_opt(d.min), String::new()),
                    ("Maximum", fmt_opt(d.max), String::new()),
                    ("Range", fmt_opt(d.range), String::new()),
                    ("Interquartile Range", fmt_opt(d.iqr), String::new()),
                    ("Skewness", fmt_opt(d.skewness), fmt_opt(d.skewness_se)),
                    ("Kurtosis", fmt_opt(d.kurtosis), fmt_opt(d.kurtosis_se)),
                ];
                for (stat, val, se) in rows {
                    out.push_str(&format!("{}\t{}\t{}\t{}\n", name, stat, val, se));
                }
            }
        }
    }
    out.push('\n');

    // Normality
    out.push_str("Tests of Normality\n");
    out.push_str("Variable\tKS Statistic\tKS df\tKS Sig\tSW Statistic\tSW df\tSW Sig\n");
    for var in &res.variables {
        for g in &var.groups {
            if let Some(n) = &g.normality {
                let name = match &g.factor_level {
                    Some(f) => format!("{} ({})", var.variable, f),
                    None => var.variable.clone(),
                };
                let (ks_stat, ks_df, ks_sig) = match &n.kolmogorov_smirnov {
                    Some(k) => (
                        format!("{:.3}", k.statistic),
                        k.df.to_string(),
                        format!("{:.3}", k.sig),
                    ),
                    None => (".".to_string(), ".".to_string(), ".".to_string()),
                };
                let (sw_stat, sw_df, sw_sig) = match &n.shapiro_wilk {
                    Some(s) => (
                        format!("{:.3}", s.statistic),
                        s.df.to_string(),
                        format!("{:.3}", s.sig),
                    ),
                    None => (".".to_string(), ".".to_string(), ".".to_string()),
                };
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    name, ks_stat, ks_df, ks_sig, sw_stat, sw_df, sw_sig
                ));
            }
        }
    }
    out.push_str("a. Lilliefors Significance Correction\n\n");

    // Extreme Values
    out.push_str("Extreme Values\n");
    out.push_str("Variable\tOrder\tCase Number\tValue\n");
    for var in &res.variables {
        for g in &var.groups {
            if let Some(ext) = &g.extremes {
                let name = match &g.factor_level {
                    Some(f) => format!("{} ({})", var.variable, f),
                    None => var.variable.clone(),
                };
                for item in &ext.highest {
                    out.push_str(&format!(
                        "{}\tHighest {}\t{}\t{:.4}\n",
                        name, item.order, item.case_number, item.value
                    ));
                }
                for item in &ext.lowest {
                    out.push_str(&format!(
                        "{}\tLowest {}\t{}\t{:.4}\n",
                        name, item.order, item.case_number, item.value
                    ));
                }
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use statcraft_algo::{ExploreCaseProcessing, ExploreGroupResult, ExploreVariableResult};

    #[test]
    fn test_explore_to_tsv_output() {
        let res = ExploreResult {
            variables: vec![ExploreVariableResult {
                variable: "salary".to_string(),
                factor_var: None,
                groups: vec![ExploreGroupResult {
                    factor_level: None,
                    case_processing: ExploreCaseProcessing {
                        variable: "salary".to_string(),
                        factor_level: None,
                        n_valid: 10,
                        pct_valid: 100.0,
                        n_missing: 0,
                        pct_missing: 0.0,
                        n_total: 10,
                        pct_total: 100.0,
                    },
                    descriptives: None,
                    normality: None,
                    extremes: None,
                }],
            }],
        };

        let tsv = explore_to_tsv(&res);
        assert!(tsv.contains("Case Processing Summary"));
        assert!(tsv.contains("salary\t10\t100.0%\t0\t0.0%\t10\t100.0%"));
    }
}
