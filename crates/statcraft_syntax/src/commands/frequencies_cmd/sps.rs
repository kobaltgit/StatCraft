use serde_json::Value;
use statcraft_algo::FrequencyCategory;

pub fn apply_order(categories: &mut [FrequencyCategory], order: &str) {
    let needs_recalc = match order.to_ascii_lowercase().as_str() {
        "dvalue" => {
            categories.reverse();
            true
        }
        "afreq" => {
            categories.sort_by_key(|c| c.count);
            true
        }
        "dfreq" => {
            categories.sort_by_key(|a| std::cmp::Reverse(a.count));
            true
        }
        _ => false,
    };

    if needs_recalc {
        let mut running = 0.0;
        for cat in categories.iter_mut() {
            if let Some(vp) = cat.valid_percent {
                running += vp;
                cat.cumulative_percent = Some(running);
            }
        }
    }
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
            return lines.join("\n");
        }
    }

    lines.push("  /ORDER=ANALYSIS.".to_string());
    lines.join("\n")
}
