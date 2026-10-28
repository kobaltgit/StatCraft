use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use statcraft_algo::{
    DescriptivesResult, FrequencyResult, IndepTTestResult, OneSampleTTestResult, OneWayAnovaResult,
    PairedTTestResult,
};

impl OutputViewerState {
    pub fn add_dataset_imported(&mut self, name: String, rows: usize, cols: usize) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let item_id = self.alloc_id();

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-dataset-imported-title",
            custom_title: None,
            is_expanded: true,
            items: vec![
                OutputItem {
                    id: log_id,
                    title_key: "output-log",
                    custom_title: None,
                    kind: OutputItemKind::Log(format!("GET DATA /TYPE=CSV /FILE='{}'", name)),
                },
                OutputItem {
                    id: item_id,
                    title_key: "output-dataset-summary",
                    custom_title: None,
                    kind: OutputItemKind::DatasetSummary { name, rows, cols },
                },
            ],
        };

        self.procedures.push(proc);
        let new_proc_idx = self.procedures.len() - 1;
        self.selected_target = Some(SelectedTarget::Item {
            proc_idx: new_proc_idx,
            item_idx: 1,
        });
        self.scroll_to_target = self.selected_target;
    }

    pub fn add_descriptives(&mut self, results: Vec<DescriptivesResult>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let var_names: Vec<&str> = results.iter().map(|r| r.variable.as_str()).collect();
        let sps_log = format!(
            "DESCRIPTIVES VARIABLES={}\n  /STATISTICS=MEAN STDDEV MIN MAX.",
            var_names.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results.first().map(|r| r.n_valid).unwrap_or(0),
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-descriptives-title",
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
                    title_key: "output-descriptives-table",
                    custom_title: None,
                    kind: OutputItemKind::DescriptivesTable(results),
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

    pub fn add_frequencies(&mut self, results: Vec<FrequencyResult>) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let var_names: Vec<&str> = results.iter().map(|r| r.variable.as_str()).collect();
        let sps_log = format!(
            "FREQUENCIES VARIABLES={}\n  /ORDER=ANALYSIS.",
            var_names.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results.first().map(|r| r.n_valid).unwrap_or(0),
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-frequencies-title",
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
                    title_key: "output-frequencies-table",
                    custom_title: None,
                    kind: OutputItemKind::FrequenciesTables(results),
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

    pub fn add_onesample_ttest(
        &mut self,
        results: Vec<OneSampleTTestResult>,
        test_value: f64,
        ci: f64,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let var_names: Vec<&str> = results.iter().map(|r| r.variable.as_str()).collect();
        let sps_log = format!(
            "T-TEST /TESTVAL={}\n  /CRITERIA=CI({:.2})\n  /VARIABLES={}.",
            test_value,
            ci,
            var_names.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results.first().map(|r| r.n_valid).unwrap_or(0),
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-ttest-onesample-title",
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
                    title_key: "output-ttest-onesample-title",
                    custom_title: None,
                    kind: OutputItemKind::OneSampleTTest {
                        results,
                        test_value,
                        ci,
                    },
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

    pub fn add_indep_ttest(
        &mut self,
        results: Vec<IndepTTestResult>,
        grouping_var: &str,
        group1: &str,
        group2: &str,
        ci: f64,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let var_names: Vec<&str> = results.iter().map(|r| r.variable.as_str()).collect();
        let sps_log = format!(
            "T-TEST GROUPS={}({} {})\n  /CRITERIA=CI({:.2})\n  /VARIABLES={}.",
            grouping_var,
            group1,
            group2,
            ci,
            var_names.join(" ")
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results
                .first()
                .map(|r| r.group1_stats.n + r.group2_stats.n)
                .unwrap_or(0),
            active_vars: results.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-ttest-indep-title",
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
                    title_key: "output-ttest-indep-title",
                    custom_title: None,
                    kind: OutputItemKind::IndepTTest {
                        results,
                        grouping_var: grouping_var.to_string(),
                        group1: group1.to_string(),
                        group2: group2.to_string(),
                        ci,
                    },
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

    pub fn add_paired_ttest(
        &mut self,
        results: Vec<PairedTTestResult>,
        pairs: &[(String, String)],
        ci: f64,
    ) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let pairs_syntax: Vec<String> = pairs
            .iter()
            .map(|(v1, v2)| format!("{} WITH {}", v1, v2))
            .collect();
        let sps_log = format!(
            "T-TEST PAIRS={}\n  /CRITERIA=CI({:.2}).",
            pairs_syntax.join(" "),
            ci
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results.first().map(|r| r.var1_stats.n).unwrap_or(0),
            active_vars: pairs.len() * 2,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-ttest-paired-title",
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
                    title_key: "output-ttest-paired-title",
                    custom_title: None,
                    kind: OutputItemKind::PairedTTest {
                        results,
                        pairs: pairs.to_vec(),
                        ci,
                    },
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

    pub fn add_oneway_anova(&mut self, results: Vec<OneWayAnovaResult>, ci: f64) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let dep_names: Vec<&str> = results.iter().map(|r| r.dep_var.as_str()).collect();
        let factor = results
            .first()
            .map(|r| r.factor_var.as_str())
            .unwrap_or("factor");
        let sps_log = format!(
            "ONEWAY {} BY {}\n  /STATISTICS DESCRIPTIVES HOMOGENEITY\n  /MISSING ANALYSIS.",
            dep_names.join(" "),
            factor
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: results
                .first()
                .and_then(|r| r.total_descriptives.as_ref())
                .map(|td| td.n)
                .unwrap_or(0),
            active_vars: results.len() + 1,
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-anova-oneway-title",
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
                    title_key: "output-anova-oneway-title",
                    custom_title: None,
                    kind: OutputItemKind::OneWayAnova { results, ci },
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

    pub fn add_entry(&mut self, title: String, content: String) {
        let proc_id = self.alloc_id();
        let item_id = self.alloc_id();

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-log",
            custom_title: Some(title.clone()),
            is_expanded: true,
            items: vec![OutputItem {
                id: item_id,
                title_key: "output-log",
                custom_title: Some(title),
                kind: OutputItemKind::CustomText {
                    title: "Log".to_string(),
                    content,
                },
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
}
