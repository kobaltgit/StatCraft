use statcraft_algo::cluster_kmeans::{compute_kmeans, KMeansOptions};
use statcraft_algo::codebook_core::{compute_codebook, CodebookOptions, RawColumnData};
use statcraft_algo::summarize_core::{compute_summarize, SummarizeOptions};
use statcraft_algo::survival_km::{compute_kaplan_meier, KMDataPoint, KMOptions};
use statcraft_algo::survival_lifetables::{compute_life_table, LifeTableCase, LifeTableOptions};
use statcraft_app::dialogs::{
    render_codebook_dialog, render_kmeans_dialog, render_summarize_dialog,
    render_survival_km_dialog, render_survival_lifetables_dialog, CodebookDialogAction,
    CodebookDialogState, KMeansDialogAction, KMeansDialogState, KaplanMeierDialogAction,
    KaplanMeierDialogState, LifeTablesDialogAction, LifeTablesDialogState, SummarizeDialogAction,
    SummarizeDialogState,
};
use statcraft_app::views::output_view::{
    codebook_to_tsv, kmeans_to_tsv, lifetables_to_tsv, render_output_view, summarize_to_tsv,
    survival_to_tsv, OutputViewerState,
};
use statcraft_core::variable::{DataType, Measure, VariableMetadata};
use statcraft_core::DataSet;

fn create_sample_dataset() -> DataSet {
    let mut ds = DataSet::empty();
    ds.add_variable("time", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable(
        "status",
        DataType::Numeric { decimals: 0 },
        Measure::Nominal,
    )
    .unwrap();
    ds.add_variable(
        "treatment",
        DataType::String { max_len: 8 },
        Measure::Nominal,
    )
    .unwrap();
    ds.add_variable("var1", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds.add_variable("var2", DataType::Numeric { decimals: 2 }, Measure::Scale)
        .unwrap();
    ds
}

#[test]
fn test_headless_render_pkg3_dialogs() {
    let _ = statcraft_i18n::init_i18n();
    let ctx = egui::Context::default();
    let ds = create_sample_dataset();

    // 1. K-Means dialog
    let mut km_state = KMeansDialogState {
        is_open: true,
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_kmeans_dialog(ctx, Some(&ds), &mut km_state);
        match action {
            KMeansDialogAction::None => {}
            _ => panic!("Expected None action on initial render"),
        }
    });

    // 2. Codebook dialog
    let mut cb_state = CodebookDialogState {
        is_open: true,
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_codebook_dialog(ctx, Some(&ds), &mut cb_state);
        match action {
            CodebookDialogAction::None => {}
            _ => panic!("Expected None action on initial render"),
        }
    });

    // 3. Summarize dialog
    let mut sm_state = SummarizeDialogState {
        is_open: true,
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_summarize_dialog(ctx, Some(&ds), &mut sm_state);
        match action {
            SummarizeDialogAction::None => {}
            _ => panic!("Expected None action on initial render"),
        }
    });

    // 4. Kaplan-Meier dialog
    let mut surv_km_state = KaplanMeierDialogState {
        is_open: true,
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_survival_km_dialog(ctx, Some(&ds), &mut surv_km_state);
        match action {
            KaplanMeierDialogAction::None => {}
            _ => panic!("Expected None action on initial render"),
        }
    });

    // 5. Life Tables dialog
    let mut lt_state = LifeTablesDialogState {
        is_open: true,
        ..Default::default()
    };
    let _ = ctx.run(Default::default(), |ctx| {
        let action = render_survival_lifetables_dialog(ctx, Some(&ds), &mut lt_state);
        match action {
            LifeTablesDialogAction::None => {}
            _ => panic!("Expected None action on initial render"),
        }
    });
}

#[test]
fn test_pkg3_output_viewer_and_tsv_export() {
    let _ = statcraft_i18n::init_i18n();
    let mut state = OutputViewerState::new();

    // 1. K-Means Output
    let data = vec![
        vec![0.0, 0.0],
        vec![1.0, 0.0],
        vec![10.0, 10.0],
        vec![11.0, 10.0],
    ];
    let var_names = vec!["x1".to_string(), "x2".to_string()];
    let km_opts = KMeansOptions {
        clusters: 2,
        max_iterations: 10,
        convergence_criterion: 1e-4,
    };
    let kmeans_res = compute_kmeans(&data, &var_names, &km_opts).expect("kmeans ok");
    state.add_kmeans(kmeans_res.clone());
    let km_tsv = kmeans_to_tsv(&kmeans_res);
    assert!(km_tsv.contains("Final Cluster Centers"));
    assert!(km_tsv.contains("x1"));

    // 2. Codebook Output
    let mut var_salary =
        VariableMetadata::new("salary", DataType::Numeric { decimals: 2 }, Measure::Scale)
            .expect("metadata ok");
    var_salary.label = "Current Salary".to_string();

    let salary_str = vec![
        Some("1000.00".to_string()),
        Some("2000.00".to_string()),
        Some("3000.00".to_string()),
    ];
    let salary_num = vec![Some(1000.0), Some(2000.0), Some(3000.0)];
    let cols = vec![RawColumnData {
        values: &salary_str,
        numeric_values: &salary_num,
    }];
    let codebook_res =
        compute_codebook(&[&var_salary], &cols, &CodebookOptions::default()).expect("codebook ok");
    state.add_codebook(codebook_res.clone());
    let cb_tsv = codebook_to_tsv(&codebook_res);
    assert!(cb_tsv.contains("salary"));

    // 3. Summarize Output
    let summarize_res = compute_summarize(
        &["salary".to_string()],
        &[salary_str],
        &[salary_num],
        &[],
        &[],
        &SummarizeOptions::default(),
    )
    .expect("summarize ok");
    state.add_summarize(summarize_res.clone());
    let sm_tsv = summarize_to_tsv(&summarize_res);
    assert!(sm_tsv.contains("Case"));

    // 4. Kaplan-Meier Output
    let km_data = vec![
        KMDataPoint {
            time: 2.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 3.0,
            is_event: true,
            group: "G1".to_string(),
        },
        KMDataPoint {
            time: 6.0,
            is_event: false,
            group: "G1".to_string(),
        },
    ];
    let km_res = compute_kaplan_meier(
        &km_data,
        "time",
        "status",
        Some("group"),
        &KMOptions::default(),
    )
    .expect("km ok");
    state.add_kaplan_meier(km_res.clone());
    let surv_tsv = survival_to_tsv(&km_res);
    assert!(surv_tsv.contains("Cumulative Survival"));

    // 5. Life Tables Output
    let lt_cases = vec![
        LifeTableCase {
            time: 3.0,
            is_event: true,
        },
        LifeTableCase {
            time: 7.0,
            is_event: true,
        },
        LifeTableCase {
            time: 8.0,
            is_event: false,
        },
    ];
    let lt_opts = LifeTableOptions {
        interval_start: 0.0,
        interval_end: 10.0,
        interval_width: 5.0,
    };
    let lt_res = compute_life_table(&lt_cases, "time", "status", &lt_opts).expect("life table ok");
    state.add_life_tables(lt_res.clone());
    let lt_tsv = lifetables_to_tsv(&lt_res);
    assert!(lt_tsv.contains("Interval"));

    // Headless render of output view with all Package 3 results
    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_output_view(ui, &mut state);
        });
    });
}
