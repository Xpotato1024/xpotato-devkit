# devkit 初期設計メモ

## 1. リポジトリ名候補の比較

今回の要件は次の通りです。

- Gale 専用ではなく、任意の Git リポジトリで使える
- 文字化け確認、diff 要約、ブロック抽出・置換、patch 適用、Git 補助を扱う
- Python + uv で開始し、将来 Rust 移行もありうる
- CLI 名は短く、覚えやすく、コマンド入力時に邪魔にならないことが望ましい

候補を比較すると、次の順が妥当です。

### 候補A: `devkit`
長所:
- 汎用開発支援ツールとして意味が広い
- 短く打ちやすい
- `devkit diff summarize` のように自然
- Python 実装でも Rust 実装でも違和感がない

短所:
- 一般名詞寄りで、既存名称とぶつかる可能性がある
- GitHub 上で同名・類似名が多い可能性がある

### 候補B: `repokit`
長所:
- repository 向けツールであることが伝わる
- Git / diff / patch 系との相性がよい
- 名前の意味が明確

短所:
- encoding や text sanity check まで含むと少し狭く見える
- `repokit block extract` は自然だが、やや凡庸

### 候補C: `patchkit`
長所:
- ブロック編集・diff・patch 適用の性格が強く出る
- 長いファイル再生成問題への対策ツールとしては分かりやすい

短所:
- Git 周りや encoding check を含むには名前が狭い
- 将来的に用途が増えると名前負けしやすい

### 候補D: `codetool` / `devtools`
長所:
- 直感的

短所:
- 広すぎる
- 既存名称との競合が強い
- CLI 名として個性が弱い

## 2. 推奨名

### 推奨リポジトリ名
**`devkit` を第一候補**とする。

理由:
- 今回のスコープ全体を最も無理なく包含できる
- Gale 固有感がない
- サブコマンド型 CLI と相性がよい
- 今後、Python / Rust / Shell の混在にも耐える

### 代替候補
GitHub 名称衝突やパッケージ名衝突が気になる場合は、次を検討する。

- `xpotato-devkit`
- `repo-devkit`
- `devkit-cli`

特に **公開リポジトリ名** と **CLI コマンド名** は分けてもよい。

例:
- GitHub repository: `xpotato-devkit`
- CLI command: `devkit`

これはかなり実務的です。  
リポジトリ名の一意性と CLI の短さを両立できます。

## 3. CLI サブコマンド体系

初期版では、責務ごとに次の5群へ分けるのが妥当です。

```bash
devkit encoding check ...
devkit encoding normalize ...

devkit diff summarize ...

devkit block extract ...
devkit block replace ...

devkit patch apply ...

devkit git commit-message ...
devkit git pr-body ...
devkit git safe-push ...
```

この体系の利点:
- 機能群ごとに責務が明確
- 人間にも Codex にも指示しやすい
- `commands/` 配下のモジュール分割が自然
- 将来の拡張先が分かりやすい

## 4. 初期ディレクトリ構成

```text
devkit/
  README.md
  pyproject.toml
  uv.lock
  src/
    devkit/
      __init__.py
      cli.py
      commands/
        __init__.py
        encoding.py
        diff.py
        block.py
        patch.py
        git.py
      core/
        __init__.py
        encoding.py
        diff.py
        block.py
        patch.py
        git.py
  tests/
    test_smoke.py
  docs/
    requirements.md
    architecture.md
```

### 分割方針
- `commands/`: CLI 引数解釈と表示
- `core/`: 純粋な処理ロジック
- `tests/`: まずは smoke test 中心
- `docs/`: 要求定義と設計

この分割はかなり重要です。  
後で Rust 化を検討するときも、`core` の責務が見えていたほうが移植判断をしやすくなります。

## 5. Python パッケージ構成の方針

### 採用
- Python 3.12 以上
- `uv` で依存管理
- `typer` で CLI
- `rich` で表示強化
- `pydantic` は必要になってから導入でよい

### 理由
- `typer` は CLI を素早く組みやすい
- サブコマンド型と相性がよい
- 初期試作の回転が速い

## 6. 最終判断

### 決定案
- **Repository name**: `xpotato-devkit` または `devkit`
- **CLI command**: `devkit`
- **Python package**: `devkit`
- **Entry point**: `devkit=devkit.cli:app`

### 推奨
公開時の衝突回避も考えると、最初は次が最も無難です。

- GitHub repository: `xpotato-devkit`
- CLI command: `devkit`
- Python package: `devkit`

これなら名称の衝突リスクを下げつつ、使う側のコマンドは短く保てます。
