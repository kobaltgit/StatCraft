use super::CrosstabsCommand;
use statcraft_algo::{CrosstabOptions, ExactMode, NonIntegerWeights, RowOrder};

impl CrosstabsCommand {
    pub fn to_sps(rows: &[String], cols: &[String], options: &CrosstabOptions) -> String {
        Self::to_sps_with_layers(rows, cols, &[], options)
    }

    pub fn to_sps_with_layers(
        rows: &[String],
        cols: &[String],
        layers: &[Vec<String>],
        options: &CrosstabOptions,
    ) -> String {
        let mut tables = format!("{} BY {}", rows.join(" "), cols.join(" "));
        for l in layers {
            if !l.is_empty() {
                tables.push_str(&format!(" BY {}", l.join(" ")));
            }
        }

        let mut clauses = vec![format!("CROSSTABS\n  /TABLES={}", tables)];

        if options.row_order == RowOrder::Descending {
            clauses.push("  /FORMAT=DVALUE TABLES".to_string());
        } else {
            clauses.push("  /FORMAT=AVALUE TABLES".to_string());
        }

        if options.suppress_tables {
            clauses.push("  /NOTABLES".to_string());
        }

        let mut stats = Vec::new();
        if options.chisq {
            stats.push("CHISQ".to_string());
        }
        if options.correlations {
            stats.push("CORR".to_string());
        }
        if options.contingency_coef {
            stats.push("CC".to_string());
        }
        if options.phi_cramer {
            stats.push("PHI".to_string());
        }
        if options.lambda {
            stats.push("LAMBDA".to_string());
        }
        if options.uncertainty_coef {
            stats.push("UC".to_string());
        }
        if options.gamma {
            stats.push("GAMMA".to_string());
        }
        if options.somers_d {
            stats.push("D".to_string());
        }
        if options.kendall_tau_b {
            stats.push("BTAU".to_string());
        }
        if options.kendall_tau_c {
            stats.push("CTAU".to_string());
        }
        if options.eta {
            stats.push("ETA".to_string());
        }
        if options.kappa {
            stats.push("KAPPA".to_string());
        }
        if options.risk {
            stats.push("RISK".to_string());
        }
        if options.mcnemar {
            stats.push("MCNEMAR".to_string());
        }
        if options.cochran_mantel_haenszel {
            if (options.cmh_common_odds_ratio - 1.0).abs() > 1e-9 {
                stats.push(format!("CMH({})", options.cmh_common_odds_ratio));
            } else {
                stats.push("CMH".to_string());
            }
        }
        if !stats.is_empty() {
            clauses.push(format!("  /STATISTICS={}", stats.join(" ")));
        }

        let mut cells = Vec::new();
        if options.observed {
            cells.push("COUNT");
        }
        if options.expected {
            cells.push("EXPECTED");
        }
        if options.row_pct {
            cells.push("ROW");
        }
        if options.col_pct {
            cells.push("COLUMN");
        }
        if options.total_pct {
            cells.push("TOTAL");
        }
        if options.residuals || options.resid_unstandardized {
            cells.push("RESID");
        }
        if options.resid_standardized {
            cells.push("SRESID");
        }
        if options.resid_adjusted {
            cells.push("ASRESID");
        }
        if options.compare_col_proportions {
            if options.bonferroni_adjust {
                cells.push("BPROP");
            } else {
                cells.push("PROP");
            }
        }
        if !cells.is_empty() {
            clauses.push(format!("  /CELLS={}", cells.join(" ")));
        }

        match options.exact_mode {
            ExactMode::AsymptoticOnly => {}
            ExactMode::MonteCarlo {
                confidence,
                samples,
            } => {
                clauses.push(format!(
                    "  /METHOD=MC CIN({}) SAMPLES({})",
                    confidence * 100.0,
                    samples
                ));
            }
            ExactMode::Exact { time_limit_min } => {
                if let Some(m) = time_limit_min {
                    clauses.push(format!("  /METHOD=EXACT TIMER({})", m));
                } else {
                    clauses.push("  /METHOD=EXACT".to_string());
                }
            }
        }

        let count_kw = match options.noninteger_weights {
            NonIntegerWeights::RoundCell => "ROUNDCELL",
            NonIntegerWeights::TruncateCell => "TRUNCATECELL",
            NonIntegerWeights::RoundCase => "ROUNDCASE",
            NonIntegerWeights::TruncateCase => "TRUNCATECASE",
            NonIntegerWeights::NoAdjust => "ASIS",
        };
        if options.noninteger_weights != NonIntegerWeights::RoundCell {
            clauses.push(format!("  /COUNT={}", count_kw));
        }

        if options.display_clustered_bar_charts {
            clauses.push("  /BARCHART".to_string());
        }

        format!("{}.", clauses.join("\n"))
    }

    pub fn to_sps_syntax(
        &self,
        rows: &[String],
        cols: &[String],
        options: &CrosstabOptions,
    ) -> String {
        Self::to_sps(rows, cols, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::EngineCommand;
    use serde_json::json;
    use statcraft_core::{DataSet, DataType, Measure};
    use statcraft_io::RawValue;

    #[test]
    fn test_crosstabs_sps_syntax() {
        let opts = CrosstabOptions::default();
        let sps = CrosstabsCommand::to_sps(&["gender".to_string()], &["jobcat".to_string()], &opts);
        assert!(sps.contains("CROSSTABS\n  /TABLES=gender BY jobcat"));
        assert!(sps.contains("/STATISTICS=CHISQ PHI"));
        assert!(sps.contains("/CELLS=COUNT EXPECTED ROW COLUMN TOTAL RESID"));
    }

    #[test]
    fn test_crosstabs_command_execution_with_layers_and_cmh() {
        let mut ds = DataSet::empty();
        let r_col = ds
            .add_variable(
                "gender",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();
        let c_col = ds
            .add_variable(
                "jobcat",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .unwrap();
        let l_col = ds
            .add_variable("dept", DataType::Numeric { decimals: 0 }, Measure::Nominal)
            .unwrap();

        for i in 0..40 {
            ds.add_empty_row();
            let g = if i % 4 < 2 { 1.0 } else { 2.0 };
            let j = if i % 2 == 0 { 1.0 } else { 2.0 };
            let d = if i < 20 { 1.0 } else { 2.0 };
            ds.set_cell(i, r_col, RawValue::Number(g)).unwrap();
            ds.set_cell(i, c_col, RawValue::Number(j)).unwrap();
            ds.set_cell(i, l_col, RawValue::Number(d)).unwrap();
        }

        let mut opt_ds = Some(ds);
        let cmd = CrosstabsCommand;
        let params = json!({
            "rows": ["gender"],
            "cols": ["jobcat"],
            "layers": [["dept"]],
            "chisq": true,
            "cochran_mantel_haenszel": true
        });
        let res = cmd
            .execute(&mut opt_ds, params)
            .expect("Command must succeed");
        let arr = res.as_array().expect("Result must be array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["layer_keys"][0]["var_name"], "dept");
        assert!(arr[0]["cmh_result"].is_object());
    }

    #[test]
    fn test_crosstabs_sps_roundtrip_all_subcommands() {
        let opts = CrosstabOptions {
            row_order: RowOrder::Descending,
            suppress_tables: true,
            display_clustered_bar_charts: true,
            chisq: true,
            correlations: true,
            contingency_coef: true,
            phi_cramer: true,
            lambda: true,
            uncertainty_coef: true,
            gamma: true,
            somers_d: true,
            kendall_tau_b: true,
            kendall_tau_c: true,
            eta: true,
            kappa: true,
            risk: true,
            mcnemar: true,
            cochran_mantel_haenszel: true,
            cmh_common_odds_ratio: 2.0,
            resid_standardized: true,
            resid_adjusted: true,
            compare_col_proportions: true,
            bonferroni_adjust: true,
            exact_mode: ExactMode::MonteCarlo {
                confidence: 0.99,
                samples: 5000,
            },
            noninteger_weights: NonIntegerWeights::TruncateCase,
            ..Default::default()
        };
        let sps = CrosstabsCommand::to_sps_with_layers(
            &["gender".to_string()],
            &["jobcat".to_string()],
            &[vec!["dept".to_string()]],
            &opts,
        );
        let blocks = crate::syntax_parser::lexer::SyntaxLexer::split_commands(&sps)
            .expect("Roundtrip tokenize must succeed");
        let cmd = crate::syntax_parser::SyntaxParser::parse_command(&blocks[0])
            .expect("Roundtrip parse must succeed");
        let mapped = crate::syntax_parser::mappers::map_stat_command(&cmd, None)
            .expect("Roundtrip map must succeed")
            .expect("Command must map");
        assert_eq!(mapped.0, "stat.crosstabs");
        let p = &mapped.1;
        assert_eq!(p["rows"], json!(["gender"]));
        assert_eq!(p["cols"], json!(["jobcat"]));
        assert_eq!(p["layers"], json!([["dept"]]));
        assert_eq!(p["gamma"], true);
        assert_eq!(p["cochran_mantel_haenszel"], true);
        assert_eq!(p["cmh_common_odds_ratio"], 2.0);
        assert_eq!(p["bonferroni_adjust"], true);
        assert_eq!(p["suppress_tables"], true);
        assert_eq!(p["display_clustered_bar_charts"], true);
    }
}
