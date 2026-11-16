pub mod blocks;
pub mod builder;
pub mod export;
pub mod model;
pub mod outline;
pub mod tables_adp;
pub mod tables_anova;
pub mod tables_cluster;
pub mod tables_common;
pub mod tables_correlations;
pub mod tables_crosstabs;
pub mod tables_curvefit;
pub mod tables_descriptives;
pub mod tables_explore;
pub mod tables_factor;
pub mod tables_frequencies;
pub mod tables_generic;
pub mod tables_glm;
pub mod tables_kruskal;
pub mod tables_logistic;
pub mod tables_means;
pub mod tables_multinomial_logistic;
pub mod tables_nonparam;
pub mod tables_nonparam_related;
pub mod tables_partial_corr;
pub mod tables_programmability;
pub mod tables_regression;
pub mod tables_reliability;
pub mod tables_reports;
pub mod tables_survival;
pub mod tables_ttest_indep;
pub mod tables_ttest_onesample;
pub mod tables_ttest_paired;
pub mod view;

pub use blocks::{render_item_context_menu, render_log_block, render_notes_block};
pub use export::{
    export_output_to_html, export_output_to_tsv, export_output_to_txt, export_procedures_to_txt,
    output_viewer_to_report_document,
};
pub use model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
pub use outline::render_outline_pane;
pub use tables_adp::{adp_to_tsv, render_spss_adp_tables};
pub use tables_anova::{anova_to_tsv, render_spss_oneway_anova_tables};
pub use tables_cluster::{kmeans_to_tsv, render_spss_kmeans_tables};
pub use tables_correlations::{correlations_to_tsv, render_spss_correlations_tables};
pub use tables_crosstabs::{crosstabs_to_tsv, render_crosstabs_item};
pub use tables_curvefit::{curvefit_to_tsv, render_spss_curvefit_tables};
pub use tables_descriptives::{descriptives_to_tsv, render_spss_descriptives_pivot_table};
pub use tables_explore::{explore_to_tsv, render_spss_explore_pivot_tables};
pub use tables_factor::{factor_to_tsv, render_spss_factor};
pub use tables_frequencies::{frequencies_to_tsv, render_spss_frequencies_tables};
pub use tables_generic::{generic_report_to_tsv, render_spss_generic_report};
pub use tables_glm::*;
pub use tables_kruskal::{kruskal_wallis_to_tsv, render_spss_kruskal_wallis_tables};
pub use tables_logistic::{logistic_to_tsv, render_spss_logistic_tables};
pub use tables_means::{means_to_tsv, render_spss_means};
pub use tables_multinomial_logistic::{
    multinomial_logistic_to_tsv, render_spss_multinomial_logistic_tables,
};
pub use tables_nonparam::{mann_whitney_to_tsv, render_spss_mann_whitney_tables};
pub use tables_nonparam_related::{related_samples_to_tsv, render_spss_related_samples_tables};
pub use tables_partial_corr::{partial_corr_to_tsv, render_spss_partial_corr_tables};
pub use tables_programmability::{programmability_to_tsv, render_spss_programmability_tables};
pub use tables_regression::{regression_to_tsv, render_spss_linear_regression};
pub use tables_reliability::{reliability_to_tsv, render_spss_reliability};
pub use tables_reports::{
    codebook_to_tsv, render_spss_codebook_tables, render_spss_summarize_tables, summarize_to_tsv,
};
pub use tables_survival::{
    lifetables_to_tsv, render_spss_lifetables, render_spss_survival_tables, survival_to_tsv,
};
pub use tables_ttest_indep::{indep_ttest_to_tsv, render_spss_indep_ttest_tables};
pub use tables_ttest_onesample::{onesample_ttest_to_tsv, render_spss_onesample_ttest_tables};
pub use tables_ttest_paired::{paired_ttest_to_tsv, render_spss_paired_ttest_tables};
pub use view::render_output_view;

pub fn extract_item_tsv(kind: &OutputItemKind) -> Option<String> {
    match kind {
        OutputItemKind::DescriptivesTable(res) => Some(descriptives_to_tsv(res)),
        OutputItemKind::FrequenciesTables(res) => Some(frequencies_to_tsv(res)),
        OutputItemKind::OneSampleTTest { results, .. } => Some(onesample_ttest_to_tsv(results)),
        OutputItemKind::IndepTTest { results, .. } => Some(indep_ttest_to_tsv(results)),
        OutputItemKind::PairedTTest { results, .. } => Some(paired_ttest_to_tsv(results)),
        OutputItemKind::OneWayAnova { results, .. } => Some(anova_to_tsv(results)),
        OutputItemKind::Crosstabs(results) => results.first().map(crosstabs_to_tsv),
        OutputItemKind::Correlations(res) => Some(correlations_to_tsv(res)),
        OutputItemKind::MannWhitney {
            results,
            grouping_var,
        } => Some(mann_whitney_to_tsv(results, grouping_var)),
        OutputItemKind::KruskalWallis {
            results,
            grouping_var,
        } => Some(kruskal_wallis_to_tsv(results, grouping_var)),
        OutputItemKind::TwoRelatedSamples(results) => Some(related_samples_to_tsv(results)),
        OutputItemKind::LinearRegression(res) => Some(regression_to_tsv(res)),
        OutputItemKind::PartialCorrelations(res) => Some(partial_corr_to_tsv(res)),
        OutputItemKind::LogisticRegression(res) => Some(logistic_to_tsv(res)),
        OutputItemKind::MultinomialLogistic(res) => Some(multinomial_logistic_to_tsv(res)),
        OutputItemKind::CurveFit(res) => Some(curvefit_to_tsv(res)),
        OutputItemKind::UnivariateGlm(res) => Some(univariate_glm_to_tsv(res)),
        OutputItemKind::MultivariateGlm(res) => Some(multivariate_glm_to_tsv(res)),
        OutputItemKind::RepeatedGlm(res) => Some(repeated_glm_to_tsv(res)),
        OutputItemKind::KMeans(res) => Some(kmeans_to_tsv(res)),
        OutputItemKind::Codebook(res) => Some(codebook_to_tsv(res)),
        OutputItemKind::Summarize(res) => Some(summarize_to_tsv(res)),
        OutputItemKind::KaplanMeier(res) => Some(survival_to_tsv(res)),
        OutputItemKind::LifeTables(res) => Some(lifetables_to_tsv(res)),
        OutputItemKind::Reliability(res) => Some(reliability_to_tsv(res)),
        OutputItemKind::Factor(res) => Some(factor_to_tsv(res)),
        OutputItemKind::Means(res) => Some(means_to_tsv(res)),
        OutputItemKind::Explore(res) => Some(explore_to_tsv(res)),
        OutputItemKind::Adp(res) => Some(adp_to_tsv(res)),
        OutputItemKind::Programmability(res) => Some(programmability_to_tsv(res)),
        OutputItemKind::GenericReport {
            command_id,
            status,
            summary,
            key_values,
            table_headers,
            table_rows,
        } => Some(generic_report_to_tsv(
            command_id,
            status,
            summary,
            key_values,
            table_headers,
            table_rows,
        )),
        _ => None,
    }
}
