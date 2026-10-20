//! 四則演算エンジンの公開API統合テスト。

use Re_CraftLaunch::calc::{evaluate, is_evaluable};

#[test]
fn test_calc_basic_operations() {
    assert_eq!(evaluate("10 + 20").unwrap().formatted_result, "30");
    assert_eq!(evaluate("100 - 45").unwrap().formatted_result, "55");
    assert_eq!(evaluate("25 * 4").unwrap().formatted_result, "100");
    assert_eq!(evaluate("100 / 4").unwrap().formatted_result, "25");
    assert_eq!(evaluate("10 % 3").unwrap().formatted_result, "1");
}

#[test]
fn test_calc_compound_and_parentheses() {
    let res = evaluate("(100 + 200) * 3").unwrap();
    assert_eq!(res.formatted_result, "900");
    assert_eq!(res.display_text, "= 900");
    assert_eq!(res.copy_text, "900");

    let nested = evaluate("((10 + 20) * 2) - 10").unwrap();
    assert_eq!(nested.formatted_result, "50");
}

#[test]
fn test_calc_decimal_and_comma_formatting() {
    let res = evaluate("1280 * 1.1").unwrap();
    assert_eq!(res.formatted_result, "1,408");
    assert_eq!(res.display_text, "= 1,408");

    // 実数除算の自動処理
    let res_div = evaluate("10 / 4").unwrap();
    assert_eq!(res_div.formatted_result, "2.5");
    assert_eq!(res_div.copy_text, "2.5");

    let res_large = evaluate("1000000 / 3").unwrap();
    assert!(res_large.formatted_result.starts_with("333,333.33"));
}

#[test]
fn test_calc_negative_and_zero_division() {
    let res = evaluate("-50 + 20").unwrap();
    assert_eq!(res.formatted_result, "-30");

    // ゼロ除算はパニックせず None を返す
    assert!(evaluate("10 / 0").is_none());
    assert!(evaluate("0 / 0").is_none());
}

#[test]
fn test_calc_invalid_and_edge_cases() {
    // 文字列混在は評価不能
    assert!(!is_evaluable("chrome.exe"));
    assert!(evaluate("chrome.exe").is_none());
    // 単なる数字のみはランチャー検索として扱うため計算評価対象外
    assert!(!is_evaluable("12345"));
    assert!(!is_evaluable("-12345"));
    assert!(!is_evaluable("   "));
}
