use super::*;
use std::collections::HashMap;

#[test]
fn test_arithmetic_precedence() {
    let expr = CompiledExpr::parse("2 + 3 * 4").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 14.0);

    let expr = CompiledExpr::parse("(2 + 3) * 4").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 20.0);

    let expr = CompiledExpr::parse("2 ** 3 ** 2").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 512.0); // 2^(3^2) = 2^9 = 512
}

#[test]
fn test_math_functions() {
    let expr = CompiledExpr::parse("SQRT(16) + abs(-5)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 9.0);

    let expr = CompiledExpr::parse("SUM(1, 2, 3, 4)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 10.0);

    let expr = CompiledExpr::parse("MEAN(10, 20, 30)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 20.0);

    let expr = CompiledExpr::parse("MIN(5, 2, 8)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 2.0);

    let expr = CompiledExpr::parse("MAX(5, 2, 8)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 8.0);
}

#[test]
fn test_nan_safety() {
    let expr = CompiledExpr::parse("10 / 0").unwrap();
    assert!(expr.eval(&|_| None).unwrap().is_nan());

    let expr = CompiledExpr::parse("SQRT(-1)").unwrap();
    assert!(expr.eval(&|_| None).unwrap().is_nan());

    let expr = CompiledExpr::parse("LN(0)").unwrap();
    assert!(expr.eval(&|_| None).unwrap().is_nan());
}

#[test]
fn test_variables_and_row_eval() {
    let expr = CompiledExpr::parse("weight / ((height / 100) ** 2)").unwrap();
    let mut row = HashMap::new();
    row.insert("weight".to_string(), 70.0);
    row.insert("height".to_string(), 175.0);

    let bmi = expr.eval_row(&row).unwrap();
    assert!((bmi - 22.8571).abs() < 1e-3);
}
