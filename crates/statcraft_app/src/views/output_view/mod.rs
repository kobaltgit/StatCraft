pub mod blocks;
pub mod builder;
pub mod export;
pub mod model;
pub mod outline;
pub mod tables_anova;
pub mod tables_common;
pub mod tables_correlations;
pub mod tables_crosstabs;
pub mod tables_descriptives;
pub mod tables_factor;
pub mod tables_frequencies;
pub mod tables_kruskal;
pub mod tables_means;
pub mod tables_nonparam;
pub mod tables_nonparam_related;
pub mod tables_regression;
pub mod tables_reliability;
pub mod tables_ttest_indep;
pub mod tables_ttest_onesample;
pub mod tables_ttest_paired;

pub use blocks::{render_item_context_menu, render_log_block, render_notes_block};
pub use export::{
    export_output_to_html, export_output_to_tsv, export_output_to_txt,
    output_viewer_to_report_document,
};
pub use model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
pub use outline::render_outline_pane;
pub use tables_anova::{anova_to_tsv, render_spss_oneway_anova_tables};
pub use tables_correlations::{correlations_to_tsv, render_spss_correlations_tables};
pub use tables_crosstabs::{crosstabs_to_tsv, render_crosstabs_item};
pub use tables_descriptives::{descriptives_to_tsv, render_spss_descriptives_pivot_table};
pub use tables_factor::{factor_to_tsv, render_spss_factor};
pub use tables_frequencies::{frequencies_to_tsv, render_spss_frequencies_tables};
pub use tables_kruskal::{kruskal_wallis_to_tsv, render_spss_kruskal_wallis_tables};
pub use tables_means::{means_to_tsv, render_spss_means};
pub use tables_nonparam::{mann_whitney_to_tsv, render_spss_mann_whitney_tables};
pub use tables_nonparam_related::{related_samples_to_tsv, render_spss_related_samples_tables};
pub use tables_regression::{regression_to_tsv, render_spss_linear_regression};
pub use tables_reliability::{reliability_to_tsv, render_spss_reliability};
pub use tables_ttest_indep::{indep_ttest_to_tsv, render_spss_indep_ttest_tables};
pub use tables_ttest_onesample::{onesample_ttest_to_tsv, render_spss_onesample_ttest_tables};
pub use tables_ttest_paired::{paired_ttest_to_tsv, render_spss_paired_ttest_tables};

use crate::theme::Tokens;
use egui::{pos2, vec2, Align, Align2, FontId, Rect, ScrollArea, Sense, Stroke, Ui};
use statcraft_i18n::t;

pub fn extract_item_tsv(kind: &OutputItemKind) -> Option<String> {
    match kind {
        OutputItemKind::DescriptivesTable(res) => Some(descriptives_to_tsv(res)),
        OutputItemKind::FrequenciesTables(res) => Some(frequencies_to_tsv(res)),
        OutputItemKind::OneSampleTTest { results, .. } => Some(onesample_ttest_to_tsv(results)),
        OutputItemKind::IndepTTest { results, .. } => Some(indep_ttest_to_tsv(results)),
        OutputItemKind::PairedTTest { results, .. } => Some(paired_ttest_to_tsv(results)),
        OutputItemKind::OneWayAnova { results, .. } => Some(anova_to_tsv(results)),
        OutputItemKind::Crosstabs(results) => results.first().map(crosstabs_to_tsv),
        OutputItemKind::Correlations(res) => Some(correlations_to_tsv(res)),
        OutputItemKind::MannWhitney {
            results,
            grouping_var,
        } => Some(mann_whitney_to_tsv(results, grouping_var)),
        OutputItemKind::KruskalWallis {
            results,
            grouping_var,
        } => Some(kruskal_wallis_to_tsv(results, grouping_var)),
        OutputItemKind::TwoRelatedSamples(results) => Some(related_samples_to_tsv(results)),
        OutputItemKind::LinearRegression(res) => Some(regression_to_tsv(res)),
        OutputItemKind::Reliability(res) => Some(reliability_to_tsv(res)),
        OutputItemKind::Factor(res) => Some(factor_to_tsv(res)),
        OutputItemKind::Means(res) => Some(means_to_tsv(res)),
        _ => None,
    }
}

pub fn render_output_view(ui: &mut Ui, state: &mut OutputViewerState) {
    if state.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(t!("output-no-analysis"));
        });
        return;
    }

    let full_rect = ui.available_rect_before_wrap();
    let outline_width = 230.0_f32;

    // 1. Left outline tree panel
    let left_rect = Rect::from_min_size(full_rect.min, vec2(outline_width, full_rect.height()));
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(left_rect), |ui| {
        ui.set_clip_rect(left_rect);
        render_outline_pane(ui, state);
    });

    // 2. Vertical separator between panels
    let sep_x = full_rect.min.x + outline_width;
    ui.painter().line_segment(
        [pos2(sep_x, full_rect.min.y), pos2(sep_x, full_rect.max.y)],
        Stroke::new(1.0_f32, Tokens::BORDER),
    );

    // 3. Right continuous vertical scroll of all procedures
    let right_rect = Rect::from_min_max(pos2(sep_x + 4.0_f32, full_rect.min.y), full_rect.max);
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(right_rect), |ui| {
        ScrollArea::both()
            .id_salt("output_continuous_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.add_space(8.0_f32);

                    let mut clicked_item: Option<SelectedTarget> = None;
                    let mut item_to_delete: Option<(usize, usize)> = None;
                    let mut proc_to_delete: Option<usize> = None;
                    let mut item_copied: Option<usize> = None;
                    let scroll_target = state.scroll_to_target;

                    for (p_idx, proc) in state.procedures.iter().enumerate() {
                        let is_proc_targeted = match scroll_target {
                            Some(SelectedTarget::Procedure(idx)) => idx == p_idx,
                            _ => false,
                        };

                        // Procedure Header Banner
                        let (proc_title_rect, proc_title_resp) = ui.allocate_exact_size(
                            vec2(ui.available_width().max(300.0), 26.0),
                            Sense::click(),
                        );
                        if is_proc_targeted {
                            ui.scroll_to_rect(proc_title_rect, Some(Align::Center));
                        }
                        if proc_title_resp.clicked() || proc_title_resp.secondary_clicked() {
                            clicked_item = Some(SelectedTarget::Procedure(p_idx));
                        }
                        proc_title_resp.context_menu(|ui| {
                            if ui.button(t!("output-menu-delete")).clicked() {
                                proc_to_delete = Some(p_idx);
                                ui.close_menu();
                            }
                        });

                        ui.painter().text(
                            pos2(proc_title_rect.min.x + 4.0, proc_title_rect.center().y),
                            Align2::LEFT_CENTER,
                            proc.title(),
                            FontId::proportional(15.0),
                            Tokens::ACCENT,
                        );

                        ui.add_space(4.0_f32);

                        // Render each item of this procedure
                        for (i_idx, item) in proc.items.iter().enumerate() {
                            let this_target = SelectedTarget::Item {
                                proc_idx: p_idx,
                                item_idx: i_idx,
                            };
                            let is_selected = state.selected_target == Some(this_target);
                            let is_item_scroll_targeted = scroll_target == Some(this_target);
                            let maybe_tsv = extract_item_tsv(&item.kind);

                            ui.vertical(|ui| {
                                let block_start_y = ui.cursor().top();
                                let is_item_copied = state.last_copied_id == Some(item.id);
                                let mut title_clicked = false;
                                let mut title_secondary_clicked = false;

                                // Item Title Header with Copy TSV button if table
                                ui.horizontal(|ui| {
                                    ui.add_space(8.0_f32);
                                    let title_resp =
                                        ui.strong(item.title()).interact(Sense::click());
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

                                    ui.add_space(12.0_f32);

                                    if let Some(tsv_data) = &maybe_tsv {
                                        let btn_label = if is_item_copied {
                                            egui::RichText::new(t!("output-menu-copied"))
                                                .color(Tokens::ACCENT)
                                        } else {
                                            egui::RichText::new(t!("output-menu-copy-tsv"))
                                        };
                                        let btn = ui
                                            .small_button(btn_label)
                                            .on_hover_text(t!("output-copied-toast"));
                                        if btn.clicked() {
                                            ui.ctx()
                                                .output_mut(|o| o.copied_text = tsv_data.clone());
                                            item_copied = Some(item.id);
                                        }
                                    }
                                });

                                if title_clicked || title_secondary_clicked {
                                    clicked_item = Some(this_target);
                                }

                                ui.add_space(4.0_f32);
                                let content_start_y = ui.cursor().top();

                                // Item Content
                                match &item.kind {
                                    OutputItemKind::Log(sps) => {
                                        render_log_block(ui, sps);
                                    }
                                    OutputItemKind::Notes(notes) => {
                                        render_notes_block(ui, notes);
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
                                    OutputItemKind::FrequenciesTables(results) => {
                                        render_spss_frequencies_tables(ui, results);
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
                                    OutputItemKind::Reliability(res) => {
                                        render_spss_reliability(ui, res);
                                    }
                                    OutputItemKind::Factor(res) => {
                                        render_spss_factor(ui, res);
                                    }
                                    OutputItemKind::Means(res) => {
                                        render_spss_means(ui, res);
                                    }
                                    OutputItemKind::Warning(msg) => {
                                        ui.colored_label(Tokens::CAPTION_CLOSE_HOVER, msg);
                                    }
                                    OutputItemKind::CustomText { content, .. } => {
                                        ui.label(content);
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

                                // Scroll to this item if it was targeted from outline
                                if is_item_scroll_targeted {
                                    ui.scroll_to_rect(total_item_rect, Some(Align::Center));
                                }

                                // Outline focus border if selected
                                if is_selected {
                                    ui.painter().rect_stroke(
                                        total_item_rect,
                                        2.0_f32,
                                        Stroke::new(1.5_f32, Tokens::ACCENT),
                                    );
                                }
                            });

                            ui.add_space(12.0_f32);
                        }

                        ui.add_space(8.0_f32);
                        ui.separator();
                        ui.add_space(8.0_f32);
                    }

                    if clicked_item.is_some() {
                        state.selected_target = clicked_item;
                    }

                    if let Some(id) = item_copied {
                        state.last_copied_id = Some(id);
                    }

                    // Process deletions if any
                    if let Some((p_idx, i_idx)) = item_to_delete {
                        if let Some(proc) = state.procedures.get_mut(p_idx) {
                            if i_idx < proc.items.len() {
                                proc.items.remove(i_idx);
                            }
                            if proc.items.is_empty() {
                                proc_to_delete = Some(p_idx);
                            }
                        }
                        state.selected_target = None;
                    }
                    if let Some(p_idx) = proc_to_delete {
                        if p_idx < state.procedures.len() {
                            state.procedures.remove(p_idx);
                        }
                        state.selected_target = None;
                    }

                    // Clear scroll target after dispatch
                    if state.scroll_to_target.is_some() {
                        state.scroll_to_target = None;
                    }
                });
            });
    });
}
