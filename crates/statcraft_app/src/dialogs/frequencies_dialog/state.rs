use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct FrequenciesStatisticsState {
    // Percentiles
    pub quartiles: bool,
    pub cutpoints_enabled: bool,
    pub cutpoints_n: String,
    pub custom_percentiles: Vec<f64>,
    pub percentile_input: String,
    pub selected_percentile_idx: Option<usize>,

    // Central Tendency
    pub mean: bool,
    pub median: bool,
    pub mode: bool,
    pub sum: bool,

    // Dispersion
    pub std_dev: bool,
    pub variance: bool,
    pub range: bool,
    pub min: bool,
    pub max: bool,
    pub se_mean: bool,

    // Distribution
    pub skewness: bool,
    pub kurtosis: bool,

    // Group midpoints
    pub group_midpoints: bool,
}

impl Default for FrequenciesStatisticsState {
    fn default() -> Self {
        Self {
            quartiles: false,
            cutpoints_enabled: false,
            cutpoints_n: "10".to_string(),
            custom_percentiles: Vec::new(),
            percentile_input: String::new(),
            selected_percentile_idx: None,

            mean: false,
            median: false,
            mode: false,
            sum: false,

            std_dev: false,
            variance: false,
            range: false,
            min: false,
            max: false,
            se_mean: false,

            skewness: false,
            kurtosis: false,

            group_midpoints: false,
        }
    }
}

impl FrequenciesStatisticsState {
    pub fn to_statistics_json(&self) -> Option<Value> {
        let has_any = self.mean
            || self.se_mean
            || self.median
            || self.mode
            || self.sum
            || self.std_dev
            || self.variance
            || self.range
            || self.min
            || self.max
            || self.skewness
            || self.kurtosis;

        if !has_any {
            return None;
        }

        Some(json!({
            "mean": self.mean,
            "se_mean": self.se_mean,
            "median": self.median,
            "mode": self.mode,
            "sum": self.sum,
            "std_dev": self.std_dev,
            "variance": self.variance,
            "range": self.range,
            "min": self.min,
            "max": self.max,
            "skewness": self.skewness,
            "kurtosis": self.kurtosis,
        }))
    }

    pub fn to_percentiles_json(&self) -> Option<Value> {
        let mut obj = serde_json::Map::new();

        if self.quartiles {
            obj.insert("quartiles".to_string(), json!(true));
        }

        if self.cutpoints_enabled {
            if let Ok(n) = self.cutpoints_n.trim().parse::<usize>() {
                if n >= 2 {
                    obj.insert("ntiles".to_string(), json!(n));
                }
            }
        }

        if !self.custom_percentiles.is_empty() {
            obj.insert("custom".to_string(), json!(self.custom_percentiles));
        }

        if !obj.is_empty() {
            Some(Value::Object(obj))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrequenciesDialogState {
    pub is_open: bool,
    pub show_statistics: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub display_tables: bool,
    pub apa_tables: bool,
    pub stats: FrequenciesStatisticsState,
    pub stats_backup: Option<FrequenciesStatisticsState>,
}

impl Default for FrequenciesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            show_statistics: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            display_tables: true,
            apa_tables: false,
            stats: FrequenciesStatisticsState::default(),
            stats_backup: None,
        }
    }
}

impl FrequenciesDialogState {
    pub fn reset(&mut self) {
        self.source_selected = None;
        self.target_selected = None;
        self.selected_vars.clear();
        self.display_tables = true;
        self.apa_tables = false;
        self.stats = FrequenciesStatisticsState::default();
        self.stats_backup = None;
        self.show_statistics = false;
    }
}

pub enum FrequenciesDialogAction {
    Run {
        vars: Vec<String>,
        display_tables: bool,
        apa_tables: bool,
        statistics: Option<Value>,
        percentiles: Option<Value>,
        group_midpoints: bool,
    },
    Paste {
        vars: Vec<String>,
        display_tables: bool,
        apa_tables: bool,
        statistics: Option<Value>,
        percentiles: Option<Value>,
        group_midpoints: bool,
    },
    None,
}
