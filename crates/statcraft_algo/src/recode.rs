use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OldValueRule {
    Exact(f64),
    SysMissing,
    SystemOrUserMissing,
    Range(f64, f64),
    RangeLowest(f64),
    RangeHighest(f64),
    AllOther,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NewValueSpec {
    Value(f64),
    SysMissing,
    CopyOld,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecodeMapping {
    pub old: OldValueRule,
    pub new: NewValueSpec,
}

impl RecodeMapping {
    pub fn new(old: OldValueRule, new: NewValueSpec) -> Self {
        Self { old, new }
    }

    pub fn to_sps(&self) -> String {
        format!("({}={})", self.old.to_sps(), self.new.to_sps())
    }

    pub fn display_label(&self) -> String {
        format!(
            "{} -> {}",
            self.old.display_label(),
            self.new.display_label()
        )
    }
}

fn format_num(val: f64) -> String {
    if val.fract() == 0.0 && val.abs() < 1e12 {
        format!("{:.0}", val)
    } else {
        format!("{}", val)
    }
}

impl OldValueRule {
    pub fn matches(&self, val: Option<f64>, is_user_missing: impl Fn(f64) -> bool) -> bool {
        match self {
            OldValueRule::Exact(target) => match val {
                Some(v) => !v.is_nan() && (v - target).abs() < 1e-12,
                None => false,
            },
            OldValueRule::SysMissing => val.is_none() || val.map(|v| v.is_nan()).unwrap_or(false),
            OldValueRule::SystemOrUserMissing => match val {
                None => true,
                Some(v) => v.is_nan() || is_user_missing(v),
            },
            OldValueRule::Range(low, high) => match val {
                Some(v) if !v.is_nan() && !is_user_missing(v) => {
                    let min_v = low.min(*high);
                    let max_v = low.max(*high);
                    v >= min_v && v <= max_v
                }
                _ => false,
            },
            OldValueRule::RangeLowest(high) => match val {
                Some(v) if !v.is_nan() && !is_user_missing(v) => v <= *high,
                _ => false,
            },
            OldValueRule::RangeHighest(low) => match val {
                Some(v) if !v.is_nan() && !is_user_missing(v) => v >= *low,
                _ => false,
            },
            OldValueRule::AllOther => true,
        }
    }

    pub fn to_sps(&self) -> String {
        match self {
            OldValueRule::Exact(v) => format_num(*v),
            OldValueRule::SysMissing => "SYSMIS".to_string(),
            OldValueRule::SystemOrUserMissing => "MISSING".to_string(),
            OldValueRule::Range(low, high) => {
                format!("{} THRU {}", format_num(*low), format_num(*high))
            }
            OldValueRule::RangeLowest(high) => format!("LOWEST THRU {}", format_num(*high)),
            OldValueRule::RangeHighest(low) => format!("{} THRU HIGHEST", format_num(*low)),
            OldValueRule::AllOther => "ELSE".to_string(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            OldValueRule::Exact(v) => format_num(*v),
            OldValueRule::SysMissing => "SYSMIS".to_string(),
            OldValueRule::SystemOrUserMissing => "MISSING".to_string(),
            OldValueRule::Range(low, high) => {
                format!("{} THRU {}", format_num(*low), format_num(*high))
            }
            OldValueRule::RangeLowest(high) => format!("LOWEST THRU {}", format_num(*high)),
            OldValueRule::RangeHighest(low) => format!("{} THRU HIGHEST", format_num(*low)),
            OldValueRule::AllOther => "ELSE".to_string(),
        }
    }
}

impl NewValueSpec {
    pub fn to_sps(&self) -> String {
        match self {
            NewValueSpec::Value(v) => format_num(*v),
            NewValueSpec::SysMissing => "SYSMIS".to_string(),
            NewValueSpec::CopyOld => "COPY".to_string(),
        }
    }

    pub fn display_label(&self) -> String {
        match self {
            NewValueSpec::Value(v) => format_num(*v),
            NewValueSpec::SysMissing => "SYSMIS".to_string(),
            NewValueSpec::CopyOld => "COPY".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RecodePlan {
    pub mappings: Vec<RecodeMapping>,
}

impl RecodePlan {
    pub fn new(mappings: Vec<RecodeMapping>) -> Self {
        Self { mappings }
    }

    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
    }

    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    pub fn add(&mut self, mapping: RecodeMapping) {
        self.mappings.push(mapping);
    }

    pub fn remove(&mut self, index: usize) -> Option<RecodeMapping> {
        if index < self.mappings.len() {
            Some(self.mappings.remove(index))
        } else {
            None
        }
    }

    pub fn update(&mut self, index: usize, mapping: RecodeMapping) -> bool {
        if let Some(target) = self.mappings.get_mut(index) {
            *target = mapping;
            true
        } else {
            false
        }
    }

    pub fn apply_value(
        &self,
        val: Option<f64>,
        is_user_missing: impl Fn(f64) -> bool,
    ) -> Option<f64> {
        for mapping in &self.mappings {
            if mapping.old.matches(val, &is_user_missing) {
                return match mapping.new {
                    NewValueSpec::Value(v) => Some(v),
                    NewValueSpec::SysMissing => None,
                    NewValueSpec::CopyOld => val,
                };
            }
        }
        // SPSS default: unchanged if no rule matches
        val
    }

    pub fn to_sps(&self) -> String {
        self.mappings
            .iter()
            .map(|m| m.to_sps())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn apply_recode<F>(
    values: &[Option<f64>],
    plan: &RecodePlan,
    is_user_missing: F,
) -> Vec<Option<f64>>
where
    F: Fn(f64) -> bool,
{
    values
        .iter()
        .map(|&v| plan.apply_value(v, &is_user_missing))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_mapping() {
        let plan = RecodePlan::new(vec![
            RecodeMapping::new(OldValueRule::Exact(1.0), NewValueSpec::Value(10.0)),
            RecodeMapping::new(OldValueRule::Exact(2.0), NewValueSpec::Value(20.0)),
        ]);
        let input = vec![Some(1.0), Some(2.0), Some(3.0)];
        let res = apply_recode(&input, &plan, |_| false);
        assert_eq!(res, vec![Some(10.0), Some(20.0), Some(3.0)]);
    }

    #[test]
    fn test_ranges_and_lowest_highest() {
        let plan = RecodePlan::new(vec![
            RecodeMapping::new(OldValueRule::RangeLowest(17.0), NewValueSpec::Value(1.0)),
            RecodeMapping::new(OldValueRule::Range(18.0, 60.0), NewValueSpec::Value(2.0)),
            RecodeMapping::new(OldValueRule::RangeHighest(61.0), NewValueSpec::Value(3.0)),
        ]);
        let input = vec![
            Some(10.0),
            Some(17.0),
            Some(18.0),
            Some(35.0),
            Some(60.0),
            Some(70.0),
        ];
        let res = apply_recode(&input, &plan, |_| false);
        assert_eq!(
            res,
            vec![
                Some(1.0),
                Some(1.0),
                Some(2.0),
                Some(2.0),
                Some(2.0),
                Some(3.0)
            ]
        );
    }

    #[test]
    fn test_missing_values_and_else() {
        let plan = RecodePlan::new(vec![
            RecodeMapping::new(OldValueRule::SysMissing, NewValueSpec::Value(999.0)),
            RecodeMapping::new(
                OldValueRule::SystemOrUserMissing,
                NewValueSpec::Value(888.0),
            ),
            RecodeMapping::new(OldValueRule::Exact(0.0), NewValueSpec::SysMissing),
            RecodeMapping::new(OldValueRule::AllOther, NewValueSpec::CopyOld),
        ]);

        let is_user_missing = |v: f64| v == -9.0;
        let input = vec![None, Some(-9.0), Some(0.0), Some(42.0)];
        let res = apply_recode(&input, &plan, is_user_missing);
        assert_eq!(res, vec![Some(999.0), Some(888.0), None, Some(42.0)]);
    }

    #[test]
    fn test_sps_formatting() {
        let plan = RecodePlan::new(vec![
            RecodeMapping::new(OldValueRule::RangeLowest(18.0), NewValueSpec::Value(1.0)),
            RecodeMapping::new(OldValueRule::Range(19.0, 35.0), NewValueSpec::Value(2.0)),
            RecodeMapping::new(OldValueRule::RangeHighest(36.0), NewValueSpec::Value(3.0)),
            RecodeMapping::new(OldValueRule::AllOther, NewValueSpec::SysMissing),
        ]);

        assert_eq!(
            plan.to_sps(),
            "(LOWEST THRU 18=1) (19 THRU 35=2) (36 THRU HIGHEST=3) (ELSE=SYSMIS)"
        );
    }
}
