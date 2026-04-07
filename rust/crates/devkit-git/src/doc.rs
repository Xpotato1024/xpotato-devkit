use crate::diff::{DiffSummary, FileDiff};

fn format_file_list(files: &[FileDiff], limit: usize) -> String {
    if files.is_empty() {
        return "- (none)\n".to_string();
    }
    let mut lines = Vec::new();
    let num = std::cmp::min(files.len(), limit);
    for entry in &files[..num] {
        let detail = if entry.is_binary {
            "binary".to_string()
        } else {
            format!("+{}/-{}", entry.additions, entry.deletions)
        };
        lines.push(format!("- `{}` ({})", entry.path, detail));
    }
    let remaining = files.len().saturating_sub(num);
    if remaining > 0 {
        lines.push(format!("- ... and {} more file(s)", remaining));
    }

    let mut res = lines.join("\n");
    res.push('\n');
    res
}

pub fn generate_impl_note(summary: Option<&DiffSummary>, lang: &str) -> String {
    if lang.to_lowercase().starts_with("ja") {
        impl_note_ja(summary)
    } else {
        impl_note_en(summary)
    }
}

fn impl_note_ja(summary: Option<&DiffSummary>) -> String {
    let mut scope_label = String::new();
    let mut files_section = String::new();
    let mut stats = String::new();

    if let Some(s) = summary {
        scope_label = s.scope.description.clone();
        files_section = format_file_list(&s.files, 15);
        stats = format!("+{}/-{}", s.total_additions, s.total_deletions);
    }

    let stats_line = if !stats.is_empty() {
        format!("差分統計: {}\n", stats)
    } else {
        String::new()
    };
    let scope_line = if !scope_label.is_empty() {
        format!("スコープ: {}\n", scope_label)
    } else {
        String::new()
    };

    format!(
        "\
## 変更概要
<!-- 1-2文で変更の要点をまとめる -->


## 背景
<!-- なぜこの変更が必要か -->


## 実装内容
<!-- 主要な変更点を箇条書きで記述 -->


## 変更ファイル
{}{}{}
## 検証
<!-- どのように動作確認したか -->
- [ ] テスト実行
- [ ] 手動確認

## 残課題
<!-- 未解決の問題や今後の改善点 -->

",
        files_section, stats_line, scope_line
    )
}

fn impl_note_en(summary: Option<&DiffSummary>) -> String {
    let mut scope_label = String::new();
    let mut files_section = String::new();
    let mut stats = String::new();

    if let Some(s) = summary {
        scope_label = s.scope.description.clone();
        files_section = format_file_list(&s.files, 15);
        stats = format!("+{}/-{}", s.total_additions, s.total_deletions);
    }

    let stats_line = if !stats.is_empty() {
        format!("Diff stats: {}\n", stats)
    } else {
        String::new()
    };
    let scope_line = if !scope_label.is_empty() {
        format!("Scope: {}\n", scope_label)
    } else {
        String::new()
    };

    format!(
        "\
## Summary
<!-- Summarize the change in 1-2 sentences -->


## Background
<!-- Why was this change needed? -->


## Changes
<!-- Key changes as bullet points -->


## Changed Files
{}{}{}
## Verification
<!-- How was the change tested? -->
- [ ] Tests run
- [ ] Manual verification

## Outstanding Issues
<!-- Unresolved problems or future improvements -->

",
        files_section, stats_line, scope_line
    )
}

pub fn generate_benchmark_note(summary: Option<&DiffSummary>, lang: &str) -> String {
    if lang.to_lowercase().starts_with("ja") {
        benchmark_note_ja(summary)
    } else {
        benchmark_note_en(summary)
    }
}

fn benchmark_note_ja(summary: Option<&DiffSummary>) -> String {
    let files_section = summary.map_or(String::new(), |s| format_file_list(&s.files, 15));

    format!(
        "\
## ベンチマーク概要
<!-- 計測の目的と対象を記述 -->


## 環境
<!-- OS, CPU, メモリ, ランタイムバージョンなど -->


## 手順
<!-- 再現手順を箇条書きで記述 -->


## 結果
| 項目 | Before | After | 変化率 |
|------|--------|-------|--------|
|      |        |       |        |

## 変更ファイル
{}
## 考察
<!-- 結果の解釈と次のアクション -->

",
        files_section
    )
}

fn benchmark_note_en(summary: Option<&DiffSummary>) -> String {
    let files_section = summary.map_or(String::new(), |s| format_file_list(&s.files, 15));

    format!(
        "\
## Benchmark Summary
<!-- Purpose and target of the benchmark -->


## Environment
<!-- OS, CPU, memory, runtime version, etc. -->


## Procedure
<!-- Reproducible steps as bullet points -->


## Results
| Metric | Before | After | Change |
|--------|--------|-------|--------|
|        |        |       |        |

## Changed Files
{}
## Analysis
<!-- Interpretation of results and next actions -->

",
        files_section
    )
}
