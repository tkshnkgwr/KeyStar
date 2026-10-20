//! 入力欄での即時四則演算・数式評価モジュール。
//!
//! `evalexpr` クレートを利用して、入力文字列から安全に計算式を評価し、
//! 整形された計算結果を生成します。

use evalexpr::{Value, eval};

/// 計算結果を保持する構造体。
#[derive(Debug, Clone, PartialEq)]
pub struct CalcResult {
    /// 計算された生の値（浮動小数点数）
    pub raw_value: f64,
    /// 3桁カンマ区切り等で整形された結果文字列（例: `"1,408"`, `"3.14159"`）
    pub formatted_result: String,
    /// ランチャー候補行に表示するテキスト（例: `"= 1,408"`）
    pub display_text: String,
    /// クリップボードにコピーするためのプレーンな数値文字列（例: `"1408"`）
    pub copy_text: String,
}

/// 入力文字列が計算式として評価可能であるかを判定します。
///
/// 単なる文字入力や単語、単一の数字のみの場合は `false` を返し、
/// 少なくとも1つの演算子（`+`, `-`, `*`, `/`, `%`）または括弧と数値を含んでいる場合に `true` を返します。
///
/// # Panics
///
/// 本関数はどのような入力に対してもパニックを引き起こしません。
///
/// # Examples
///
/// ```
/// use Re_CraftLaunch::calc::is_evaluable;
///
/// assert!(is_evaluable("1 + 2"));
/// assert!(is_evaluable("1280 * 1.1"));
/// assert!(is_evaluable("(10 + 20) / 2"));
/// assert!(!is_evaluable("chrome"));
/// assert!(!is_evaluable("123")); // 単なる数字のみはランチャー検索とみなす
/// ```
#[must_use]
pub fn is_evaluable(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return false;
    }

    // 数字が含まれているか
    let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());
    if !has_digit {
        return false;
    }

    // 計算演算子または括弧が含まれているか
    let has_operator = trimmed
        .chars()
        .any(|c| matches!(c, '+' | '-' | '*' | '/' | '%' | '(' | ')'));
    if !has_operator {
        return false;
    }

    // 単なる負の数（例: "-123"）や符号付き単一数値は数字のみとみなして除外
    if (trimmed.starts_with('-') || trimmed.starts_with('+'))
        && trimmed[1..].chars().all(|c| c.is_ascii_digit() || c == '.')
    {
        return false;
    }

    true
}

/// 入力文字列を数式として評価し、整形された計算結果を返します。
///
/// 四則演算（加減乗除・剰余）および括弧の優先順位を正しく評価します。
/// 式内に除算 `/` が含まれる場合は、電卓としての実用性を確保するため自動的に実数除算として計算されます。
/// 式が無効、パースエラー、ゼロ除算、または数値以外の結果（真偽値等）となった場合は `None` を返します。
///
/// # Panics
///
/// ゼロ除算やオーバーフロー、不正な文法などの異常入力に対しても安全に `None` を返し、パニックは発生しません。
///
/// # Examples
///
/// ```
/// use Re_CraftLaunch::calc::evaluate;
///
/// let res = evaluate("1280 * 1.1").unwrap();
/// assert_eq!(res.formatted_result, "1,408");
/// assert_eq!(res.display_text, "= 1,408");
///
/// // 実数除算の自動処理
/// let res_div = evaluate("10 / 4").unwrap();
/// assert_eq!(res_div.formatted_result, "2.5");
///
/// assert!(evaluate("1 / 0").is_none()); // ゼロ除算はNone
/// assert!(evaluate("invalid expr").is_none());
/// ```
#[must_use]
pub fn evaluate(query: &str) -> Option<CalcResult> {
    if !is_evaluable(query) {
        return None;
    }

    // 除算 '/' が含まれている場合は実数除算として優先評価（10 / 4 = 2.5 を実現）
    let value = if query.contains('/') && !query.contains('%') {
        let promoted = promote_integers_to_floats(query);
        eval(&promoted).ok().or_else(|| eval(query).ok())?
    } else {
        eval(query).ok()?
    };

    let float_val = match value {
        Value::Int(i) => i as f64,
        Value::Float(f) => {
            if f.is_nan() || f.is_infinite() {
                return None;
            }
            f
        }
        _ => return None,
    };

    let (formatted, copy_val) = format_number(float_val);
    let display = format!("= {formatted}");

    Some(CalcResult {
        raw_value: float_val,
        formatted_result: formatted,
        display_text: display,
        copy_text: copy_val,
    })
}

/// 式内の整数リテラル（例: `10`）を浮動小数点数（例: `10.0`）に昇格させます。
fn promote_integers_to_floats(expr: &str) -> String {
    let mut result = String::with_capacity(expr.len() + 16);
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_ascii_digit() {
            let start = i;
            let mut has_dot = false;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                if chars[i] == '.' {
                    has_dot = true;
                }
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            result.push_str(&num_str);
            if !has_dot {
                result.push_str(".0");
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

/// 数値を3桁カンマ区切りの文字列と、クリップボード用プレーン文字列に整形します。
fn format_number(val: f64) -> (String, String) {
    // 整数かどうか判定（誤差考慮）
    let is_integer = (val.fract().abs() < 1e-9) && val.abs() <= (i64::MAX as f64);

    if is_integer {
        let int_val = val.round() as i64;
        let formatted = format_integer_with_commas(int_val);
        let plain = int_val.to_string();
        (formatted, plain)
    } else {
        // 小数の場合、最大6桁程度で余計なゼロを省く
        let raw_str = format!("{val:.6}");
        let trimmed_str = raw_str.trim_end_matches('0').trim_end_matches('.');

        // 整数部と小数部に分離
        if let Some((int_part, frac_part)) = trimmed_str.split_once('.') {
            if let Ok(int_val) = int_part.parse::<i64>() {
                let formatted_int = format_integer_with_commas(int_val);
                (
                    format!("{formatted_int}.{frac_part}"),
                    trimmed_str.to_string(),
                )
            } else {
                (trimmed_str.to_string(), trimmed_str.to_string())
            }
        } else {
            (trimmed_str.to_string(), trimmed_str.to_string())
        }
    }
}

/// 整数値を3桁カンマ区切りの文字列に変換します。
fn format_integer_with_commas(n: i64) -> String {
    let is_negative = n < 0;
    let abs_str = n.unsigned_abs().to_string();
    let len = abs_str.len();

    let mut result = String::with_capacity(len + (len / 3) + 1);
    if is_negative {
        result.push('-');
    }

    let remainder = len % 3;
    let mut chars = abs_str.chars();

    if remainder > 0 {
        for _ in 0..remainder {
            if let Some(c) = chars.next() {
                result.push(c);
            }
        }
        if len > remainder {
            result.push(',');
        }
    }

    for (count, c) in chars.enumerate() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_evaluable() {
        assert!(is_evaluable("1 + 2"));
        assert!(is_evaluable("100 * 1.1"));
        assert!(is_evaluable("(1 + 2) * 3"));
        assert!(is_evaluable("10 % 3"));
        assert!(!is_evaluable(""));
        assert!(!is_evaluable("   "));
        assert!(!is_evaluable("chrome"));
        assert!(!is_evaluable("12345"));
        assert!(!is_evaluable("-12345"));
        assert!(!is_evaluable("+123.45"));
        assert!(!is_evaluable("abc + def")); // 数字なし
    }

    #[test]
    fn test_evaluate_basic_arithmetic() {
        let res = evaluate("1 + 2").unwrap();
        assert_eq!(res.raw_value, 3.0);
        assert_eq!(res.formatted_result, "3");
        assert_eq!(res.display_text, "= 3");
        assert_eq!(res.copy_text, "3");

        let res = evaluate("1280 * 1.1").unwrap();
        assert_eq!(res.raw_value, 1408.0);
        assert_eq!(res.formatted_result, "1,408");
        assert_eq!(res.display_text, "= 1,408");

        let res = evaluate("(10 + 20) * 30").unwrap();
        assert_eq!(res.raw_value, 900.0);
        assert_eq!(res.formatted_result, "900");
    }

    #[test]
    fn test_evaluate_large_numbers_and_decimals() {
        let res = evaluate("1000000 + 2500").unwrap();
        assert_eq!(res.formatted_result, "1,002,500");

        let res = evaluate("10 / 4").unwrap();
        assert_eq!(res.formatted_result, "2.5");
        assert_eq!(res.copy_text, "2.5");

        let res_mod = evaluate("10 % 3").unwrap();
        assert_eq!(res_mod.formatted_result, "1");
    }

    #[test]
    fn test_evaluate_negative_numbers() {
        let res = evaluate("-10 + 5").unwrap();
        assert_eq!(res.formatted_result, "-5");
        assert_eq!(res.display_text, "= -5");
        assert_eq!(res.copy_text, "-5");

        let res2 = evaluate("10 * -2").unwrap();
        assert_eq!(res2.formatted_result, "-20");
    }

    #[test]
    fn test_evaluate_nested_parentheses() {
        let res = evaluate("((2 + 3) * (4 + 1)) / 5").unwrap();
        assert_eq!(res.formatted_result, "5");
    }

    #[test]
    fn test_evaluate_invalid_cases() {
        assert!(evaluate("1 / 0").is_none());
        assert!(evaluate("1 + +").is_none());
        assert!(evaluate("foo + bar").is_none());
        assert!(evaluate("").is_none());
        assert!(evaluate("   ").is_none());
    }

    #[test]
    fn test_format_integer_with_commas() {
        assert_eq!(format_integer_with_commas(0), "0");
        assert_eq!(format_integer_with_commas(12), "12");
        assert_eq!(format_integer_with_commas(123), "123");
        assert_eq!(format_integer_with_commas(1234), "1,234");
        assert_eq!(format_integer_with_commas(1234567), "1,234,567");
        assert_eq!(format_integer_with_commas(-1234), "-1,234");
        assert_eq!(format_integer_with_commas(-1000000), "-1,000,000");
    }
}
