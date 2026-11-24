use serde_json::Value;
use statcraft_algo::{sort_categories, CategoryOrder, FrequencyCategory};

pub fn apply_order(categories: &mut [FrequencyCategory], order: &str) {
    sort_categories(categories, CategoryOrder::from_str_loose(order));
}

pub fn to_sps(vars: &[String]) -> String {
    format!(
        "FREQUENCIES VARIABLES={}\n  /STATISTICS=STDDEV MEAN MEDIAN\n  /ORDER=ANALYSIS.",
        vars.join(" ")
    )
}

/// Converts a full parameter JSON into canonical SPSS syntax.
pub fn to_sps_from_params(params: &Value) -> String {
    let vars = params
        .get("vars")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        })
        .unwrap_or_default();

    let mut lines = vec![format!("FREQUENCIES VARIABLES={}", vars)];

    // FORMAT
    let display_tables = params
        .get("display_tables")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let order = params.get("order").and_then(|v| v.as_str());
    let multiple_vars_mode = params.get("multiple_vars_mode").and_then(|v| v.as_str());
    let suppress_tables_limit = params.get("suppress_tables_limit").and_then(|v| v.as_u64());

    let mut format_tokens = Vec::new();
    if !display_tables {
        format_tokens.push("NOTABLE".to_string());
    }
    if let Some(ord) = order {
        match ord.to_ascii_uppercase().as_str() {
            "DVALUE" | "AFREQ" | "DFREQ" | "AVALUE" => {
                format_tokens.push(ord.to_ascii_uppercase());
            }
            _ => {}
        }
    }
    if let Some(mvm) = multiple_vars_mode {
        if mvm.eq_ignore_ascii_case("variable") {
            format_tokens.push("VARIABLE".to_string());
        }
    }
    if let Some(limit) = suppress_tables_limit {
        format_tokens.push(format!("LIMIT({})", limit));
    }
    if !format_tokens.is_empty() {
        lines.push(format!("  /FORMAT={}", format_tokens.join(" ")));
    }

    // STATISTICS
    if let Some(stats) = params.get("statistics") {
        let stat_keys = [
            ("mean", "MEAN"),
            ("std_dev", "STDDEV"),
            ("median", "MEDIAN"),
            ("mode", "MODE"),
            ("sum", "SUM"),
            ("variance", "VARIANCE"),
            ("range", "RANGE"),
            ("min", "MINIMUM"),
            ("max", "MAXIMUM"),
            ("se_mean", "SEMEAN"),
            ("skewness", "SKEWNESS"),
            ("kurtosis", "KURTOSIS"),
        ];
        let mut stat_names = Vec::new();
        for (key, sps_kw) in stat_keys {
            if stats.get(key).and_then(|v| v.as_bool()).unwrap_or(false) {
                stat_names.push(sps_kw);
            }
        }
        if !stat_names.is_empty() {
            lines.push(format!("  /STATISTICS={}", stat_names.join(" ")));
        }
    }

    // PERCENTILES
    if let Some(pcts) = params.get("percentiles") {
        let mut custom_tokens = Vec::new();
        if pcts
            .get("quartiles")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            custom_tokens.push("QUARTILES".to_string());
        }
        if let Some(arr) = pcts.get("custom").and_then(|v| v.as_array()) {
            for v in arr.iter().filter_map(|x| x.as_f64()) {
                custom_tokens.push(format!("{}", v));
            }
        }
        if !custom_tokens.is_empty() {
            lines.push(format!("  /PERCENTILES={}", custom_tokens.join(" ")));
        }

        if let Some(nt) = pcts.get("ntiles").and_then(|v| v.as_u64()) {
            lines.push(format!("  /NTILES={}", nt));
        }
    }

    // CHARTS
    if let Some(charts) = params.get("charts") {
        let chart_type = charts
            .get("chart_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        match chart_type {
            "bar" => {
                let is_percent = charts
                    .get("values")
                    .and_then(|v| v.as_str())
                    .map(|s| {
                        s.eq_ignore_ascii_case("percentages") || s.eq_ignore_ascii_case("percent")
                    })
                    .unwrap_or(false);
                let val = if is_percent { "PERCENT" } else { "FREQ" };
                lines.push(format!("  /BARCHART {}", val));
            }
            "pie" => {
                let is_percent = charts
                    .get("values")
                    .and_then(|v| v.as_str())
                    .map(|s| {
                        s.eq_ignore_ascii_case("percentages") || s.eq_ignore_ascii_case("percent")
                    })
                    .unwrap_or(false);
                let val = if is_percent { "PERCENT" } else { "FREQ" };
                lines.push(format!("  /PIECHART {}", val));
            }
            "histogram" => {
                let normal = charts
                    .get("normal_curve")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if normal {
                    lines.push("  /HISTOGRAM NORMAL".to_string());
                } else {
                    lines.push("  /HISTOGRAM".to_string());
                }
            }
            _ => {}
        }
    }

    // ORDER
    if let Some(ord) = order {
        if ord.eq_ignore_ascii_case("variable") {
            lines.push("  /ORDER=VARIABLE.".to_string());
        } else {
            lines.push("  /ORDER=ANALYSIS.".to_string());
        }
    } else {
        lines.push("  /ORDER=ANALYSIS.".to_string());
    }

    let freqs_sps = lines.join("\n");

    if let Some(styles_arr) = params.get("table_styles").and_then(|v| v.as_array()) {
        if let Some(modify_cmd) = format_output_modify(styles_arr) {
            return format!("{}\n\n{}", freqs_sps, modify_cmd);
        }
    }

    freqs_sps
}

pub fn format_output_modify(styles: &[Value]) -> Option<String> {
    if styles.is_empty() {
        return None;
    }

    let mut out = String::from("OUTPUT MODIFY\n  /SELECT TABLES\n  /IF SUBTYPES=['Frequencies']");

    let num_rules = styles.len();
    for (idx, rule) in styles.iter().enumerate() {
        let is_last = idx + 1 == num_rules;

        // 1. SELECT and SELECTDIMENSION
        let select_str = match rule.get("value_target") {
            Some(Value::String(s)) if s == "AllDataCells" => "SELECT=ALL".to_string(),
            Some(Value::Object(map)) => {
                if let Some(Value::String(lbl)) = map.get("Label") {
                    format!("SELECT=['{}']", lbl)
                } else {
                    "SELECT=ALL".to_string()
                }
            }
            _ => "SELECT=ALL".to_string(),
        };

        let dim_str = match rule.get("dimension").and_then(|v| v.as_str()) {
            Some("Rows") => "SELECTDIMENSION=ROWS",
            Some("Both") => "SELECTDIMENSION=BOTH",
            _ => "SELECTDIMENSION=COLUMNS",
        };

        out.push_str(&format!("\n  /TABLECELLS {} {}", select_str, dim_str));

        // 2. CONDITION
        if let Some(cond) = rule.get("condition") {
            let cond_str = format_condition_sps(cond);
            if !cond_str.is_empty() {
                out.push_str(&format!("\n    CONDITION={}", cond_str));
            }
        }

        // 3. STYLE
        let mut style_parts = Vec::new();
        if let Some(fmt) = rule.get("format") {
            if let Some(true) = fmt.get("reset_defaults").and_then(|v| v.as_bool()) {
                style_parts.push("RESET=YES".to_string());
            } else {
                if let Some(bg) = fmt.get("bg_color").and_then(|v| v.as_array()) {
                    if bg.len() == 3 {
                        let r = bg[0].as_u64().unwrap_or(0);
                        let g = bg[1].as_u64().unwrap_or(0);
                        let b = bg[2].as_u64().unwrap_or(0);
                        style_parts.push(format!("BACKGROUNDCOLOR(RGB({}, {}, {}))", r, g, b));
                    }
                }
                if let Some(fg) = fmt.get("text_color").and_then(|v| v.as_array()) {
                    if fg.len() == 3 {
                        let r = fg[0].as_u64().unwrap_or(0);
                        let g = fg[1].as_u64().unwrap_or(0);
                        let b = fg[2].as_u64().unwrap_or(0);
                        style_parts.push(format!("TEXTCOLOR(RGB({}, {}, {}))", r, g, b));
                    }
                }
                let bold = fmt.get("bold").and_then(|v| v.as_bool()).unwrap_or(false);
                let italic = fmt.get("italic").and_then(|v| v.as_bool()).unwrap_or(false);
                if bold && italic {
                    style_parts.push("FONTSTYLE=BOLD ITALIC".to_string());
                } else if bold {
                    style_parts.push("FONTSTYLE=BOLD".to_string());
                } else if italic {
                    style_parts.push("FONTSTYLE=ITALIC".to_string());
                }
            }
        }

        if !style_parts.is_empty() {
            out.push_str(&format!("\n    STYLE={}", style_parts.join(" ")));
        }

        if is_last {
            out.push('.');
        }
    }

    Some(out)
}

fn format_condition_sps(cond: &Value) -> String {
    match cond {
        Value::String(s) if s == "All" => "ALL".to_string(),
        Value::String(s) if s == "SysMis" => "SYSMIS".to_string(),
        Value::Object(map) => {
            if let Some(v) = map.get("Equal").and_then(|x| x.as_f64()) {
                format!("EQ({})", v)
            } else if let Some(v) = map.get("NotEqual").and_then(|x| x.as_f64()) {
                format!("NE({})", v)
            } else if let Some(v) = map.get("LessThan").and_then(|x| x.as_f64()) {
                format!("LT({})", v)
            } else if let Some(v) = map.get("LessOrEqual").and_then(|x| x.as_f64()) {
                format!("LE({})", v)
            } else if let Some(v) = map.get("GreaterThan").and_then(|x| x.as_f64()) {
                format!("GT({})", v)
            } else if let Some(v) = map.get("GreaterOrEqual").and_then(|x| x.as_f64()) {
                format!("GE({})", v)
            } else if let Some(arr) = map.get("Between").and_then(|x| x.as_array()) {
                if arr.len() == 2 {
                    let low = arr[0].as_f64().unwrap_or(0.0);
                    let high = arr[1].as_f64().unwrap_or(0.0);
                    format!("BETWEEN({}, {})", low, high)
                } else {
                    String::new()
                }
            } else if let Some(arr) = map.get("NotBetween").and_then(|x| x.as_array()) {
                if arr.len() == 2 {
                    let low = arr[0].as_f64().unwrap_or(0.0);
                    let high = arr[1].as_f64().unwrap_or(0.0);
                    format!("NOTBETWEEN({}, {})", low, high)
                } else {
                    String::new()
                }
            } else if let Some(n) = map.get("TopN").and_then(|x| x.as_u64()) {
                format!("TOP({})", n)
            } else if let Some(n) = map.get("BottomN").and_then(|x| x.as_u64()) {
                format!("BOTTOM({})", n)
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}
