use super::{display_var_name, TurfOutputItem};
use statcraft_i18n::t;
use statcraft_reporting::model::{CellAlignment, ReportTable};

pub fn turf_to_tsv(item: &TurfOutputItem) -> String {
    let mut out = String::new();
    let res = &item.result;
    let opts = &item.options;

    // 1. Summary table
    out.push_str(&t!("output-turf-summary-title"));
    out.push('\n');
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-col-param"),
        t!("output-turf-col-value")
    ));
    out.push_str(&format!(
        "{}\t{:.0}\n",
        t!("output-turf-param-valid-n"),
        res.total_valid_cases
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-param-positive-vals"),
        opts.positive_values.join(", ")
    ));

    let excl_str = if res.excluded_variables.is_empty() {
        t!("output-turf-param-none")
    } else {
        res.excluded_variables
            .iter()
            .map(|(v, p)| format!("{} ({p:.1}%)", display_var_name(item, v)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-param-excluded-vars"),
        excl_str
    ));

    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-param-est-ops"),
        res.cycles
            .first()
            .map(|c| c.estimated_operations.to_string())
            .unwrap_or_else(|| "0".to_string())
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-param-advanced-heats"),
        if opts.use_advanced_selection {
            t!(
                "output-turf-heats-detail",
                yes = t!("output-turf-param-yes"),
                size = opts.advanced_size,
                winners = opts.advanced_winners
            )
        } else {
            t!("output-turf-param-no")
        }
    ));
    out.push_str(&format!(
        "{}\t{}\n",
        t!("output-turf-param-always-required"),
        opts.always_required
    ));
    out.push_str(&format!(
        "{}\t{}\n\n",
        t!("output-turf-param-removal-cycles"),
        opts.removal_cycles
    ));

    // 2. Reach & Frequency tables per cycle
    let has_weights = !opts.importance_weights.is_empty();
    for cycle in &res.cycles {
        if cycle.cycle_index > 0 {
            out.push_str(&format!(
                "[{}: {}]\n",
                t!("output-turf-reach-freq-cycle", cycle = cycle.cycle_index),
                cycle
                    .removed_variables
                    .iter()
                    .map(|v| display_var_name(item, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        } else {
            out.push_str(&t!("output-turf-reach-freq-title"));
            out.push('\n');
        }

        if has_weights {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                t!("output-turf-col-combo-size"),
                t!("output-turf-col-variables"),
                t!("output-turf-col-reach"),
                t!("output-turf-col-reach-pct"),
                t!("output-turf-col-frequency"),
                t!("output-turf-col-frequency-pct"),
                t!("output-turf-col-mean-responses"),
                t!("output-turf-col-score")
            ));
        } else {
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                t!("output-turf-col-combo-size"),
                t!("output-turf-col-variables"),
                t!("output-turf-col-reach"),
                t!("output-turf-col-reach-pct"),
                t!("output-turf-col-frequency"),
                t!("output-turf-col-frequency-pct"),
                t!("output-turf-col-mean-responses")
            ));
        }

        for (size, rows) in &cycle.rows_by_size {
            for row in rows {
                out.push_str(&format!(
                    "{}\t{}\t{:.0}\t{:.1}%\t{:.0}\t{:.1}%\t{:.2}",
                    size,
                    row.variables
                        .iter()
                        .map(|v| display_var_name(item, v))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    row.reach_count,
                    row.reach_pct,
                    row.frequency_count,
                    row.frequency_pct,
                    row.mean_responses_per_reached
                ));
                if has_weights {
                    let score_str = row
                        .weighted_score
                        .map(|s| format!("{s:.4}"))
                        .unwrap_or_else(|| "-".to_string());
                    out.push_str(&format!("\t{score_str}"));
                }
                out.push('\n');
            }
        }
        out.push('\n');
    }

    out
}

pub fn turf_to_report_tables(item: &TurfOutputItem) -> Vec<ReportTable> {
    let mut tables = Vec::new();
    let res = &item.result;
    let opts = &item.options;

    // 1. Summary ReportTable
    let mut summary_table = ReportTable::new(t!("output-turf-summary-title"));
    summary_table.headers = vec![t!("output-turf-col-param"), t!("output-turf-col-value")];
    summary_table.alignments = vec![CellAlignment::Left, CellAlignment::Left];

    let excl_str = if res.excluded_variables.is_empty() {
        t!("output-turf-param-none")
    } else {
        res.excluded_variables
            .iter()
            .map(|(v, p)| format!("{} ({p:.1}%)", display_var_name(item, v)))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let heats_str = if opts.use_advanced_selection {
        t!(
            "output-turf-heats-detail",
            yes = t!("output-turf-param-yes"),
            size = opts.advanced_size,
            winners = opts.advanced_winners
        )
    } else {
        t!("output-turf-param-no")
    };
    let ops_str = res
        .cycles
        .first()
        .map(|c| c.estimated_operations.to_string())
        .unwrap_or_else(|| "0".to_string());

    summary_table.rows = vec![
        vec![
            t!("output-turf-param-valid-n"),
            format!("{:.0}", res.total_valid_cases),
        ],
        vec![
            t!("output-turf-param-positive-vals"),
            opts.positive_values.join(", "),
        ],
        vec![t!("output-turf-param-excluded-vars"), excl_str],
        vec![t!("output-turf-param-est-ops"), ops_str],
        vec![t!("output-turf-param-advanced-heats"), heats_str],
        vec![
            t!("output-turf-param-always-required"),
            opts.always_required.to_string(),
        ],
        vec![
            t!("output-turf-param-removal-cycles"),
            opts.removal_cycles.to_string(),
        ],
    ];
    tables.push(summary_table);

    // 2. Reach & Frequency ReportTables per cycle
    let has_weights = !opts.importance_weights.is_empty();
    for cycle in &res.cycles {
        let title = if cycle.cycle_index > 0 {
            let removed = cycle
                .removed_variables
                .iter()
                .map(|v| display_var_name(item, v))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "{} ({}: {})",
                t!("output-turf-reach-freq-title"),
                t!("output-turf-reach-freq-cycle", cycle = cycle.cycle_index),
                removed
            )
        } else {
            t!("output-turf-reach-freq-title")
        };

        let mut rf_table = ReportTable::new(title);
        if has_weights {
            rf_table.headers = vec![
                t!("output-turf-col-combo-size"),
                t!("output-turf-col-variables"),
                t!("output-turf-col-reach"),
                t!("output-turf-col-reach-pct"),
                t!("output-turf-col-frequency"),
                t!("output-turf-col-frequency-pct"),
                t!("output-turf-col-mean-responses"),
                t!("output-turf-col-score"),
            ];
            rf_table.alignments = vec![
                CellAlignment::Right,
                CellAlignment::Left,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
            ];
        } else {
            rf_table.headers = vec![
                t!("output-turf-col-combo-size"),
                t!("output-turf-col-variables"),
                t!("output-turf-col-reach"),
                t!("output-turf-col-reach-pct"),
                t!("output-turf-col-frequency"),
                t!("output-turf-col-frequency-pct"),
                t!("output-turf-col-mean-responses"),
            ];
            rf_table.alignments = vec![
                CellAlignment::Right,
                CellAlignment::Left,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
                CellAlignment::Right,
            ];
        }

        for (size, rows) in &cycle.rows_by_size {
            for row in rows {
                let mut r = vec![
                    size.to_string(),
                    row.variables
                        .iter()
                        .map(|v| display_var_name(item, v))
                        .collect::<Vec<_>>()
                        .join(" + "),
                    format!("{:.0}", row.reach_count),
                    format!("{:.1}%", row.reach_pct),
                    format!("{:.0}", row.frequency_count),
                    format!("{:.1}%", row.frequency_pct),
                    format!("{:.2}", row.mean_responses_per_reached),
                ];
                if has_weights {
                    let score_str = row
                        .weighted_score
                        .map(|s| format!("{s:.4}"))
                        .unwrap_or_else(|| "-".to_string());
                    r.push(score_str);
                }
                rf_table.rows.push(r);
            }
        }

        tables.push(rf_table);
    }

    tables
}
