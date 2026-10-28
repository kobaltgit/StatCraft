use super::app::{StatCraftApp, StatusMessage};
use super::dialogs::{
    render_anova_oneway_dialog, render_correlations_dialog, render_crosstabs_dialog,
    render_descriptives_dialog, render_frequencies_dialog, render_missing_values_dialog,
    render_nonparam_twosample_dialog, render_ttest_indep_dialog, render_ttest_onesample_dialog,
    render_ttest_paired_dialog, render_value_labels_dialog, render_variable_type_dialog,
    CorrelationsDialogAction, CrosstabsDialogAction, DescriptivesDialogAction,
    FrequenciesDialogAction, IndependentSamplesTTestDialogAction, MissingValuesDialogAction,
    NonparamTwoSampleDialogAction, OneSampleTTestDialogAction, OneWayAnovaDialogAction,
    PairedSamplesTTestDialogAction, ValueLabelsDialogAction, VariableTypeDialogAction,
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

        // 8. Value Labels Dialog
        let val_action = render_value_labels_dialog(ctx, &mut self.value_labels_dialog_state);
        if let ValueLabelsDialogAction::Apply { col_index, mapping } = val_action {
            let params = json!({
                "col_index": col_index,
                "labels": mapping
            });
            match self
                .command_bus
                .execute("var.set_value_labels", &mut self.dataset, params)
            {
                Ok(_) => {
                    self.status_message = StatusMessage::Ready;
                }
                Err(e) => {
                    self.status_message = StatusMessage::Error(e.to_string());
                }
            }
        }

        // 9. Missing Values Dialog
        let miss_action = render_missing_values_dialog(ctx, &mut self.missing_values_dialog_state);
        if let MissingValuesDialogAction::Apply {
            col_index,
            discrete,
            range,
        } = miss_action
        {
            let mut params_map = serde_json::Map::new();
            params_map.insert("col_index".to_string(), json!(col_index));
            params_map.insert("discrete".to_string(), json!(discrete));
            if let Some((low, high)) = range {
                params_map.insert("range".to_string(), json!([low, high]));
            }
            match self.command_bus.execute(
                "var.set_missing_values",
                &mut self.dataset,
                serde_json::Value::Object(params_map),
            ) {
                Ok(_) => {
                    self.status_message = StatusMessage::Ready;
                }
                Err(e) => {
                    self.status_message = StatusMessage::Error(e.to_string());
                }
            }
        }

        // 10. Variable Type Dialog
        let type_action = render_variable_type_dialog(ctx, &mut self.variable_type_dialog_state);
        if let VariableTypeDialogAction::Apply {
            col_index,
            var_type,
            width,
            decimals,
        } = type_action
        {
            if let Some(ds) = &mut self.dataset {
                let _ = ds.set_variable_type(col_index, var_type, width, decimals);
            }
        }

        // 11. Two Independent Samples Nonparametric Dialog
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
    }
}
