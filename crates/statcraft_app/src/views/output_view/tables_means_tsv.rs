use super::tables_common::{
    bootstrap_specs_to_tsv, bootstrap_table_to_tsv, format_opt, format_sig,
};
use super::tables_means::build_means_bootstrap_rows;
use statcraft_algo::MeansProcedureResult;
use statcraft_i18n::t;

pub fn means_to_tsv(res: &MeansProcedureResult) -> String {
    let mut lines = Vec::new();
    if let Some(opts) = res
        .tables
        .iter()
        .find_map(|t| t.bootstrap.as_ref().map(|b| &b.options))
    {
        lines.push(bootstrap_specs_to_tsv(opts));
        lines.push(String::new());
    }

    for table in &res.tables {
        lines.push(format!(
            "Case Processing Summary ({} * {})",
            table.dep_var, table.factor_var
        ));
        lines.push(
            "Pair\tIncluded N\tIncluded %\tExcluded N\tExcluded %\tTotal N\tTotal %".to_string(),
        );
        lines.push(format!(
            "{} * {}\t{}\t{:.1}%\t{}\t{:.1}%\t{}\t{:.1}%",
            table.dep_var,
            table.factor_var,
            table.summary.included_n,
            table.summary.included_percent,
            table.summary.excluded_n,
            table.summary.excluded_percent,
            table.summary.total_n,
            table.summary.total_percent
        ));
        lines.push(String::new());

        lines.push(format!("Report ({})", table.dep_var));
        lines.push(format!(
            "{}\tMean\tN\tStd. Deviation\tMedian",
            table.factor_var
        ));
        for cell in &table.cells {
            lines.push(format!(
                "{}\t{}\t{}\t{}\t{}",
                cell.group_label,
                format_opt(cell.mean),
                cell.count,
                format_opt(cell.std_dev),
                format_opt(cell.median)
            ));
        }
        lines.push(format!(
            "Total\t{}\t{}\t{}\t{}",
            format_opt(table.total.mean),
            table.total.count,
            format_opt(table.total.std_dev),
            format_opt(table.total.median)
        ));
        lines.push(String::new());

        if let Some((headers, rows)) = build_means_bootstrap_rows(table) {
            let title = format!(
                "{} ({} - {})",
                t!("output-means-report-title"),
                table.dep_var,
                t!("output-bootstrap-group-header")
            );
            lines.push(bootstrap_table_to_tsv(&title, &headers, &rows));
            lines.push(String::new());
        }

        if let Some(ref anova) = table.anova {
            lines.push(format!(
                "ANOVA Table ({} * {})",
                table.dep_var, table.factor_var
            ));
            lines.push("Source\tSum of Squares\tdf\tMean Square\tF\tSig.".to_string());
            lines.push(format!(
                "Between Groups\t{:.3}\t{}\t{}\t{}\t{}",
                anova.between_groups.sum_of_squares,
                anova.between_groups.df,
                format_opt(anova.between_groups.mean_square),
                format_opt(anova.between_groups.f_stat),
                format_sig(anova.between_groups.p_value)
            ));
            lines.push(format!(
                "Within Groups\t{:.3}\t{}\t{}\t\t",
                anova.within_groups.sum_of_squares,
                anova.within_groups.df,
                format_opt(anova.within_groups.mean_square)
            ));
            lines.push(format!(
                "Total\t{:.3}\t{}\t\t\t",
                anova.total.sum_of_squares, anova.total.df
            ));
            lines.push(String::new());
        }

        if let Some(ref assoc) = table.association {
            lines.push(format!(
                "Measures of Association ({} * {})",
                table.dep_var, table.factor_var
            ));
            lines.push("Pair\tEta\tEta Squared".to_string());
            lines.push(format!(
                "{} * {}\t{:.3}\t{:.3}",
                table.dep_var, table.factor_var, assoc.eta, assoc.eta_squared
            ));
            lines.push(String::new());
        }
    }

    lines.join("\n")
}
