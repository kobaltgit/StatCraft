pub mod charts;
pub mod export;
pub mod matrix;
pub mod stats_tables;

pub use export::{crosstabs_to_report_tables, crosstabs_to_tsv};

use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use crate::dialogs::table_style_dialog::{format_condition_summary, format_style_summary};
use crate::theme::Tokens;
use egui::{FontId, Ui};
use statcraft_algo::{BootstrapOptions, BootstrapSamplingMethod, CrosstabOptions, CrosstabResult};
use statcraft_i18n::t;
use statcraft_reporting::model::{CellAlignment, ReportTable};
use statcraft_reporting::styling::{ConditionalFormattingEngine, TableStyleRule, TargetValue};

impl OutputViewerState {
    pub fn add_crosstabs(&mut self, results: Vec<CrosstabResult>, options: CrosstabOptions) {
        self.add_crosstabs_styled(results, options, None, None, false, None);
    }

    pub fn add_crosstabs_styled(
        &mut self,
        results: Vec<CrosstabResult>,
        options: CrosstabOptions,
        custom_sps_log: Option<String>,
        table_styles: Option<&[TableStyleRule]>,
        create_style_report: bool,
        bootstrap: Option<&BootstrapOptions>,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();

        let rows: Vec<String> = results.iter().map(|r| r.row_var.clone()).collect();
        let cols: Vec<String> = results.iter().map(|r| r.col_var.clone()).collect();
        let sps_log = custom_sps_log.unwrap_or_else(|| {
            statcraft_syntax::commands::CrosstabsCommand::to_sps(&rows, &cols, &options)
        });
        let active_cases = results
            .first()
            .map(|r| r.n_valid + r.n_missing)
            .unwrap_or(0);
        let notes = OutputNotes {
            dataset_name: self.active_dataset_name.clone(),
            timestamp: "Session".to_string(),
            active_cases,
            active_vars: results.len(),
        };

        let mut items = vec![
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
        ];

        if let Some(boot_opts) = bootstrap {
            let spec_table = build_bootstrap_specs_report_table(boot_opts);
            items.push(OutputItem {
                id: self.alloc_id(),
                title_key: "output-bootstrap-specs-title",
                custom_title: None,
                kind: OutputItemKind::ReportTable(spec_table),
            });
        }

        let has_styles = table_styles.map(|r| !r.is_empty()).unwrap_or(false);
        if has_styles || create_style_report {
            let (styled_tables, audit_report) =
                apply_crosstabs_styles(&results, table_styles, create_style_report);
            for tab in styled_tables {
                items.push(OutputItem {
                    id: self.alloc_id(),
                    title_key: "output-crosstabs-title",
                    custom_title: Some(tab.title.clone()),
                    kind: OutputItemKind::ReportTable(tab),
                });
            }
            if let Some(rep) = audit_report {
                items.push(OutputItem {
                    id: self.alloc_id(),
                    title_key: "dialog-style-report-title",
                    custom_title: None,
                    kind: OutputItemKind::ReportTable(rep),
                });
            }
        } else {
            items.push(OutputItem {
                id: self.alloc_id(),
                title_key: "output-crosstabs-title",
                custom_title: None,
                kind: OutputItemKind::Crosstabs(results),
            });
        }

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-crosstabs-title",
            custom_title: None,
            dataset_name: Some(self.active_dataset_name.clone()),
            is_expanded: true,
            items,
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        let target_item_idx = if items_len_gt_2(&self.procedures[new_proc_idx].items) {
            2
        } else {
            0
        };
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: target_item_idx,
        });
        self.scroll_to_target = self.selected_target;
    }
}

fn items_len_gt_2(items: &[OutputItem]) -> bool {
    items.len() > 2
}

pub fn render_crosstabs_item(ui: &mut Ui, results: &[CrosstabResult], selected: bool) {
    for (res_idx, res) in results.iter().enumerate() {
        if res_idx > 0 {
            ui.add_space(16.0);
        }

        matrix::render_case_processing_summary(ui, res, selected);
        ui.add_space(12.0);

        if res.options.suppress_tables {
            ui.label(
                egui::RichText::new(t!("output-crosstabs-suppressed-note"))
                    .italics()
                    .font(FontId::proportional(11.5))
                    .color(Tokens::text_muted()),
            );
            ui.add_space(8.0);
        } else {
            matrix::render_crosstab_matrix(ui, res, selected);
            ui.add_space(12.0);
        }

        stats_tables::render_all_stats_tables(ui, res, selected);

        if res.options.display_clustered_bar_charts {
            ui.add_space(12.0);
            charts::render_clustered_bar_chart(ui, res, selected);
        }
    }
}

fn apply_crosstabs_styles(
    results: &[CrosstabResult],
    rules: Option<&[TableStyleRule]>,
    create_style_report: bool,
) -> (Vec<ReportTable>, Option<ReportTable>) {
    let mut styled_tables = Vec::new();
    let mut audit_rows = Vec::new();

    for res in results {
        for mut table in crosstabs_to_report_tables(res) {
            if let Some(rule_list) = rules {
                for rule in rule_list {
                    if !ConditionalFormattingEngine::matches_table(&rule.table_target, "Crosstabs")
                    {
                        continue;
                    }
                    let modified = ConditionalFormattingEngine::apply_single_rule(&mut table, rule);
                    if create_style_report {
                        let target_str = match &rule.value_target {
                            TargetValue::AllDataCells => t!("dialog-table-style-all-cells"),
                            TargetValue::Label(lbl) => lbl.clone(),
                        };
                        audit_rows.push(vec![
                            table.title.clone(),
                            target_str,
                            format_condition_summary(&rule.condition),
                            format_style_summary(&rule.format),
                            format!("{}", modified),
                        ]);
                    }
                }
            }
            styled_tables.push(table);
        }
    }

    let report_table = if create_style_report && !audit_rows.is_empty() {
        let mut report = ReportTable::new(t!("dialog-style-report-title"));
        report.headers = vec![
            t!("dialog-style-report-col-table"),
            t!("dialog-style-report-col-target"),
            t!("dialog-style-report-col-condition"),
            t!("dialog-style-report-col-format"),
            t!("dialog-style-report-col-modified"),
        ];
        report.alignments = vec![
            CellAlignment::Left,
            CellAlignment::Left,
            CellAlignment::Left,
            CellAlignment::Left,
            CellAlignment::Right,
        ];
        report.rows = audit_rows;
        Some(report)
    } else {
        None
    };

    (styled_tables, report_table)
}

fn build_bootstrap_specs_report_table(opts: &BootstrapOptions) -> ReportTable {
    let mut table = ReportTable::new(t!("output-bootstrap-specs-title"));
    table.headers = vec![
        t!("output-bootstrap-col-statistic"),
        t!("output-crosstabs-value"),
    ];
    table.alignments = vec![CellAlignment::Left, CellAlignment::Right];
    let sampling_str = match &opts.sampling_method {
        BootstrapSamplingMethod::Simple => t!("output-bootstrap-sampling-simple"),
        BootstrapSamplingMethod::Stratified { strata_vars } => {
            let joined = strata_vars.join(", ");
            t!(
                "output-bootstrap-sampling-stratified",
                vars = joined.as_str()
            )
        }
    };
    let ci_method_str = match opts.ci_method {
        statcraft_algo::BootstrapCiMethod::Percentile => t!("output-bootstrap-ci-percentile"),
        statcraft_algo::BootstrapCiMethod::Bca => t!("output-bootstrap-ci-bca"),
    };
    table.rows = vec![
        vec![t!("output-bootstrap-spec-sampling"), sampling_str],
        vec![
            t!("output-bootstrap-spec-samples"),
            opts.samples.to_string(),
        ],
        vec![
            t!("output-bootstrap-spec-ci-level"),
            format!("{:.1}%", opts.ci_level),
        ],
        vec![t!("output-bootstrap-spec-ci-type"), ci_method_str],
    ];
    table
}
