//! インクリメンタル補完・検索エンジンモジュール。
//!
//! ユーザーの入力文字列をもとに、ショートカット一覧から前方一致および部分一致で候補を絞り込み、
//! 電卓の計算結果が存在する場合は最優先候補として統合します。

use crate::calc::CalcResult;
use crate::indexer::ShortcutItem;

/// 候補アイテムの種類。
#[derive(Debug, Clone, PartialEq)]
pub enum CandidateKind {
    /// 数式評価・電卓の計算結果
    Calc(CalcResult),
    /// ショートカット・実行可能アイテム
    Shortcut(ShortcutItem),
}

/// ランチャーの候補リストに表示される個別アイテム。
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    /// UIに表示するテキスト（例: `"= 1,408"`, `"Chrome"`, `"BuildScript (.bat)"`）
    pub label: String,
    /// 候補の種別データ
    pub kind: CandidateKind,
    /// ソート用のマッチングスコア（高いほど優先）
    pub score: i32,
}

impl Candidate {
    /// 電卓結果から最優先候補アイテムを生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    ///
    /// # Examples
    ///
    /// ```
    /// use KeyStar::calc::CalcResult;
    /// use KeyStar::search::Candidate;
    ///
    /// let calc = CalcResult {
    ///     raw_value: 42.0,
    ///     formatted_result: "42".to_string(),
    ///     display_text: "= 42".to_string(),
    ///     copy_text: "42".to_string(),
    /// };
    /// let cand = Candidate::from_calc(calc);
    /// assert_eq!(cand.label, "= 42");
    /// assert_eq!(cand.score, 10_000);
    /// ```
    #[must_use]
    pub fn from_calc(calc: CalcResult) -> Self {
        let label = calc.display_text.clone();
        Self {
            label,
            kind: CandidateKind::Calc(calc),
            score: 10_000, // 電卓結果は最優先
        }
    }

    /// ショートカットアイテムから候補アイテムを生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    #[must_use]
    pub fn from_shortcut(item: ShortcutItem, score: i32) -> Self {
        let label = format!("{} (.{})", item.name, item.extension);
        Self {
            label,
            kind: CandidateKind::Shortcut(item),
            score,
        }
    }
}

/// 入力文字列とショートカット群から候補リストをフィルタリング・生成します。
///
/// 1. 入力が数式として成立する場合は、最優先候補として電卓結果を先頭（index 0）に配置します。
/// 2. ショートカット名に対して前方一致（高スコア）および部分一致（中スコア）で評価します。
/// 3. クエリが空の場合は、ショートカット一覧の先頭から `max_results` 件を返します。
/// 4. 結果件数は `max_results` 件以内に収まるよう切り詰められます。
///
/// # Panics
///
/// どのような入力文字列やアイテム群に対してもパニックを引き起こしません。
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use KeyStar::calc::evaluate;
/// use KeyStar::indexer::ShortcutItem;
/// use KeyStar::search::filter_candidates;
///
/// let shortcuts = vec![
///     ShortcutItem {
///         name: "Chrome".to_string(),
///         path: PathBuf::from("Chrome.lnk"),
///         extension: "lnk".to_string(),
///     },
/// ];
///
/// // 前方一致検索
/// let candidates = filter_candidates("ch", &shortcuts, None, 10);
/// assert_eq!(candidates.len(), 1);
/// assert_eq!(candidates[0].label, "Chrome (.lnk)");
///
/// // 電卓優先マージ
/// let calc = evaluate("1 + 2");
/// let candidates = filter_candidates("1 + 2", &shortcuts, calc, 10);
/// assert_eq!(candidates[0].label, "= 3");
/// ```
#[must_use]
pub fn filter_candidates(
    query: &str,
    shortcuts: &[ShortcutItem],
    calc_result: Option<CalcResult>,
    max_results: usize,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    // 1. 電卓結果があれば最優先で先頭に追加
    if let Some(calc) = calc_result {
        candidates.push(Candidate::from_calc(calc));
    }

    let trimmed = query.trim();

    // 2. クエリが空の場合のデフォルト表示
    if trimmed.is_empty() {
        let remaining = max_results.saturating_sub(candidates.len());
        for item in shortcuts.iter().take(remaining) {
            candidates.push(Candidate::from_shortcut(item.clone(), 0));
        }
        return candidates;
    }

    // 3. ショートカットのマッチングとスコアリング
    let lower_query = trimmed.to_lowercase();
    let mut matched_shortcuts = Vec::new();

    for item in shortcuts {
        let lower_name = item.name.to_lowercase();

        if lower_name.starts_with(&lower_query) {
            // 前方一致: 高スコア（余計な文字数が少ないほど高得点）
            let diff = lower_name.len().saturating_sub(lower_query.len()) as i32;
            let score = 1000 - diff;
            matched_shortcuts.push((item, score));
        } else if let Some(pos) = lower_name.find(&lower_query) {
            // 部分一致: 中スコア（前方に出現するほど、かつ余計な文字数が少ないほど高得点）
            let score = 500 - (pos as i32 * 10) - (lower_name.len() as i32);
            matched_shortcuts.push((item, score));
        }
    }

    // スコア降順（高い順）、同点なら名前昇順でソート
    matched_shortcuts.sort_by(|(a_item, a_score), (b_item, b_score)| {
        b_score
            .cmp(a_score)
            .then_with(|| a_item.name.to_lowercase().cmp(&b_item.name.to_lowercase()))
    });

    // 最大件数まで追加
    let remaining_slots = max_results.saturating_sub(candidates.len());
    for (item, score) in matched_shortcuts.into_iter().take(remaining_slots) {
        candidates.push(Candidate::from_shortcut(item.clone(), score));
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_item(name: &str, ext: &str) -> ShortcutItem {
        ShortcutItem {
            name: name.to_string(),
            path: PathBuf::from(format!("{name}.{ext}")),
            extension: ext.to_string(),
        }
    }

    #[test]
    fn test_filter_prefix_matching() {
        let shortcuts = vec![
            make_item("Chrome", "lnk"),
            make_item("Firefox", "lnk"),
            make_item("CommandPrompt", "lnk"),
        ];

        let results = filter_candidates("ch", &shortcuts, None, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Chrome (.lnk)");

        // 大文字小文字不問
        let results_caps = filter_candidates("CH", &shortcuts, None, 10);
        assert_eq!(results_caps.len(), 1);
        assert_eq!(results_caps[0].label, "Chrome (.lnk)");
    }

    #[test]
    fn test_filter_substring_matching() {
        let shortcuts = vec![make_item("GoogleChrome", "lnk"), make_item("Chrome", "lnk")];

        // "Chrome" は前方一致、"GoogleChrome" は部分一致で前方一致が上位
        let results = filter_candidates("chrome", &shortcuts, None, 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].label, "Chrome (.lnk)");
        assert_eq!(results[1].label, "GoogleChrome (.lnk)");
    }

    #[test]
    fn test_filter_with_calc_result() {
        let shortcuts = vec![make_item("123Launcher", "lnk")];
        let calc = CalcResult {
            raw_value: 3.0,
            formatted_result: "3".to_string(),
            display_text: "= 3".to_string(),
            copy_text: "3".to_string(),
        };

        let results = filter_candidates("1 + 2", &shortcuts, Some(calc), 10);
        assert_eq!(results[0].label, "= 3");
        assert!(matches!(results[0].kind, CandidateKind::Calc(_)));
    }

    #[test]
    fn test_filter_empty_query_and_limit() {
        let shortcuts = vec![
            make_item("AppA", "lnk"),
            make_item("AppB", "lnk"),
            make_item("AppC", "lnk"),
        ];

        let results = filter_candidates("", &shortcuts, None, 2);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].label, "AppA (.lnk)");
        assert_eq!(results[1].label, "AppB (.lnk)");
    }

    #[test]
    fn test_filter_no_match() {
        let shortcuts = vec![make_item("Chrome", "lnk")];
        let results = filter_candidates("notepad", &shortcuts, None, 10);
        assert!(results.is_empty());
    }
}
