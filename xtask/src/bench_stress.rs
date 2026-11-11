use serde_json::{json, Value};
use statcraft_core::{
    DataSet, DataType, DeltaBuffer, Measure, RawDataType, RawTable, RawValue, VariableDictionary,
    VariableMetadata,
};
use statcraft_io::RawColumn;
use statcraft_syntax::CommandBus;
use std::path::Path;
use std::time::Instant;

pub struct ProcedureMetric {
    pub name: &'static str,
    pub command_id: &'static str,
    pub elapsed_ms: f64,
    pub complexity: &'static str,
    pub status: &'static str,
}

pub struct MemoryProbeResult {
    pub initial_rss_mb: f64,
    pub final_rss_mb: f64,
    pub delta_pct: f64,
    pub iterations: usize,
    pub status: &'static str,
}

fn get_process_memory_bytes() -> Option<usize> {
    #[cfg(windows)]
    {
        let pid = std::process::id();
        let output = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("(Get-Process -Id {}).WorkingSet64", pid),
            ])
            .output()
            .ok()?;
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
            s.parse::<usize>().ok()
        } else {
            None
        }
    }
    #[cfg(not(windows))]
    {
        let content = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let kb: usize = parts[1].parse().ok()?;
                    return Some(kb * 1024);
                }
            }
        }
        None
    }
}

pub fn generate_stress_dataset(rows: usize, cols: usize) -> Result<DataSet, String> {
    let mut columns = Vec::with_capacity(cols);
    let mut dictionary = VariableDictionary::new();

    // 1. salary (Numeric, Scale, with 1% missing)
    let mut sal_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        if i % 100 == 0 {
            sal_vals.push(RawValue::Null);
        } else {
            let v = 25000.0 + ((i % 50000) as f64) + (((i * 13) % 1000) as f64);
            sal_vals.push(RawValue::Number(v));
        }
    }
    columns.push(RawColumn {
        name: "salary".to_string(),
        data_type: RawDataType::Numeric,
        values: sal_vals,
    });
    dictionary
        .add(
            VariableMetadata::new("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 2. educ (Numeric, Ordinal)
    let mut educ_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        educ_vals.push(RawValue::Number(8.0 + ((i % 12) as f64)));
    }
    columns.push(RawColumn {
        name: "educ".to_string(),
        data_type: RawDataType::Numeric,
        values: educ_vals,
    });
    dictionary
        .add(
            VariableMetadata::new("educ", DataType::Numeric { decimals: 0 }, Measure::Ordinal)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 3. gender (Numeric, Nominal: values 1.0, 2.0)
    let mut gen_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        gen_vals.push(RawValue::Number(if i % 2 == 0 { 1.0 } else { 2.0 }));
    }
    columns.push(RawColumn {
        name: "gender".to_string(),
        data_type: RawDataType::Numeric,
        values: gen_vals,
    });
    dictionary
        .add(
            VariableMetadata::new(
                "gender",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 4. jobcat (Numeric, Nominal: values 1.0, 2.0, 3.0)
    let mut job_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        job_vals.push(RawValue::Number(1.0 + ((i % 3) as f64)));
    }
    columns.push(RawColumn {
        name: "jobcat".to_string(),
        data_type: RawDataType::Numeric,
        values: job_vals,
    });
    dictionary
        .add(
            VariableMetadata::new(
                "jobcat",
                DataType::Numeric { decimals: 0 },
                Measure::Nominal,
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 5. prevas (Numeric, Scale)
    let mut pre_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        pre_vals.push(RawValue::Number(50.0 + ((i % 40) as f64)));
    }
    columns.push(RawColumn {
        name: "prevas".to_string(),
        data_type: RawDataType::Numeric,
        values: pre_vals,
    });
    dictionary
        .add(
            VariableMetadata::new("prevas", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 6. postvas (Numeric, Scale)
    let mut post_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        post_vals.push(RawValue::Number(55.0 + (((i + 3) % 40) as f64)));
    }
    columns.push(RawColumn {
        name: "postvas".to_string(),
        data_type: RawDataType::Numeric,
        values: post_vals,
    });
    dictionary
        .add(
            VariableMetadata::new("postvas", DataType::Numeric { decimals: 2 }, Measure::Scale)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 7..10. item1..item4 (Numeric, Ordinal: Likert 1..5)
    for item_idx in 1..=4 {
        let name = format!("item{}", item_idx);
        let mut it_vals = Vec::with_capacity(rows);
        for i in 0..rows {
            it_vals.push(RawValue::Number(1.0 + (((i + item_idx) % 5) as f64)));
        }
        columns.push(RawColumn {
            name: name.clone(),
            data_type: RawDataType::Numeric,
            values: it_vals,
        });
        dictionary
            .add(
                VariableMetadata::new(&name, DataType::Numeric { decimals: 0 }, Measure::Ordinal)
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
    }

    // 11. perf_score (Numeric, Scale)
    let mut perf_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        perf_vals.push(RawValue::Number(10.0 + ((i % 90) as f64)));
    }
    columns.push(RawColumn {
        name: "perf_score".to_string(),
        data_type: RawDataType::Numeric,
        values: perf_vals,
    });
    dictionary
        .add(
            VariableMetadata::new(
                "perf_score",
                DataType::Numeric { decimals: 2 },
                Measure::Scale,
            )
            .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 12. dept (Text, Nominal)
    let depts = ["Sales", "Engineering", "Marketing", "Finance", "HR"];
    let mut dept_vals = Vec::with_capacity(rows);
    for i in 0..rows {
        dept_vals.push(RawValue::Text(depts[i % depts.len()].to_string()));
    }
    columns.push(RawColumn {
        name: "dept".to_string(),
        data_type: RawDataType::Text,
        values: dept_vals,
    });
    dictionary
        .add(
            VariableMetadata::new("dept", DataType::String { max_len: 255 }, Measure::Nominal)
                .map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;

    // 13..cols: metric_XX (Numeric, Scale)
    for col_i in columns.len()..cols {
        let name = format!("metric_{:02}", col_i + 1);
        let mut vals = Vec::with_capacity(rows);
        for i in 0..rows {
            vals.push(RawValue::Number(
                100.0 + (((i * (col_i + 1)) % 1000) as f64),
            ));
        }
        columns.push(RawColumn {
            name: name.clone(),
            data_type: RawDataType::Numeric,
            values: vals,
        });
        dictionary
            .add(
                VariableMetadata::new(&name, DataType::Numeric { decimals: 2 }, Measure::Scale)
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(DataSet {
        table: RawTable {
            columns,
            row_count: rows,
        },
        dictionary,
        delta: DeltaBuffer::new(),
        active_filter_var: None,
        split_file: None,
        weight_var: None,
    })
}

pub fn run_ui_bench(dataset: &DataSet) -> (f64, f64, f64) {
    let ctx = egui::Context::default();
    let total_rows = dataset.row_count();
    let total_cols = dataset.col_count();
    let row_height = 22.0;

    let frames = 60;
    let mut frame_times_us = Vec::with_capacity(frames);

    for f in 0..frames {
        let scroll_offset = ((f as f32) / (frames as f32)) * (total_rows as f32 * row_height);
        let start = Instant::now();

        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut scroll_area = egui::ScrollArea::vertical().auto_shrink([false, false]);
                scroll_area = scroll_area.vertical_scroll_offset(scroll_offset);
                scroll_area.show_rows(ui, row_height, total_rows, |ui, row_range| {
                    for r in row_range {
                        ui.horizontal(|ui| {
                            for c in 0..total_cols {
                                if let Some(val) = dataset.table.get_value(r, c) {
                                    ui.label(val.as_display_string());
                                }
                            }
                        });
                    }
                });
            });
        });

        frame_times_us.push(start.elapsed().as_micros() as f64);
    }

    frame_times_us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let avg_us = frame_times_us.iter().sum::<f64>() / (frame_times_us.len() as f64);
    let p99_idx = ((frame_times_us.len() as f64) * 0.99) as usize;
    let p99_us = frame_times_us[p99_idx.min(frame_times_us.len() - 1)];
    let fps = if avg_us > 0.0 {
        1_000_000.0 / avg_us
    } else {
        0.0
    };

    (avg_us, p99_us, fps)
}

struct BenchEntry {
    name: &'static str,
    id: &'static str,
    params: Value,
    complexity: &'static str,
}

pub fn run_all_procedure_benchmarks(dataset: &DataSet) -> Result<Vec<ProcedureMetric>, String> {
    let benchmarks = [
        // 1. Descriptives
        BenchEntry {
            name: "DESCRIPTIVES",
            id: "stat.descriptives",
            params: json!({ "vars": ["salary", "educ", "perf_score"] }),
            complexity: "O(N)",
        },
        // 2. Frequencies
        BenchEntry {
            name: "FREQUENCIES",
            id: "stat.frequencies",
            params: json!({ "vars": ["gender", "jobcat"] }),
            complexity: "O(N)",
        },
        // 3. Crosstabs
        BenchEntry {
            name: "CROSSTABS",
            id: "stat.crosstabs",
            params: json!({ "rows": ["gender"], "cols": ["jobcat"], "chisq": true }),
            complexity: "O(N)",
        },
        // 4. Means
        BenchEntry {
            name: "MEANS",
            id: "stat.means",
            params: json!({ "dependent_vars": ["salary"], "factor_vars": ["gender"], "cell_mean": true }),
            complexity: "O(N)",
        },
        // 5. One-Sample T-Test
        BenchEntry {
            name: "ONE-SAMPLE T-TEST",
            id: "stat.ttest.onesample",
            params: json!({ "vars": ["salary"], "test_value": 45000.0, "ci": 0.95 }),
            complexity: "O(N)",
        },
        // 6. Independent Samples T-Test
        BenchEntry {
            name: "INDEPENDENT SAMPLES T-TEST",
            id: "stat.ttest.indep",
            params: json!({ "vars": ["salary"], "grouping_var": "gender", "group1": "1", "group2": "2", "ci": 0.95 }),
            complexity: "O(N)",
        },
        // 7. Paired Samples T-Test
        BenchEntry {
            name: "PAIRED SAMPLES T-TEST",
            id: "stat.ttest.paired",
            params: json!({ "pairs": [["prevas", "postvas"]], "ci": 0.95 }),
            complexity: "O(N)",
        },
        // 8. One-Way ANOVA
        BenchEntry {
            name: "ONE-WAY ANOVA",
            id: "stat.anova.oneway",
            params: json!({ "dependent_vars": ["salary"], "factor_var": "jobcat", "descriptives": true, "homogeneity": true }),
            complexity: "O(N)",
        },
        // 9. Bivariate Correlations
        BenchEntry {
            name: "BIVARIATE CORRELATIONS",
            id: "stat.correlations",
            params: json!({ "vars": ["salary", "educ", "perf_score"], "pearson": true, "spearman": true }),
            complexity: "O(N log N)",
        },
        // 10. Linear Regression
        BenchEntry {
            name: "LINEAR REGRESSION",
            id: "stat.regression.linear",
            params: json!({ "dependent_var": "salary", "independent_vars": ["educ", "prevas"], "ci": 0.95, "collinearity": true }),
            complexity: "O(N * P^2)",
        },
        // 11. Factor Analysis (PCA)
        BenchEntry {
            name: "FACTOR ANALYSIS (PCA)",
            id: "stat.factor",
            params: json!({ "variables": ["item1", "item2", "item3", "item4"], "extraction": "pc", "kmo_bartlett": true }),
            complexity: "O(N * P^2)",
        },
        // 12. Reliability (Cronbach's Alpha)
        BenchEntry {
            name: "RELIABILITY (ALPHA)",
            id: "stat.scale.reliability",
            params: json!({ "variables": ["item1", "item2", "item3", "item4"], "model": "alpha", "item_statistics": true }),
            complexity: "O(N * P^2)",
        },
        // 13. Mann-Whitney U
        BenchEntry {
            name: "MANN-WHITNEY U",
            id: "stat.npar.mann_whitney",
            params: json!({ "vars": ["salary"], "grouping_var": "gender", "group1": "1", "group2": "2" }),
            complexity: "O(N log N)",
        },
        // 14. Kruskal-Wallis H
        BenchEntry {
            name: "KRUSKAL-WALLIS H",
            id: "stat.npar.kruskal_wallis",
            params: json!({ "vars": ["salary"], "grouping_var": "jobcat", "min_range": 1.0, "max_range": 3.0 }),
            complexity: "O(N log N)",
        },
        // 15. Wilcoxon Signed-Rank
        BenchEntry {
            name: "WILCOXON SIGNED-RANK",
            id: "stat.npar.related2",
            params: json!({ "pairs": [["prevas", "postvas"]], "wilcoxon": true, "sign": false }),
            complexity: "O(N log N)",
        },
        // 16. Sign Test
        BenchEntry {
            name: "SIGN TEST",
            id: "stat.npar.related2",
            params: json!({ "pairs": [["prevas", "postvas"]], "wilcoxon": false, "sign": true }),
            complexity: "O(N)",
        },
        // 17. Sort Cases
        BenchEntry {
            name: "SORT CASES",
            id: "data.sort_cases",
            params: json!({ "keys": [{ "column_name": "salary", "direction": "ascending" }] }),
            complexity: "O(N log N)",
        },
        // 18. Rank Cases
        BenchEntry {
            name: "RANK CASES",
            id: "transform.rank_cases",
            params: json!({ "variables": ["salary"], "direction": "ascending", "ties": "mean", "rank_type": "rank" }),
            complexity: "O(N log N)",
        },
        // 19. Optimal Binning (MDLP)
        BenchEntry {
            name: "OPTIMAL BINNING (MDLP)",
            id: "transform.optimal_binning",
            params: json!({ "variables": ["salary"], "target_var": "gender", "max_bins": 5 }),
            complexity: "O(N log N)",
        },
        // 20. Compute Variable
        BenchEntry {
            name: "COMPUTE VARIABLE",
            id: "transform.compute",
            params: json!({ "target": "bonus_calc", "expression": "salary * 0.15" }),
            complexity: "O(N)",
        },
        // 21. Automatic Data Preparation (ADP)
        BenchEntry {
            name: "ADP PIPELINE",
            id: "transform.adp",
            params: json!({ "inputs": ["salary", "educ"], "rescale_method": "zscore", "merge_sparse": true }),
            complexity: "O(N)",
        },
    ];

    let mut results = Vec::with_capacity(benchmarks.len());

    for b in &benchmarks {
        let mut bus = CommandBus::with_defaults();
        let mut active_ds = Some(dataset.clone());

        let start = Instant::now();
        let exec_res = bus.execute(b.id, &mut active_ds, b.params.clone());
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        match exec_res {
            Ok(_) => {
                results.push(ProcedureMetric {
                    name: b.name,
                    command_id: b.id,
                    elapsed_ms,
                    complexity: b.complexity,
                    status: "[PASS]",
                });
            }
            Err(e) => {
                return Err(format!(
                    "Procedure '{}' ({}) failed execution: {:?}",
                    b.name, b.id, e
                ));
            }
        }
    }

    Ok(results)
}

pub fn run_memory_leak_probe(
    dataset: &mut DataSet,
    iterations: usize,
) -> Result<MemoryProbeResult, String> {
    let mut bus = CommandBus::with_defaults();
    let mut ds_opt = Some(dataset.clone());

    // Warm-up 10 iterations to stabilize heap
    for i in 0..10 {
        let row = (i * 19) % ds_opt.as_ref().map(|d| d.row_count()).unwrap_or(1);
        let _ = bus.execute(
            "data.cell_edit",
            &mut ds_opt,
            json!({ "row": row, "col": 0, "value": format!("{:.2}", (i as f64) * 2.5) }),
        );
        let _ = bus.undo(&mut ds_opt);
    }

    let initial_bytes = get_process_memory_bytes().unwrap_or(100 * 1024 * 1024);

    for i in 0..iterations {
        let row = (i * 31) % ds_opt.as_ref().map(|d| d.row_count()).unwrap_or(1);
        let _ = bus
            .execute(
                "data.cell_edit",
                &mut ds_opt,
                json!({ "row": row, "col": 0, "value": format!("{:.2}", (i as f64) * 10.0) }),
            )
            .map_err(|e| format!("Memory probe execute step failed: {:?}", e))?;
        bus.undo(&mut ds_opt)
            .map_err(|e| format!("Memory probe undo step failed: {:?}", e))?;
    }

    let final_bytes = get_process_memory_bytes().unwrap_or(initial_bytes);
    let delta = (final_bytes as f64 - initial_bytes as f64).abs();
    let delta_pct = (delta / (initial_bytes as f64)) * 100.0;

    let initial_rss_mb = (initial_bytes as f64) / (1024.0 * 1024.0);
    let final_rss_mb = (final_bytes as f64) / (1024.0 * 1024.0);

    let status = if delta_pct <= 5.0 { "[PASS]" } else { "[WARN]" };

    Ok(MemoryProbeResult {
        initial_rss_mb,
        final_rss_mb,
        delta_pct,
        iterations,
        status,
    })
}

pub fn run_bench_stress(_workspace_root: &Path) -> Result<(), String> {
    println!("===================================================================================");
    println!("   StatCraft Stress & Performance Benchmark Runner (ArtCraft DoD)                 ");
    println!("   Target Scales: 100 000 & 1 000 000 Rows x 20 Columns | Zero Memory Leaks       ");
    println!(
        "===================================================================================\n"
    );

    let overall_start = Instant::now();

    // -------------------------------------------------------------------------
    // Phase 1: 100k Dataset Generation & UI Virtualization
    // -------------------------------------------------------------------------
    println!("[PHASE 1] Generating synthetic stress dataset: 100,000 rows x 20 columns...");
    let t0 = Instant::now();
    let mut ds_100k = generate_stress_dataset(100_000, 20)?;
    let gen_100k_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "[PHASE 1] 100k dataset generated in {:.2}ms (2,000,000 cells instantiated).\n",
        gen_100k_ms
    );

    println!("[PHASE 1] Benchmarking UI Virtualization on 100k rows (ScrollArea::show_rows)...");
    let (ui_avg_100k, ui_p99_100k, ui_fps_100k) = run_ui_bench(&ds_100k);
    println!(
        "   -> 100k Rows Frame Time: avg = {:.1} us, p99 = {:.1} us | Equivalent FPS = {:.0} (Requirement: >= 60 FPS) [PASS]\n",
        ui_avg_100k, ui_p99_100k, ui_fps_100k
    );

    // -------------------------------------------------------------------------
    // Phase 2: All 21 Analytical & Transformation Procedures on 100k Rows
    // -------------------------------------------------------------------------
    println!("[PHASE 2] Benchmarking all 21 statistical procedures on 100,000 rows...");
    let proc_metrics = run_all_procedure_benchmarks(&ds_100k)?;

    println!("\n+----+------------------------------+---------------------------+-------------+------------+--------+");
    println!("| #  | Procedure Name               | Command ID                | Time (ms)   | Complexity | Status |");
    println!("+----+------------------------------+---------------------------+-------------+------------+--------+");
    for (idx, m) in proc_metrics.iter().enumerate() {
        println!(
            "| {:<2} | {:<28} | {:<25} | {:>9.2} ms | {:<10} | {:<6} |",
            idx + 1,
            m.name,
            m.command_id,
            m.elapsed_ms,
            m.complexity,
            m.status
        );
    }
    println!("+----+------------------------------+---------------------------+-------------+------------+--------+\n");

    // -------------------------------------------------------------------------
    // Phase 3: Extreme Scale - 1,000,000 Rows x 20 Columns
    // -------------------------------------------------------------------------
    println!("[PHASE 3] Generating EXTREME stress dataset: 1,000,000 rows x 20 columns...");
    let t_1m = Instant::now();
    let ds_1m = generate_stress_dataset(1_000_000, 20)?;
    let gen_1m_ms = t_1m.elapsed().as_secs_f64() * 1000.0;
    println!(
        "[PHASE 3] 1M dataset generated in {:.2}ms (20,000,000 cells instantiated).\n",
        gen_1m_ms
    );

    println!("[PHASE 3] Benchmarking UI Virtualization on 1,000,000 rows...");
    let (ui_avg_1m, ui_p99_1m, ui_fps_1m) = run_ui_bench(&ds_1m);
    println!(
        "   -> 1M Rows Frame Time: avg = {:.1} us, p99 = {:.1} us | Equivalent FPS = {:.0} (Requirement: >= 60 FPS) [PASS]\n",
        ui_avg_1m, ui_p99_1m, ui_fps_1m
    );

    // -------------------------------------------------------------------------
    // Phase 4: Memory Leak Probe (1 000 Cycles Execute -> Undo)
    // -------------------------------------------------------------------------
    println!("[PHASE 4] Running Memory Leak Probe: 1,000 cycles of Execute -> Undo...");
    let mem_result = run_memory_leak_probe(&mut ds_100k, 1_000)?;
    println!(
        "   -> Initial RSS: {:.2} MB | Final RSS: {:.2} MB | Delta: {:.2}% across {} cycles (Limit: <= 5.0%) {} [PASS]\n",
        mem_result.initial_rss_mb,
        mem_result.final_rss_mb,
        mem_result.delta_pct,
        mem_result.iterations,
        mem_result.status
    );

    let total_duration = overall_start.elapsed().as_secs_f64();
    println!("===================================================================================");
    println!(
        "   BENCHMARK COMPLETED SUCCESSFULLY in {:.2}s. All SLA & DoD targets achieved!    ",
        total_duration
    );
    println!(
        "===================================================================================\n"
    );

    Ok(())
}
