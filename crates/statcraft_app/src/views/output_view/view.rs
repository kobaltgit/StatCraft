use super::blocks::render_item_context_menu;
use super::model::{OutputItemKind, OutputViewerState, SelectedTarget};
use super::outline::render_outline_pane;
use super::{extract_item_tsv, render_crosstabs_item, render_log_block, render_notes_block};
use super::{
    render_frequency_charts, render_report_table, render_spss_adp_tables,
    render_spss_codebook_tables, render_spss_correlations_tables, render_spss_cox_regression,
    render_spss_curvefit_tables, render_spss_deming_regression,
    render_spss_descriptives_pivot_table, render_spss_explore_pivot_tables, render_spss_factor,
    render_spss_frequencies_tables, render_spss_generic_report, render_spss_indep_ttest_tables,
    render_spss_kmeans_tables, render_spss_kruskal_wallis_tables, render_spss_lifetables,
    render_spss_linear_regression, render_spss_logistic_tables, render_spss_mann_whitney_tables,
    render_spss_means, render_spss_multinomial_logistic_tables,
    render_spss_multivariate_glm_tables, render_spss_onesample_ttest_tables,
    render_spss_oneway_anova_tables, render_spss_paired_ttest_tables,
    render_spss_partial_corr_tables, render_spss_programmability_tables,
    render_spss_related_samples_tables, render_spss_reliability, render_spss_repeated_glm_tables,
    render_spss_summarize_tables, render_spss_survival_tables, render_spss_theil_sen,
    render_spss_univariate_glm_tables,
};
use crate::theme::Tokens;
use egui::{pos2, vec2, Align, Rect, ScrollArea, Sense, Stroke, Ui};
use statcraft_i18n::t;

pub fn render_output_view(ui: &mut Ui, state: &mut OutputViewerState) {
    render_output_view_with_workspace(ui, state, None);
}

pub fn render_output_view_with_workspace(
    ui: &mut Ui,
    state: &mut OutputViewerState,
    workspace: Option<&statcraft_core::WorkspaceManager>,
) {
    if state.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(t!("output-no-analysis"));
        });
        return;
    }

    let full_rect = ui.available_rect_before_wrap();
    ui.painter()
        .rect_filled(full_rect, 0.0_f32, Tokens::bg_app());
    let outline_width = 230.0_f32.min((full_rect.width() - 40.0).max(40.0) * 0.45);

    // 1. Left outline tree panel
    let left_rect = Rect::from_min_size(full_rect.min, vec2(outline_width, full_rect.height()));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        ui.set_clip_rect(left_rect);
        render_outline_pane(ui, state, workspace);
    });

    // 2. Vertical separator between panels
    let sep_x = full_rect.min.x + outline_width;
    ui.painter().line_segment(
        [pos2(sep_x, full_rect.min.y), pos2(sep_x, full_rect.max.y)],
        Stroke::new(1.0_f32, Tokens::border()),
    );

    // 3. Right continuous vertical scroll of all procedures
    let right_rect = Rect::from_min_max(pos2(sep_x + 4.0_f32, full_rect.min.y), full_rect.max);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
        ScrollArea::vertical()
            .id_salt("output_viewer_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(right_rect.width() - 16.0);
                ui.add_space(8.0_f32);

                let mut clicked_item: Option<SelectedTarget> = None;
                let mut scroll_target_rect: Option<Rect> = None;
                let mut item_copied: Option<usize> = None;
                let mut item_to_delete: Option<(usize, usize)> = None;
                let mut proc_to_delete: Option<usize> = None;

                for (p_idx, proc) in state.procedures.iter().enumerate() {
                    // Procedure Heading Banner
                    let proc_target = SelectedTarget::Procedure(p_idx);
                    let _is_proc_selected = state.selected_target == Some(proc_target);
                    let proc_header_start_y = ui.cursor().top();

                    ui.horizontal(|ui| {
                        let resp = ui.heading(proc.title()).interact(Sense::click());
                        if resp.clicked() || resp.secondary_clicked() {
                            clicked_item = Some(proc_target);
                        }
                    });
                    ui.add_space(4.0_f32);
                    let proc_header_end_y = ui.cursor().top();
                    let proc_header_rect = Rect::from_x_y_ranges(
                        right_rect.x_range(),
                        proc_header_start_y..=proc_header_end_y,
                    );
                    if state.scroll_to_target == Some(proc_target) {
                        scroll_target_rect = Some(proc_header_rect);
                    }

                    // Child Items (tables, notes, log)
                    for (i_idx, item) in proc.items.iter().enumerate() {
                        let this_target = SelectedTarget::Item {
                            proc_idx: p_idx,
                            item_idx: i_idx,
                        };
                        let is_selected = state.selected_target == Some(this_target);
                        let is_item_scroll_targeted = state.scroll_to_target == Some(this_target);

                        let maybe_tsv = extract_item_tsv(&item.kind);

                        ui.vertical(|ui| {
                            let block_start_y = ui.cursor().top();
                            let is_item_copied = state.last_copied_id == Some(item.id);
                            let mut title_clicked = false;
                            let mut title_secondary_clicked = false;

                            // Item Title Header with Copy TSV button if table
                            ui.horizontal(|ui| {
                                ui.add_space(8.0_f32);
                                let title_resp = ui.strong(item.title()).interact(Sense::click());
                                if title_resp.clicked() {
                                    title_clicked = true;
                                }
                                if title_resp.secondary_clicked() {
                                    title_secondary_clicked = true;
                                }
                                title_resp.context_menu(|ui| {
                                    render_item_context_menu(
                                        ui,
                                        item,
                                        &maybe_tsv,
                                        is_item_copied,
                                        (p_idx, i_idx),
                                        &mut item_copied,
                                        &mut item_to_delete,
                                    );
                                });

                                if let Some(ref tsv) = maybe_tsv {
                                    ui.add_space(12.0_f32);
                                    let btn_label = if is_item_copied {
                                        egui::RichText::new(t!("output-menu-copied"))
                                            .color(Tokens::accent())
                                    } else {
                                        egui::RichText::new(t!("output-menu-copy-tsv"))
                                    };
                                    let btn = ui
                                        .button(btn_label)
                                        .on_hover_text(t!("output-copied-toast"));
                                    if btn.clicked() {
                                        ui.output_mut(|o| {
                                            o.copied_text = tsv.clone();
                                        });
                                        item_copied = Some(item.id);
                                    }
                                }
                            });

                            if title_clicked || title_secondary_clicked {
                                clicked_item = Some(this_target);
                            }

                            ui.add_space(4.0_f32);
                            let content_start_y = ui.cursor().top();

                            // Content Rendering
                            ui.indent(ui.id().with(("indent", p_idx, i_idx)), |ui| {
                                match &item.kind {
                                    OutputItemKind::Log(sps) => {
                                        render_log_block(ui, sps);
                                    }
                                    OutputItemKind::Notes(notes) => {
                                        render_notes_block(ui, notes, workspace);
                                    }
                                    OutputItemKind::DatasetSummary { name, rows, cols } => {
                                        ui.label(t!(
                                            "output-dataset-imported",
                                            name = name.as_str(),
                                            rows = *rows,
                                            cols = *cols
                                        ));
                                    }
                                    OutputItemKind::DescriptivesTable(results) => {
                                        render_spss_descriptives_pivot_table(ui, results);
                                    }
                                    OutputItemKind::FrequenciesTables(tables) => {
                                        render_spss_frequencies_tables(ui, tables);
                                    }
                                    OutputItemKind::FrequencyCharts(charts) => {
                                        render_frequency_charts(ui, charts);
                                    }
                                    OutputItemKind::OneSampleTTest {
                                        results,
                                        test_value,
                                        ci,
                                    } => {
                                        render_spss_onesample_ttest_tables(
                                            ui,
                                            results,
                                            *test_value,
                                            *ci,
                                        );
                                    }
                                    OutputItemKind::IndepTTest {
                                        results,
                                        grouping_var,
                                        group1,
                                        group2,
                                        ci,
                                    } => {
                                        render_spss_indep_ttest_tables(
                                            ui,
                                            results,
                                            grouping_var,
                                            group1,
                                            group2,
                                            *ci,
                                        );
                                    }
                                    OutputItemKind::PairedTTest { results, pairs, ci } => {
                                        render_spss_paired_ttest_tables(ui, results, pairs, *ci);
                                    }
                                    OutputItemKind::OneWayAnova { results, ci } => {
                                        render_spss_oneway_anova_tables(ui, results, *ci);
                                    }
                                    OutputItemKind::Crosstabs(results) => {
                                        render_crosstabs_item(ui, results, is_selected);
                                    }
                                    OutputItemKind::Correlations(res) => {
                                        render_spss_correlations_tables(ui, res);
                                    }
                                    OutputItemKind::MannWhitney {
                                        results,
                                        grouping_var,
                                    } => {
                                        render_spss_mann_whitney_tables(ui, results, grouping_var);
                                    }
                                    OutputItemKind::KruskalWallis {
                                        results,
                                        grouping_var,
                                    } => {
                                        render_spss_kruskal_wallis_tables(
                                            ui,
                                            results,
                                            grouping_var,
                                        );
                                    }
                                    OutputItemKind::TwoRelatedSamples(results) => {
                                        render_spss_related_samples_tables(ui, results);
                                    }
                                    OutputItemKind::LinearRegression(res) => {
                                        render_spss_linear_regression(ui, res);
                                    }
                                    OutputItemKind::TheilSen(item) => {
                                        render_spss_theil_sen(ui, item);
                                    }
                                    OutputItemKind::DemingRegression(item) => {
                                        render_spss_deming_regression(ui, item);
                                    }
                                    OutputItemKind::PartialCorrelations(res) => {
                                        render_spss_partial_corr_tables(ui, res);
                                    }
                                    OutputItemKind::LogisticRegression(res) => {
                                        render_spss_logistic_tables(ui, res);
                                    }
                                    OutputItemKind::MultinomialLogistic(res) => {
                                        render_spss_multinomial_logistic_tables(ui, res);
                                    }
                                    OutputItemKind::CurveFit(res) => {
                                        render_spss_curvefit_tables(ui, res);
                                    }
                                    OutputItemKind::UnivariateGlm(res) => {
                                        render_spss_univariate_glm_tables(ui, res);
                                    }
                                    OutputItemKind::MultivariateGlm(res) => {
                                        render_spss_multivariate_glm_tables(ui, res);
                                    }
                                    OutputItemKind::RepeatedGlm(res) => {
                                        render_spss_repeated_glm_tables(ui, res);
                                    }
                                    OutputItemKind::KMeans(res) => {
                                        render_spss_kmeans_tables(ui, res);
                                    }
                                    OutputItemKind::Codebook(res) => {
                                        render_spss_codebook_tables(ui, res);
                                    }
                                    OutputItemKind::Summarize(res) => {
                                        render_spss_summarize_tables(ui, res);
                                    }
                                    OutputItemKind::KaplanMeier(res) => {
                                        render_spss_survival_tables(ui, res);
                                    }
                                    OutputItemKind::LifeTables(res) => {
                                        render_spss_lifetables(ui, res);
                                    }
                                    OutputItemKind::CoxRegression(res) => {
                                        render_spss_cox_regression(ui, res);
                                    }
                                    OutputItemKind::Reliability(res) => {
                                        render_spss_reliability(ui, res);
                                    }
                                    OutputItemKind::Factor(res) => {
                                        render_spss_factor(ui, res);
                                    }
                                    OutputItemKind::Means(res) => {
                                        render_spss_means(ui, res);
                                    }
                                    OutputItemKind::Explore(res) => {
                                        render_spss_explore_pivot_tables(ui, res);
                                    }
                                    OutputItemKind::Adp(res) => {
                                        render_spss_adp_tables(ui, res);
                                    }
                                    OutputItemKind::Programmability(res) => {
                                        render_spss_programmability_tables(ui, res);
                                    }
                                    OutputItemKind::Warning(msg) => {
                                        ui.colored_label(Tokens::caption_close_hover(), msg);
                                    }
                                    OutputItemKind::CustomText { content, .. } => {
                                        ui.label(content);
                                    }
                                    OutputItemKind::GenericReport {
                                        command_id,
                                        status,
                                        summary,
                                        key_values,
                                        table_headers,
                                        table_rows,
                                    } => {
                                        render_spss_generic_report(
                                            ui,
                                            command_id,
                                            status,
                                            summary,
                                            key_values,
                                            table_headers,
                                            table_rows,
                                        );
                                    }
                                    OutputItemKind::ReportTable(table) => {
                                        render_report_table(ui, table);
                                    }
                                }

                                let block_end_y = ui.cursor().top();
                                let total_item_rect = Rect::from_x_y_ranges(
                                    right_rect.x_range(),
                                    block_start_y..=block_end_y,
                                );
                                let content_rect = Rect::from_x_y_ranges(
                                    right_rect.x_range(),
                                    content_start_y..=block_end_y,
                                );

                                let content_resp = ui.interact(
                                    content_rect,
                                    ui.id().with(("item_content", p_idx, i_idx)),
                                    Sense::click(),
                                );
                                if content_resp.clicked() || content_resp.secondary_clicked() {
                                    clicked_item = Some(this_target);
                                }

                                content_resp.context_menu(|ui| {
                                    render_item_context_menu(
                                        ui,
                                        item,
                                        &maybe_tsv,
                                        is_item_copied,
                                        (p_idx, i_idx),
                                        &mut item_copied,
                                        &mut item_to_delete,
                                    );
                                });

                                // Target this item for scroll after all layout completes
                                if is_item_scroll_targeted {
                                    scroll_target_rect = Some(total_item_rect);
                                }

                                // Outline focus border if selected
                                if is_selected {
                                    ui.painter().rect_stroke(
                                        total_item_rect,
                                        2.0_f32,
                                        Stroke::new(1.5_f32, Tokens::accent()),
                                    );
                                }
                            });

                            ui.add_space(16.0_f32);
                        });
                    }
                    ui.add_space(24.0_f32);
                }

                // Apply deferred actions
                if let Some(target) = clicked_item {
                    state.selected_target = Some(target);
                }
                if let Some(id) = item_copied {
                    state.last_copied_id = Some(id);
                }
                if let Some((p, i)) = item_to_delete {
                    if let Some(proc) = state.procedures.get_mut(p) {
                        if i < proc.items.len() {
                            proc.items.remove(i);
                        }
                        if proc.items.is_empty() {
                            proc_to_delete = Some(p);
                        }
                    }
                    state.selected_target = None;
                }
                if let Some(p) = proc_to_delete {
                    if p < state.procedures.len() {
                        state.procedures.remove(p);
                    }
                    state.selected_target = None;
                }

                // Execute scroll-to after all children have finished layout
                // (avoids nested horizontal scroll areas clearing vertical scroll_target)
                if let Some(target_rect) = scroll_target_rect {
                    ui.scroll_to_rect(target_rect, Some(Align::TOP));
                    ui.ctx().request_repaint();
                }

                // Reset scroll-to once rendered
                state.scroll_to_target = None;
            });
    });
}
