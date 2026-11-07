use crate::syntax_parser::error::SyntaxError;
use crate::syntax_parser::parser::ParsedCommand;
use serde_json::{json, Value};

pub fn map_help_command(cmd: &ParsedCommand) -> Result<Option<(&'static str, Value)>, SyntaxError> {
    if !cmd.name.eq_ignore_ascii_case("HELP") {
        return Ok(None);
    }

    let topic = cmd
        .subcommands
        .first()
        .and_then(|sub| {
            sub.tokens.first().or(if !sub.raw_value.is_empty() {
                Some(&sub.raw_value)
            } else {
                None
            })
        })
        .map(|s| s.trim().to_ascii_uppercase())
        .filter(|s| !s.is_empty());

    let help_text = generate_help(topic.as_deref());

    Ok(Some((
        "util.help",
        json!({
            "topic": topic,
            "output": help_text
        }),
    )))
}

pub fn generate_help(topic: Option<&str>) -> String {
    let t = match topic {
        Some(s) if !s.trim().is_empty() => s.trim().to_ascii_uppercase(),
        _ => return generate_general_help(),
    };

    match t.as_str() {
        "DESCRIPTIVES" => "\
PROCEDURE: DESCRIPTIVES
Computes univariate summary statistics for scale variables.

SYNTAX:
  DESCRIPTIVES VARIABLES=varlist
    [/STATISTICS=[MEAN] [STDDEV] [MIN] [MAX] [SEMEAN] [VARIANCE]].

EXAMPLE:
  DESCRIPTIVES VARIABLES=salary age
    /STATISTICS=MEAN STDDEV MIN MAX SEMEAN.
"
        .to_string(),

        "FREQUENCIES" => "\
PROCEDURE: FREQUENCIES
Produces frequency counts, percentages, and percentiles.

SYNTAX:
  FREQUENCIES VARIABLES=varlist
    [/STATISTICS=[MEAN] [MEDIAN] [MODE] [STDDEV]]
    [/FORMAT=[AVALUE | DVALUE]].

EXAMPLE:
  FREQUENCIES VARIABLES=jobcat gender
    /STATISTICS=MEAN MEDIAN.
"
        .to_string(),

        "CROSSTABS" => "\
PROCEDURE: CROSSTABS
Generates two-way contingency tables and association statistics.

SYNTAX:
  CROSSTABS /TABLES=row_vars BY col_vars
    [/STATISTICS=[CHISQ] [PHI] [CC]].

EXAMPLE:
  CROSSTABS /TABLES=jobcat BY gender
    /STATISTICS=CHISQ PHI.
"
        .to_string(),

        "T-TEST" | "TTEST" => "\
PROCEDURE: T-TEST
Performs Student's t-tests (Independent samples, Paired, or One-sample).

SYNTAX:
  1. Independent Samples:
     T-TEST GROUPS=group_var(val1 val2) /VARIABLES=varlist [/CRITERIA=CI(level)].
  2. Paired Samples:
     T-TEST PAIRS=var1 WITH var2 (PAIRED) [/CRITERIA=CI(level)].
  3. One-Sample:
     T-TEST /TESTVAL=value /VARIABLES=varlist [/CRITERIA=CI(level)].

EXAMPLES:
  T-TEST GROUPS=gender(1 2) /VARIABLES=salary /CRITERIA=CI(0.95).
  T-TEST PAIRS=before WITH after (PAIRED).
  T-TEST /TESTVAL=50000 /VARIABLES=salary.
"
        .to_string(),

        "ONEWAY" | "ANOVA" => "\
PROCEDURE: ONEWAY
Conducts One-Way Analysis of Variance with Levene's test and descriptives.

SYNTAX:
  ONEWAY dep_vars BY factor_var
    [/STATISTICS=[DESCRIPTIVES] [HOMOGENEITY]]
    [/CRITERIA=CI(level)].

EXAMPLE:
  ONEWAY salary BY jobcat
    /STATISTICS DESCRIPTIVES HOMOGENEITY
    /CRITERIA=CI(0.95).
"
        .to_string(),

        "MEANS" => "\
PROCEDURE: MEANS
Calculates subgroup means and one-way ANOVA table across categories.

SYNTAX:
  MEANS TABLES=dep_vars BY factor_vars
    [/STATISTICS=[ANOVA]].

EXAMPLE:
  MEANS TABLES=salary BY jobcat
    /STATISTICS=ANOVA.
"
        .to_string(),

        "CORRELATIONS" | "CORRELATION" => "\
PROCEDURE: CORRELATIONS
Computes Pearson, Spearman, and Kendall bivariate correlation coefficients.

SYNTAX:
  CORRELATIONS /VARIABLES=varlist
    [/PRINT=[TWOTAIL | ONETAIL] [NOSIG]]
    [/STATISTICS=[DESCRIPTIVES] [XPROD]].

EXAMPLE:
  CORRELATIONS /VARIABLES=salary educ prevexp
    /PRINT=TWOTAIL
    /STATISTICS=DESCRIPTIVES.
"
        .to_string(),

        "REGRESSION" => "\
PROCEDURE: REGRESSION
Fits linear regression models with ANOVA, coefficients, and collinearity diagnostics.

SYNTAX:
  REGRESSION
    /DEPENDENT=dep_var
    /METHOD=ENTER indep_vars.

EXAMPLE:
  REGRESSION
    /DEPENDENT=salary
    /METHOD=ENTER educ prevexp jobtime.
"
        .to_string(),

        "RELIABILITY" => "\
PROCEDURE: RELIABILITY
Computes Cronbach's Alpha and item-total reliability statistics.

SYNTAX:
  RELIABILITY /VARIABLES=varlist
    [/MODEL=ALPHA]
    [/SUMMARY=TOTAL].

EXAMPLE:
  RELIABILITY /VARIABLES=item1 item2 item3 item4
    /MODEL=ALPHA.
"
        .to_string(),

        "FACTOR" => "\
PROCEDURE: FACTOR
Performs Principal Component Factor Analysis with KMO and Bartlett tests.

SYNTAX:
  FACTOR /VARIABLES=varlist
    [/EXTRACTION=PC]
    [/CRITERIA=FACTORS(k) | MINEIGEN(val)]
    [/ROTATION=VARIMAX].

EXAMPLE:
  FACTOR /VARIABLES=q1 q2 q3 q4 q5
    /EXTRACTION=PC.
"
        .to_string(),

        "VARIABLE LEVEL" => "\
COMMAND: VARIABLE LEVEL
Assigns measurement levels (SCALE, ORDINAL, NOMINAL) to variables.

SYNTAX:
  VARIABLE LEVEL varlist (SCALE | ORDINAL | NOMINAL) [/varlist2 (...)].

EXAMPLE:
  VARIABLE LEVEL salary age (SCALE) /gender (NOMINAL).
"
        .to_string(),

        "VARIABLE LABELS" => "\
COMMAND: VARIABLE LABELS
Assigns descriptive labels to variables.

SYNTAX:
  VARIABLE LABELS var 'Label string' [var2 'Label 2'].

EXAMPLE:
  VARIABLE LABELS salary 'Annual Salary in USD' age 'Age in Years'.
"
        .to_string(),

        "VALUE LABELS" => "\
COMMAND: VALUE LABELS
Assigns descriptive labels to categorical values.

SYNTAX:
  VALUE LABELS var val1 'Label 1' val2 'Label 2' [/var2 ...].

EXAMPLE:
  VALUE LABELS gender 1 'Male' 2 'Female'.
"
        .to_string(),

        "MISSING VALUES" => "\
COMMAND: MISSING VALUES
Specifies user-missing discrete values or numeric ranges.

SYNTAX:
  MISSING VALUES var (val1, val2, val3) | (low THRU high).

EXAMPLE:
  MISSING VALUES salary (99, 999).
  MISSING VALUES age (0 THRU 17).
"
        .to_string(),

        "COMPUTE" => "\
COMMAND: COMPUTE
Creates or updates a variable from an arithmetic expression.

SYNTAX:
  COMPUTE target_var = expression.

EXAMPLE:
  COMPUTE log_salary = LN(salary).
  COMPUTE total_score = q1 + q2 + q3.
"
        .to_string(),

        "SORT CASES" => "\
COMMAND: SORT CASES
Sorts active dataset rows by one or more variables.

SYNTAX:
  SORT CASES BY var1 [(A|D)] [var2 ...].

EXAMPLE:
  SORT CASES BY department (A) salary (D).
"
        .to_string(),

        "SPLIT FILE" => "\
COMMAND: SPLIT FILE
Divides dataset into subgroups for comparative analysis.

SYNTAX:
  SPLIT FILE BY varlist.
  SPLIT FILE OFF.

EXAMPLE:
  SPLIT FILE BY gender.
"
        .to_string(),

        "WEIGHT" => "\
COMMAND: WEIGHT
Applies case weights to subsequent statistical procedures.

SYNTAX:
  WEIGHT BY weight_var.
  WEIGHT OFF.
"
        .to_string(),

        "FILTER" => "\
COMMAND: FILTER
Filters cases based on a binary or indicator variable.

SYNTAX:
  FILTER BY filter_var.
  FILTER OFF.
"
        .to_string(),

        other => {
            let suggestion = find_closest_command(other);
            format!(
                "No specific help entry for '{}'. Did you mean '{}'?\nType 'HELP.' for the full command directory.",
                other, suggestion
            )
        }
    }
}

pub fn generate_general_help() -> String {
    "\
================================================================================
  StatCraft Autonomous Code Mode - SPSS Syntax Quick Reference
================================================================================

Commands are executed sequentially in batch mode. Each command must end with a period (.).

1. DESCRIPTIVE STATISTICS:
   * DESCRIPTIVES VARIABLES=vars [/STATISTICS=MEAN STDDEV MIN MAX SEMEAN].
   * FREQUENCIES VARIABLES=vars [/STATISTICS=MEAN MEDIAN MODE STDDEV].
   * CROSSTABS /TABLES=row_vars BY col_vars [/STATISTICS=CHISQ PHI CC].

2. COMPARE MEANS:
   * MEANS TABLES=dep_vars BY factor_vars [/STATISTICS=ANOVA].
   * T-TEST GROUPS=var(v1 v2) /VARIABLES=vars [/CRITERIA=CI(0.95)].
   * T-TEST PAIRS=v1 WITH v2 (PAIRED).
   * T-TEST /TESTVAL=value /VARIABLES=vars.
   * ONEWAY dep_vars BY factor_var [/STATISTICS DESCRIPTIVES HOMOGENEITY].

3. CORRELATION & REGRESSION:
   * CORRELATIONS /VARIABLES=vars [/PRINT=TWOTAIL] [/STATISTICS=DESCRIPTIVES].
   * REGRESSION /DEPENDENT=var /METHOD=ENTER indep_vars.

4. SCALE & FACTOR ANALYSIS:
   * RELIABILITY /VARIABLES=vars [/MODEL=ALPHA].
   * FACTOR /VARIABLES=vars [/EXTRACTION=PC].

5. DATA TRANSFORMATIONS & FILTERING:
   * COMPUTE target = expression.
   * SORT CASES BY var1 (A) var2 (D).
   * SPLIT FILE BY varlist. | SPLIT FILE OFF.
   * WEIGHT BY weight_var. | WEIGHT OFF.
   * FILTER BY filter_var. | FILTER OFF.

6. VARIABLE DICTIONARY & METADATA:
   * VARIABLE LEVEL varlist (SCALE | ORDINAL | NOMINAL).
   * VARIABLE LABELS var 'Label' [var2 'Label 2'].
   * VALUE LABELS var 1 'Label 1' 2 'Label 2'.
   * MISSING VALUES var (val1, val2) | (low THRU high).

To see detailed syntax and options for a specific command, run:
  HELP <COMMAND>.   (e.g., HELP ONEWAY. or HELP T-TEST.)
Dynamic MCP resource: statcraft://syntax-reference
"
    .to_string()
}

pub fn generate_full_syntax_reference() -> String {
    let mut ref_doc = String::with_capacity(8192);
    ref_doc.push_str("# StatCraft SPSS Syntax & Procedure Reference Manual\n\n");
    ref_doc.push_str(&generate_general_help());
    ref_doc.push_str("\n---\n\n## Detailed Command Specifications\n\n");

    let commands = [
        "DESCRIPTIVES",
        "FREQUENCIES",
        "CROSSTABS",
        "MEANS",
        "T-TEST",
        "ONEWAY",
        "CORRELATIONS",
        "REGRESSION",
        "RELIABILITY",
        "FACTOR",
        "VARIABLE LEVEL",
        "VARIABLE LABELS",
        "VALUE LABELS",
        "MISSING VALUES",
        "COMPUTE",
        "SORT CASES",
        "SPLIT FILE",
        "WEIGHT",
        "FILTER",
    ];

    for cmd in commands {
        ref_doc.push_str(&format!(
            "### {}\n```sps\n{}```\n\n",
            cmd,
            generate_help(Some(cmd))
        ));
    }

    ref_doc
}

pub fn find_closest_command(input: &str) -> &'static str {
    let candidates = [
        "DESCRIPTIVES",
        "FREQUENCIES",
        "CROSSTABS",
        "MEANS",
        "T-TEST",
        "ONEWAY",
        "CORRELATIONS",
        "REGRESSION",
        "RELIABILITY",
        "FACTOR",
        "VARIABLE LEVEL",
        "VARIABLE LABELS",
        "VALUE LABELS",
        "MISSING VALUES",
        "COMPUTE",
        "SORT CASES",
        "SPLIT FILE",
        "WEIGHT",
        "FILTER",
        "HELP",
    ];

    let upper = input.trim().to_ascii_uppercase();
    let mut best = candidates[0];
    let mut best_score = usize::MAX;

    for &cand in &candidates {
        if cand.contains(&upper) || upper.contains(cand) {
            return cand;
        }
        let dist = levenshtein(&upper, cand);
        if dist < best_score {
            best_score = dist;
            best = cand;
        }
    }

    best
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev.copy_from_slice(&curr);
    }

    prev[n]
}
