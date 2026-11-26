use crate::anova_oneway::AlgoError;
use crate::turf::{estimate_turf_operations, evaluate_exact_turf, VarBitset};
use crate::turf_types::{TurfCombinationRow, TurfOptions};
use std::collections::HashSet;

pub(crate) fn run_turf_heats(
    pool: &[VarBitset],
    case_weights: &[f64],
    total_valid_cases: f64,
    has_importance: bool,
    opts: &TurfOptions,
    max_k: usize,
) -> Result<Vec<(usize, Vec<TurfCombinationRow>)>, AlgoError> {
    let forced: Vec<VarBitset> = pool.iter().filter(|v| v.is_forced).cloned().collect();
    let free: Vec<VarBitset> = pool.iter().filter(|v| !v.is_forced).cloned().collect();

    if free.is_empty() {
        return Ok(evaluate_exact_turf(
            pool,
            case_weights,
            total_valid_cases,
            has_importance,
            opts,
            max_k,
        ));
    }

    let mut current_free = free;
    let max_ops_u128 = opts.max_operations as u128;
    let heat_size = opts.advanced_size.max(2);
    let mut winners_per_heat = opts.advanced_winners.max(1);

    for _round in 0..16 {
        let mut combined_pool = forced.clone();
        combined_pool.extend(current_free.iter().cloned());
        let est = estimate_turf_operations(combined_pool.len(), forced.len(), max_k);
        if est <= max_ops_u128 {
            return Ok(evaluate_exact_turf(
                &combined_pool,
                case_weights,
                total_valid_cases,
                has_importance,
                opts,
                max_k,
            ));
        }

        let eff_heat_size = if opts.advanced_auto {
            auto_heat_size(heat_size, forced.len(), max_k, max_ops_u128)
        } else {
            heat_size
        };

        let n_heats = current_free.len().div_ceil(eff_heat_size).max(2);
        let chunk_size = current_free.len().div_ceil(n_heats);
        let mut next_free: Vec<VarBitset> = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        for chunk in current_free.chunks(chunk_size) {
            let mut heat_pool = forced.clone();
            heat_pool.extend(chunk.iter().cloned());
            let heat_max_k = max_k.min(heat_pool.len());
            let heat_opts = TurfOptions {
                show_only_max_combo: false,
                num_to_display: Some(10),
                ..opts.clone()
            };
            let heat_rows = evaluate_exact_turf(
                &heat_pool,
                case_weights,
                total_valid_cases,
                has_importance,
                &heat_opts,
                heat_max_k,
            );
            let picked = pick_heat_winners(chunk, &heat_rows, winners_per_heat);
            for w in picked {
                if seen_names.insert(w.name.clone()) {
                    next_free.push(w);
                }
            }
        }

        if next_free.len() >= current_free.len() {
            if winners_per_heat > 1 {
                winners_per_heat = 1;
                continue;
            }
            if opts.limit_exceeded_abort {
                return Err(AlgoError::InvalidInput(format!(
                    "TURF heats could not reduce problem below MAXSETS ({})",
                    opts.max_operations
                )));
            }
            break;
        }
        current_free = next_free;
    }

    let mut final_pool = forced;
    final_pool.extend(current_free);
    Ok(evaluate_exact_turf(
        &final_pool,
        case_weights,
        total_valid_cases,
        has_importance,
        opts,
        max_k,
    ))
}

fn auto_heat_size(base_size: usize, n_forced: usize, max_k: usize, max_ops: u128) -> usize {
    let mut size = base_size.max(2);
    while size > 2 {
        let est = estimate_turf_operations(n_forced + size, n_forced, max_k.min(n_forced + size));
        if est <= max_ops {
            return size;
        }
        size = (size * 3 / 4).max(2);
        if size == 2 {
            break;
        }
    }
    size
}

fn pick_heat_winners(
    chunk: &[VarBitset],
    rows_by_size: &[(usize, Vec<TurfCombinationRow>)],
    count: usize,
) -> Vec<VarBitset> {
    let mut winners = Vec::new();
    let mut seen = HashSet::new();

    if let Some((_, best_rows)) = rows_by_size.last() {
        for row in best_rows {
            for name in &row.variables {
                if winners.len() >= count {
                    return winners;
                }
                if let Some(v) = chunk.iter().find(|v| &v.name == name) {
                    if seen.insert(v.name.clone()) {
                        winners.push(v.clone());
                    }
                }
            }
        }
    }

    if winners.len() < count {
        let mut sorted = chunk.to_vec();
        sorted.sort_by(|a, b| b.reach.total_cmp(&a.reach));
        for v in sorted {
            if winners.len() >= count {
                break;
            }
            if seen.insert(v.name.clone()) {
                winners.push(v);
            }
        }
    }

    winners
}
