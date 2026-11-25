use crate::bootstrap::normal_cdf;

/// Pairwise column proportion z-test within each row.
/// Returns a matrix of subscript group strings (e.g. `"a"`, `"a, b"`) for each cell `(i, j)`.
/// Cells whose column proportions do not differ significantly (at alpha = 0.05) share a letter.
pub fn compute_column_proportion_groups(
    obs: &[Vec<f64>],
    col_totals: &[f64],
    bonferroni: bool,
) -> Vec<Vec<Option<String>>> {
    let nr = obs.len();
    let nc = if nr > 0 { obs[0].len() } else { 0 };
    let mut result = vec![vec![None; nc]; nr];
    if nr == 0 || nc < 2 {
        return result;
    }

    let num_pairs = (nc * (nc - 1)) / 2;
    let alpha = if bonferroni && num_pairs > 0 {
        0.05 / (num_pairs as f64)
    } else {
        0.05
    };

    for i in 0..nr {
        let mut adj = vec![vec![true; nc]; nc];
        for j in 0..nc {
            for k in (j + 1)..nc {
                let n1 = col_totals[j];
                let n2 = col_totals[k];
                if n1 <= 0.0 || n2 <= 0.0 {
                    continue;
                }
                let p1 = obs[i][j] / n1;
                let p2 = obs[i][k] / n2;
                let p_pool = (obs[i][j] + obs[i][k]) / (n1 + n2);
                if p_pool <= 0.0 || p_pool >= 1.0 {
                    continue;
                }
                let se = (p_pool * (1.0 - p_pool) * (1.0 / n1 + 1.0 / n2)).sqrt();
                if se <= 0.0 {
                    continue;
                }
                let z = (p1 - p2).abs() / se;
                let p_val = 2.0 * (1.0 - normal_cdf(z));
                if p_val < alpha {
                    adj[j][k] = false;
                    adj[k][j] = false;
                }
            }
        }

        // Assign column letters ("a", "b", ...) to maximal cliques or pairwise non-different sets
        // Standard SPSS style: each column j gets its own letter if any pair differs,
        // and shares letters with columns that do not differ from it.
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for j in 0..nc {
            let mut placed = false;
            for g in &mut groups {
                if g.iter().all(|&m| adj[j][m]) {
                    g.push(j);
                    placed = true;
                    break;
                }
            }
            if !placed {
                groups.push(vec![j]);
            }
        }
        // Also ensure every non-significant pair shares at least one letter
        for j in 0..nc {
            for k in (j + 1)..nc {
                if adj[j][k] && !groups.iter().any(|g| g.contains(&j) && g.contains(&k)) {
                    groups.push(vec![j, k]);
                }
            }
        }

        for j in 0..nc {
            let mut letters = Vec::new();
            for (g_idx, g) in groups.iter().enumerate() {
                if g.contains(&j) {
                    letters.push(col_letter(g_idx));
                }
            }
            if !letters.is_empty() {
                result[i][j] = Some(letters.join(", "));
            }
        }
    }

    result
}

fn col_letter(idx: usize) -> String {
    let c = (b'a' + (idx % 26) as u8) as char;
    if idx < 26 {
        c.to_string()
    } else {
        format!("{}{}", c, idx / 26)
    }
}
