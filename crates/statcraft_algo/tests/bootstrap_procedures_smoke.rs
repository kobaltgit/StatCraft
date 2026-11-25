use statcraft_algo::bootstrap::{BootstrapCiMethod, BootstrapOptions, BootstrapSamplingMethod};
use statcraft_algo::{
    compute_means_bootstrap, compute_means_breakdown, compute_onesample_ttest_bootstrap,
    compute_oneway_anova, compute_oneway_anova_bootstrap, compute_paired_ttest_bootstrap,
    MeansOptions, OneSampleTTestOptions, OneWayAnovaOptions,
};

#[test]
fn smoke_means_onesample_paired_and_anova_bootstrap() {
    let boot_opts = BootstrapOptions {
        enabled: true,
        samples: 100,
        seed: Some(2_000_000),
        ci_level: 95.0,
        ci_method: BootstrapCiMethod::Percentile,
        sampling_method: BootstrapSamplingMethod::Simple,
    };

    let dep = [
        Some(10.0),
        Some(12.0),
        Some(14.0),
        Some(20.0),
        Some(22.0),
        Some(25.0),
    ];
    let fac: Vec<Option<String>> = ["A", "A", "A", "B", "B", "B"]
        .iter()
        .map(|s| Some((*s).to_string()))
        .collect();

    let means_opts = MeansOptions::default();
    let means_base =
        compute_means_breakdown("score", "grp", &dep, &fac, None, &means_opts).unwrap();
    let means_boot = compute_means_bootstrap(
        "score",
        "grp",
        &dep,
        &fac,
        None,
        &means_opts,
        &means_base,
        None,
        &boot_opts,
    )
    .unwrap();
    assert_eq!(means_boot.groups.len(), 2);
    assert!(means_boot.total.mean.is_some());

    let os_boot = compute_onesample_ttest_bootstrap(
        "score",
        &dep,
        &OneSampleTTestOptions::default(),
        None,
        &boot_opts,
    )
    .unwrap();
    assert!(os_boot.mean.is_some());
    assert!(os_boot.mean_diff.is_some());

    let p2 = [
        Some(8.0),
        Some(10.0),
        Some(11.0),
        Some(16.0),
        Some(18.0),
        Some(20.0),
    ];
    let paired_boot =
        compute_paired_ttest_bootstrap(1, "pre", &dep, "post", &p2, 0.95, None, &boot_opts)
            .unwrap();
    assert!(paired_boot.mean_diff.is_some());

    let anova_opts = OneWayAnovaOptions::default();
    let anova_base = compute_oneway_anova("score", "grp", &dep, &fac, &anova_opts).unwrap();
    let anova_boot = compute_oneway_anova_bootstrap(
        "score",
        "grp",
        &dep,
        &fac,
        &anova_opts,
        &anova_base,
        None,
        &boot_opts,
    )
    .unwrap();
    assert_eq!(anova_boot.groups.len(), 2);
    assert_eq!(anova_boot.pairwise_diffs.len(), 1);
}
