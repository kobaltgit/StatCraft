use crate::theme::Tokens;
use egui::Color32;
use statcraft_i18n::t;

#[derive(Debug, Clone)]
pub enum StatusMessage {
    Ready,
    DatasetImported {
        name: String,
        rows: usize,
        cols: usize,
    },
    DescriptivesExecuted {
        count: usize,
    },
    FrequenciesExecuted {
        count: usize,
    },
    TTestExecuted {
        count: usize,
    },
    IndepTTestExecuted {
        count: usize,
    },
    PairedTTestExecuted {
        count: usize,
    },
    AnovaExecuted {
        count: usize,
    },
    CrosstabsExecuted {
        count: usize,
    },
    CorrelationsExecuted {
        count: usize,
    },
    MannWhitneyExecuted {
        count: usize,
    },
    KruskalWallisExecuted {
        count: usize,
    },
    RelatedSamplesExecuted {
        count: usize,
    },
    ComputeExecuted {
        target: String,
        rows: usize,
    },
    CountValuesExecuted {
        target: String,
        variables: usize,
        rows: usize,
    },
    ShiftValuesExecuted {
        target: String,
        rows: usize,
    },
    DateTimeExecuted {
        target: String,
    },
    RecodeExecuted {
        count: usize,
    },
    AutoRecodeExecuted {
        count: usize,
    },
    VisualBinningExecuted {
        target: String,
        rows: usize,
    },
    OptimalBinningExecuted {
        count: usize,
    },
    CreateDummiesExecuted {
        count: usize,
    },
    ReplaceMissingExecuted {
        count: usize,
    },
    RandomSeedExecuted {
        seed: Option<u64>,
    },
    SortCasesExecuted {
        count: usize,
    },
    SelectCasesExecuted {
        mode: String,
    },
    SplitFileExecuted {
        mode: String,
        count: usize,
    },
    WeightCasesExecuted {
        var: Option<String>,
    },
    RegressionExecuted {
        dep_var: String,
        indep_count: usize,
    },
    ReliabilityExecuted {
        scale: String,
        count: usize,
    },
    FactorExecuted {
        count: usize,
        factors: usize,
    },
    MeansExecuted {
        tables: usize,
        cases: usize,
    },
    NewDatasetCreated,
    NewSyntaxCreated,
    NewProjectCreated,
    ProjectSaved {
        path: String,
    },
    ProjectOpened {
        name: String,
        rows: usize,
        cols: usize,
    },
    DataSaved {
        path: String,
    },
    DataExported {
        path: String,
    },
    OutputExported {
        path: String,
    },
    Error(String),
}

impl StatusMessage {
    pub fn text(&self) -> String {
        match self {
            StatusMessage::Ready => t!("status-ready"),
            StatusMessage::NewDatasetCreated => t!("status-new-dataset-created"),
            StatusMessage::NewSyntaxCreated => t!("status-new-syntax-created"),
            StatusMessage::NewProjectCreated => t!("status-new-project-created"),
            StatusMessage::ProjectSaved { path } => {
                t!("status-project-saved", path = path.as_str())
            }
            StatusMessage::ProjectOpened { name, rows, cols } => {
                t!(
                    "status-project-opened",
                    name = name.as_str(),
                    rows = *rows,
                    cols = *cols
                )
            }
            StatusMessage::DataSaved { path } => {
                t!("status-data-saved", path = path.as_str())
            }
            StatusMessage::DataExported { path } => {
                t!("status-data-exported", path = path.as_str())
            }
            StatusMessage::OutputExported { path } => {
                t!("status-output-exported", path = path.as_str())
            }
            StatusMessage::DatasetImported { name, rows, cols } => {
                t!(
                    "output-dataset-imported",
                    name = name.as_str(),
                    rows = *rows,
                    cols = *cols
                )
            }
            StatusMessage::DescriptivesExecuted { count } => {
                t!("output-descriptives-executed", count = *count)
            }
            StatusMessage::FrequenciesExecuted { count } => {
                t!("output-frequencies-executed", count = *count)
            }
            StatusMessage::TTestExecuted { count } => {
                t!("output-ttest-executed", count = *count)
            }
            StatusMessage::IndepTTestExecuted { count } => {
                t!("output-ttest-indep-executed", count = *count)
            }
            StatusMessage::PairedTTestExecuted { count } => {
                t!("output-ttest-paired-executed", count = *count)
            }
            StatusMessage::AnovaExecuted { count } => {
                t!("output-anova-executed", count = *count)
            }
            StatusMessage::CrosstabsExecuted { count } => {
                t!("output-crosstabs-executed", count = *count)
            }
            StatusMessage::CorrelationsExecuted { count } => {
                t!("output-correlations-executed", count = *count)
            }
            StatusMessage::MannWhitneyExecuted { count } => {
                t!("output-npar-mann-whitney-executed", count = *count)
            }
            StatusMessage::KruskalWallisExecuted { count } => {
                t!("output-npar-kruskal-executed", count = *count)
            }
            StatusMessage::RelatedSamplesExecuted { count } => {
                t!("output-npar-related-executed", count = *count)
            }
            StatusMessage::ComputeExecuted { target, rows } => {
                t!(
                    "output-compute-executed",
                    target = target.as_str(),
                    rows = *rows
                )
            }
            StatusMessage::CountValuesExecuted {
                target,
                variables,
                rows,
            } => {
                t!(
                    "output-count-executed",
                    target = target.as_str(),
                    variables = *variables,
                    rows = *rows
                )
            }
            StatusMessage::ShiftValuesExecuted { target, rows } => {
                t!(
                    "output-shift-executed",
                    target = target.as_str(),
                    rows = *rows
                )
            }
            StatusMessage::DateTimeExecuted { target } => {
                t!("status-datetime-executed", target = target.as_str())
            }
            StatusMessage::RecodeExecuted { count } => {
                t!("output-recode-executed", count = *count)
            }
            StatusMessage::AutoRecodeExecuted { count } => {
                t!("output-autorecode-executed", count = *count)
            }
            StatusMessage::VisualBinningExecuted { target, rows } => {
                t!(
                    "output-binning-executed",
                    target = target.as_str(),
                    rows = *rows
                )
            }
            StatusMessage::OptimalBinningExecuted { count } => {
                t!("status-optbin-executed", count = *count)
            }
            StatusMessage::CreateDummiesExecuted { count } => {
                t!("status-dummies-executed", count = *count)
            }
            StatusMessage::ReplaceMissingExecuted { count } => {
                t!("status-missing-executed", count = *count)
            }
            StatusMessage::RandomSeedExecuted { seed } => match seed {
                Some(s) => t!("status-random-seed-fixed", seed = *s),
                None => t!("status-random-seed-random"),
            },
            StatusMessage::SortCasesExecuted { count } => {
                t!("output-sort-cases-executed", count = *count)
            }
            StatusMessage::SelectCasesExecuted { mode } => {
                t!("output-select-cases-executed", mode = mode.as_str())
            }
            StatusMessage::SplitFileExecuted { mode, count } => {
                t!(
                    "output-split-file-executed",
                    mode = mode.as_str(),
                    count = *count
                )
            }
            StatusMessage::WeightCasesExecuted { var } => match var {
                Some(v) => t!("status-weight-executed", var = v.as_str()),
                None => t!("status-weight-off-executed"),
            },
            StatusMessage::RegressionExecuted {
                dep_var,
                indep_count,
            } => {
                t!(
                    "output-regression-executed",
                    dep = dep_var.as_str(),
                    count = *indep_count
                )
            }
            StatusMessage::ReliabilityExecuted { scale, count } => {
                t!(
                    "output-reliability-executed",
                    scale = scale.as_str(),
                    count = *count
                )
            }
            StatusMessage::FactorExecuted { count, factors } => {
                t!("output-factor-executed", count = *count, factors = *factors)
            }
            StatusMessage::MeansExecuted { tables, cases } => {
                t!("output-means-executed", tables = *tables, cases = *cases)
            }
            StatusMessage::Error(e) => format!("{}: {}", t!("status-error"), e),
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            StatusMessage::ProjectSaved { .. }
            | StatusMessage::DataSaved { .. }
            | StatusMessage::DataExported { .. }
            | StatusMessage::OutputExported { .. } => Tokens::MEASURE_NOMINAL,
            StatusMessage::Error(_) => Color32::from_rgb(235, 80, 80),
            _ => Tokens::TEXT_MUTED,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_message_colors() {
        assert_eq!(
            StatusMessage::ProjectSaved {
                path: "test.stc".to_string()
            }
            .color(),
            Tokens::MEASURE_NOMINAL
        );
        assert_eq!(
            StatusMessage::DataSaved {
                path: "test.csv".to_string()
            }
            .color(),
            Tokens::MEASURE_NOMINAL
        );
        assert_eq!(
            StatusMessage::Error("failed".to_string()).color(),
            Color32::from_rgb(235, 80, 80)
        );
        assert_eq!(StatusMessage::Ready.color(), Tokens::TEXT_MUTED);
    }
}
