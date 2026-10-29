use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, Ui};
use statcraft_algo::{
    FactorAnalysisResult, FactorExtractionCriteria, FactorExtractionMethod, FactorOptions,
};
use statcraft_i18n::t;

fn format_3dec(val: f64) -> String {
    format!("{:.3}", val)
}

fn format_opt_3dec(val: Option<f64>) -> String {
    val.map(|v| format!("{:.3}", v))
        .unwrap_or_else(|| "-".to_string())
}

impl OutputViewerState {
    pub fn add_factor(&mut self, result: FactorAnalysisResult, var_names: Vec<String>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let options = FactorOptions {
            extraction: FactorExtractionMethod::PC,
            criteria: FactorExtractionCriteria::Mineigen(1.0),
            kmo_bartlett: result.kmo_bartlett.is_some(),
        };

        let sps_log = statcraft_syntax::commands::FactorCommand::to_sps(&var_names, &options);

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_cases,
            active_vars: var_names.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-factor-title",
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
                    title_key: "output-factor-table-title",
                    custom_title: None,
                    kind: OutputItemKind::Factor(Box::new(result)),
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

pub fn render_spss_factor(ui: &mut Ui, res: &FactorAnalysisResult) {
    ui.vertical(|ui| {
        // Table 1: KMO and Bartlett's Test
        if let Some(ref kb) = res.kmo_bartlett {
            render_kmo_bartlett_table(ui, kb);
            ui.add_space(14.0);
        }

        // Table 2: Communalities
        render_communalities_table(ui, res);
        ui.add_space(14.0);

        // Table 3: Total Variance Explained
        render_variance_explained_table(ui, res);
        ui.add_space(14.0);

        // Table 4: Component Matrix
        render_component_matrix_table(ui, res);
    });
}

fn render_kmo_bartlett_table(ui: &mut Ui, kb: &statcraft_algo::KmoBartlettResult) {
    ui.label(
        egui::RichText::new(t!("output-factor-kmo-title"))
            .font(egui::FontId::proportional(13.0))
            .color(Tokens::TEXT_PRIMARY),
    );
    ui.add_space(3.0);

    let mut cols = vec![
        (t!("output-factor-kmo-measure-col"), 260.0_f32),
        (t!("output-factor-value-col"), 110.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    render_header_row(ui, &cols, row_h);

    let rows: [(&'static str, String); 4] = [
        ("output-factor-kmo-stat", format_3dec(kb.kmo)),
        (
            "output-factor-bartlett-approx-chi",
            format_3dec(kb.bartlett_approx_chi_square),
        ),
        ("output-factor-bartlett-df", kb.bartlett_df.to_string()),
        ("output-factor-bartlett-sig", format_3dec(kb.bartlett_sig)),
    ];

    for (idx, (key, val)) in rows.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::BG_ROW_EVEN
        } else {
            Tokens::BG_ROW_ODD
        };
        let vals = vec![
            (t!(key), Align2::LEFT_CENTER),
            (val.clone(), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &vals, row_h, bg);
    }
}

fn render_communalities_table(ui: &mut Ui, res: &FactorAnalysisResult) {
    ui.label(
        egui::RichText::new(t!("output-factor-communalities-title"))
            .font(egui::FontId::proportional(13.0))
            .color(Tokens::TEXT_PRIMARY),
    );
    ui.add_space(3.0);

    let mut cols = vec![
        (t!("output-factor-variable-col"), 180.0_f32),
        (t!("output-factor-initial-col"), 100.0_f32),
        (t!("output-factor-extraction-col"), 110.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    render_header_row(ui, &cols, row_h);

    for (idx, row) in res.communalities.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::BG_ROW_EVEN
        } else {
            Tokens::BG_ROW_ODD
        };
        let vals = vec![
            (row.variable.clone(), Align2::LEFT_CENTER),
            (format_3dec(row.initial), Align2::RIGHT_CENTER),
            (format_3dec(row.extraction), Align2::RIGHT_CENTER),
        ];
        render_data_row(ui, &cols, &vals, row_h, bg);
    }
}

fn render_variance_explained_table(ui: &mut Ui, res: &FactorAnalysisResult) {
    ui.label(
        egui::RichText::new(t!("output-factor-total-variance-title"))
            .font(egui::FontId::proportional(13.0))
            .color(Tokens::TEXT_PRIMARY),
    );
    ui.add_space(3.0);

    let mut cols = vec![
        (t!("output-factor-component-col"), 90.0_f32),
        (t!("output-factor-initial-total-col"), 100.0_f32),
        (t!("output-factor-initial-pct-col"), 110.0_f32),
        (t!("output-factor-initial-cum-col"), 120.0_f32),
        (t!("output-factor-extract-total-col"), 100.0_f32),
        (t!("output-factor-extract-pct-col"), 110.0_f32),
        (t!("output-factor-extract-cum-col"), 120.0_f32),
    ];
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    render_header_row(ui, &cols, row_h);

    for (idx, row) in res.total_variance_explained.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::BG_ROW_EVEN
        } else {
            Tokens::BG_ROW_ODD
        };
        let vals = vec![
            (row.component.to_string(), Align2::CENTER_CENTER),
            (format_3dec(row.initial_total), Align2::RIGHT_CENTER),
            (format_3dec(row.initial_pct_variance), Align2::RIGHT_CENTER),
            (
                format_3dec(row.initial_cumulative_pct),
                Align2::RIGHT_CENTER,
            ),
            (format_opt_3dec(row.extraction_total), Align2::RIGHT_CENTER),
            (
                format_opt_3dec(row.extraction_pct_variance),
                Align2::RIGHT_CENTER,
            ),
            (
                format_opt_3dec(row.extraction_cumulative_pct),
                Align2::RIGHT_CENTER,
            ),
        ];
        render_data_row(ui, &cols, &vals, row_h, bg);
    }
}

fn render_component_matrix_table(ui: &mut Ui, res: &FactorAnalysisResult) {
    ui.label(
        egui::RichText::new(t!("output-factor-component-matrix-title"))
            .font(egui::FontId::proportional(13.0))
            .color(Tokens::TEXT_PRIMARY),
    );
    ui.add_space(3.0);

    let mut cols = vec![(t!("output-factor-variable-col"), 180.0_f32)];
    for k in 0..res.n_components_extracted {
        cols.push((format!("{} {}", t!("output-factor-comp-col"), k + 1), 90.0));
    }
    adjust_widths(ui, &mut cols);

    let row_h = 22.0_f32;
    render_header_row(ui, &cols, row_h);

    for (idx, row) in res.component_matrix.iter().enumerate() {
        let bg = if idx % 2 == 0 {
            Tokens::BG_ROW_EVEN
        } else {
            Tokens::BG_ROW_ODD
        };
        let mut vals = vec![(row.variable.clone(), Align2::LEFT_CENTER)];
        for &load in &row.loadings {
            vals.push((format_3dec(load), Align2::RIGHT_CENTER));
        }
        render_data_row(ui, &cols, &vals, row_h, bg);
    }
}

pub fn factor_to_tsv(res: &FactorAnalysisResult) -> String {
    let mut out = String::new();

    if let Some(ref kb) = res.kmo_bartlett {
        out.push_str("KMO and Bartlett's Test\n");
        out.push_str(&format!(
            "Kaiser-Meyer-Olkin Measure of Sampling Adequacy\t{:.3}\n",
            kb.kmo
        ));
        out.push_str(&format!(
            "Bartlett's Test of Sphericity Approx. Chi-Square\t{:.3}\n",
            kb.bartlett_approx_chi_square
        ));
        out.push_str(&format!("df\t{}\n", kb.bartlett_df));
        out.push_str(&format!("Sig.\t{:.3}\n\n", kb.bartlett_sig));
    }

    out.push_str("Communalities\nVariable\tInitial\tExtraction\n");
    for row in &res.communalities {
        out.push_str(&format!(
            "{}\t{:.3}\t{:.3}\n",
            row.variable, row.initial, row.extraction
        ));
    }
    out.push('\n');

    out.push_str("Total Variance Explained\nComponent\tInitial Total\tInitial % of Variance\tInitial Cumulative %\tExtraction Total\tExtraction % of Variance\tExtraction Cumulative %\n");
    for row in &res.total_variance_explained {
        out.push_str(&format!(
            "{}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\t{}\n",
            row.component,
            row.initial_total,
            row.initial_pct_variance,
            row.initial_cumulative_pct,
            format_opt_3dec(row.extraction_total),
            format_opt_3dec(row.extraction_pct_variance),
            format_opt_3dec(row.extraction_cumulative_pct)
        ));
    }
    out.push('\n');

    out.push_str("Component Matrix\nVariable");
    for k in 0..res.n_components_extracted {
        out.push_str(&format!("\tComponent {}", k + 1));
    }
    out.push('\n');
    for row in &res.component_matrix {
        out.push_str(&row.variable);
        for &load in &row.loadings {
            out.push_str(&format!("\t{:.3}", load));
        }
        out.push('\n');
    }

    out
}
