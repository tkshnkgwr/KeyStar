//! インクリメンタル補完・検索エンジンモジュール。
//!
//! ユーザーの入力文字列をもとに、ショートカット一覧から前方一致および部分一致で候補を絞り込み、
//! 電卓の計算結果が存在する場合は最優先候補として統合します。
//! 引数付き入力（例: `code .` や `chrome https://...`）のコマンド単語検出にも対応しています。

use crate::calc::CalcResult;
use crate::command::{BuiltinCommand, BuiltinCommandDef, match_builtin_commands};
use crate::indexer::ShortcutItem;

/// 候補アイテムの種類。
#[derive(Debug, Clone, PartialEq)]
pub enum CandidateKind {
    /// 数式評価・電卓の計算結果
    Calc(CalcResult),
    /// ショートカット・実行可能アイテム
    Shortcut(ShortcutItem),
    /// ビルトイン（スラッシュ）コマンド
    Builtin(BuiltinCommand),
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

    /// ビルトイン（スラッシュ）コマンド定義から候補アイテムを生成します。
    ///
    /// # Panics
    ///
    /// パニックは発生しません。
    #[must_use]
    pub fn from_builtin(def: BuiltinCommandDef) -> Self {
        let label = format!("{} — {}", def.name, def.description);
        Self {
            label,
            kind: CandidateKind::Builtin(def.command),
            score: 10_000,
        }
    }
}

/// 入力文字列をコマンド検索用プレフィックスと起動時引数に分割します。
///
/// 先頭の空白をトリムした後、最初の空白文字（スペース等）でコマンド部分と引数部分に分割します。
/// 空白文字が含まれない場合は、引数部分は空文字列 `""` となります。
///
/// # Examples
///
/// ```
/// use KeyStar::search::split_command_and_args;
///
/// assert_eq!(split_command_and_args("notepad"), ("notepad", ""));
/// assert_eq!(split_command_and_args("notepad "), ("notepad", ""));
/// assert_eq!(split_command_and_args("notepad test.txt"), ("notepad", "test.txt"));
/// assert_eq!(split_command_and_args("  code  . "), ("code", ". "));
/// ```
#[must_use]
pub fn split_command_and_args(input: &str) -> (&str, &str) {
    let trimmed = input.trim_start();
    if let Some(idx) = trimmed.find(|c: char| c.is_whitespace()) {
        let cmd = &trimmed[..idx];
        let args = trimmed[idx..].trim_start();
        (cmd, args)
    } else {
        (trimmed, "")
    }
}

/// 入力文字列とショートカット群から候補リストをフィルタリング・生成します。
///
/// 1. 入力が数式として成立する場合は、最優先候補として電卓結果を先頭（index 0）に配置します。
/// 2. ショートカット名に対して前方一致（高スコア）および部分一致（中スコア）で評価します。
/// 3. 引数付きクエリ（例: `chrome https://...`）の場合、第1単語でのショートカット照合も行います。
/// 4. クエリが空の場合は、ショートカット一覧の先頭から `max_results` 件を返します。
/// 5. 結果件数は `max_results` 件以内に収まるよう切り詰められます。
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
/// // 引数付きクエリでの検索維持
/// let candidates_with_args = filter_candidates("ch https://google.com", &shortcuts, None, 10);
/// assert_eq!(candidates_with_args.len(), 1);
/// assert_eq!(candidates_with_args[0].label, "Chrome (.lnk)");
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
    let trimmed = query.trim();

    // 0. スラッシュコマンド判定（クエリが "/" で始まる場合は専用モード）
    if trimmed.starts_with('/') {
        let matched_commands = match_builtin_commands(trimmed);
        return matched_commands
            .into_iter()
            .take(max_results)
            .map(Candidate::from_builtin)
            .collect();
    }

    let mut candidates = Vec::new();

    // 1. 電卓結果があれば最優先で先頭に追加
    if let Some(calc) = calc_result {
        candidates.push(Candidate::from_calc(calc));
    }

    // 2. クエリが空の場合のデフォルト表示
    if trimmed.is_empty() {
        let remaining = max_results.saturating_sub(candidates.len());
        for item in shortcuts.iter().take(remaining) {
            candidates.push(Candidate::from_shortcut(item.clone(), 0));
        }
        return candidates;
    }

    // 3. ショートカットのマッチングとスコアリング
    // 引数対応: クエリ全体（スペース含む）および コマンド部分（先頭単語）の両方でスコアリング
    let (cmd_part, _args_part) = split_command_and_args(trimmed);
    let lower_full = trimmed.to_lowercase();
    let lower_cmd = cmd_part.to_lowercase();
    let has_args = !cmd_part.is_empty() && trimmed.len() > cmd_part.len();

    let mut matched_shortcuts = Vec::new();

    for item in shortcuts {
        let lower_name = item.name.to_lowercase();
        let mut best_score: Option<i32> = None;

        // A. クエリ全体での判定（ショートカット名自体にスペースが含まれている場合など）
        if lower_name.starts_with(&lower_full) {
            let diff = lower_name.len().saturating_sub(lower_full.len()) as i32;
            best_score = Some(1000 - diff);
        } else if let Some(pos) = lower_name.find(&lower_full) {
            best_score = Some(500 - (pos as i32 * 10) - (lower_name.len() as i32));
        }

        // B. 引数入力中（スペースあり）の場合、コマンド部分（第1単語）での判定
        if has_args && !lower_cmd.is_empty() {
            let cmd_score = if lower_name.starts_with(&lower_cmd) {
                let diff = lower_name.len().saturating_sub(lower_cmd.len()) as i32;
                Some(900 - diff)
            } else {
                lower_name
                    .find(&lower_cmd)
                    .map(|pos| 400 - (pos as i32 * 10) - (lower_name.len() as i32))
            };

            if let Some(cs) = cmd_score {
                best_score = Some(best_score.map_or(cs, |bs| bs.max(cs)));
            }
        }

        if let Some(score) = best_score {
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
    fn test_split_command_and_args() {
        assert_eq!(split_command_and_args(""), ("", ""));
        assert_eq!(split_command_and_args("notepad"), ("notepad", ""));
        assert_eq!(split_command_and_args("notepad "), ("notepad", ""));
        assert_eq!(
            split_command_and_args("notepad test.txt"),
            ("notepad", "test.txt")
        );
        assert_eq!(
            split_command_and_args("code  C:\\path\\dir  "),
            ("code", "C:\\path\\dir  ")
        );
        assert_eq!(
            split_command_and_args("  cmd /k ping google.com"),
            ("cmd", "/k ping google.com")
        );
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
    fn test_filter_with_command_and_args() {
        let shortcuts = vec![make_item("Chrome", "lnk"), make_item("Notepad", "lnk")];

        // 引数を入力しても、コマンド名に前方一致していれば候補が残り続ける
        let results = filter_candidates("chrome https://google.com", &shortcuts, None, 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "Chrome (.lnk)");

        // 部分一致のコマンドでも同様
        let results_partial = filter_candidates("pad myfile.txt", &shortcuts, None, 10);
        assert_eq!(results_partial.len(), 1);
        assert_eq!(results_partial[0].label, "Notepad (.lnk)");
    }

    #[test]
    fn test_filter_with_space_in_shortcut_name() {
        let shortcuts = vec![
            make_item("Visual Studio Code", "lnk"),
            make_item("Visual Studio", "lnk"),
        ];

        // ショートカット名自体に空白を含む場合、より長く一致する方が最上位になる
        let results = filter_candidates("visual studio c", &shortcuts, None, 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].label, "Visual Studio Code (.lnk)");
    }

    #[test]
    fn test_filter_substring_matching() {
        let shortcuts = vec![make_item("GoogleChrome", "lnk"), make_item("Chrome", "lnk")];

        // "Chrome" は前方一致、"GoogleChrome" は部分一致で前方一致が上位
        let results = filter_candidates("chrome", &shortcuts, None, 2);
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
        let results = filter_candidates("xyz_no_match", &shortcuts, None, 10);
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_slash_commands() {
        let shortcuts = vec![make_item("Chrome", "lnk")];

        // "/" 単体で全スラッシュコマンドが展開される
        let results_all = filter_candidates("/", &shortcuts, None, 20);
        assert!(!results_all.is_empty());
        assert!(results_all.iter().any(|c| c.label.contains("/task")));

        // "/ta" でタスクマネージャーが最上位
        let results_task = filter_candidates("/ta", &shortcuts, None, 10);
        assert!(!results_task.is_empty());
        assert_eq!(
            results_task[0].kind,
            CandidateKind::Builtin(BuiltinCommand::Task)
        );
    }
}
