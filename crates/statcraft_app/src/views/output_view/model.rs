use statcraft_algo::{
    DescriptivesResult, FrequencyResult, IndepTTestResult, OneSampleTTestResult, OneWayAnovaResult,
    PairedTTestResult,
};
use statcraft_i18n::t;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedTarget {
    Procedure(usize),
    Item { proc_idx: usize, item_idx: usize },
}

#[derive(Debug, Clone)]
pub struct OutputNotes {
    pub dataset_name: String,
    pub timestamp: String,
    pub active_cases: usize,
    pub active_vars: usize,
}

#[derive(Debug, Clone)]
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
    Warning(String),
    CustomText {
        title: String,
        content: String,
    },
}

#[derive(Debug, Clone)]
pub struct OutputItem {
    pub id: usize,
    pub title_key: &'static str,
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

#[derive(Debug, Clone)]
pub struct OutputProcedure {
    pub id: usize,
    pub title_key: &'static str,
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
}
