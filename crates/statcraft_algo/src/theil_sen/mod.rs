pub mod engine;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::{compute_theil_sen, compute_theil_sen_bootstrap};
pub use types::{
    TheilSenBootstrap, TheilSenOptions, TheilSenPoint, TheilSenResult, MAX_EXACT_PAIRS_N,
};
