use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_anova_oneway_dialog, render_correlations_dialog, render_crosstabs_dialog,
    render_descriptives_dialog, render_factor_dialog, render_frequencies_dialog,
    render_kruskal_wallis_dialog, render_linear_regression_dialog, render_means_dialog,
    render_nonparam_related_dialog, render_nonparam_twosample_dialog, render_reliability_dialog,
    render_ttest_indep_dialog, render_ttest_onesample_dialog, render_ttest_paired_dialog,
    CorrelationsDialogAction, CrosstabsDialogAction, DescriptivesDialogAction, FactorDialogAction,
    FrequenciesDialogAction, IndependentSamplesTTestDialogAction, KruskalWallisDialogAction,
    LinearRegressionDialogAction, MeansDialogAction, NonparamRelatedDialogAction,
    NonparamTwoSampleDialogAction, OneSampleTTestDialogAction, OneWayAnovaDialogAction,
    PairedSamplesTTestDialogAction, ReliabilityDialogAction,
};
use serde_json::json;
use statcraft_i18n::t;

impl StatCraftApp {
    pub(crate) fn handle_dialogs(&mut self, ctx: &egui::Context) {
        // 1. Descriptives Dialog
        let desc_action = render_descriptives_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.descriptives_dialog_state,
        );

        match desc_action {
            DescriptivesDialogAction::Run { vars, options } => {
                let params = json!({
                    "vars": vars,
                    "options": options
                });
                match self
                    .command_bus
                    .execute("stat.descriptives", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::DescriptivesResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_descriptives(results);
                            self.status_message = StatusMessage::DescriptivesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            DescriptivesDialogAction::Paste { vars } => {
                let sps = statcraft_syntax::commands::DescriptivesCommand::to_sps(&vars);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            DescriptivesDialogAction::None => {}
        }

        // 2. Frequencies Dialog
        let freq_action = render_frequencies_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.frequencies_dialog_state,
        );

        match freq_action {
            FrequenciesDialogAction::Run {
                vars,
                display_tables,
            } => {
                let params = json!({
                    "vars": vars,
                    "display_tables": display_tables
                });
                match self
                    .command_bus
                    .execute("stat.frequencies", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::FrequencyResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            if display_tables {
                                self.output_view_state.add_frequencies(results);
                            }
                            self.status_message = StatusMessage::FrequenciesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            FrequenciesDialogAction::Paste { vars } => {
                let sps = statcraft_syntax::commands::FrequenciesCommand::to_sps(&vars);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            FrequenciesDialogAction::None => {}
        }

        // 3. Crosstabs Dialog
        let crosstabs_action =
            render_crosstabs_dialog(ctx, self.dataset.as_ref(), &mut self.crosstabs_dialog_state);

        match crosstabs_action {
            CrosstabsDialogAction::Run {
                rows,
                cols,
                options,
            } => {
                let params = json!({
                    "rows": rows,
                    "cols": cols,
                    "expected": options.expected,
                    "row_pct": options.row_pct,
                    "col_pct": options.col_pct,
                    "total_pct": options.total_pct,
                    "residuals": options.residuals,
                    "chisq": options.chisq,
                    "phi_cramer": options.phi_cramer,
                });
                match self
                    .command_bus
                    .execute("stat.crosstabs", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::CrosstabResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_crosstabs(results, options);
                            self.status_message = StatusMessage::CrosstabsExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CrosstabsDialogAction::Paste {
                rows,
                cols,
                options,
            } => {
                let sps =
                    statcraft_syntax::commands::CrosstabsCommand::to_sps(&rows, &cols, &options);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CrosstabsDialogAction::None => {}
        }

        // 4. One-Sample T-Test Dialog
        let ttest_action = render_ttest_onesample_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.ttest_onesample_dialog_state,
        );

        match ttest_action {
            OneSampleTTestDialogAction::Run {
                vars,
                test_value,
                ci,
            } => {
                let params = json!({
                    "vars": vars,
                    "test_value": test_value,
                    "ci": ci,
                });
                match self
                    .command_bus
                    .execute("stat.ttest.onesample", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::OneSampleTTestResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state
                                .add_onesample_ttest(results, test_value, ci);
                            self.status_message = StatusMessage::TTestExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            OneSampleTTestDialogAction::Paste {
                vars,
                test_value,
                ci,
            } => {
                let sps = statcraft_syntax::commands::OneSampleTTestCommand::to_sps(
                    &vars, test_value, ci,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            OneSampleTTestDialogAction::None => {}
        }

        // 5. Independent-Samples T-Test Dialog
        let indep_action = render_ttest_indep_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.ttest_indep_dialog_state,
        );

        match indep_action {
            IndependentSamplesTTestDialogAction::Run {
                vars,
                grouping_var,
                group1,
                group2,
                ci,
            } => {
                let params = json!({
                    "vars": vars,
                    "grouping_var": grouping_var,
                    "group1": group1,
                    "group2": group2,
                    "ci": ci,
                });
                match self
                    .command_bus
                    .execute("stat.ttest.indep", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::IndepTTestResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_indep_ttest(
                                results,
                                &grouping_var,
                                &group1,
                                &group2,
                                ci,
                            );
                            self.status_message = StatusMessage::IndepTTestExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            IndependentSamplesTTestDialogAction::Paste {
                vars,
                grouping_var,
                group1,
                group2,
                ci,
            } => {
                let sps = statcraft_syntax::commands::IndependentSamplesTTestCommand::to_sps(
                    &vars,
                    &grouping_var,
                    &group1,
                    &group2,
                    ci,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            IndependentSamplesTTestDialogAction::None => {}
        }

        // 6. Paired-Samples T-Test Dialog
        let paired_action = render_ttest_paired_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.ttest_paired_dialog_state,
        );

        match paired_action {
            PairedSamplesTTestDialogAction::Run { pairs, ci } => {
                let params = json!({
                    "pairs": pairs,
                    "ci": ci,
                });
                match self
                    .command_bus
                    .execute("stat.ttest.paired", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::PairedTTestResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_paired_ttest(results, &pairs, ci);
                            self.status_message = StatusMessage::PairedTTestExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            PairedSamplesTTestDialogAction::Paste { pairs, ci } => {
                let sps = statcraft_syntax::commands::PairedSamplesTTestCommand::to_sps(&pairs, ci);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            PairedSamplesTTestDialogAction::None => {}
        }

        // 7. One-Way ANOVA Dialog
        let anova_action = render_anova_oneway_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.anova_oneway_dialog_state,
        );

        match anova_action {
            OneWayAnovaDialogAction::Run {
                dependent_vars,
                factor_var,
                descriptives,
                homogeneity,
                ci,
            } => {
                let params = json!({
                    "dependent_vars": dependent_vars,
                    "factor_var": factor_var,
                    "descriptives": descriptives,
                    "homogeneity": homogeneity,
                    "ci": ci,
                });
                match self
                    .command_bus
                    .execute("stat.anova.oneway", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::OneWayAnovaResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_oneway_anova(results, ci);
                            self.status_message = StatusMessage::AnovaExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            OneWayAnovaDialogAction::Paste {
                dependent_vars,
                factor_var,
                descriptives,
                homogeneity,
                ..
            } => {
                let sps = statcraft_syntax::commands::OneWayAnovaCommand::to_sps(
                    &dependent_vars,
                    &factor_var,
                    descriptives,
                    homogeneity,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            OneWayAnovaDialogAction::None => {}
        }

        // Bivariate Correlations Dialog
        let corr_action = render_correlations_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.correlations_dialog_state,
        );

        match corr_action {
            CorrelationsDialogAction::Run { vars, options } => {
                let params = json!({
                    "vars": vars,
                    "pearson": options.pearson,
                    "spearman": options.spearman,
                    "kendall": options.kendall,
                    "two_tailed": options.significance == statcraft_algo::SignificanceTest::TwoTailed,
                    "flag_significant": options.flag_significant,
                    "descriptives": options.descriptives,
                    "cross_prod_cov": options.cross_prod_cov,
                    "missing": match options.missing {
                        statcraft_algo::MissingHandling::Listwise => "listwise",
                        _ => "pairwise",
                    },
                });
                match self
                    .command_bus
                    .execute("stat.correlations", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(res) = serde_json::from_value::<
                            statcraft_algo::BivariateCorrelationsResult,
                        >(result_value)
                        {
                            let count = vars.len();
                            self.output_view_state.add_correlations(res, &vars);
                            self.status_message = StatusMessage::CorrelationsExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            CorrelationsDialogAction::Paste { vars, options } => {
                let sps = statcraft_syntax::commands::CorrelationsCommand::to_sps(&vars, &options);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            CorrelationsDialogAction::None => {}
        }

        // 9. Two Independent Samples Nonparametric Dialog
        let npar_action = render_nonparam_twosample_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.nonparam_twosample_dialog_state,
        );

        match npar_action {
            NonparamTwoSampleDialogAction::Run {
                vars,
                grouping_var,
                group1,
                group2,
            } => {
                let params = json!({
                    "vars": vars,
                    "grouping_var": grouping_var,
                    "group1": group1,
                    "group2": group2,
                });
                match self
                    .command_bus
                    .execute("stat.npar.mann_whitney", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::MannWhitneyResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_mann_whitney(
                                results,
                                &grouping_var,
                                &group1,
                                &group2,
                            );
                            self.status_message = StatusMessage::MannWhitneyExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            NonparamTwoSampleDialogAction::Paste {
                vars,
                grouping_var,
                group1,
                group2,
            } => {
                let sps = statcraft_syntax::commands::MannWhitneyCommand::to_sps(
                    &vars,
                    &grouping_var,
                    &group1,
                    &group2,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            NonparamTwoSampleDialogAction::None => {}
        }

        // 12. Two Related Samples Nonparametric Dialog
        let npar_related_action = render_nonparam_related_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.nonparam_related_dialog_state,
        );

        match npar_related_action {
            NonparamRelatedDialogAction::Run {
                pairs,
                wilcoxon,
                sign,
            } => {
                let params = json!({
                    "pairs": pairs.iter().map(|(v1, v2)| vec![v1.clone(), v2.clone()]).collect::<Vec<_>>(),
                    "wilcoxon": wilcoxon,
                    "sign": sign,
                });
                match self
                    .command_bus
                    .execute("stat.npar.related2", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::RelatedPairTestResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state
                                .add_related_samples(results, &pairs, wilcoxon, sign);
                            self.status_message = StatusMessage::RelatedSamplesExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            NonparamRelatedDialogAction::Paste {
                pairs,
                wilcoxon,
                sign,
            } => {
                let sps = statcraft_syntax::commands::RelatedSamplesTestCommand::to_sps(
                    &pairs, wilcoxon, sign,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            NonparamRelatedDialogAction::None => {}
        }

        // 13. K Independent Samples Nonparametric Dialog (Kruskal-Wallis)
        let kruskal_action = render_kruskal_wallis_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.kruskal_wallis_dialog_state,
        );

        match kruskal_action {
            KruskalWallisDialogAction::Run {
                vars,
                grouping_var,
                min_range,
                max_range,
            } => {
                let params = json!({
                    "vars": vars,
                    "grouping_var": grouping_var,
                    "min_range": min_range,
                    "max_range": max_range,
                });
                match self.command_bus.execute(
                    "stat.npar.kruskal_wallis",
                    &mut self.dataset,
                    params,
                ) {
                    Ok(result_value) => {
                        if let Ok(results) = serde_json::from_value::<
                            Vec<statcraft_algo::KruskalWallisResult>,
                        >(result_value)
                        {
                            let count = results.len();
                            self.output_view_state.add_kruskal_wallis(
                                results,
                                &grouping_var,
                                min_range,
                                max_range,
                            );
                            self.status_message = StatusMessage::KruskalWallisExecuted { count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            KruskalWallisDialogAction::Paste {
                vars,
                grouping_var,
                min_range,
                max_range,
            } => {
                let sps = statcraft_syntax::commands::KruskalWallisCommand::to_sps(
                    &vars,
                    &grouping_var,
                    min_range,
                    max_range,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            KruskalWallisDialogAction::None => {}
        }

        // 12. Linear Regression Dialog
        let reg_action = render_linear_regression_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.linear_regression_dialog_state,
        );

        match reg_action {
            LinearRegressionDialogAction::Run {
                dependent_var,
                independent_vars,
                ci,
                collinearity,
            } => {
                let params = json!({
                    "dependent_var": dependent_var,
                    "independent_vars": independent_vars,
                    "ci": ci,
                    "collinearity": collinearity,
                });
                match self
                    .command_bus
                    .execute("stat.regression.linear", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::LinearRegressionResult,
                        >(result_value)
                        {
                            let indep_count = result.indep_vars.len();
                            let dep_var = result.dep_var.clone();
                            self.output_view_state.add_linear_regression(result);
                            self.status_message = StatusMessage::RegressionExecuted {
                                dep_var,
                                indep_count,
                            };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            LinearRegressionDialogAction::Paste {
                dependent_var,
                independent_vars,
                ..
            } => {
                let sps = statcraft_syntax::commands::LinearRegressionCommand::to_sps(
                    &dependent_var,
                    &independent_vars,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            LinearRegressionDialogAction::None => {}
        }

        // 13. Scale Reliability Dialog
        let rel_action = render_reliability_dialog(
            ctx,
            self.dataset.as_ref(),
            &mut self.reliability_dialog_state,
        );

        match rel_action {
            ReliabilityDialogAction::Run {
                variables,
                scale_name,
                model,
                options,
            } => {
                let params = json!({
                    "variables": variables,
                    "scale_name": scale_name,
                    "model": match model {
                        statcraft_algo::ReliabilityModel::Alpha => "alpha",
                    },
                    "item_statistics": options.item_statistics,
                    "scale_statistics": options.scale_statistics,
                    "scale_if_item_deleted": options.scale_if_item_deleted,
                });
                match self
                    .command_bus
                    .execute("stat.scale.reliability", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::ReliabilityResult,
                        >(result_value)
                        {
                            let scale = result.scale_name.clone();
                            let count = result.reliability_stats.n_of_items;
                            let var_names = result
                                .item_stats
                                .iter()
                                .map(|s| s.item_name.clone())
                                .collect::<Vec<_>>();
                            self.output_view_state.add_reliability(result, var_names);
                            self.status_message =
                                StatusMessage::ReliabilityExecuted { scale, count };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            ReliabilityDialogAction::Paste {
                variables,
                scale_name,
                model,
                options,
            } => {
                let sps = statcraft_syntax::commands::ReliabilityCommand::to_sps(
                    &variables,
                    &scale_name,
                    model,
                    &options,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            ReliabilityDialogAction::None => {}
        }

        // 14. Factor Analysis Dialog
        let factor_action =
            render_factor_dialog(ctx, self.dataset.as_ref(), &mut self.factor_dialog_state);

        match factor_action {
            FactorDialogAction::Run { variables, options } => {
                let params = json!({
                    "variables": variables,
                    "extraction": match options.extraction {
                        statcraft_algo::FactorExtractionMethod::PC => "pc",
                    },
                    "mineigen": match options.criteria {
                        statcraft_algo::FactorExtractionCriteria::Mineigen(val) => Some(val),
                        _ => None,
                    },
                    "nfactors": match options.criteria {
                        statcraft_algo::FactorExtractionCriteria::Factors(k) => Some(k),
                        _ => None,
                    },
                    "kmo_bartlett": options.kmo_bartlett,
                });
                match self
                    .command_bus
                    .execute("stat.factor", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::FactorAnalysisResult,
                        >(result_value)
                        {
                            let count = result.n_cases;
                            let factors = result.n_components_extracted;
                            let var_names = result.variables.clone();
                            self.output_view_state.add_factor(result, var_names);
                            self.status_message = StatusMessage::FactorExecuted { count, factors };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            FactorDialogAction::Paste { variables, options } => {
                let sps = statcraft_syntax::commands::FactorCommand::to_sps(&variables, &options);
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            FactorDialogAction::None => {}
        }

        // 15. Means Dialog
        let means_action =
            render_means_dialog(ctx, self.dataset.as_ref(), &mut self.means_dialog_state);

        match means_action {
            MeansDialogAction::Run {
                dependent_vars,
                factor_vars,
                options,
            } => {
                let params = json!({
                    "dependent_vars": dependent_vars,
                    "factor_vars": factor_vars,
                    "cell_mean": options.cell_mean,
                    "cell_count": options.cell_count,
                    "cell_stddev": options.cell_stddev,
                    "cell_variance": options.cell_variance,
                    "cell_median": options.cell_median,
                    "cell_min": options.cell_min,
                    "cell_max": options.cell_max,
                    "anova": options.anova,
                });
                match self
                    .command_bus
                    .execute("stat.means", &mut self.dataset, params)
                {
                    Ok(result_value) => {
                        if let Ok(result) = serde_json::from_value::<
                            statcraft_algo::MeansProcedureResult,
                        >(result_value)
                        {
                            let tables = result.tables.len();
                            let cases = result
                                .tables
                                .first()
                                .map(|t| t.summary.total_n)
                                .unwrap_or(0);
                            self.output_view_state.add_means(
                                result,
                                dependent_vars,
                                factor_vars,
                                &options,
                            );
                            self.status_message = StatusMessage::MeansExecuted { tables, cases };
                        }
                    }
                    Err(e) => {
                        self.status_message = StatusMessage::Error(e.to_string());
                    }
                }
            }
            MeansDialogAction::Paste {
                dependent_vars,
                factor_vars,
                options,
            } => {
                let sps = statcraft_syntax::commands::MeansCommand::to_sps(
                    &dependent_vars,
                    &factor_vars,
                    &options,
                );
                self.output_view_state.add_entry(t!("output-log"), sps);
            }
            MeansDialogAction::None => {}
        }

        // 16. Data and Variable Management Dialogs
        self.handle_data_dialogs(ctx);
    }
}
