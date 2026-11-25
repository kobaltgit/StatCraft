use crate::dialogs::bootstrap_dialog::BootstrapDialogState;
use serde_json::Value;
use statcraft_algo::{
    ExploreBoxplotDisplay, ExploreMissingHandling, ExploreOptions, ExploreOutputDisplay,
    SpreadLevelMode,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExploreDisplayMode {
    #[default]
    Both,
    Statistics,
    Plots,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExploreBoxplotMode {
    #[default]
    FactorLevelsTogether,
    DependentsTogether,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExploreSpreadLevelMode {
    #[default]
    None,
    PowerEstimation,
    Transformed,
    Untransformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExploreSpreadPower {
    #[default]
    NaturalLog,
    InvSqrt,
    Reciprocal,
    Sqrt,
    Square,
    Cube,
}

impl ExploreSpreadPower {
    pub const ALL: [Self; 6] = [
        Self::NaturalLog,
        Self::InvSqrt,
        Self::Reciprocal,
        Self::Sqrt,
        Self::Square,
        Self::Cube,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::NaturalLog => "dialog-explore-power-ln",
            Self::InvSqrt => "dialog-explore-power-inv-sqrt",
            Self::Reciprocal => "dialog-explore-power-reciprocal",
            Self::Sqrt => "dialog-explore-power-sqrt",
            Self::Square => "dialog-explore-power-square",
            Self::Cube => "dialog-explore-power-cube",
        }
    }

    pub fn as_f64(self) -> f64 {
        match self {
            Self::NaturalLog => 0.0,
            Self::InvSqrt => -0.5,
            Self::Reciprocal => -1.0,
            Self::Sqrt => 0.5,
            Self::Square => 2.0,
            Self::Cube => 3.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExploreMissingMode {
    #[default]
    Listwise,
    Pairwise,
    Report,
}

impl ExploreMissingMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listwise => "listwise",
            Self::Pairwise => "pairwise",
            Self::Report => "report",
        }
    }

    pub fn to_algo(self) -> ExploreMissingHandling {
        match self {
            Self::Listwise => ExploreMissingHandling::Listwise,
            Self::Pairwise => ExploreMissingHandling::Pairwise,
            Self::Report => ExploreMissingHandling::Report,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExploreDialogState {
    pub is_open: bool,
    pub source_selected: Option<usize>,
    pub dep_selected: Option<usize>,
    pub factor_selected: Option<usize>,
    pub selected_deps: Vec<String>,
    pub selected_factors: Vec<String>,
    pub label_var: Option<String>,
    pub display_mode: ExploreDisplayMode,
    // Statistics subdialog
    pub compute_descriptives: bool,
    pub ci_level_str: String,
    pub compute_m_estimators: bool,
    pub compute_extremes: bool,
    pub compute_percentiles: bool,
    // Plots subdialog
    pub boxplot_mode: ExploreBoxplotMode,
    pub plot_stem_leaf: bool,
    pub plot_histogram: bool,
    pub compute_normality: bool,
    pub spread_level_mode: ExploreSpreadLevelMode,
    pub spread_level_power: ExploreSpreadPower,
    // Options subdialog
    pub missing_mode: ExploreMissingMode,
    // Subdialog visibility
    pub show_statistics: bool,
    pub show_plots: bool,
    pub show_options: bool,
    pub bootstrap: BootstrapDialogState,
}

impl Default for ExploreDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            source_selected: None,
            dep_selected: None,
            factor_selected: None,
            selected_deps: Vec::new(),
            selected_factors: Vec::new(),
            label_var: None,
            display_mode: ExploreDisplayMode::Both,
            compute_descriptives: true,
            ci_level_str: "95".to_string(),
            compute_m_estimators: false,
            compute_extremes: true,
            compute_percentiles: false,
            boxplot_mode: ExploreBoxplotMode::FactorLevelsTogether,
            plot_stem_leaf: true,
            plot_histogram: false,
            compute_normality: true,
            spread_level_mode: ExploreSpreadLevelMode::None,
            spread_level_power: ExploreSpreadPower::NaturalLog,
            missing_mode: ExploreMissingMode::Listwise,
            show_statistics: false,
            show_plots: false,
            show_options: false,
            bootstrap: BootstrapDialogState::default(),
        }
    }
}

impl ExploreDialogState {
    pub fn reset(&mut self) {
        let is_open = self.is_open;
        *self = Self::default();
        self.is_open = is_open;
    }

    pub fn confidence_level(&self) -> f64 {
        self.ci_level_str
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite() && *v > 0.0 && *v < 100.0)
            .map(|p| p / 100.0)
            .unwrap_or(0.95)
    }

    pub fn spread_level_params(&self) -> (&'static str, Option<f64>) {
        if self.display_mode == ExploreDisplayMode::Statistics {
            return ("none", None);
        }
        match self.spread_level_mode {
            ExploreSpreadLevelMode::None => ("none", None),
            ExploreSpreadLevelMode::PowerEstimation => ("power_estimation", None),
            ExploreSpreadLevelMode::Transformed => match self.spread_level_power {
                ExploreSpreadPower::NaturalLog => ("natural_log", None),
                other => ("power", Some(other.as_f64())),
            },
            ExploreSpreadLevelMode::Untransformed => ("power", Some(1.0)),
        }
    }

    pub fn to_explore_options(&self) -> ExploreOptions {
        let include_stats = self.display_mode != ExploreDisplayMode::Plots;
        let include_plots = self.display_mode != ExploreDisplayMode::Statistics;
        let spread_level_mode = if !include_plots {
            SpreadLevelMode::None
        } else {
            match self.spread_level_mode {
                ExploreSpreadLevelMode::None => SpreadLevelMode::None,
                ExploreSpreadLevelMode::PowerEstimation => SpreadLevelMode::PowerEstimation,
                ExploreSpreadLevelMode::Transformed => match self.spread_level_power {
                    ExploreSpreadPower::NaturalLog => SpreadLevelMode::NaturalLog,
                    other => SpreadLevelMode::Power(other.as_f64()),
                },
                ExploreSpreadLevelMode::Untransformed => SpreadLevelMode::Power(1.0),
            }
        };

        let display_mode = match self.display_mode {
            ExploreDisplayMode::Both => ExploreOutputDisplay::Both,
            ExploreDisplayMode::Statistics => ExploreOutputDisplay::Statistics,
            ExploreDisplayMode::Plots => ExploreOutputDisplay::Plots,
        };
        let boxplot_mode = match self.boxplot_mode {
            ExploreBoxplotMode::FactorLevelsTogether => ExploreBoxplotDisplay::FactorLevelsTogether,
            ExploreBoxplotMode::DependentsTogether => ExploreBoxplotDisplay::DependentsTogether,
            ExploreBoxplotMode::None => ExploreBoxplotDisplay::None,
        };

        ExploreOptions {
            dependent_vars: self.selected_deps.clone(),
            factor_vars: self.selected_factors.clone(),
            label_var: self.label_var.clone(),
            compute_descriptives: include_stats && self.compute_descriptives,
            compute_m_estimators: include_stats && self.compute_m_estimators,
            compute_percentiles: include_stats && self.compute_percentiles,
            compute_normality: include_plots && self.compute_normality,
            compute_extremes: include_stats && self.compute_extremes,
            spread_level_mode,
            display_mode,
            boxplot_mode,
            plot_stem_leaf: include_plots && self.plot_stem_leaf,
            plot_histogram: include_plots && self.plot_histogram,
            confidence_level: self.confidence_level(),
            missing: self.missing_mode.to_algo(),
        }
    }
}

pub enum ExploreDialogAction {
    Run {
        dependent_vars: Vec<String>,
        factor_vars: Vec<String>,
        label_var: Option<String>,
        compute_descriptives: bool,
        compute_m_estimators: bool,
        compute_percentiles: bool,
        compute_normality: bool,
        compute_extremes: bool,
        spread_level_mode: &'static str,
        spread_level_power: Option<f64>,
        display_mode: &'static str,
        boxplot_mode: &'static str,
        plot_stem_leaf: bool,
        plot_histogram: bool,
        confidence_level: f64,
        missing: &'static str,
        bootstrap: Option<Value>,
    },
    Paste {
        options: ExploreOptions,
    },
    None,
}
