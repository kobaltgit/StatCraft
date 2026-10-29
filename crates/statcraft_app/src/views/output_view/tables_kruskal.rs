use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_opt_2dec, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::KruskalWallisResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_kruskal_wallis(
        &mut self,
        results: Vec<KruskalWallisResult>,
        grouping_var: &str,
        min_range: f64,
        max_range: f64,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let vars: Vec<String> = results.iter().map(|r| r.variable.clone()).collect();
        let sps_log = statcraft_syntax::commands::KruskalWallisCommand::to_sps(
            &vars,
            grouping_var,
            min_range,
            max_range,
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
            title_key: "output-npar-kruskal-title",
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
                    title_key: "output-npar-kruskal-title",
                    custom_title: None,
                    kind: OutputItemKind::KruskalWallis {
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

pub fn render_spss_kruskal_wallis_tables(
    ui: &mut Ui,
    results: &[KruskalWallisResult],
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
        ];
        adjust_widths(ui, &mut cols_ranks);

        ScrollArea::horizontal()
            .id_salt("kruskal_ranks_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_ranks, row_height);

                    for (r_idx, res) in results.iter().enumerate() {
                        let bg = if r_idx % 2 == 0 {
                            Tokens::BG_ROW_EVEN
                        } else {
                            Tokens::BG_ROW_ODD
                        };

                        for (g_idx, group) in res.groups.iter().enumerate() {
                            let var_str = if g_idx == 0 {
                                res.variable.clone()
                            } else {
                                String::new()
                            };

                            let row = [
                                (var_str, Align2::LEFT_CENTER),
                                (group.group_val.clone(), Align2::LEFT_CENTER),
                                (format!("{}", group.n), Align2::RIGHT_CENTER),
                                (format_opt_2dec(Some(group.mean_rank)), Align2::RIGHT_CENTER),
                            ];
                            render_data_row(ui, &cols_ranks, &row, row_height, bg);
                        }

                        let row_tot = [
                            (String::new(), Align2::LEFT_CENTER),
                            (t!("output-col-total"), Align2::LEFT_CENTER),
                            (format!("{}", res.total_n), Align2::RIGHT_CENTER),
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
            .id_salt("kruskal_test_stats_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols_stats, row_height);

                    // Row: Kruskal-Wallis H
                    let mut row_h = vec![(t!("output-row-kruskal-wallis-h"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_h.push((format_opt(Some(res.kruskal_wallis_h)), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_h, row_height, Tokens::BG_ROW_EVEN);

                    // Row: df
                    let mut row_df = vec![(t!("output-row-df"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_df.push((format!("{}", res.df), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_df, row_height, Tokens::BG_ROW_ODD);

                    // Row: Asymp. Sig.
                    let mut row_p = vec![(t!("output-row-asymp-sig"), Align2::LEFT_CENTER)];
                    for res in results {
                        row_p.push((format_sig(Some(res.asymp_sig)), Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols_stats, &row_p, row_height, Tokens::BG_ROW_EVEN);
                });
            });

        ui.add_space(4.0_f32);
        ui.label(
            egui::RichText::new(t!("output-footnote-kruskal-a"))
                .size(11.0)
                .color(Tokens::TEXT_MUTED),
        );
        ui.label(
            egui::RichText::new(format!(
                "b. {}: {}",
                t!("output-row-grouping-var"),
                grouping_var
            ))
            .size(11.0)
            .color(Tokens::TEXT_MUTED),
        );
        ui.add_space(16.0_f32);
    });
}

pub fn kruskal_wallis_to_tsv(results: &[KruskalWallisResult], grouping_var: &str) -> String {
    let mut out = String::new();

    // Table 1: Ranks
    out.push_str("Ranks\n");
    out.push_str(&format!("Variable\t{}\tN\tMean Rank\n", grouping_var));
    for r in results {
        for (g_idx, g) in r.groups.iter().enumerate() {
            let var_name = if g_idx == 0 { &r.variable } else { "" };
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                var_name,
                g.group_val,
                g.n,
                format_opt_2dec(Some(g.mean_rank))
            ));
        }
        out.push_str(&format!("\tTotal\t{}\t\n", r.total_n));
    }

    out.push_str("\nTest Statistics\n\t");
    for (i, r) in results.iter().enumerate() {
        out.push_str(&r.variable);
        if i + 1 < results.len() {
            out.push('\t');
        }
    }
    out.push('\n');

    out.push_str(&format!(
        "Kruskal-Wallis H\t{}\n",
        results
            .iter()
            .map(|r| format_opt(Some(r.kruskal_wallis_h)))
            .collect::<Vec<_>>()
            .join("\t")
    ));

    out.push_str(&format!(
        "df\t{}\n",
        results
            .iter()
            .map(|r| format!("{}", r.df))
            .collect::<Vec<_>>()
            .join("\t")
    ));

    out.push_str(&format!(
        "Asymp. Sig.\t{}\n",
        results
            .iter()
            .map(|r| format_sig(Some(r.asymp_sig)))
            .collect::<Vec<_>>()
            .join("\t")
    ));

    out
}
