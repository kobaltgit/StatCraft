use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_opt_2dec, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::MannWhitneyResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_mann_whitney(
        &mut self,
        results: Vec<MannWhitneyResult>,
        grouping_var: &str,
        group1: &str,
        group2: &str,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let vars: Vec<String> = results.iter().map(|r| r.variable.clone()).collect();
        let sps_log = statcraft_syntax::commands::MannWhitneyCommand::to_sps(
            &vars,
            grouping_var,
            group1,
            group2,
        );

        let active_cases = results.first().map(|r| r.total_n).unwrap_or(0);
        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-npar-mann-whitney-title",
            custom_title: None,
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(sps_log),
                },
                OutputItem {
                    id: notes_id,
                    title_key: "output-notes",
                    custom_title: None,
                    kind: OutputItemKind::Notes(notes),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-npar-mann-whitney-title",
                    custom_title: None,
                    kind: OutputItemKind::MannWhitney {
                        results,
                        grouping_var: grouping_var.to_string(),
                    },
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 2,
        });
        self.scroll_to_target = self.selected_target;
    }
}

pub fn render_spss_mann_whitney_tables(
    ui: &mut Ui,
    results: &[MannWhitneyResult],
    grouping_var: &str,
) {
    if results.is_empty() {
        return;
    }

    let row_height = 22.0_f32;

    ui.vertical(|ui| {
        // Table 1: Ranks
        ui.strong(t!("output-npar-ranks-title"));
        ui.add_space(4.0_f32);

        let mut cols_ranks: Vec<(String, f32)> = vec![
            (t!("output-col-variable"), 120.0_f32),
            (grouping_var.to_string(), 90.0_f32),
            (t!("output-col-n"), 60.0_f32),
            (t!("output-col-mean-rank"), 90.0_f32),
            (t!("output-col-sum-ranks"), 100.0_f32),
        ];
        adjust_widths(ui, &mut cols_ranks);

        ScrollArea::horizontal()
            .id_salt("npar_ranks_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_ranks, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
                        };

                        let row1 = [
                            (res.variable.clone(), Align2::LEFT_CENTER),
                            (res.group1_stats.group_val.clone(), Align2::LEFT_CENTER),
                            (format!("{}", res.group1_stats.n), Align2::RIGHT_CENTER),
                            (
                                format_opt_2dec(Some(res.group1_stats.mean_rank)),
                                Align2::RIGHT_CENTER,
                            ),
                            (
                                format_opt_2dec(Some(res.group1_stats.sum_ranks)),
                                Align2::RIGHT_CENTER,
                            ),
                        ];
                        render_data_row(ui, &cols_ranks, &row1, row_height, bg);

                        let row2 = [
                            (String::new(), Align2::LEFT_CENTER),
                            (res.group2_stats.group_val.clone(), Align2::LEFT_CENTER),
                            (format!("{}", res.group2_stats.n), Align2::RIGHT_CENTER),
                            (
                                format_opt_2dec(Some(res.group2_stats.mean_rank)),
                                Align2::RIGHT_CENTER,
                            ),
                            (
                                format_opt_2dec(Some(res.group2_stats.sum_ranks)),
                                Align2::RIGHT_CENTER,
                            ),
                        ];
                        render_data_row(ui, &cols_ranks, &row2, row_height, bg);

                        let row_tot = [
                            (String::new(), Align2::LEFT_CENTER),
                            (t!("output-col-total"), Align2::LEFT_CENTER),
                            (format!("{}", res.total_n), Align2::RIGHT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                            (String::new(), Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols_ranks, &row_tot, row_height, bg);
                    }
                });
            });

        ui.add_space(14.0_f32);

        // Table 2: Test Statistics
        ui.strong(t!("output-npar-test-stats-title"));
        ui.add_space(4.0_f32);

        let mut cols_stats: Vec<(String, f32)> = vec![(String::new(), 160.0_f32)];
        for res in results {
            cols_stats.push((res.variable.clone(), 100.0_f32));
        }
        adjust_widths(ui, &mut cols_stats);

        ScrollArea::horizontal()
            .id_salt("npar_test_stats_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_stats, row_height);

                    // Row: Mann-Whitney U
                    let mut row_u = vec![(t!("output-row-mann-whitney-u"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_u.push((format_opt(Some(res.mann_whitney_u)), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_u, row_height, Tokens::BG_ROW_EVEN);

                    // Row: Wilcoxon W
                    let mut row_w = vec![(t!("output-row-wilcoxon-w"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_w.push((format_opt(Some(res.wilcoxon_w)), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_w, row_height, Tokens::BG_ROW_ODD);

                    // Row: Z
                    let mut row_z = vec![(t!("output-row-z"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_z.push((format_opt(Some(res.z_stat)), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_z, row_height, Tokens::BG_ROW_EVEN);

                    // Row: Asymp. Sig. (2-tailed)
                    let mut row_p = vec![(t!("output-row-asymp-sig"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_p.push((
                            format_sig(Some(res.asymp_sig_2tailed)),
                            Align2::RIGHT_CENTER,
                        ));
                    }
                    render_data_row(ui, &cols_stats, &row_p, row_height, Tokens::BG_ROW_ODD);
                });
            });

        ui.add_space(4.0_f32);
        ui.label(
            egui::RichText::new(format!(
                "a. {}: {}",
                t!("output-row-grouping-var"),
                grouping_var
            ))
            .size(11.0)
            .color(Tokens::TEXT_MUTED),
        );
        ui.add_space(16.0_f32);
    });
}

pub fn mann_whitney_to_tsv(results: &[MannWhitneyResult], grouping_var: &str) -> String {
    let mut out = String::new();

    // Table 1: Ranks
    out.push_str("Ranks\n");
    out.push_str(&format!(
        "Variable\t{}\tN\tMean Rank\tSum of Ranks\n",
        grouping_var
    ));
    for r in results {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            r.variable,
            r.group1_stats.group_val,
            r.group1_stats.n,
            format_opt_2dec(Some(r.group1_stats.mean_rank)),
            format_opt_2dec(Some(r.group1_stats.sum_ranks))
        ));
        out.push_str(&format!(
            "\t{}\t{}\t{}\t{}\n",
            r.group2_stats.group_val,
            r.group2_stats.n,
            format_opt_2dec(Some(r.group2_stats.mean_rank)),
            format_opt_2dec(Some(r.group2_stats.sum_ranks))
        ));
        out.push_str(&format!("\tTotal\t{}\t\t\n", r.total_n));
    }

    out.push('\n');

    // Table 2: Test Statistics
    out.push_str("Test Statistics\n");
    let mut header_line = String::from("Statistic");
    for r in results {
        header_line.push('\t');
        header_line.push_str(&r.variable);
    }
    header_line.push('\n');
    out.push_str(&header_line);

    // Mann-Whitney U
    let mut line_u = String::from("Mann-Whitney U");
    for r in results {
        line_u.push('\t');
        line_u.push_str(&format_opt(Some(r.mann_whitney_u)));
    }
    line_u.push('\n');
    out.push_str(&line_u);

    // Wilcoxon W
    let mut line_w = String::from("Wilcoxon W");
    for r in results {
        line_w.push('\t');
        line_w.push_str(&format_opt(Some(r.wilcoxon_w)));
    }
    line_w.push('\n');
    out.push_str(&line_w);

    // Z
    let mut line_z = String::from("Z");
    for r in results {
        line_z.push('\t');
        line_z.push_str(&format_opt(Some(r.z_stat)));
    }
    line_z.push('\n');
    out.push_str(&line_z);

    // Asymp. Sig. (2-tailed)
    let mut line_p = String::from("Asymp. Sig. (2-tailed)");
    for r in results {
        line_p.push('\t');
        line_p.push_str(&format_sig(Some(r.asymp_sig_2tailed)));
    }
    line_p.push('\n');
    out.push_str(&line_p);

    out.push_str(&format!("a. Grouping Variable: {}\n", grouping_var));

    out
}
