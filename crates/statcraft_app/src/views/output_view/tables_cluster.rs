use super::model::{
    OutputItem, OutputItemKind, OutputNotes, OutputProcedure, OutputViewerState, SelectedTarget,
};
use super::tables_common::{adjust_widths, format_sig, render_data_row, render_header_row};
use crate::theme::Tokens;
use egui::{Align2, ScrollArea, Ui};
use statcraft_algo::KMeansResult;
use statcraft_i18n::t;

impl OutputViewerState {
    pub fn add_kmeans(&mut self, result: KMeansResult) {
        let proc_id = self.alloc_id();
        let log_id = self.alloc_id();
        let notes_id = self.alloc_id();
        let item_id = self.alloc_id();

        let sps_log = format!(
            "QUICK CLUSTER {}\n  /CRITERIA = CLUSTERS({}) MXITER({})\n  /PRINT = INITIAL FINAL DISTAN ANOVA.",
            result.variables.join(" "),
            result.k,
            result.iterations_performed
        );

        let notes = OutputNotes {
            dataset_name: "DataSet1".to_string(),
            timestamp: "Session".to_string(),
            active_cases: result.valid_cases_count,
            active_vars: result.variables.len(),
        };

        let proc = OutputProcedure {
            id: proc_id,
            title_key: "output-kmeans-title",
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
                    title_key: "output-kmeans-table-title",
                    custom_title: None,
                    kind: OutputItemKind::KMeans(Box::new(result)),
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

pub fn render_spss_kmeans_tables(ui: &mut Ui, res: &KMeansResult) {
    ui.vertical(|ui| {
        // 1. Initial Cluster Centers
        ui.strong(t!("output-kmeans-initial-centers"));
        ui.add_space(4.0);
        render_center_matrix(ui, &res.variables, &res.initial_centers, "kmeans_init");

        ui.add_space(16.0);

        // 2. Final Cluster Centers
        ui.strong(t!("output-kmeans-final-centers"));
        ui.add_space(4.0);
        render_center_matrix(ui, &res.variables, &res.final_centers, "kmeans_final");

        ui.add_space(16.0);

        // 3. Distances between Final Cluster Centers
        ui.strong(t!("output-kmeans-distances-title"));
        ui.add_space(4.0);
        render_distance_matrix(ui, res.k, &res.center_distances);

        ui.add_space(16.0);

        // 4. ANOVA Table
        ui.strong(t!("output-kmeans-anova-title"));
        ui.add_space(4.0);
        render_kmeans_anova(ui, &res.anova_table);

        ui.add_space(16.0);

        // 5. Cluster Sizes
        ui.strong(t!("output-kmeans-cases-title"));
        ui.add_space(4.0);
        render_cluster_sizes(ui, &res.cluster_sizes, res.valid_cases_count);

        ui.add_space(16.0);
    });
}

fn render_center_matrix(ui: &mut Ui, vars: &[String], centers: &[Vec<f64>], id_salt: &str) {
    let k = centers.len();
    let mut cols: Vec<(String, f32)> = Vec::with_capacity(k + 1);
    cols.push((t!("output-common-variable"), 140.0));
    for i in 1..=k {
        cols.push((format!("{} {}", t!("output-kmeans-cluster"), i), 90.0));
    }
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal().id_salt(id_salt).show(ui, |ui| {
        ui.vertical(|ui| {
            render_header_row(ui, &cols, row_height);
            for (v_idx, v_name) in vars.iter().enumerate() {
                let bg = if v_idx % 2 == 0 {
                    Tokens::bg_row_even()
                } else {
                    Tokens::bg_row_odd()
                };
                let mut vals: Vec<(String, Align2)> = Vec::with_capacity(k + 1);
                vals.push((v_name.clone(), Align2::LEFT_CENTER));
                for c in centers {
                    let val_str = c
                        .get(v_idx)
                        .map(|x| format!("{:.3}", x))
                        .unwrap_or_default();
                    vals.push((val_str, Align2::RIGHT_CENTER));
                }
                render_data_row(ui, &cols, &vals, row_height, bg);
            }
        });
    });
}

fn render_distance_matrix(ui: &mut Ui, k: usize, distances: &[Vec<f64>]) {
    let mut cols: Vec<(String, f32)> = Vec::with_capacity(k + 1);
    cols.push((t!("output-kmeans-cluster"), 100.0));
    for i in 1..=k {
        cols.push((format!("{}", i), 80.0));
    }
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("kmeans_dist")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                for i in 0..k {
                    let bg = if i % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let mut vals = Vec::with_capacity(k + 1);
                    vals.push((format!("{}", i + 1), Align2::LEFT_CENTER));
                    for j in 0..k {
                        let d_str = distances
                            .get(i)
                            .and_then(|r| r.get(j))
                            .map(|d| format!("{:.3}", d))
                            .unwrap_or_default();
                        vals.push((d_str, Align2::RIGHT_CENTER));
                    }
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_kmeans_anova(ui: &mut Ui, anova_table: &[statcraft_algo::KMeansAnovaRow]) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-common-variable"), 130.0),
        (t!("output-kmeans-cluster-ms"), 90.0),
        (t!("output-common-df"), 50.0),
        (t!("output-kmeans-error-ms"), 90.0),
        (t!("output-common-df"), 50.0),
        (t!("output-common-f"), 70.0),
        (t!("output-common-sig"), 70.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("kmeans_anova")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                for (idx, row) in anova_table.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (row.variable.clone(), Align2::LEFT_CENTER),
                        (format!("{:.3}", row.cluster_ms), Align2::RIGHT_CENTER),
                        (format!("{}", row.cluster_df), Align2::RIGHT_CENTER),
                        (format!("{:.3}", row.error_ms), Align2::RIGHT_CENTER),
                        (format!("{}", row.error_df), Align2::RIGHT_CENTER),
                        (format!("{:.3}", row.f_stat), Align2::RIGHT_CENTER),
                        (format_sig(Some(row.p_value)), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
            });
        });
}

fn render_cluster_sizes(ui: &mut Ui, sizes: &[usize], valid_cases: usize) {
    let mut cols: Vec<(String, f32)> = vec![
        (t!("output-kmeans-cluster"), 100.0),
        (t!("output-common-n"), 80.0),
    ];
    adjust_widths(ui, &mut cols);

    let row_height = 20.0;
    ScrollArea::horizontal()
        .id_salt("kmeans_sizes")
        .show(ui, |ui| {
            ui.vertical(|ui| {
                render_header_row(ui, &cols, row_height);
                for (idx, &count) in sizes.iter().enumerate() {
                    let bg = if idx % 2 == 0 {
                        Tokens::bg_row_even()
                    } else {
                        Tokens::bg_row_odd()
                    };
                    let vals = [
                        (
                            format!("{} {}", t!("output-kmeans-cluster"), idx + 1),
                            Align2::LEFT_CENTER,
                        ),
                        (format!("{}", count), Align2::RIGHT_CENTER),
                    ];
                    render_data_row(ui, &cols, &vals, row_height, bg);
                }
                let valid_vals = [
                    (t!("output-common-valid"), Align2::LEFT_CENTER),
                    (format!("{}", valid_cases), Align2::RIGHT_CENTER),
                ];
                render_data_row(ui, &cols, &valid_vals, row_height, Tokens::bg_row_odd());
            });
        });
}

pub fn kmeans_to_tsv(res: &KMeansResult) -> String {
    let mut out = String::new();
    out.push_str("Final Cluster Centers\nVariable");
    for i in 1..=res.k {
        out.push_str(&format!("\tCluster {}", i));
    }
    out.push('\n');

    for (v_idx, var) in res.variables.iter().enumerate() {
        out.push_str(var);
        for c in &res.final_centers {
            if let Some(val) = c.get(v_idx) {
                out.push_str(&format!("\t{:.4}", val));
            }
        }
        out.push('\n');
    }
    out
}
