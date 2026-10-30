use super::rank_cases::*;
use statcraft_core::RawValue;

#[test]
fn test_compute_ranks_no_ties() {
    let vals = vec![
        RawValue::Number(30.0),
        RawValue::Number(10.0),
        RawValue::Number(20.0),
    ];

    let asc = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::SimpleRank,
    );
    assert_eq!(
        asc,
        vec![
            RawValue::Number(3.0),
            RawValue::Number(1.0),
            RawValue::Number(2.0),
        ]
    );

    let desc = compute_ranks(
        &vals,
        RankDirection::Descending,
        TiesHandling::Mean,
        RankType::SimpleRank,
    );
    assert_eq!(
        desc,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(3.0),
            RawValue::Number(2.0),
        ]
    );
}

#[test]
fn test_compute_ranks_with_ties() {
    // Values: 10, 20, 20, 30
    let vals = vec![
        RawValue::Number(10.0),
        RawValue::Number(20.0),
        RawValue::Number(20.0),
        RawValue::Number(30.0),
    ];

    // Mean: 1, 2.5, 2.5, 4
    let mean = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::SimpleRank,
    );
    assert_eq!(
        mean,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(2.5),
            RawValue::Number(2.5),
            RawValue::Number(4.0),
        ]
    );

    // Low: 1, 2, 2, 4
    let low = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Low,
        RankType::SimpleRank,
    );
    assert_eq!(
        low,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(2.0),
            RawValue::Number(4.0),
        ]
    );

    // High: 1, 3, 3, 4
    let high = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::High,
        RankType::SimpleRank,
    );
    assert_eq!(
        high,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(3.0),
            RawValue::Number(3.0),
            RawValue::Number(4.0),
        ]
    );

    // Seq: 1, 2, 3, 4
    let seq = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Seq,
        RankType::SimpleRank,
    );
    assert_eq!(
        seq,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(3.0),
            RawValue::Number(4.0),
        ]
    );
}

#[test]
fn test_compute_ranks_with_missing() {
    let vals = vec![
        RawValue::Number(100.0),
        RawValue::Null,
        RawValue::Number(50.0),
        RawValue::Text(String::new()),
        RawValue::Number(150.0),
    ];

    let res = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::SimpleRank,
    );
    assert_eq!(
        res,
        vec![
            RawValue::Number(2.0),
            RawValue::Null,
            RawValue::Number(1.0),
            RawValue::Null,
            RawValue::Number(3.0),
        ]
    );
}

#[test]
fn test_compute_ranks_fractional_and_ntiles() {
    let vals = vec![
        RawValue::Number(10.0),
        RawValue::Number(20.0),
        RawValue::Number(30.0),
        RawValue::Number(40.0),
    ];

    // FractionalRank (r / 4)
    let frac = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::FractionalRank,
    );
    assert_eq!(
        frac,
        vec![
            RawValue::Number(0.25),
            RawValue::Number(0.50),
            RawValue::Number(0.75),
            RawValue::Number(1.00),
        ]
    );

    // FractionalPercent ((r / 4) * 100)
    let pct = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::FractionalPercent,
    );
    assert_eq!(
        pct,
        vec![
            RawValue::Number(25.0),
            RawValue::Number(50.0),
            RawValue::Number(75.0),
            RawValue::Number(100.0),
        ]
    );

    // Ntiles (2 groups)
    let ntiles = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::Ntiles(2),
    );
    assert_eq!(
        ntiles,
        vec![
            RawValue::Number(1.0),
            RawValue::Number(1.0),
            RawValue::Number(2.0),
            RawValue::Number(2.0),
        ]
    );
}

#[test]
fn test_inv_normal_cdf_accuracy() {
    // Median prob = 0.5 -> 0.0
    assert!((inv_normal_cdf(0.5) - 0.0).abs() < 1e-12);

    // 1 standard deviation ~ 0.841344746
    let z1 = inv_normal_cdf(0.8413447460685429);
    assert!((z1 - 1.0).abs() < 1e-6);

    // -1 standard deviation
    let zn1 = inv_normal_cdf(0.15865525393145705);
    assert!((zn1 - (-1.0)).abs() < 1e-6);

    // 97.5% quantile ~ 1.95996398454
    let z975 = inv_normal_cdf(0.975);
    assert!((z975 - 1.95996398454).abs() < 1e-5);

    // 2.5% quantile ~ -1.95996398454
    let z025 = inv_normal_cdf(0.025);
    assert!((z025 - (-1.95996398454)).abs() < 1e-5);
}

#[test]
fn test_normal_scores() {
    let vals = vec![
        RawValue::Number(10.0),
        RawValue::Number(20.0),
        RawValue::Number(30.0),
    ];

    let blom = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::NormalBlom,
    );
    // N = 3, w = 3.
    // rank 2: (2 - 3/8) / (3 + 1/4) = (1.625) / (3.25) = 0.5 -> z = 0.0
    if let RawValue::Number(z) = blom[1] {
        assert!(z.abs() < 1e-6);
    } else {
        panic!("Expected Number");
    }

    let rankit = compute_ranks(
        &vals,
        RankDirection::Ascending,
        TiesHandling::Mean,
        RankType::NormalRankit,
    );
    // rank 2: (2 - 0.5) / 3 = 1.5 / 3 = 0.5 -> z = 0.0
    if let RawValue::Number(z) = rankit[1] {
        assert!(z.abs() < 1e-6);
    } else {
        panic!("Expected Number");
    }
}

#[test]
fn test_empty_and_all_null() {
    assert_eq!(
        compute_ranks(
            &[],
            RankDirection::Ascending,
            TiesHandling::Mean,
            RankType::SimpleRank
        ),
        Vec::<RawValue>::new()
    );

    let nulls = vec![RawValue::Null, RawValue::Null];
    assert_eq!(
        compute_ranks(
            &nulls,
            RankDirection::Ascending,
            TiesHandling::Mean,
            RankType::SimpleRank
        ),
        vec![RawValue::Null, RawValue::Null]
    );
}
