use crate::anova_oneway::AlgoError;
use crate::turf_heats::run_turf_heats;
use crate::turf_types::{TurfCombinationRow, TurfCycleResult, TurfOptions, TurfResult};
use statcraft_core::{DataSet, RawValue, VariableMetadata};
use std::cmp::Ordering;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub(crate) struct VarBitset {
    pub name: String,
    pub bits: Vec<u64>,
    pub reach: f64,
    pub freq: f64,
    pub weight: f64,
    pub is_forced: bool,
}

pub fn estimate_turf_operations(n_pool: usize, n_forced: usize, max_k: usize) -> u128 {
    if n_pool == 0 || max_k == 0 {
        return 0;
    }
    let eff_forced = n_forced.min(n_pool);
    let eff_max_k = max_k.min(n_pool);
    if eff_forced > eff_max_k {
        return 0;
    }
    let n_free = n_pool - eff_forced;
    let start_k = eff_forced.max(1);
    let mut total: u128 = 0;
    for k in start_k..=eff_max_k {
        let r = k - eff_forced;
        let comb = binom_u128(n_free, r);
        total = total.saturating_add(comb);
    }
    total
}

fn binom_u128(n: usize, k: usize) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut res: u128 = 1;
    for i in 0..k {
        let num = (n - i) as u128;
        let den = (i + 1) as u128;
        res = match res.checked_mul(num) {
            Some(v) => v / den,
            None => return u128::MAX,
        };
    }
    res
}

fn is_positive_value(raw: &RawValue, meta: Option<&VariableMetadata>, criteria: &[String]) -> bool {
    if let Some(m) = meta {
        if m.missing.is_missing(raw) {
            return false;
        }
    }
    match raw {
        RawValue::Null => false,
        RawValue::Number(n) => {
            if !n.is_finite() {
                return false;
            }
            let disp = raw.as_display_string();
            criteria.iter().any(|c| {
                let trimmed = c.trim();
                if let Ok(parsed) = trimmed.parse::<f64>() {
                    if parsed.is_finite() && (n - parsed).abs() <= 1e-9 {
                        return true;
                    }
                }
                disp == trimmed
            })
        }
        RawValue::Text(s) => {
            let s_trim = s.trim();
            criteria.iter().any(|c| s_trim == c.trim())
        }
    }
}

pub(crate) fn union_reach(bits: &[u64], case_weights: &[f64]) -> f64 {
    let mut reach = 0.0;
    for (word_idx, &word) in bits.iter().enumerate() {
        let mut w = word;
        let base = word_idx * 64;
        while w != 0 {
            let bit = w.trailing_zeros() as usize;
            let case_idx = base + bit;
            if let Some(&cw) = case_weights.get(case_idx) {
                reach += cw;
            }
            w &= w - 1;
        }
    }
    reach
}

fn cmp_f64_desc(a: f64, b: f64) -> Ordering {
    b.total_cmp(&a)
}

pub(crate) fn rank_and_truncate(rows: &mut Vec<TurfCombinationRow>, limit: Option<usize>) {
    rows.sort_by(|a, b| {
        let ord = match (a.weighted_score, b.weighted_score) {
            (Some(wa), Some(wb)) => cmp_f64_desc(wa, wb),
            _ => Ordering::Equal,
        };
        ord.then_with(|| cmp_f64_desc(a.reach_count, b.reach_count))
            .then_with(|| cmp_f64_desc(a.frequency_count, b.frequency_count))
            .then_with(|| a.variables.cmp(&b.variables))
    });
    if let Some(m) = limit {
        rows.truncate(m);
    }
}

pub(crate) fn evaluate_exact_turf(
    vars: &[VarBitset],
    case_weights: &[f64],
    total_valid_cases: f64,
    has_importance: bool,
    opts: &TurfOptions,
    max_k: usize,
) -> Vec<(usize, Vec<TurfCombinationRow>)> {
    let forced_indices: Vec<usize> = vars
        .iter()
        .enumerate()
        .filter(|(_, v)| v.is_forced)
        .map(|(i, _)| i)
        .collect();
    let free_indices: Vec<usize> = vars
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_forced)
        .map(|(i, _)| i)
        .collect();

    let n_forced = forced_indices.len();
    let eff_max_k = max_k.min(vars.len());
    let start_k = n_forced.max(1);
    if start_k > eff_max_k {
        return Vec::new();
    }

    let word_len = case_weights.len().div_ceil(64);
    let mut base_bits = vec![0u64; word_len];
    let mut base_freq = 0.0;
    let mut base_w_score = 0.0;
    let mut base_names = Vec::with_capacity(n_forced);

    for &idx in &forced_indices {
        let v = &vars[idx];
        for (dst, &src) in base_bits.iter_mut().zip(v.bits.iter()) {
            *dst |= src;
        }
        base_freq += v.freq;
        base_w_score += v.weight * v.freq;
        base_names.push(v.name.clone());
    }

    let total_positive_responses: f64 = vars.iter().map(|v| v.freq).sum();

    let mut by_size: Vec<(usize, Vec<TurfCombinationRow>)> = Vec::new();
    for k in start_k..=eff_max_k {
        if opts.show_only_max_combo && k != eff_max_k {
            continue;
        }
        let need_free = k - n_forced;
        if need_free > free_indices.len() {
            continue;
        }
        let mut rows = Vec::new();
        let mut cur_bits = base_bits.clone();
        let mut cur_names = base_names.clone();
        let mut ctx = ComboSearchCtx {
            vars,
            free_indices: &free_indices,
            case_weights,
            total_valid_cases,
            total_positive_responses,
            has_importance,
            target_k: k,
            rows: &mut rows,
        };
        if need_free == 0 {
            push_combo_row(&mut ctx, &cur_bits, &cur_names, base_freq, base_w_score);
        } else {
            dfs_combos(
                &mut ctx,
                0,
                need_free,
                &mut cur_bits,
                &mut cur_names,
                base_freq,
                base_w_score,
            );
        }
        rank_and_truncate(&mut rows, opts.num_to_display);
        by_size.push((k, rows));
    }
    by_size
}

struct ComboSearchCtx<'a> {
    vars: &'a [VarBitset],
    free_indices: &'a [usize],
    case_weights: &'a [f64],
    total_valid_cases: f64,
    total_positive_responses: f64,
    has_importance: bool,
    target_k: usize,
    rows: &'a mut Vec<TurfCombinationRow>,
}

fn push_combo_row(
    ctx: &mut ComboSearchCtx<'_>,
    bits: &[u64],
    names: &[String],
    freq: f64,
    w_score: f64,
) {
    let reach_count = union_reach(bits, ctx.case_weights);
    let reach_pct = if ctx.total_valid_cases > 0.0 {
        100.0 * reach_count / ctx.total_valid_cases
    } else {
        0.0
    };
    let frequency_pct = if ctx.total_positive_responses > 0.0 {
        100.0 * freq / ctx.total_positive_responses
    } else {
        0.0
    };
    let mean_responses_per_reached = if reach_count > 0.0 {
        freq / reach_count
    } else {
        0.0
    };
    ctx.rows.push(TurfCombinationRow {
        combo_size: ctx.target_k,
        variables: names.to_vec(),
        reach_count,
        reach_pct,
        frequency_count: freq,
        frequency_pct,
        mean_responses_per_reached,
        weighted_score: if ctx.has_importance {
            Some(w_score)
        } else {
            None
        },
    });
}

fn dfs_combos(
    ctx: &mut ComboSearchCtx<'_>,
    start_pos: usize,
    remaining: usize,
    cur_bits: &mut [u64],
    cur_names: &mut Vec<String>,
    cur_freq: f64,
    cur_w_score: f64,
) {
    if remaining == 0 {
        push_combo_row(ctx, cur_bits, cur_names, cur_freq, cur_w_score);
        return;
    }
    let end = ctx.free_indices.len().saturating_sub(remaining) + 1;
    let saved_bits = cur_bits.to_vec();
    for pos in start_pos..end {
        let var_idx = ctx.free_indices[pos];
        let v = &ctx.vars[var_idx];
        for (dst, &src) in cur_bits.iter_mut().zip(v.bits.iter()) {
            *dst |= src;
        }
        cur_names.push(v.name.clone());
        dfs_combos(
            ctx,
            pos + 1,
            remaining - 1,
            cur_bits,
            cur_names,
            cur_freq + v.freq,
            cur_w_score + v.weight * v.freq,
        );
        cur_names.pop();
        cur_bits.copy_from_slice(&saved_bits);
    }
}

pub fn compute_turf(dataset: &DataSet, opts: &TurfOptions) -> Result<TurfResult, AlgoError> {
    if opts.variables.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one variable is required for TURF analysis".to_string(),
        ));
    }
    if opts.positive_values.is_empty() {
        return Err(AlgoError::InvalidInput(
            "At least one positive response criterion is required".to_string(),
        ));
    }

    let mut seen = HashSet::new();
    let mut col_indices = Vec::with_capacity(opts.variables.len());
    for name in &opts.variables {
        if !seen.insert(name.as_str()) {
            return Err(AlgoError::InvalidInput(format!(
                "Duplicate variable '{name}' in TURF variable list"
            )));
        }
        let idx = dataset
            .dictionary
            .find_index(name)
            .ok_or_else(|| AlgoError::InvalidInput(format!("Variable '{name}' not found")))?;
        col_indices.push((name.clone(), idx));
    }

    let has_importance = !opts.importance_weights.is_empty();
    if has_importance && opts.weights_strict {
        for (name, _) in &col_indices {
            if !opts.importance_weights.contains_key(name) {
                return Err(AlgoError::InvalidInput(format!(
                    "Missing importance weight for variable '{name}' in STRICT mode"
                )));
            }
        }
    }

    let active_rows = dataset.active_row_indices();
    let mut case_weights = Vec::with_capacity(active_rows.len());
    let mut valid_rows = Vec::with_capacity(active_rows.len());
    for &r in &active_rows {
        let w = dataset.row_weight(r);
        if w > 0.0 && w.is_finite() {
            case_weights.push(w);
            valid_rows.push(r);
        }
    }

    let total_cases_unweighted = valid_rows.len();
    let total_valid_cases: f64 = case_weights.iter().sum();
    if total_cases_unweighted == 0 || total_valid_cases <= 0.0 {
        return Err(AlgoError::InsufficientData(
            "No valid active cases with positive weight".to_string(),
        ));
    }

    let word_len = valid_rows.len().div_ceil(64);
    let mut all_vars = Vec::with_capacity(col_indices.len());
    let mut excluded_variables = Vec::new();

    for (pos_idx, (name, col_idx)) in col_indices.iter().enumerate() {
        let meta = dataset.dictionary.get(*col_idx);
        let mut bits = vec![0u64; word_len];
        let mut reach = 0.0;
        for (case_i, &row_idx) in valid_rows.iter().enumerate() {
            if let Some(raw) = dataset.get_cell(row_idx, *col_idx) {
                if is_positive_value(raw, meta, &opts.positive_values) {
                    bits[case_i / 64] |= 1u64 << (case_i % 64);
                    reach += case_weights[case_i];
                }
            }
        }
        let pos_pct = 100.0 * reach / total_valid_cases;
        if let Some(min_pct) = opts.min_positive_pct {
            if pos_pct < min_pct {
                excluded_variables.push((name.clone(), pos_pct));
                continue;
            }
        }
        let weight = if has_importance {
            opts.importance_weights.get(name).copied().unwrap_or(1.0)
        } else {
            1.0
        };
        all_vars.push(VarBitset {
            name: name.clone(),
            bits,
            reach,
            freq: reach,
            weight,
            is_forced: pos_idx < opts.always_required,
        });
    }

    if all_vars.is_empty() {
        return Err(AlgoError::InsufficientData(
            "All variables were excluded by MINPERCENT threshold".to_string(),
        ));
    }

    let mut current_pool = all_vars;
    let mut removed_so_far: Vec<String> = Vec::new();
    let mut cycles = Vec::new();

    for cycle_idx in 0..=opts.removal_cycles {
        if current_pool.is_empty() {
            break;
        }
        let n_forced = current_pool.iter().filter(|v| v.is_forced).count();
        let max_k = opts
            .max_combinations
            .unwrap_or(current_pool.len())
            .clamp(1, current_pool.len());
        let est_ops = estimate_turf_operations(current_pool.len(), n_forced, max_k);
        let max_ops_u128 = opts.max_operations as u128;

        let (rows_by_size, used_heats) = if est_ops > max_ops_u128 {
            if opts.use_advanced_selection {
                let rows = run_turf_heats(
                    &current_pool,
                    &case_weights,
                    total_valid_cases,
                    has_importance,
                    opts,
                    max_k,
                )?;
                (rows, true)
            } else if opts.limit_exceeded_abort {
                return Err(AlgoError::InvalidInput(format!(
                    "Estimated TURF operations ({est_ops}) exceed MAXSETS ({})",
                    opts.max_operations
                )));
            } else {
                let rows = evaluate_exact_turf(
                    &current_pool,
                    &case_weights,
                    total_valid_cases,
                    has_importance,
                    opts,
                    max_k,
                );
                (rows, false)
            }
        } else {
            let rows = evaluate_exact_turf(
                &current_pool,
                &case_weights,
                total_valid_cases,
                has_importance,
                opts,
                max_k,
            );
            (rows, false)
        };

        let next_to_remove = pick_removal_candidate(&current_pool, &rows_by_size);
        cycles.push(TurfCycleResult {
            cycle_index: cycle_idx,
            removed_variables: removed_so_far.clone(),
            rows_by_size,
            used_heats,
            estimated_operations: est_ops,
        });

        if cycle_idx < opts.removal_cycles {
            if let Some(rem_name) = next_to_remove {
                removed_so_far.push(rem_name.clone());
                current_pool.retain(|v| v.name != rem_name);
            } else {
                break;
            }
        }
    }

    Ok(TurfResult {
        total_valid_cases,
        total_cases_unweighted,
        excluded_variables,
        cycles,
    })
}

fn pick_removal_candidate(
    pool: &[VarBitset],
    rows_by_size: &[(usize, Vec<TurfCombinationRow>)],
) -> Option<String> {
    let best_row = rows_by_size.last().and_then(|(_, rows)| rows.first())?;
    let mut best_name: Option<String> = None;
    let mut best_reach = f64::NEG_INFINITY;
    for name in &best_row.variables {
        if let Some(v) = pool.iter().find(|v| &v.name == name && !v.is_forced) {
            if v.reach > best_reach {
                best_reach = v.reach;
                best_name = Some(v.name.clone());
            }
        }
    }
    best_name
}
