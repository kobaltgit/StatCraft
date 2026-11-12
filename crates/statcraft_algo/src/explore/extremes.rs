use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtremeValueItem {
    pub order: usize,       // 1 to 5
    pub case_number: usize, // 1-based original row index
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtremeValuesResult {
    pub variable: String,
    pub factor_level: Option<String>,
    pub highest: Vec<ExtremeValueItem>,
    pub lowest: Vec<ExtremeValueItem>,
}

/// Computes the 5 lowest and 5 highest extreme values with their 1-based case numbers.
/// Properly handles ties and datasets with fewer than 5 observations.
pub fn compute_extreme_values(
    var_name: &str,
    factor_level: Option<String>,
    indexed_values: &[(usize, f64)],
) -> ExtremeValuesResult {
    if indexed_values.is_empty() {
        return ExtremeValuesResult {
            variable: var_name.to_string(),
            factor_level,
            highest: Vec::new(),
            lowest: Vec::new(),
        };
    }

    // Sort by value ascending, breaking ties by case number ascending
    let mut sorted = indexed_values.to_vec();
    sorted.sort_by(|a, b| match a.1.partial_cmp(&b.1) {
        Some(ord) if ord != std::cmp::Ordering::Equal => ord,
        _ => a.0.cmp(&b.0),
    });

    let n = sorted.len();
    let take = 5.min(n);

    // 5 Lowest cases (in ascending order, order 1 is lowest)
    let mut lowest = Vec::with_capacity(take);
    for (k, &(case_num, val)) in sorted.iter().take(take).enumerate() {
        lowest.push(ExtremeValueItem {
            order: k + 1,
            case_number: case_num,
            value: val,
        });
    }

    // 5 Highest cases (in descending order, order 1 is highest)
    let mut highest = Vec::with_capacity(take);
    for k in 0..take {
        let (case_num, val) = sorted[n - 1 - k];
        highest.push(ExtremeValueItem {
            order: k + 1,
            case_number: case_num,
            value: val,
        });
    }

    ExtremeValuesResult {
        variable: var_name.to_string(),
        factor_level,
        highest,
        lowest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extremes_standard() {
        let data = vec![
            (1, 10.0),
            (2, 50.0),
            (3, 20.0),
            (4, 90.0),
            (5, 30.0),
            (6, 80.0),
            (7, 40.0),
            (8, 70.0),
            (9, 60.0),
            (10, 100.0),
        ];
        let res = compute_extreme_values("score", None, &data);

        assert_eq!(res.lowest.len(), 5);
        assert_eq!(res.lowest[0].order, 1);
        assert_eq!(res.lowest[0].case_number, 1);
        assert_eq!(res.lowest[0].value, 10.0);
        assert_eq!(res.lowest[4].order, 5);
        assert_eq!(res.lowest[4].value, 50.0);

        assert_eq!(res.highest.len(), 5);
        assert_eq!(res.highest[0].order, 1);
        assert_eq!(res.highest[0].case_number, 10);
        assert_eq!(res.highest[0].value, 100.0);
        assert_eq!(res.highest[4].order, 5);
        assert_eq!(res.highest[4].value, 60.0);
    }

    #[test]
    fn test_extremes_fewer_than_5() {
        let data = vec![(2, 15.0), (5, 5.0), (8, 25.0)];
        let res = compute_extreme_values("v", Some("Group1".to_string()), &data);

        assert_eq!(res.lowest.len(), 3);
        assert_eq!(res.lowest[0].case_number, 5);
        assert_eq!(res.lowest[0].value, 5.0);

        assert_eq!(res.highest.len(), 3);
        assert_eq!(res.highest[0].case_number, 8);
        assert_eq!(res.highest[0].value, 25.0);
    }

    #[test]
    fn test_extremes_with_ties() {
        let data = vec![
            (1, 10.0),
            (2, 10.0),
            (3, 10.0),
            (4, 20.0),
            (5, 30.0),
            (6, 30.0),
        ];
        let res = compute_extreme_values("ties", None, &data);
        assert_eq!(res.lowest[0].case_number, 1);
        assert_eq!(res.lowest[1].case_number, 2);
        assert_eq!(res.lowest[2].case_number, 3);
    }
}
