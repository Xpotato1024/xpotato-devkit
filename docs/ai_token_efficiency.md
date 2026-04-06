# AI トークン効率改善設計書

## 背景

xpotato-devkit は「AI に渡す文脈を最小化する」思想で設計されている。
しかし現時点では、各コマンドの出力は人間向けのリッチ表示（Rich テーブル、装飾付きメッセージ）が中心であり、
AI がツール呼び出しの結果を受け取る際にはむしろ冗長になるケースがある。

AI コーディングエージェントがこのツールキットを使う際の典型的な無駄は以下の通り：

1. **結果確認の冗長性** — 成功/失敗だけ知りたいのに、テーブル全体が返る（encoding check, diff summarize）
2. **構造把握の非効率** — ファイルの関数一覧は取れるが、シグネチャが取れない。結局全文を読む羽目になる
3. **ファイルツリー取得の不在** — リポジトリの構造把握に `find` / `ls -R` の生出力を使わざるを得ず、大量トークンを消費する
4. **差分ファイル一覧のみの取得手段がない** — テーブル描画なしにファイルパスだけ欲しい場合がある
5. **トークン予算を意識した文脈収集ができない** — 「この関数の周辺 N 行だけ」という指定が行範囲の手計算になる

---

## 改善提案

### A. `--brief` 全コマンド共通出力モード

**動機**: AI は多くの場合、操作の成否だけを確認できればよい。

**設計**:
- 全サブコマンドに `--brief` フラグを追加
- `--brief` が付くと、出力を **1行** に制限する
- exit code で成否を判定可能であることは前提

**出力例**:

```
# 通常モード
encoding check *.py → Rich テーブル（5列×N行）

# --brief モード
encoding check *.py --brief
OK: 12 files checked, 0 issues

encoding check *.py --brief  (問題あり)
FAIL: 12 files checked, 3 issues (src/a.py: BOM, src/b.py: mixed_newlines, src/c.py: invalid_utf8)
```

```
# 通常モード
diff summarize --staged → Rich テーブル

# --brief モード
diff summarize --staged --brief
OK: 5 files, +120/-45
```

```
# 通常モード
block replace ... → 成功メッセージ + （dry-run 時には diff）

# --brief モード
block replace ... --brief
OK: replaced 15 lines with 20 lines in src/app.py
```

```
# 通常モード
patch apply --patch-file f.patch → 成功メッセージ + hunk 情報

# --brief モード
patch apply --patch-file f.patch --brief
OK: 3 hunks applied to 2 files

# 失敗時
FAIL: 1/3 hunks failed (src/app.py: context mismatch at L42)
```

```
# md / doc コマンド
md append-section doc.md --heading Log --content "- item" --brief
OK: appended to 'Log' in doc.md
```

**実装方針**:
- コマンド層で `--brief` フラグを受け取り、出力分岐
- コア層は変更不要
- `--brief` と `--json` は排他で、`--json` が優先

**トークン削減効果**: 高。encoding check のテーブル出力は 20 ファイルで 500+ トークン → 1 行 30 トークン程度に削減。

---

### B. `devkit block outline` — シグネチャのみの抽出

**動機**: AI がファイルの API 構造を把握するために全文を読む必要がある。
`--list-functions` は名前と行番号しか返さず、引数・返り値型・デコレータが分からない。

**設計**:
- `devkit block outline <file>` コマンドの新設
- 関数/クラスのシグネチャ行だけを抽出（ボディは省略）
- デコレータ・アノテーションも含む

**出力例**:

```python
# devkit block outline src/devkit/core/block.py
L15: def list_markdown_headings(filepath: Path) -> List[dict[str, object]]:
L28: def list_functions(filepath: Path) -> List[dict[str, object]]:
L39: def suggest_candidates(target: str, choices: List[str], limit: int = 3) -> List[str]:
L88: def _find_function_end_python(lines: List[str], start_idx: int) -> int:
L101: def _find_function_end_braces(lines: List[str], start_idx: int) -> int:
L115: def _find_function_end_fallback(lines: List[str], start_idx: int) -> int:
L122: def _detect_end_strategy(filepath: Optional[Path]):
L132: def _find_heading_end(lines: List[str], start_idx: int, heading_level: int) -> int:
L186: def find_block_bounds(lines, line_range, marker, heading, function, *, heading_exact, filepath) -> Tuple[int, int]:
L250: def extract_block(filepath, line_range, marker, heading, function, *, heading_exact) -> str:
L264: def replace_block(filepath, replacement, line_range, marker, heading, function, dry_run, *, heading_exact) -> Tuple[str, str]:
L285: def diff_preview(old_block: str, new_block: str, filepath: Path | str = "file") -> str:
```

**オプション**:
- `--imports` — import 文も含める（ファイルの依存関係が分かる）
- `--docstrings` — 各関数の docstring 1 行目も含める

**トークン削減効果**: 非常に高。200 行のファイル全文を読む代わりに 15 行の概要で構造を把握できる。

---

### C. `devkit diff summarize --files-only` — ファイルパス一覧のみ

**動機**: 変更ファイルの一覧だけが欲しいケースが AI では頻出する。

**設計**:
- `--files-only` フラグ追加
- 出力は改行区切りのファイルパスのみ（テーブル・装飾なし）

**出力例**:

```
devkit diff summarize --staged --files-only
src/devkit/core/block.py
src/devkit/commands/block.py
tests/test_block.py
```

**トークン削減効果**: 中。テーブル出力の装飾・ヘッダ・統計行を省略。

---

### D. `devkit tree` — コンパクトなプロジェクト構造表示

**動機**: AI がリポジトリの全体構造を把握するために `find` や `ls -R` を使うと
巨大な出力になる。特に node_modules, .venv, __pycache__ が含まれると壊滅的。

**設計**:
- `devkit tree [path]` コマンドの新設
- `devkit.toml` の ignore パターンを自動適用
- デフォルトは `.gitignore` も尊重

**出力例**:

```
devkit tree
src/
  devkit/
    __init__.py (0B)
    cli.py (693B)
    bootstrap.py (2.1KB)
    commands/
      __init__.py (0B)
      block.py (5.6KB)
      diff.py (1.8KB)
      doc.py (2.3KB)
      encoding.py (3.5KB)
      git.py (4.9KB)
      md.py (4.8KB)
      patch.py (3.7KB)
    core/
      __init__.py (0B)
      block.py (6.4KB)
      config.py (913B)
      diff.py (3.5KB)
      doc.py (3.2KB)
      encoding.py (1.7KB)
      git.py (7.9KB)
      md.py (6.8KB)
      patch.py (4.5KB)
tests/
  test_block.py (5.2KB)
  test_doc.py (2.1KB)
  test_md.py (4.8KB)
  ...
12 directories, 28 files
```

**オプション**:
- `--max-depth N` — 深さ制限
- `--ext .py,.rs` — 拡張子フィルタ
- `--dirs-only` — ディレクトリのみ
- `--no-gitignore` — .gitignore を無視

**トークン削減効果**: 高。`find . -type f` の出力を 1/5〜1/10 に削減。

---

### E. `devkit block context` — 周辺コンテキスト収集

**動機**: AI が「ある関数の前後 N 行」を読みたい場合、現状では行範囲を手計算する必要がある。

**設計**:
- `devkit block context <file> --symbol <name> --margin N`
- 指定シンボルを中心に前後 N 行を付加して返す
- 行番号付きで返すことで、patch 指定が容易になる

**出力例**:

```
devkit block context src/app.py --symbol greet --margin 5
--- src/app.py L10-35 ---
10: import os
11:
12: # Configuration
13: MAX_RETRIES = 3
14:
15: def greet(name: str) -> str:
16:     """Say hello."""
17:     msg = f"Hello, {name}"
18:     return msg
19:
20: def farewell(name: str) -> str:
```

**トークン削減効果**: 中。全ファイルを渡す代わりに、必要な周辺だけを安全に渡せる。

---

## 優先順位

| 優先度 | 機能 | トークン削減効果 | 実装コスト |
|--------|------|-----------------|-----------|
| 1 | A. `--brief` 全コマンド共通 | 高 | 中（各コマンドに分岐追加） |
| 2 | B. `block outline` | 非常に高 | 小（既存パターン解析の延長） |
| 3 | D. `devkit tree` | 高 | 小（pathlib + gitignore 解析） |
| 4 | C. `--files-only` | 中 | 極小（1フラグ追加） |
| 5 | E. `block context` | 中 | 小（既存 extract の拡張） |

---

## 実装順序

1. **A + C**: `--brief` を全コマンドに追加 + `--files-only` — 既存コードの出力層変更のみ
2. **B**: `block outline` — 新サブコマンド
3. **D**: `devkit tree` — 新コマンドグループ
4. **E**: `block context` — 新サブコマンド
