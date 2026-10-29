use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{
    adjust_widths, format_opt, format_opt_2dec, format_sig, render_data_row, render_header_row,
};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::RelatedPairTestResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_related_samples(
        &mut self,
        results: Vec<RelatedPairTestResult>,
        pairs: &[(String, String)],
        wilcoxon: bool,
        sign: bool,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log =
            statcraft_syntax::commands::RelatedSamplesTestCommand::to_sps(pairs, wilcoxon, sign);

        let active_cases = results
            .first()
            .and_then(|r| {
                r.wilcoxon
                    .as_ref()
                    .map(|w| w.ranks.total_n)
                    .or_else(|| r.sign.as_ref().map(|s| s.diffs.total_n))
            })
            .unwrap_or(0);

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: pairs.len() * 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-npar-related-title",
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
                    title_key: "output-npar-related-title",
                    custom_title: None,
                    kind: OutputItemKind::TwoRelatedSamples(results),
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

pub fn render_spss_related_samples_tables(ui: &mut Ui, results: &[RelatedPairTestResult]) {
    if results.is_empty() {
        return;
    }

    let has_wilcoxon = results.iter().any(|r| r.wilcoxon.is_some());
    let has_sign = results.iter().any(|r| r.sign.is_some());
    let row_height = 22.0_f32;

    ui.vertical(|ui| {
        if has_wilcoxon {
            // Table 1: Ranks
            ui.strong(t!("output-npar-ranks-title"));
            ui.add_space(4.0_f32);

            let mut cols_ranks: Vec<(String, f32)> = vec![
                (String::new(), 140.0_f32),
                (String::new(), 130.0_f32),
                (t!("output-col-n"), 60.0_f32),
                (t!("output-col-mean-rank"), 90.0_f32),
                (t!("output-col-sum-ranks"), 100.0_f32),
            ];
            adjust_widths(ui, &mut cols_ranks);

            ScrollArea::horizontal()
                .id_salt("npar_related_ranks_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        render_header_row(ui, &cols_ranks, row_height);

                        for (r_idx, res) in results.iter().enumerate() {
                            if let Some(w) = &res.wilcoxon {
                                let bg = if r_idx % 2 == 0 {
                                    Tokens::BG_ROW_EVEN
                                } else {
                                    Tokens::BG_ROW_ODD
                                };

                                let pair_label = format!("{} - {}", w.var2, w.var1);

                                let row_neg = [
                                    (pair_label, Align2::LEFT_CENTER),
                                    (
                                        format!("{}(a)", t!("output-npar-ranks-negative")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (
                                        format!("{}", w.ranks.negative_ranks.n),
                                        Align2::RIGHT_CENTER,
                                    ),
                                    (
                                        format_opt_2dec(Some(w.ranks.negative_ranks.mean_rank)),
                                        Align2::RIGHT_CENTER,
                                    ),
                                    (
                                        format_opt_2dec(Some(w.ranks.negative_ranks.sum_ranks)),
                                        Align2::RIGHT_CENTER,
                                    ),
                                ];
                                render_data_row(ui, &cols_ranks, &row_neg, row_height, bg);

                                let row_pos = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (
                                        format!("{}(b)", t!("output-npar-ranks-positive")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (
                                        format!("{}", w.ranks.positive_ranks.n),
                                        Align2::RIGHT_CENTER,
                                    ),
                                    (
                                        format_opt_2dec(Some(w.ranks.positive_ranks.mean_rank)),
                                        Align2::RIGHT_CENTER,
                                    ),
                                    (
                                        format_opt_2dec(Some(w.ranks.positive_ranks.sum_ranks)),
                                        Align2::RIGHT_CENTER,
                                    ),
                                ];
                                render_data_row(ui, &cols_ranks, &row_pos, row_height, bg);

                                let row_ties = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (
                                        format!("{}(c)", t!("output-npar-ranks-ties")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (format!("{}", w.ranks.ties_count), Align2::RIGHT_CENTER),
                                    (String::new(), Align2::RIGHT_CENTER),
                                    (String::new(), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_ranks, &row_ties, row_height, bg);

                                let row_tot = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (t!("output-col-total"), Align2::LEFT_CENTER),
                                    (format!("{}", w.ranks.total_n), Align2::RIGHT_CENTER),
                                    (String::new(), Align2::RIGHT_CENTER),
                                    (String::new(), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_ranks, &row_tot, row_height, bg);
                            }
                        }
                    });
                });

            ui.add_space(4.0_f32);
            for res in results {
                if let Some(w) = &res.wilcoxon {
                    ui.label(
                        egui::RichText::new(format!(
                            "a. {} < {}\nb. {} > {}\nc. {} = {}",
                            w.var2, w.var1, w.var2, w.var1, w.var2, w.var1
                        ))
                        .size(11.0)
                        .color(Tokens::TEXT_MUTED),
                    );
                }
            }

            ui.add_space(14.0_f32);

            // Table 2: Test Statistics
            ui.strong(t!("output-npar-test-stats-title"));
            ui.add_space(4.0_f32);

            let mut cols_stats: Vec<(String, f32)> = vec![(String::new(), 160.0_f32)];
            for res in results {
                if let Some(w) = &res.wilcoxon {
                    cols_stats.push((format!("{} - {}", w.var2, w.var1), 110.0_f32));
                }
            }
            adjust_widths(ui, &mut cols_stats);

            ScrollArea::horizontal()
                .id_salt("npar_related_wilcoxon_stats_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        render_header_row(ui, &cols_stats, row_height);

                        // Row: Z
                        let mut row_z = vec![(t!("output-row-z"), Align2::LEFT_CENTER)];
                        for res in results {
                            if let Some(w) = &res.wilcoxon {
                                row_z.push((format_opt(Some(w.z_stat)), Align2::RIGHT_CENTER));
                            }
                        }
                        render_data_row(ui, &cols_stats, &row_z, row_height, Tokens::BG_ROW_EVEN);

                        // Row: Asymp. Sig. (2-tailed)
                        let mut row_p = vec![(t!("output-row-asymp-sig"), Align2::LEFT_CENTER)];
                        for res in results {
                            if let Some(w) = &res.wilcoxon {
                                row_p.push((
                                    format_sig(Some(w.asymp_sig_2tailed)),
                                    Align2::RIGHT_CENTER,
                                ));
                            }
                        }
                        render_data_row(ui, &cols_stats, &row_p, row_height, Tokens::BG_ROW_ODD);
                    });
                });

            ui.add_space(4.0_f32);
            ui.label(
                egui::RichText::new(format!("a. {}", t!("output-npar-footnote-wilcoxon")))
                    .size(11.0)
                    .color(Tokens::TEXT_MUTED),
            );
            for res in results {
                if let Some(w) = &res.wilcoxon {
                    let footnote_b = if w.based_on_negative {
                        t!("output-npar-footnote-based-negative")
                    } else {
                        t!("output-npar-footnote-based-positive")
                    };
                    ui.label(
                        egui::RichText::new(format!("b. {}", footnote_b))
                            .size(11.0)
                            .color(Tokens::TEXT_MUTED),
                    );
                }
            }
        }

        if has_sign {
            if has_wilcoxon {
                ui.add_space(16.0_f32);
            }

            // Table 3: Sign Test Frequencies
            ui.strong(t!("output-npar-sign-frequencies"));
            ui.add_space(4.0_f32);

            let mut cols_sign_freq: Vec<(String, f32)> = vec![
                (String::new(), 140.0_f32),
                (String::new(), 150.0_f32),
                (t!("output-col-n"), 60.0_f32),
            ];
            adjust_widths(ui, &mut cols_sign_freq);

            ScrollArea::horizontal()
                .id_salt("npar_sign_freq_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        render_header_row(ui, &cols_sign_freq, row_height);

                        for (r_idx, res) in results.iter().enumerate() {
                            if let Some(s) = &res.sign {
                                let bg = if r_idx % 2 == 0 {
                                    Tokens::BG_ROW_EVEN
                                } else {
                                    Tokens::BG_ROW_ODD
                                };

                                let pair_label = format!("{} - {}", s.var2, s.var1);

                                let row_neg = [
                                    (pair_label, Align2::LEFT_CENTER),
                                    (
                                        format!("{}(a)", t!("output-npar-sign-negative-diff")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (format!("{}", s.diffs.negative_count), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_sign_freq, &row_neg, row_height, bg);

                                let row_pos = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (
                                        format!("{}(b)", t!("output-npar-sign-positive-diff")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (format!("{}", s.diffs.positive_count), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_sign_freq, &row_pos, row_height, bg);

                                let row_ties = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (
                                        format!("{}(c)", t!("output-npar-sign-ties")),
                                        Align2::LEFT_CENTER,
                                    ),
                                    (format!("{}", s.diffs.ties_count), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_sign_freq, &row_ties, row_height, bg);

                                let row_tot = [
                                    (String::new(), Align2::LEFT_CENTER),
                                    (t!("output-col-total"), Align2::LEFT_CENTER),
                                    (format!("{}", s.diffs.total_n), Align2::RIGHT_CENTER),
                                ];
                                render_data_row(ui, &cols_sign_freq, &row_tot, row_height, bg);
                            }
                        }
                    });
                });

            ui.add_space(4.0_f32);
            for res in results {
                if let Some(s) = &res.sign {
                    ui.label(
                        egui::RichText::new(format!(
                            "a. {} < {}\nb. {} > {}\nc. {} = {}",
                            s.var2, s.var1, s.var2, s.var1, s.var2, s.var1
                        ))
                        .size(11.0)
                        .color(Tokens::TEXT_MUTED),
                    );
                }
            }

            ui.add_space(14.0_f32);

            // Table 4: Test Statistics (Sign Test)
            ui.strong(t!("output-npar-test-stats-title"));
            ui.add_space(4.0_f32);

            let mut cols_sign_stats: Vec<(String, f32)> = vec![(String::new(), 160.0_f32)];
            for res in results {
                if let Some(s) = &res.sign {
                    cols_sign_stats.push((format!("{} - {}", s.var2, s.var1), 110.0_f32));
                }
            }
            adjust_widths(ui, &mut cols_sign_stats);

            ScrollArea::horizontal()
                .id_salt("npar_sign_stats_scroll")
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        render_header_row(ui, &cols_sign_stats, row_height);

                        let mut row_p =
                            vec![(t!("output-npar-exact-sig-2tailed"), Align2::LEFT_CENTER)];
                        for res in results {
                            if let Some(s) = &res.sign {
                                row_p.push((
                                    format_sig(Some(s.exact_sig_2tailed)),
                                    Align2::RIGHT_CENTER,
                                ));
                            }
                        }
                        render_data_row(
                            ui,
                            &cols_sign_stats,
                            &row_p,
                            row_height,
                            Tokens::BG_ROW_EVEN,
                        );
                    });
                });

            ui.add_space(4.0_f32);
            ui.label(
                egui::RichText::new(format!("a. {}", t!("output-npar-footnote-sign")))
                    .size(11.0)
                    .color(Tokens::TEXT_MUTED),
            );
        }
    });
}

pub fn related_samples_to_tsv(results: &[RelatedPairTestResult]) -> String {
    let mut out = String::new();

    // Wilcoxon section if present
    if results.iter().any(|r| r.wilcoxon.is_some()) {
        out.push_str("Ranks\n");
        out.push_str("Pair\tCategory\tN\tMean Rank\tSum of Ranks\n");
        for res in results {
            if let Some(w) = &res.wilcoxon {
                let pair = format!("{} - {}", w.var2, w.var1);
                out.push_str(&format!(
                    "{}\tNegative Ranks\t{}\t{:.2}\t{:.2}\n",
                    pair,
                    w.ranks.negative_ranks.n,
                    w.ranks.negative_ranks.mean_rank,
                    w.ranks.negative_ranks.sum_ranks
                ));
                out.push_str(&format!(
                    "{}\tPositive Ranks\t{}\t{:.2}\t{:.2}\n",
                    pair,
                    w.ranks.positive_ranks.n,
                    w.ranks.positive_ranks.mean_rank,
                    w.ranks.positive_ranks.sum_ranks
                ));
                out.push_str(&format!("{}\tTies\t{}\t\t\n", pair, w.ranks.ties_count));
                out.push_str(&format!("{}\tTotal\t{}\t\t\n", pair, w.ranks.total_n));
            }
        }
        out.push('\n');
        out.push_str("Test Statistics (Wilcoxon Signed Ranks)\n");
        out.push_str("Statistic");
        for res in results {
            if let Some(w) = &res.wilcoxon {
                out.push_str(&format!("\t{} - {}", w.var2, w.var1));
            }
        }
        out.push('\n');

        out.push('Z');
        for res in results {
            if let Some(w) = &res.wilcoxon {
                out.push_str(&format!("\t{:.3}", w.z_stat));
            }
        }
        out.push('\n');

        out.push_str("Asymp. Sig. (2-tailed)");
        for res in results {
            if let Some(w) = &res.wilcoxon {
                out.push_str(&format!("\t{:.3}", w.asymp_sig_2tailed));
            }
        }
        out.push('\n');
    }

    // Sign test section if present
    if results.iter().any(|r| r.sign.is_some()) {
        out.push_str("\nFrequencies (Sign Test)\n");
        out.push_str("Pair\tCategory\tN\n");
        for res in results {
            if let Some(s) = &res.sign {
                let pair = format!("{} - {}", s.var2, s.var1);
                out.push_str(&format!(
                    "{}\tNegative Differences\t{}\n",
                    pair, s.diffs.negative_count
                ));
                out.push_str(&format!(
                    "{}\tPositive Differences\t{}\n",
                    pair, s.diffs.positive_count
                ));
                out.push_str(&format!("{}\tTies\t{}\n", pair, s.diffs.ties_count));
                out.push_str(&format!("{}\tTotal\t{}\n", pair, s.diffs.total_n));
            }
        }
        out.push('\n');
        out.push_str("Test Statistics (Sign Test)\n");
        out.push_str("Statistic");
        for res in results {
            if let Some(s) = &res.sign {
                out.push_str(&format!("\t{} - {}", s.var2, s.var1));
            }
        }
        out.push('\n');

        out.push_str("Exact Sig. (2-tailed)");
        for res in results {
            if let Some(s) = &res.sign {
                out.push_str(&format!("\t{:.3}", s.exact_sig_2tailed));
            }
        }
        out.push('\n');
    }

    out
}
