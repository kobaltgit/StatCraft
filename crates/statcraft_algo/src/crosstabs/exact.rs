use crate::cox::normal_quantile;
use crate::ttest_onesample::ln_gamma;
use std::time::{Duration, Instant};

/// Fisher's Exact Test two-sided p-value for a 2x2 contingency table.
pub fn fisher_exact_2x2(a: usize, b: usize, c: usize, d: usize) -> f64 {
    let r1 = a + b;
    let r2 = c + d;
    let c1 = a + c;
    let n = r1 + r2;

    if n == 0 {
        return 1.0;
    }

    let log_hyper = |k: usize| -> f64 {
        let k_i = k as f64;
        let r1_f = r1 as f64;
        let r2_f = r2 as f64;
        let c1_f = c1 as f64;
        let c2_f = (b + d) as f64;
        let n_f = n as f64;

        ln_gamma(r1_f + 1.0) + ln_gamma(r2_f + 1.0) + ln_gamma(c1_f + 1.0) + ln_gamma(c2_f + 1.0)
            - ln_gamma(n_f + 1.0)
            - ln_gamma(k_i + 1.0)
            - ln_gamma(r1_f - k_i + 1.0)
            - ln_gamma(c1_f - k_i + 1.0)
            - ln_gamma(r2_f - c1_f + k_i + 1.0)
    };

    let p_observed = log_hyper(a).exp();
    let min_k = c1.saturating_sub(r2);
    let max_k = std::cmp::min(r1, c1);

    let mut p_two_sided = 0.0_f64;
    for k in min_k..=max_k {
        let p_k = log_hyper(k).exp();
        if p_k <= p_observed * (1.0 + 1.0e-11) {
            p_two_sided += p_k;
        }
    }

    p_two_sided.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy)]
pub struct MonteCarloResult {
    pub p_value: f64,
    pub ci_lower: f64,
    pub ci_upper: f64,
}

struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    fn next_f64(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
    }
}

/// Patefield's (1981) algorithm AS 159 for sampling random r x c contingency tables
/// with fixed row and column marginals.
fn sample_patefield(
    row_totals: &[usize],
    col_totals: &[usize],
    rng: &mut SplitMix64,
) -> Vec<Vec<usize>> {
    let nrow = row_totals.len();
    let ncol = col_totals.len();
    let ntotal: usize = row_totals.iter().sum();
    let mut table = vec![vec![0usize; ncol]; nrow];
    if ntotal == 0 || nrow == 0 || ncol == 0 {
        return table;
    }

    let mut fact = vec![0.0_f64; ntotal + 1];
    for i in 1..=ntotal {
        fact[i] = fact[i - 1] + (i as f64).ln();
    }

    let mut jwork = col_totals.to_vec();
    let mut jc = ntotal;

    for l in 0..(nrow - 1) {
        let mut ia = row_totals[l];
        let mut ic = jc;
        jc -= ia;

        for m in 0..(ncol - 1) {
            let id = jwork[m];
            let ie = ic;
            ic -= id;
            let ib = ie.saturating_sub(ia);

            if ie == 0 || ia == 0 || id == 0 {
                table[l][m] = 0;
                continue;
            }

            let mut nlm = ((ia * id) as f64 / (ie as f64) + 0.5) as usize;
            let iap = ia + 1;
            let idp = id + 1;
            let igp = idp.saturating_sub(ib);

            let mut x = (fact[iap - 1] + fact[ib] + fact[id] + fact[ic]
                - fact[ie]
                - fact[nlm]
                - fact[ia - nlm]
                - fact[id - nlm]
                - fact[nlm + ib - id])
                .exp();

            let mut r = rng.next_f64();
            let mut sum_pr = x;
            let mut y = x;
            let mut nll = nlm;

            loop {
                if r <= sum_pr {
                    table[l][m] = nlm;
                    break;
                }
                let mut lsp = false;
                let mut lsm = false;
                if nlm < ia.min(id) {
                    nlm += 1;
                    x = x * ((iap - nlm) as f64 * (idp - nlm) as f64)
                        / ((nlm as f64) * ((nlm + ib - id) as f64));
                    sum_pr += x;
                    if r <= sum_pr {
                        table[l][m] = nlm;
                        break;
                    }
                    lsp = true;
                }
                if nll > igp {
                    let prev_nll = nll;
                    nll -= 1;
                    y = y * ((prev_nll as f64) * ((prev_nll + ib - id) as f64))
                        / (((iap - prev_nll) as f64) * ((idp - prev_nll) as f64));
                    sum_pr += y;
                    if r <= sum_pr {
                        table[l][m] = nll;
                        break;
                    }
                    lsm = true;
                }
                if !lsp && !lsm {
                    r = rng.next_f64();
                    sum_pr = 0.0;
                    nlm = ((ia * id) as f64 / (ie as f64) + 0.5) as usize;
                    x = (fact[iap - 1] + fact[ib] + fact[id] + fact[ic]
                        - fact[ie]
                        - fact[nlm]
                        - fact[ia - nlm]
                        - fact[id - nlm]
                        - fact[nlm + ib - id])
                        .exp();
                    sum_pr += x;
                    y = x;
                    nll = nlm;
                }
            }

            ia -= table[l][m];
            jwork[m] -= table[l][m];
        }
        table[l][ncol - 1] = ia;
        jwork[ncol - 1] = jwork[ncol - 1].saturating_sub(ia);
    }

    table[nrow - 1][..ncol].copy_from_slice(&jwork[..ncol]);

    table
}

fn pearson_stat(table: &[Vec<usize>], row_totals: &[usize], col_totals: &[usize], n: f64) -> f64 {
    let mut stat = 0.0;
    for (i, row) in table.iter().enumerate() {
        for (j, &obs) in row.iter().enumerate() {
            let exp = (row_totals[i] as f64 * col_totals[j] as f64) / n;
            if exp > 0.0 {
                let d = (obs as f64) - exp;
                stat += d * d / exp;
            }
        }
    }
    stat
}

/// Monte Carlo estimate of exact p-value for Pearson Chi-Square on an r x c table.
pub fn monte_carlo_rxc(
    obs: &[Vec<usize>],
    row_totals: &[usize],
    col_totals: &[usize],
    confidence: f64,
    samples: usize,
) -> MonteCarloResult {
    let n: usize = row_totals.iter().sum();
    if n == 0 || samples == 0 {
        return MonteCarloResult {
            p_value: 1.0,
            ci_lower: 0.0,
            ci_upper: 1.0,
        };
    }

    let obs_stat = pearson_stat(obs, row_totals, col_totals, n as f64);
    let mut rng = SplitMix64(0x2026_1125_5747_c2af);
    let mut hits = 0usize;

    for _ in 0..samples {
        let sampled = sample_patefield(row_totals, col_totals, &mut rng);
        let stat = pearson_stat(&sampled, row_totals, col_totals, n as f64);
        if stat >= obs_stat - 1.0e-11 {
            hits += 1;
        }
    }

    let p = (hits as f64) / (samples as f64);
    let alpha = (1.0 - confidence.clamp(0.5, 0.9999)) * 0.5;
    let z = normal_quantile(1.0 - alpha).abs();
    let se = (p * (1.0 - p) / (samples as f64)).sqrt();
    MonteCarloResult {
        p_value: p.clamp(0.0, 1.0),
        ci_lower: (p - z * se).clamp(0.0, 1.0),
        ci_upper: (p + z * se).clamp(0.0, 1.0),
    }
}

/// Bounded exact p-value for r x c tables using recursive margin-constrained enumeration.
/// Returns `None` if the time limit is exceeded before completion.
pub fn exact_rxc(
    obs: &[Vec<usize>],
    row_totals: &[usize],
    col_totals: &[usize],
    time_limit_min: Option<u32>,
) -> Option<f64> {
    let nr = row_totals.len();
    let nc = col_totals.len();
    let n: usize = row_totals.iter().sum();
    if nr == 2 && nc == 2 {
        return Some(fisher_exact_2x2(obs[0][0], obs[0][1], obs[1][0], obs[1][1]));
    }
    if n == 0 || nr < 2 || nc < 2 {
        return Some(1.0);
    }

    let deadline =
        Instant::now() + Duration::from_secs(u64::from(time_limit_min.unwrap_or(5)) * 60);
    let mut fact = vec![0.0_f64; n + 1];
    for i in 1..=n {
        fact[i] = fact[i - 1] + (i as f64).ln();
    }

    let obs_stat = pearson_stat(obs, row_totals, col_totals, n as f64);
    let log_const: f64 = row_totals.iter().map(|&r| fact[r]).sum::<f64>()
        + col_totals.iter().map(|&c| fact[c]).sum::<f64>()
        - fact[n];

    let mut cur = vec![vec![0usize; nc]; nr];
    let mut rem_col = col_totals.to_vec();
    let mut p_sum = 0.0_f64;
    let mut steps = 0usize;

    #[allow(clippy::too_many_arguments)]
    fn dfs(
        r: usize,
        c: usize,
        rem_row: usize,
        rem_col: &mut [usize],
        row_totals: &[usize],
        col_totals: &[usize],
        n: f64,
        obs_stat: f64,
        log_const: f64,
        fact: &[f64],
        cur: &mut [Vec<usize>],
        p_sum: &mut f64,
        steps: &mut usize,
        deadline: Instant,
    ) -> bool {
        *steps += 1;
        if (*steps & 4095) == 0 && Instant::now() >= deadline {
            return false;
        }
        let nr = row_totals.len();
        let nc = col_totals.len();
        if r == nr - 1 {
            cur[r][..nc].copy_from_slice(&rem_col[..nc]);
            let stat = pearson_stat(cur, row_totals, col_totals, n);
            if stat >= obs_stat - 1.0e-11 {
                let mut log_denom = 0.0;
                for row in cur.iter() {
                    for &v in row {
                        log_denom += fact[v];
                    }
                }
                *p_sum += (log_const - log_denom).exp();
            }
            return true;
        }
        if c == nc - 1 {
            if rem_row > rem_col[c] {
                return true;
            }
            cur[r][c] = rem_row;
            rem_col[c] -= rem_row;
            let ok = dfs(
                r + 1,
                0,
                row_totals[r + 1],
                rem_col,
                row_totals,
                col_totals,
                n,
                obs_stat,
                log_const,
                fact,
                cur,
                p_sum,
                steps,
                deadline,
            );
            rem_col[c] += rem_row;
            return ok;
        }
        let max_v = rem_row.min(rem_col[c]);
        for v in 0..=max_v {
            cur[r][c] = v;
            rem_col[c] -= v;
            let ok = dfs(
                r,
                c + 1,
                rem_row - v,
                rem_col,
                row_totals,
                col_totals,
                n,
                obs_stat,
                log_const,
                fact,
                cur,
                p_sum,
                steps,
                deadline,
            );
            rem_col[c] += v;
            if !ok {
                return false;
            }
        }
        true
    }

    if dfs(
        0,
        0,
        row_totals[0],
        &mut rem_col,
        row_totals,
        col_totals,
        n as f64,
        obs_stat,
        log_const,
        &fact,
        &mut cur,
        &mut p_sum,
        &mut steps,
        deadline,
    ) {
        Some(p_sum.clamp(0.0, 1.0))
    } else {
        None
    }
}
