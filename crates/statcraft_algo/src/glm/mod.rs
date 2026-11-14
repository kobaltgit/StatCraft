pub mod design_matrix;
pub mod eigen;
pub mod multivariate;
pub mod repeated;
pub mod type3_ss;
pub mod types;
pub mod univariate;

#[cfg(test)]
mod glm_tests;

pub use multivariate::compute_multivariate_glm;
pub use repeated::compute_repeated_glm;
pub use types::*;
pub use univariate::compute_univariate_glm;
