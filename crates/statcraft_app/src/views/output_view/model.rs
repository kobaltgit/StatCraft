use serde::{Deserialize, Serialize};
use statcraft_algo::{
    BivariateCorrelationsResult, CrosstabResult, DescriptivesResult, FactorAnalysisResult,
    FrequencyResult, IndepTTestResult, KruskalWallisResult, LinearRegressionResult,
    MannWhitneyResult, MeansProcedureResult, OneSampleTTestResult, OneWayAnovaResult,
    PairedTTestResult, RelatedPairTestResult, ReliabilityResult,
};
use statcraft_i18n::t;

pub mod static_str_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(val: &&'static str, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_str(val)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<&'static str, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Ok(Box::leak(s.into_boxed_str()))
    }
}

pub type StaticStr = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTarget {
    Procedure(usize),
    Item { proc_idx: usize, item_idx: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputNotes {
    pub dataset_name: String,
    pub timestamp: String,
    pub active_cases: usize,
    pub active_vars: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputItemKind {
    Log(String),
    Notes(OutputNotes),
    DatasetSummary {
        name: String,
        rows: usize,
        cols: usize,
    },
    DescriptivesTable(Vec<DescriptivesResult>),
    FrequenciesTables(Vec<FrequencyResult>),
    OneSampleTTest {
        results: Vec<OneSampleTTestResult>,
        test_value: f64,
        ci: f64,
    },
    IndepTTest {
        results: Vec<IndepTTestResult>,
        grouping_var: String,
        group1: String,
        group2: String,
        ci: f64,
    },
    PairedTTest {
        results: Vec<PairedTTestResult>,
        pairs: Vec<(String, String)>,
        ci: f64,
    },
    OneWayAnova {
        results: Vec<OneWayAnovaResult>,
        ci: f64,
    },
    Crosstabs(Vec<CrosstabResult>),
    Correlations(BivariateCorrelationsResult),
    MannWhitney {
        results: Vec<MannWhitneyResult>,
        grouping_var: String,
    },
    KruskalWallis {
        results: Vec<KruskalWallisResult>,
        grouping_var: String,
    },
    TwoRelatedSamples(Vec<RelatedPairTestResult>),
    LinearRegression(Box<LinearRegressionResult>),
    Reliability(Box<ReliabilityResult>),
    Factor(Box<FactorAnalysisResult>),
    Means(Box<MeansProcedureResult>),
    Warning(String),
    CustomText {
        title: String,
        content: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputItem {
    pub id: usize,
    #[serde(with = "static_str_serde")]
    pub title_key: StaticStr,
    pub custom_title: Option<String>,
    pub kind: OutputItemKind,
}

impl OutputItem {
    pub fn title(&self) -> String {
        if let Some(custom) = &self.custom_title {
            custom.clone()
        } else {
            t!(self.title_key)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputProcedure {
    pub id: usize,
    #[serde(with = "static_str_serde")]
    pub title_key: StaticStr,
    pub custom_title: Option<String>,
    pub is_expanded: bool,
    pub items: Vec<OutputItem>,
}

impl OutputProcedure {
    pub fn title(&self) -> String {
        if let Some(custom) = &self.custom_title {
            custom.clone()
        } else {
            t!(self.title_key)
        }
    }
}

#[derive(Debug, Clone)]
pub struct OutputViewerState {
    pub procedures: Vec<OutputProcedure>,
    pub next_id: usize,
    pub selected_target: Option<SelectedTarget>,
    pub scroll_to_target: Option<SelectedTarget>,
    pub last_copied_id: Option<usize>,
}

impl Default for OutputViewerState {
    fn default() -> Self {
        Self {
            procedures: Vec::new(),
            next_id: 1,
            selected_target: None,
            scroll_to_target: None,
            last_copied_id: None,
        }
    }
}

impl OutputViewerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn is_empty(&self) -> bool {
        self.procedures.is_empty()
    }

    pub fn clear(&mut self) {
        self.procedures.clear();
        self.selected_target = None;
        self.scroll_to_target = None;
        self.last_copied_id = None;
    }

    pub fn add_log(&mut self, log_message: impl Into<String>) {
        let msg = log_message.into();
        let proc_id = self.alloc_id();
        let item_id = self.alloc_id();

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-log",
            custom_title: None,
            is_expanded: true,
            items: vec![OutputItem {
                id: item_id,
                title_key: "output-log",
                custom_title: None,
                kind: OutputItemKind::Log(msg),
            }],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 0,
        });
        self.scroll_to_target = self.selected_target;
    }

    pub fn expand_all(&mut self) {
        for proc in &mut self.procedures {
            proc.is_expanded = true;
        }
    }

    pub fn collapse_all(&mut self) {
        for proc in &mut self.procedures {
            proc.is_expanded = false;
        }
    }

    pub fn delete_selected(&mut self) {
        if let Some(target) = self.selected_target {
            match target {
                SelectedTarget::Procedure(p_idx) => {
                    if p_idx < self.procedures.len() {
                        self.procedures.remove(p_idx);
                        self.selected_target = None;
                        self.scroll_to_target = None;
                    }
                }
                SelectedTarget::Item { proc_idx, item_idx } => {
                    if proc_idx < self.procedures.len() {
                        let proc = &mut self.procedures[proc_idx];
                        if item_idx < proc.items.len() {
                            proc.items.remove(item_idx);
                            if proc.items.is_empty() {
                                self.procedures.remove(proc_idx);
                            }
                            self.selected_target = None;
                            self.scroll_to_target = None;
                        }
                    }
                }
            }
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.procedures)
    }

    pub fn load_from_json(&mut self, json_str: &str) -> Result<(), serde_json::Error> {
        #[derive(Deserialize)]
        #[serde(bound(deserialize = ""))]
        struct Wrapped {
            procedures: Vec<OutputProcedure>,
        }

        let loaded_procs =
            if let Ok(direct) = serde_json::from_str::<Vec<OutputProcedure>>(json_str) {
                direct
            } else {
                let wrapped: Wrapped = serde_json::from_str(json_str)?;
                wrapped.procedures
            };

        let mut max_id = 0;
        for proc in &loaded_procs {
            if proc.id > max_id {
                max_id = proc.id;
            }
            for item in &proc.items {
                if item.id > max_id {
                    max_id = item.id;
                }
            }
        }
        self.procedures = loaded_procs;
        self.next_id = max_id + 1;
        self.selected_target = None;
        self.scroll_to_target = None;
        self.last_copied_id = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_viewer_state_to_and_from_json() {
        let mut state = OutputViewerState::new();
        state.add_log("GET FILE='data.sav'.");
        let item_id = state.alloc_id();
        let proc_id = state.alloc_id();

        state.procedures.push(OutputProcedure {
            id: proc_id,
            title_key: "output-descriptives-title",
            custom_title: Some("My Descriptives".to_string()),
            is_expanded: true,
            items: vec![OutputItem {
                id: item_id,
                title_key: "output-descriptives-table",
                custom_title: None,
                kind: OutputItemKind::Warning("Check missing data".to_string()),
            }],
        });

        let json = state.to_json().unwrap();
        assert!(!json.is_empty());

        let mut restored = OutputViewerState::new();
        restored.load_from_json(&json).unwrap();

        assert_eq!(restored.procedures.len(), 2);
        assert_eq!(
            restored.procedures[1].custom_title.as_deref(),
            Some("My Descriptives")
        );
        assert_eq!(
            restored.procedures[1].title_key,
            "output-descriptives-title"
        );
        assert!(restored.next_id > proc_id && restored.next_id > item_id);
    }
}
