use super::types::EvalError;

pub fn eval_function(name: &str, args: &[f64]) -> Result<f64, EvalError> {
    match name {
        "ABS" => check_1_arg(name, args, |x| x.abs()),
        "SQRT" => check_1_arg(name, args, |x| if x < 0.0 { f64::NAN } else { x.sqrt() }),
        "LN" => check_1_arg(name, args, |x| if x <= 0.0 { f64::NAN } else { x.ln() }),
        "LOG10" => check_1_arg(name, args, |x| if x <= 0.0 { f64::NAN } else { x.log10() }),
        "EXP" => check_1_arg(name, args, |x| x.exp()),
        "RND" | "ROUND" => check_1_arg(name, args, |x| x.round()),
        "TRUNC" => check_1_arg(name, args, |x| x.trunc()),
        "SUM" => {
            if args.is_empty() {
                return Err(EvalError::InvalidArgCount {
                    name: name.to_string(),
                    expected: 1,
                    got: 0,
                });
            }
            let valid: Vec<f64> = args.iter().copied().filter(|x| !x.is_nan()).collect();
            if valid.is_empty() {
                Ok(f64::NAN)
            } else {
                Ok(valid.iter().sum())
            }
        }
        "MEAN" => {
            if args.is_empty() {
                return Err(EvalError::InvalidArgCount {
                    name: name.to_string(),
                    expected: 1,
                    got: 0,
                });
            }
            let valid: Vec<f64> = args.iter().copied().filter(|x| !x.is_nan()).collect();
            if valid.is_empty() {
                Ok(f64::NAN)
            } else {
                let sum: f64 = valid.iter().sum();
                Ok(sum / (valid.len() as f64))
            }
        }
        "MIN" => {
            if args.is_empty() {
                return Err(EvalError::InvalidArgCount {
                    name: name.to_string(),
                    expected: 1,
                    got: 0,
                });
            }
            let mut min_val = f64::NAN;
            for &x in args {
                if !x.is_nan() && (min_val.is_nan() || x < min_val) {
                    min_val = x;
                }
            }
            Ok(min_val)
        }
        "MAX" => {
            if args.is_empty() {
                return Err(EvalError::InvalidArgCount {
                    name: name.to_string(),
                    expected: 1,
                    got: 0,
                });
            }
            let mut max_val = f64::NAN;
            for &x in args {
                if !x.is_nan() && (max_val.is_nan() || x > max_val) {
                    max_val = x;
                }
            }
            Ok(max_val)
        }
        _ => Err(EvalError::UnknownFunction(name.to_string())),
    }
}

fn check_1_arg<F>(name: &str, args: &[f64], f: F) -> Result<f64, EvalError>
where
    F: Fn(f64) -> f64,
{
    if args.len() != 1 {
        return Err(EvalError::InvalidArgCount {
            name: name.to_string(),
            expected: 1,
            got: args.len(),
        });
    }
    let val = args[0];
    if val.is_nan() {
        Ok(f64::NAN)
    } else {
        let res = f(val);
        Ok(if res.is_nan() || res.is_infinite() {
            f64::NAN
        } else {
            res
        })
    }
}
