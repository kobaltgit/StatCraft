use crate::dialogs::table_style_dialog::{format_condition_summary, format_style_summary};
use statcraft_algo::FrequencyResult;
use statcraft_i18n::t;
use statcraft_reporting::model::{CellAlignment, ReportTable};
use statcraft_reporting::styling::{ConditionalFormattingEngine, TableStyleRule, TargetValue};

pub fn frequency_result_to_report_table(res: &FrequencyResult) -> ReportTable {
    let mut table = ReportTable::new(res.display_title());
    table.headers = vec![
        t!("output-col-value"),
        t!("output-col-frequency"),
        t!("output-col-percent"),
        t!("output-col-valid-percent"),
        t!("output-col-cum-percent"),
    ];
    table.alignments = vec![
        CellAlignment::Left,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
        CellAlignment::Right,
    ];

    let n_total = res.n_valid + res.n_missing;
    let valid_percent_total = if n_total > 0 {
        (res.n_valid as f64 / n_total as f64) * 100.0
    } else {
        0.0
    };
    let missing_percent = if n_total > 0 {
        (res.n_missing as f64 / n_total as f64) * 100.0
    } else {
        0.0
    };

    for cat in &res.categories {
        table.rows.push(vec![
            cat.display_label().to_string(),
            format!("{}", cat.count),
            format!("{:.1}", cat.percent),
            cat.valid_percent
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "-".to_string()),
            cat.cumulative_percent
                .map(|v| format!("{:.1}", v))
                .unwrap_or_else(|| "-".to_string()),
        ]);
    }

    table.rows.push(vec![
        t!("output-row-total-valid"),
        format!("{}", res.n_valid),
        format!("{:.1}", valid_percent_total),
        "100.0".to_string(),
        String::new(),
    ]);

    if res.n_missing > 0 {
        table.rows.push(vec![
            t!("output-row-system-missing"),
            format!("{}", res.n_missing),
            format!("{:.1}", missing_percent),
            String::new(),
            String::new(),
        ]);
    }

    table.rows.push(vec![
        t!("output-row-total"),
        format!("{}", n_total),
        "100.0".to_string(),
        String::new(),
        String::new(),
    ]);

    table
}

pub fn build_frequencies_styled_tables(
    results: &[FrequencyResult],
    rules: Option<&[TableStyleRule]>,
    create_style_report: bool,
) -> (Vec<ReportTable>, Option<ReportTable>) {
    let mut styled_tables = Vec::new();
    let mut audit_rows = Vec::new();

    for res in results {
        if res.categories.is_empty() && !res.table_suppressed {
            continue;
        }

        let mut table = frequency_result_to_report_table(res);

        if let Some(rule_list) = rules {
            for rule in rule_list {
                if !ConditionalFormattingEngine::matches_table(&rule.table_target, "Frequencies") {
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

pub fn build_frequencies_procedure(
    state: &mut crate::views::output_view::model::OutputViewerState,
    results: Vec<FrequencyResult>,
    sps_log: String,
    table_styles: Option<&[TableStyleRule]>,
    create_style_report: bool,
) {
    use crate::views::output_view::model::{
        OutputItem, OutputItemKind, OutputProcedure, SelectedTarget,
    };

    let has_rules = table_styles.map(|r| !r.is_empty()).unwrap_or(false);
    if !has_rules && !create_style_report {
        state.add_frequencies_with_log(results, sps_log);
        return;
    }

    let proc_id = state.alloc_id();
    let log_id = state.alloc_id();
    let notes_id = state.alloc_id();

    let notes = state.make_notes(
        results.first().map(|r| r.n_valid).unwrap_or(0),
        results.len(),
    );

    let charts: Vec<statcraft_algo::FrequencyChartResult> =
        results.iter().filter_map(|r| r.chart.clone()).collect();
    let mode = results
        .first()
        .map(|r| r.multiple_vars_mode)
        .unwrap_or(statcraft_algo::MultipleVarsMode::Compare);

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
            kind: OutputItemKind::Notes(notes.clone()),
        },
    ];

    let (styled_tables, report_table) =
        build_frequencies_styled_tables(&results, table_styles, create_style_report);

    for tab in styled_tables {
        items.push(OutputItem {
            id: state.alloc_id(),
            title_key: "output-frequencies-table",
            custom_title: Some(tab.title.clone()),
            kind: OutputItemKind::ReportTable(tab),
        });
    }

    if let Some(rep) = report_table {
        items.push(OutputItem {
            id: state.alloc_id(),
            title_key: "dialog-style-report-title",
            custom_title: None,
            kind: OutputItemKind::ReportTable(rep),
        });
    }

    if mode == statcraft_algo::MultipleVarsMode::Compare && !charts.is_empty() {
        items.push(OutputItem {
            id: state.alloc_id(),
            title_key: "output-frequencies-charts",
            custom_title: None,
            kind: OutputItemKind::FrequencyCharts(charts),
        });
    }

    let proc = OutputProcedure {
        id: proc_id,
        title_key: "output-frequencies",
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
