use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use statcraft_algo::{CategoryOrder, MultipleVarsMode};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrequenciesFormatState {
    /// Порядок категорий: AValue = Ascending Values, DValue = Descending Values, AFreq = Ascending Counts, DFreq = Descending Counts
    pub order: CategoryOrder,
    /// Организация нескольких переменных: Compare = Compare Variables, Variable = Organize by Variables
    pub multiple_vars_mode: MultipleVarsMode,
    /// Чекбокс «Отключить таблицы со многими категориями»
    pub suppress_tables: bool,
    /// Значение поля «Максимальное число категорий: [10]»
    pub max_categories_input: String,
}

impl Default for FrequenciesFormatState {
    fn default() -> Self {
        Self {
            order: CategoryOrder::AValue,
            multiple_vars_mode: MultipleVarsMode::Compare,
            suppress_tables: false,
            max_categories_input: "10".to_string(),
        }
    }
}

impl FrequenciesFormatState {
    pub fn order_str(&self) -> Option<String> {
        match self.order {
            CategoryOrder::AValue => None,
            CategoryOrder::DValue => Some("dvalue".to_string()),
            CategoryOrder::AFreq => Some("afreq".to_string()),
            CategoryOrder::DFreq => Some("dfreq".to_string()),
        }
    }

    pub fn multiple_vars_mode_str(&self) -> Option<String> {
        match self.multiple_vars_mode {
            MultipleVarsMode::Compare => None,
            MultipleVarsMode::Variable => Some("variable".to_string()),
        }
    }

    pub fn suppress_tables_limit(&self) -> Option<usize> {
        if self.suppress_tables {
            self.max_categories_input
                .trim()
                .parse::<usize>()
                .ok()
                .filter(|&n| n >= 1)
                .or(Some(10))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequenciesChartType {
    None,
    Bar,
    Pie,
    Histogram,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequenciesChartValue {
    Frequencies,
    Percentages,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrequenciesChartOptions {
    pub chart_type: FrequenciesChartType,
    pub values: FrequenciesChartValue,
    pub normal_curve: bool,
}

impl Default for FrequenciesChartOptions {
    fn default() -> Self {
        Self {
            chart_type: FrequenciesChartType::None,
            values: FrequenciesChartValue::Frequencies,
            normal_curve: false,
        }
    }
}

impl FrequenciesChartOptions {
    pub fn to_json(&self) -> Option<Value> {
        if self.chart_type == FrequenciesChartType::None {
            return None;
        }
        Some(json!({
            "chart_type": match self.chart_type {
                FrequenciesChartType::None => "none",
                FrequenciesChartType::Bar => "bar",
                FrequenciesChartType::Pie => "pie",
                FrequenciesChartType::Histogram => "histogram",
            },
            "values": match self.values {
                FrequenciesChartValue::Frequencies => "frequencies",
                FrequenciesChartValue::Percentages => "percentages",
            },
            "normal_curve": self.normal_curve,
        }))
    }
}

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
    pub show_charts: bool,
    pub show_format: bool,
    pub source_selected: Option<usize>,
    pub target_selected: Option<usize>,
    pub selected_vars: Vec<String>,
    pub display_tables: bool,
    pub apa_tables: bool,
    pub stats: FrequenciesStatisticsState,
    pub stats_backup: Option<FrequenciesStatisticsState>,
    pub charts: FrequenciesChartOptions,
    pub charts_backup: Option<FrequenciesChartOptions>,
    pub format: FrequenciesFormatState,
    pub format_backup: Option<FrequenciesFormatState>,
    pub table_style: crate::dialogs::table_style_dialog::TableStyleState,
    pub bootstrap: crate::dialogs::bootstrap_dialog::BootstrapDialogState,
}

impl Default for FrequenciesDialogState {
    fn default() -> Self {
        Self {
            is_open: false,
            show_statistics: false,
            show_charts: false,
            show_format: false,
            source_selected: None,
            target_selected: None,
            selected_vars: Vec::new(),
            display_tables: true,
            apa_tables: false,
            stats: FrequenciesStatisticsState::default(),
            stats_backup: None,
            charts: FrequenciesChartOptions::default(),
            charts_backup: None,
            format: FrequenciesFormatState::default(),
            format_backup: None,
            table_style: crate::dialogs::table_style_dialog::TableStyleState::default(),
            bootstrap: crate::dialogs::bootstrap_dialog::BootstrapDialogState::default(),
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
        self.charts = FrequenciesChartOptions::default();
        self.charts_backup = None;
        self.format = FrequenciesFormatState::default();
        self.format_backup = None;
        self.table_style.reset();
        self.bootstrap.reset();
        self.show_statistics = false;
        self.show_charts = false;
        self.show_format = false;
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
        charts: Option<Value>,
        order: Option<String>,
        multiple_vars_mode: Option<String>,
        suppress_tables_limit: Option<usize>,
        table_styles: Option<Value>,
        create_style_report: bool,
        bootstrap: Option<Value>,
    },
    Paste {
        vars: Vec<String>,
        display_tables: bool,
        apa_tables: bool,
        statistics: Option<Value>,
        percentiles: Option<Value>,
        group_midpoints: bool,
        charts: Option<Value>,
        order: Option<String>,
        multiple_vars_mode: Option<String>,
        suppress_tables_limit: Option<usize>,
        table_styles: Option<Value>,
        create_style_report: bool,
        bootstrap: Option<Value>,
    },
    None,
}
