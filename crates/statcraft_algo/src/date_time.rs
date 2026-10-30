use chrono::{Datelike, Duration, Months, NaiveDate, NaiveDateTime, Timelike};
use serde::{Deserialize, Serialize};
use statcraft_core::RawValue;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DateTimeOperation {
    ExtractPart(DatePart),
    Difference { unit: DateUnit },
    AddDuration { amount: f64, unit: DateUnit },
    ConstructDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatePart {
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    DayOfWeek,
    Quarter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DateUnit {
    Years,
    Months,
    Weeks,
    Days,
    Hours,
    Minutes,
    Seconds,
}

pub fn parse_date_value(val: &RawValue) -> Option<NaiveDateTime> {
    match val {
        RawValue::Text(s) => parse_date_str(s.trim()),
        RawValue::Number(num) => {
            if num.is_nan() || num.is_infinite() {
                return None;
            }
            let n = *num;
            if (1900.0..=2100.0).contains(&n) && n.fract() == 0.0 {
                NaiveDate::from_ymd_opt(n as i32, 1, 1).and_then(|d| d.and_hms_opt(0, 0, 0))
            } else if n > 1e11 {
                let secs = (n / 1000.0) as i64;
                let nsecs = ((n % 1000.0) * 1_000_000.0) as u32;
                DateTime::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
            } else if n > 0.0 {
                let secs = n as i64;
                let nsecs = (n.fract() * 1_000_000_000.0) as u32;
                DateTime::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
            } else {
                None
            }
        }
        RawValue::Null => None,
    }
}

use chrono::DateTime;

fn parse_date_str(s: &str) -> Option<NaiveDateTime> {
    if s.is_empty() {
        return None;
    }

    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return Some(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.naive_utc());
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%d.%m.%Y %H:%M:%S") {
        return Some(dt);
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(s, "%m/%d/%Y %H:%M:%S") {
        return Some(dt);
    }

    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d.and_hms_opt(0, 0, 0);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d.%m.%Y") {
        return d.and_hms_opt(0, 0, 0);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%m/%d/%Y") {
        return d.and_hms_opt(0, 0, 0);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y/%m/%d") {
        return d.and_hms_opt(0, 0, 0);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
        return d.and_hms_opt(0, 0, 0);
    }

    if let Ok(secs) = s.parse::<i64>() {
        if secs > 1e11 as i64 {
            return DateTime::from_timestamp(secs / 1000, ((secs % 1000) * 1_000_000) as u32)
                .map(|dt| dt.naive_utc());
        }
        if secs > 1e8 as i64 {
            return DateTime::from_timestamp(secs, 0).map(|dt| dt.naive_utc());
        }
    }

    None
}

fn compute_difference(dt1: NaiveDateTime, dt2: NaiveDateTime, unit: DateUnit) -> f64 {
    match unit {
        DateUnit::Seconds => (dt1 - dt2).num_seconds() as f64,
        DateUnit::Minutes => (dt1 - dt2).num_seconds() as f64 / 60.0,
        DateUnit::Hours => (dt1 - dt2).num_seconds() as f64 / 3600.0,
        DateUnit::Days => (dt1.date() - dt2.date()).num_days() as f64,
        DateUnit::Weeks => ((dt1.date() - dt2.date()).num_days() as f64 / 7.0).floor(),
        DateUnit::Months => {
            let mut months =
                (dt1.year() - dt2.year()) * 12 + (dt1.month() as i32 - dt2.month() as i32);
            if dt1 > dt2 && dt1.day() < dt2.day() {
                months -= 1;
            } else if dt1 < dt2 && dt1.day() > dt2.day() {
                months += 1;
            }
            months as f64
        }
        DateUnit::Years => {
            let mut years = dt1.year() - dt2.year();
            if dt1 > dt2 && (dt1.month(), dt1.day()) < (dt2.month(), dt2.day()) {
                years -= 1;
            } else if dt1 < dt2 && (dt1.month(), dt1.day()) > (dt2.month(), dt2.day()) {
                years += 1;
            }
            years as f64
        }
    }
}

fn add_duration(dt: NaiveDateTime, amount: f64, unit: DateUnit) -> Option<NaiveDateTime> {
    if amount.is_nan() || amount.is_infinite() {
        return None;
    }
    match unit {
        DateUnit::Seconds => dt.checked_add_signed(Duration::seconds(amount as i64)),
        DateUnit::Minutes => dt.checked_add_signed(Duration::minutes(amount as i64)),
        DateUnit::Hours => dt.checked_add_signed(Duration::hours(amount as i64)),
        DateUnit::Days => dt.checked_add_signed(Duration::days(amount as i64)),
        DateUnit::Weeks => dt.checked_add_signed(Duration::days((amount * 7.0) as i64)),
        DateUnit::Months => {
            let months_count = amount as i32;
            if months_count >= 0 {
                dt.checked_add_months(Months::new(months_count as u32))
            } else {
                dt.checked_sub_months(Months::new((-months_count) as u32))
            }
        }
        DateUnit::Years => {
            let months_count = (amount as i32).checked_mul(12)?;
            if months_count >= 0 {
                dt.checked_add_months(Months::new(months_count as u32))
            } else {
                dt.checked_sub_months(Months::new((-months_count) as u32))
            }
        }
    }
}

pub fn format_date_output(dt: NaiveDateTime) -> String {
    if dt.hour() == 0 && dt.minute() == 0 && dt.second() == 0 {
        dt.format("%Y-%m-%d").to_string()
    } else {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

pub fn compute_date_operation(
    op: DateTimeOperation,
    primary_col: &[RawValue],
    secondary_col: Option<&[RawValue]>,
) -> Result<Vec<RawValue>, String> {
    let n = primary_col.len();
    let mut out = Vec::with_capacity(n);

    match op {
        DateTimeOperation::ExtractPart(part) => {
            for v in primary_col {
                if let Some(dt) = parse_date_value(v) {
                    let num = match part {
                        DatePart::Year => dt.year() as f64,
                        DatePart::Month => dt.month() as f64,
                        DatePart::Day => dt.day() as f64,
                        DatePart::Hour => dt.hour() as f64,
                        DatePart::Minute => dt.minute() as f64,
                        DatePart::Second => dt.second() as f64,
                        DatePart::DayOfWeek => dt.weekday().number_from_sunday() as f64,
                        DatePart::Quarter => ((dt.month() - 1) / 3 + 1) as f64,
                    };
                    out.push(RawValue::Number(num));
                } else {
                    out.push(RawValue::Null);
                }
            }
        }
        DateTimeOperation::Difference { unit } => {
            let sec_col = secondary_col.ok_or_else(|| {
                "Secondary date column is required for date difference".to_string()
            })?;
            for i in 0..n {
                let v1 = primary_col.get(i);
                let v2 = sec_col.get(i);
                if let (Some(val1), Some(val2)) = (v1, v2) {
                    if let (Some(dt1), Some(dt2)) = (parse_date_value(val1), parse_date_value(val2))
                    {
                        let diff = compute_difference(dt1, dt2, unit);
                        out.push(RawValue::Number(diff));
                    } else {
                        out.push(RawValue::Null);
                    }
                } else {
                    out.push(RawValue::Null);
                }
            }
        }
        DateTimeOperation::AddDuration { amount, unit } => {
            for v in primary_col {
                if let Some(dt) = parse_date_value(v) {
                    if let Some(res_dt) = add_duration(dt, amount, unit) {
                        out.push(RawValue::Text(format_date_output(res_dt)));
                    } else {
                        out.push(RawValue::Null);
                    }
                } else {
                    out.push(RawValue::Null);
                }
            }
        }
        DateTimeOperation::ConstructDate => {
            for v in primary_col {
                if let Some(dt) = parse_date_value(v) {
                    out.push(RawValue::Text(format_date_output(dt)));
                } else {
                    out.push(RawValue::Null);
                }
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_formats() {
        let v1 = RawValue::Text("2024-02-29".to_string());
        let dt1 = parse_date_value(&v1);
        assert!(dt1.is_some());
        if let Some(d) = dt1 {
            assert_eq!(d.year(), 2024);
            assert_eq!(d.month(), 2);
            assert_eq!(d.day(), 29);
        }

        let v2 = RawValue::Text("15.08.2023 14:30:00".to_string());
        let dt2 = parse_date_value(&v2);
        assert!(dt2.is_some());
        if let Some(d) = dt2 {
            assert_eq!(d.year(), 2023);
            assert_eq!(d.month(), 8);
            assert_eq!(d.day(), 15);
            assert_eq!(d.hour(), 14);
        }

        let v_null = RawValue::Null;
        assert!(parse_date_value(&v_null).is_none());

        let v_invalid = RawValue::Text("not a date".to_string());
        assert!(parse_date_value(&v_invalid).is_none());
    }

    #[test]
    fn test_extract_parts() {
        let col = vec![
            RawValue::Text("2023-05-14".to_string()), // Sunday
            RawValue::Null,
        ];

        let years =
            compute_date_operation(DateTimeOperation::ExtractPart(DatePart::Year), &col, None)
                .unwrap();
        assert_eq!(years[0], RawValue::Number(2023.0));
        assert_eq!(years[1], RawValue::Null);

        let months =
            compute_date_operation(DateTimeOperation::ExtractPart(DatePart::Month), &col, None)
                .unwrap();
        assert_eq!(months[0], RawValue::Number(5.0));

        let wkday = compute_date_operation(
            DateTimeOperation::ExtractPart(DatePart::DayOfWeek),
            &col,
            None,
        )
        .unwrap();
        // Sunday is 1 in SPSS
        assert_eq!(wkday[0], RawValue::Number(1.0));

        let qtr = compute_date_operation(
            DateTimeOperation::ExtractPart(DatePart::Quarter),
            &col,
            None,
        )
        .unwrap();
        assert_eq!(qtr[0], RawValue::Number(2.0));
    }

    #[test]
    fn test_date_difference() {
        let col1 = vec![
            RawValue::Text("2023-05-15".to_string()),
            RawValue::Text("2024-02-29".to_string()),
        ];
        let col2 = vec![
            RawValue::Text("2020-05-15".to_string()),
            RawValue::Text("2023-02-28".to_string()),
        ];

        let years_diff = compute_date_operation(
            DateTimeOperation::Difference {
                unit: DateUnit::Years,
            },
            &col1,
            Some(&col2),
        )
        .unwrap();
        assert_eq!(years_diff[0], RawValue::Number(3.0));
        assert_eq!(years_diff[1], RawValue::Number(1.0));

        let days_diff = compute_date_operation(
            DateTimeOperation::Difference {
                unit: DateUnit::Days,
            },
            &col1,
            Some(&col2),
        )
        .unwrap();
        assert_eq!(days_diff[0], RawValue::Number(1095.0));
        assert_eq!(days_diff[1], RawValue::Number(366.0)); // Leap year 2024
    }

    #[test]
    fn test_add_duration() {
        let col = vec![RawValue::Text("2024-01-31".to_string()), RawValue::Null];

        let res_months = compute_date_operation(
            DateTimeOperation::AddDuration {
                amount: 1.0,
                unit: DateUnit::Months,
            },
            &col,
            None,
        )
        .unwrap();
        // 2024 is a leap year, Jan 31 + 1 month = Feb 29
        assert_eq!(res_months[0], RawValue::Text("2024-02-29".to_string()));
        assert_eq!(res_months[1], RawValue::Null);

        let res_days = compute_date_operation(
            DateTimeOperation::AddDuration {
                amount: -10.0,
                unit: DateUnit::Days,
            },
            &col,
            None,
        )
        .unwrap();
        assert_eq!(res_days[0], RawValue::Text("2024-01-21".to_string()));
    }
}
