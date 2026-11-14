pub mod multivariate;
pub mod repeated;
pub mod univariate;

pub use multivariate::*;
pub use repeated::*;
pub use univariate::*;

use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use statcraft_algo::glm::{MultivariateGlmResult, RepeatedGlmResult, UnivariateGlmResult};

impl OutputViewerState {
    pub fn add_univariate_glm(&mut self, result: UnivariateGlmResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "UNIANOVA {}\n  /METHOD = SSTYPE(3)\n  /INTERCEPT = INCLUDE.",
            result.dependent
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_valid,
            active_vars: result.tests.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-glm-univariate-title",
            custom_title: None,
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
                    title_key: "output-glm-between-subjects-title",
                    custom_title: None,
                    kind: OutputItemKind::UnivariateGlm(Box::new(result)),
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

    pub fn add_multivariate_glm(&mut self, result: MultivariateGlmResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "GLM {}\n  /METHOD = SSTYPE(3)\n  /PRINT = TEST(MULTIVAR).",
            result.dependents.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_valid,
            active_vars: result.dependents.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-glm-multivariate-title",
            custom_title: None,
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
                    title_key: "output-glm-multivariate-tests-title",
                    custom_title: None,
                    kind: OutputItemKind::MultivariateGlm(Box::new(result)),
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

    pub fn add_repeated_glm(&mut self, result: RepeatedGlmResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "GLM {}\n  /WSFACTOR = {} {} Polynomial\n  /METHOD = SSTYPE(3).",
            result.measure_variables.join(" "),
            result.factor_name,
            result.measure_variables.len()
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.n_valid,
            active_vars: result.measure_variables.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-glm-repeated-title",
            custom_title: None,
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
                    title_key: "output-glm-within-subjects-title",
                    custom_title: None,
                    kind: OutputItemKind::RepeatedGlm(Box::new(result)),
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
