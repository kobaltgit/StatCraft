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

#[test]
fn test_relational_and_logical_operators() {
    let expr = CompiledExpr::parse("age >= 18 & (salary > 30000 | bonus = 5000)").unwrap();
    let mut row1 = HashMap::new();
    row1.insert("age".to_string(), 25.0);
    row1.insert("salary".to_string(), 35000.0);
    row1.insert("bonus".to_string(), 1000.0);
    assert_eq!(expr.eval_row(&row1).unwrap(), 1.0);

    let mut row2 = HashMap::new();
    row2.insert("age".to_string(), 16.0);
    row2.insert("salary".to_string(), 40000.0);
    row2.insert("bonus".to_string(), 5000.0);
    assert_eq!(expr.eval_row(&row2).unwrap(), 0.0);

    let mut row3 = HashMap::new();
    row3.insert("age".to_string(), 30.0);
    row3.insert("salary".to_string(), 20000.0);
    row3.insert("bonus".to_string(), 5000.0);
    assert_eq!(expr.eval_row(&row3).unwrap(), 1.0);

    // Test text keywords and negation
    let expr2 = CompiledExpr::parse("NOT (gender EQ 1) AND age GT 20").unwrap();
    let mut row4 = HashMap::new();
    row4.insert("gender".to_string(), 2.0);
    row4.insert("age".to_string(), 25.0);
    assert_eq!(expr2.eval_row(&row4).unwrap(), 1.0);

    let mut row5 = HashMap::new();
    row5.insert("gender".to_string(), 1.0);
    row5.insert("age".to_string(), 25.0);
    assert_eq!(expr2.eval_row(&row5).unwrap(), 0.0);
}

#[test]
fn test_spss_precedence_canon() {
    // Exponentiation precedes unary minus in IBM SPSS: -2**2 == -4.0
    let expr = CompiledExpr::parse("-2**2").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), -4.0);

    let expr = CompiledExpr::parse("(-2)**2").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 4.0);

    let expr = CompiledExpr::parse("-(-2)**2").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), -4.0);

    // Relational operators precede NOT: NOT 5 = 5 -> NOT (5 = 5) -> 0.0
    let expr = CompiledExpr::parse("NOT 5 = 5").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 0.0);

    let expr = CompiledExpr::parse("NOT 5 <> 5").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 1.0);

    let expr = CompiledExpr::parse("NOT 0 AND 1").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 1.0);
}

#[test]
fn test_kleene_3vl_logic() {
    // Kleene OR: TRUE OR NaN -> 1.0, NaN OR TRUE -> 1.0
    let expr1 = CompiledExpr::parse("1 OR SQRT(-1)").unwrap();
    assert_eq!(expr1.eval(&|_| None).unwrap(), 1.0);

    let expr2 = CompiledExpr::parse("SQRT(-1) OR 1").unwrap();
    assert_eq!(expr2.eval(&|_| None).unwrap(), 1.0);

    // FALSE OR NaN -> NaN
    let expr3 = CompiledExpr::parse("0 OR SQRT(-1)").unwrap();
    assert!(expr3.eval(&|_| None).unwrap().is_nan());

    // Kleene AND: FALSE AND NaN -> 0.0, NaN AND FALSE -> 0.0
    let expr4 = CompiledExpr::parse("0 AND SQRT(-1)").unwrap();
    assert_eq!(expr4.eval(&|_| None).unwrap(), 0.0);

    let expr5 = CompiledExpr::parse("SQRT(-1) AND 0").unwrap();
    assert_eq!(expr5.eval(&|_| None).unwrap(), 0.0);

    // TRUE AND NaN -> NaN
    let expr6 = CompiledExpr::parse("1 AND SQRT(-1)").unwrap();
    assert!(expr6.eval(&|_| None).unwrap().is_nan());

    // NOT NaN -> NaN
    let expr7 = CompiledExpr::parse("NOT SQRT(-1)").unwrap();
    assert!(expr7.eval(&|_| None).unwrap().is_nan());
}

#[test]
fn test_spss_math_functions_canon() {
    let expr = CompiledExpr::parse("SIN(0) + COS(0)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 1.0);

    let expr = CompiledExpr::parse("TAN(0)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 0.0);

    let expr = CompiledExpr::parse("MOD(10, 3)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 1.0);

    let expr = CompiledExpr::parse("MOD(10, 0)").unwrap();
    assert!(expr.eval(&|_| None).unwrap().is_nan());

    let expr = CompiledExpr::parse("LG10(100)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 2.0);

    // Strict MEAN without .n returns NaN when any argument is NaN
    let expr = CompiledExpr::parse("MEAN(10, 20, 30)").unwrap();
    assert_eq!(expr.eval(&|_| None).unwrap(), 20.0);

    let expr = CompiledExpr::parse("MEAN(10, SQRT(-1), 30)").unwrap();
    assert!(expr.eval(&|_| None).unwrap().is_nan());
}
