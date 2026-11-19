use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::{CurveFitResult, CurveModelType};
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_curve_fit(&mut self, result: CurveFitResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let models: Vec<CurveModelType> = result.models.iter().map(|m| m.model_type).collect();
        let sps_log = statcraft_syntax::commands::CurveFitCommand::to_sps(
            &result.dependent_var,
            &result.independent_var,
            &models,
            true,
        );

        let active_cases = result.models.first().map(|m| m.n_cases).unwrap_or(0);

        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-curvefit-title",
            custom_title: None,
            dataset_name: Some(self.active_dataset_name.clone()),
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
                    title_key: "output-curvefit-table-title",
                    custom_title: None,
                    kind: OutputItemKind::CurveFit(Box::new(result)),
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

pub fn render_spss_curvefit_tables(ui: &mut Ui, res: &CurveFitResult) {
    ui.vertical(|ui| {
        ui.strong(t!("output-curvefit-model-summary-title"));
        ui.add_space(4.0);

        let mut cols: Vec<(String, f32)> = vec![
            (t!("output-curvefit-col-equation"), 110.0),
            (t!("output-curvefit-col-rsq"), 85.0),
            (t!("output-curvefit-col-f"), 85.0),
            (t!("output-curvefit-col-df1"), 55.0),
            (t!("output-curvefit-col-df2"), 55.0),
            (t!("output-curvefit-col-sig"), 75.0),
            (t!("output-curvefit-col-constant"), 85.0),
            (t!("output-curvefit-col-b1"), 85.0),
            (t!("output-curvefit-col-b2"), 85.0),
        ];
        adjust_widths(ui, &mut cols);

        let row_height = 20.0;
        ScrollArea::horizontal()
            .id_salt("curvefit_summary_scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    render_header_row(ui, &cols, row_height);

                    for (idx, row) in res.models.iter().enumerate() {
                        let bg = if idx % 2 == 0 {
                            Tokens::bg_row_even()
                        } else {
                            Tokens::bg_row_odd()
                        };
                        let eq_name = match row.model_type {
                            CurveModelType::Linear => t!("dialog-curvefit-linear"),
                            CurveModelType::Logarithmic => t!("dialog-curvefit-logarithmic"),
                            CurveModelType::Inverse => t!("dialog-curvefit-inverse"),
                            CurveModelType::Quadratic => t!("dialog-curvefit-quadratic"),
                            CurveModelType::Power => t!("dialog-curvefit-power"),
                            CurveModelType::Exponential => t!("dialog-curvefit-exponential"),
                        };

                        let b2_str = row
                            .b2
                            .map(|v| format!("{:.3}", v))
                            .unwrap_or_else(|| "".to_string());

                        let vals = [
                            (eq_name, Align2::LEFT_CENTER),
                            (format!("{:.3}", row.r_squared), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.f_stat), Align2::RIGHT_CENTER),
                            (format!("{}", row.df1), Align2::RIGHT_CENTER),
                            (format!("{}", row.df2), Align2::RIGHT_CENTER),
                            (format_sig(Some(row.sig)), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.b0), Align2::RIGHT_CENTER),
                            (format!("{:.3}", row.b1), Align2::RIGHT_CENTER),
                            (b2_str, Align2::RIGHT_CENTER),
                        ];
                        render_data_row(ui, &cols, &vals, row_height, bg);
                    }
                });
            });

        ui.add_space(4.0);
        ui.label(t!(
            "output-curvefit-dep-note",
            dep = res.dependent_var.as_str()
        ));
        ui.add_space(16.0);
    });
}

pub fn curvefit_to_tsv(res: &CurveFitResult) -> String {
    let mut out = String::new();
    out.push_str("Equation\tR Square\tF\tdf1\tdf2\tSig.\tConstant\tb1\tb2\n");

    for row in &res.models {
        let b2_str = row
            .b2
            .map(|v| format!("{:.4}", v))
            .unwrap_or_else(|| "".to_string());

        out.push_str(&format!(
            "{}\t{:.4}\t{:.4}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{}\n",
            row.model_type.name(),
            row.r_squared,
            row.f_stat,
            row.df1,
            row.df2,
            format_sig(Some(row.sig)),
            row.b0,
            row.b1,
            b2_str,
        ));
    }

    out
}
