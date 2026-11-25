use super::super::model::{
    OutputItem, OutputItemKind, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::{active_columns, compute_listwise_n};
use crate::dialogs::table_style_dialog::{format_condition_summary, format_style_summary};
use statcraft_algo::DescriptivesResult;
use statcraft_i18n::t;
use statcraft_reporting::model::{CellAlignment, ReportTable};
use statcraft_reporting::styling::{ConditionalFormattingEngine, TableStyleRule, TargetValue};

pub fn descriptives_to_report_table(results: &[DescriptivesResult]) -> ReportTable {
    let mut table = ReportTable::new(t!("output-descriptives-title"));
    let active = active_columns(results);

    table.headers = active.iter().map(|c| c.header()).collect();
    table.alignments = active
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if i == 0 {
                CellAlignment::Left
            } else {
                CellAlignment::Right
            }
        })
        .collect();

    for res in results {
        table
            .rows
            .push(active.iter().map(|c| c.format_cell(res, ".")).collect());
    }

    let listwise_n = compute_listwise_n(results);
    let mut summary_row = vec![String::new(); active.len()];
    if !summary_row.is_empty() {
        summary_row[0] = t!("output-valid-n-listwise");
    }
    if summary_row.len() > 1 {
        summary_row[1] = listwise_n.to_string();
    }
    table.rows.push(summary_row);

    table
}

pub fn build_descriptives_styled_table(
    results: &[DescriptivesResult],
    rules: Option<&[TableStyleRule]>,
    create_style_report: bool,
) -> (ReportTable, Option<ReportTable>) {
    let mut table = descriptives_to_report_table(results);
    let mut audit_rows = Vec::new();

    if let Some(rule_list) = rules {
        for rule in rule_list {
            if !ConditionalFormattingEngine::matches_table(&rule.table_target, "Descriptives") {
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

    (table, report_table)
}

pub fn build_descriptives_procedure(
    state: &mut OutputViewerState,
    results: Vec<DescriptivesResult>,
    sps_log: String,
    table_styles: Option<&[TableStyleRule]>,
    create_style_report: bool,
) {
    let has_rules = table_styles.map(|r| !r.is_empty()).unwrap_or(false);
    let notes = state.make_notes(
        results.first().map(|r| r.n_valid).unwrap_or(0),
        results.len(),
    );

    if !has_rules && !create_style_report {
        state.push_standard_procedure(
            "output-descriptives-title",
            sps_log,
            notes,
            "output-descriptives-table",
            OutputItemKind::DescriptivesTable(results),
        );
        return;
    }

    let mut items = vec![
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-log",
            custom_title: None,
            kind: OutputItemKind::Log(sps_log),
        },
        OutputItem {
            id: state.alloc_id(),
            title_key: "output-notes",
            custom_title: None,
            kind: OutputItemKind::Notes(notes.clone()),
        },
    ];

    let (table, report_table) =
        build_descriptives_styled_table(&results, table_styles, create_style_report);
    items.push(OutputItem {
        id: state.alloc_id(),
        title_key: "output-descriptives-table",
        custom_title: Some(table.title.clone()),
        kind: OutputItemKind::ReportTable(table),
    });

    if let Some(rep) = report_table {
        items.push(OutputItem {
            id: state.alloc_id(),
            title_key: "dialog-style-report-title",
            custom_title: None,
            kind: OutputItemKind::ReportTable(rep),
        });
    }

    let proc = OutputProcedure {
        id: state.alloc_id(),
        title_key: "output-descriptives-title",
        custom_title: None,
        dataset_name: Some(notes.dataset_name.clone()),
        is_expanded: true,
        items,
    };

    state.procedures.push(proc);
    let new_proc_idx = state.procedures.len() - 1;
    state.selected_target = Some(SelectedTarget::Item {
        proc_idx: new_proc_idx,
        item_idx: 2,
    });
    state.scroll_to_target = state.selected_target;
}
