//! K-Means Cluster Analysis (QUICK CLUSTER).
//! Implements iterative Euclidean partitioning, ANOVA significance tests,
//! distance matrix, and cluster membership according to IBM SPSS Statistics 24 Algorithms Guide.

use crate::anova_oneway::{f_distribution_sf, AlgoError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KMeansOptions {
    pub clusters: usize,
    pub max_iterations: usize,
    pub convergence_criterion: f64,
}

impl Default for KMeansOptions {
    fn default() -> Self {
        Self {
            clusters: 3,
            max_iterations: 10,
            convergence_criterion: 0.0001,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMeansIterationRecord {
    pub iteration: usize,
    pub cluster_changes: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMeansAnovaRow {
    pub variable: String,
    pub cluster_ms: f64,
    pub cluster_df: usize,
    pub error_ms: f64,
    pub error_df: usize,
    pub f_stat: f64,
    pub p_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KMeansResult {
    pub variables: Vec<String>,
    pub k: usize,
    pub iterations_performed: usize,
    pub initial_centers: Vec<Vec<f64>>,
    pub final_centers: Vec<Vec<f64>>,
    pub iteration_history: Vec<KMeansIterationRecord>,
    pub center_distances: Vec<Vec<f64>>,
    pub cluster_sizes: Vec<usize>,
    pub cluster_assignments: Vec<usize>,
    pub distances_to_center: Vec<f64>,
    pub valid_cases_count: usize,
    pub anova_table: Vec<KMeansAnovaRow>,
}

/// Euclidean distance between two vectors.
fn euclidean_dist(a: &[f64], b: &[f64]) -> f64 {
    let mut sum = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = x - y;
        sum += d * d;
    }
    sum.sqrt()
}

/// Compute K-Means clustering.
///
/// `data` is slice of rows (each row is P variables).
/// `var_names` is names of P variables.
pub fn compute_kmeans(
    data: &[Vec<f64>],
    var_names: &[String],
    options: &KMeansOptions,
) -> Result<KMeansResult, AlgoError> {
    let n = data.len();
    if n == 0 {
        return Err(AlgoError::InsufficientData("Data is empty".to_string()));
    }
    let p = var_names.len();
    if p == 0 {
        return Err(AlgoError::InvalidInput(
            "No variables specified".to_string(),
        ));
    }
    for row in data {
        if row.len() != p {
            return Err(AlgoError::InvalidInput(
                "Inconsistent row length in data matrix".to_string(),
            ));
        }
    }

    let k = options.clusters;
    if k < 2 {
        return Err(AlgoError::InvalidInput(
            "Number of clusters K must be at least 2".to_string(),
        ));
    }
    if n < k {
        return Err(AlgoError::InsufficientData(format!(
            "Number of valid cases ({n}) must be greater than or equal to clusters K ({k})"
        )));
    }

    // 1. Initial centers selection (deterministic maximin distance heuristic)
    let mut centers: Vec<Vec<f64>> = Vec::with_capacity(k);
    centers.push(data[0].clone());

    while centers.len() < k {
        let mut best_idx = 0;
        let mut best_min_dist = -1.0;

        for (idx, row) in data.iter().enumerate() {
            let mut min_dist = f64::MAX;
            for c in &centers {
                let d = euclidean_dist(row, c);
                if d < min_dist {
                    min_dist = d;
                }
            }
            if min_dist > best_min_dist {
                best_min_dist = min_dist;
                best_idx = idx;
            }
        }
        centers.push(data[best_idx].clone());
    }

    let initial_centers = centers.clone();

    // 2. Iterations
    let mut assignments = vec![0usize; n];
    let mut iteration_history = Vec::new();
    let mut iterations_performed = 0;

    let max_iter = if options.max_iterations == 0 {
        10
    } else {
        options.max_iterations
    };
    let tol = options.convergence_criterion.max(0.0);

    for iter in 1..=max_iter {
        iterations_performed = iter;

        // Step A: Assign points to closest center
        for (i, row) in data.iter().enumerate() {
            let mut min_d = f64::MAX;
            let mut best_c = 0;
            for (c_idx, c) in centers.iter().enumerate() {
                let d = euclidean_dist(row, c);
                if d < min_d {
                    min_d = d;
                    best_c = c_idx;
                }
            }
            assignments[i] = best_c;
        }

        // Step B: Recompute centers and measure shift
        let mut new_centers = vec![vec![0.0; p]; k];
        let mut counts = vec![0usize; k];

        for (i, row) in data.iter().enumerate() {
            let c_idx = assignments[i];
            counts[c_idx] += 1;
            for j in 0..p {
                new_centers[c_idx][j] += row[j];
            }
        }

        let mut cluster_changes = vec![0.0; k];
        let mut max_shift: f64 = 0.0;

        for c_idx in 0..k {
            if counts[c_idx] > 0 {
                let count_f = counts[c_idx] as f64;
                for val in &mut new_centers[c_idx] {
                    *val /= count_f;
                }
            } else {
                // Empty cluster fallback: keep previous center
                new_centers[c_idx] = centers[c_idx].clone();
            }

            let shift = euclidean_dist(&centers[c_idx], &new_centers[c_idx]);
            cluster_changes[c_idx] = shift;
            if shift > max_shift {
                max_shift = shift;
            }
        }

        iteration_history.push(KMeansIterationRecord {
            iteration: iter,
            cluster_changes,
        });

        centers = new_centers;

        if max_shift <= tol {
            break;
        }
    }

    // Final assignment pass to guarantee consistency with final centers
    let mut cluster_sizes = vec![0usize; k];
    let mut distances_to_center = vec![0.0; n];
    let mut final_assignments = vec![0usize; n];

    for (i, row) in data.iter().enumerate() {
        let mut min_d = f64::MAX;
        let mut best_c = 0;
        for (c_idx, c) in centers.iter().enumerate() {
            let d = euclidean_dist(row, c);
            if d < min_d {
                min_d = d;
                best_c = c_idx;
            }
        }
        final_assignments[i] = best_c + 1; // 1-based cluster ID
        distances_to_center[i] = min_d;
        cluster_sizes[best_c] += 1;
    }

    // 3. Center distances matrix (K x K)
    let mut center_distances = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in 0..k {
            if i != j {
                center_distances[i][j] = euclidean_dist(&centers[i], &centers[j]);
            }
        }
    }

    // 4. ANOVA Table
    let mut anova_table = Vec::with_capacity(p);
    let df_cluster = if k > 1 { k - 1 } else { 1 };
    let df_error = if n > k { n - k } else { 1 };

    for j in 0..p {
        // Overall mean for variable j
        let mut grand_sum = 0.0;
        for row in data {
            grand_sum += row[j];
        }
        let grand_mean = grand_sum / (n as f64);

        // SS between (cluster)
        let mut ss_between = 0.0;
        for c_idx in 0..k {
            let size = cluster_sizes[c_idx] as f64;
            let diff = centers[c_idx][j] - grand_mean;
            ss_between += size * diff * diff;
        }

        // SS within (error)
        let mut ss_within = 0.0;
        for (i, row) in data.iter().enumerate() {
            let c_idx = final_assignments[i] - 1;
            let diff = row[j] - centers[c_idx][j];
            ss_within += diff * diff;
        }

        let ms_between = ss_between / (df_cluster as f64);
        let ms_error = if df_error > 0 {
            ss_within / (df_error as f64)
        } else {
            0.0
        };

        let f_stat = if ms_error > 1e-12 {
            ms_between / ms_error
        } else {
            0.0
        };

        let p_val = if ms_error > 1e-12 && f_stat > 0.0 {
            f_distribution_sf(f_stat, df_cluster as f64, df_error as f64)
        } else {
            1.0
        };

        anova_table.push(KMeansAnovaRow {
            variable: var_names[j].clone(),
            cluster_ms: ms_between,
            cluster_df: df_cluster,
            error_ms: ms_error,
            error_df: df_error,
            f_stat,
            p_value: p_val,
        });
    }

    Ok(KMeansResult {
        variables: var_names.to_vec(),
        k,
        iterations_performed,
        initial_centers,
        final_centers: centers,
        iteration_history,
        center_distances,
        cluster_sizes,
        cluster_assignments: final_assignments,
        distances_to_center,
        valid_cases_count: n,
        anova_table,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmeans_basic() {
        let data = vec![
            vec![1.0, 1.0],
            vec![1.5, 2.0],
            vec![1.2, 0.8],
            vec![10.0, 10.0],
            vec![10.5, 9.5],
            vec![9.8, 10.2],
        ];
        let var_names = vec!["x".to_string(), "y".to_string()];
        let options = KMeansOptions {
            clusters: 2,
            max_iterations: 10,
            convergence_criterion: 0.0001,
        };

        let res = compute_kmeans(&data, &var_names, &options).expect("kmeans calculation failed");
        assert_eq!(res.k, 2);
        assert_eq!(res.valid_cases_count, 6);
        assert_eq!(res.cluster_sizes, vec![3, 3]);

        // Points 0,1,2 should belong to one cluster, 3,4,5 to another
        assert_eq!(res.cluster_assignments[0], res.cluster_assignments[1]);
        assert_eq!(res.cluster_assignments[1], res.cluster_assignments[2]);
        assert_eq!(res.cluster_assignments[3], res.cluster_assignments[4]);
        assert_eq!(res.cluster_assignments[4], res.cluster_assignments[5]);
        assert_ne!(res.cluster_assignments[0], res.cluster_assignments[3]);

        // ANOVA F should be very large
        assert!(res.anova_table[0].f_stat > 100.0);
        assert!(res.anova_table[0].p_value < 0.001);
    }
}
