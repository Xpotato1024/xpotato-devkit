# Why devkit

`devkit` は、AI 支援開発で発生しやすい「全文読込」「全文再生成」「雑な patch 適用」を減らすための CLI です。

## 何がつらいのか

一般的な shell コマンドだけでも開発は進められますが、AI や人間が次のような流れに戻りやすくなります。

- `git diff` の生出力を長く読み直す
- 長いファイルを `Get-Content` や `cat` で丸ごと読む
- 該当箇所の見当がつかず `Get-ChildItem -Recurse` や `tree` を雑に叩く
- 関数 1 個の修正でもファイル全体を書き換える
- patch の失敗理由が曖昧なまま全文再読込に戻る

`devkit` はこの固定費を、構造化された deterministic なコマンドに置き換えるための道具です。

## devkit が重視する流れ

`devkit` は `inspect -> edit -> verify` を崩さないことを重視します。

1. `devkit diff summarize` で変更の全体像を先に掴む
2. `devkit tree` や `devkit block outline/context/extract` で必要な部分だけを読む
3. 小さい patch や block 単位で編集する
4. `devkit patch diagnose` で適用可否を確認してから `devkit patch apply` する
5. 最後に `devkit diff summarize` を再実行して結果を確認する

この流れは、全文再生成型の編集よりも token 消費、レビュー負荷、破壊的変更の混入を抑えやすくします。

## なぜ PowerShell ワンライナーより専用 CLI なのか

PowerShell ワンライナーは柔軟ですが、次の弱点があります。

- 出力が毎回人間向けで、後続ツールが読みづらい
- 再利用時に quoting や path 展開が不安定になりやすい
- `Get-Content` や `Get-ChildItem -Recurse` による過剰読込へ流れやすい
- 何を「正しい最小単位」とするかが利用者に委ねられる

`devkit` は `--brief` と JSON を前提に、同じ入力に対して同じ種類の出力を返すことを重視します。

## Windows で特に効く場面

- `Get-ChildItem -Recurse` の代わりに `devkit tree --max-depth 2 --limit 40`
- `Get-Content` の代わりに `devkit block context` / `devkit block extract`
- `git diff` の概要確認の代わりに `devkit diff summarize --name-status`
- 既定 shell に依存した重い one-liner の代わりに `--brief` / JSON 出力

## FAQ

### grep や git diff だけではだめなのか

だめではありません。ただし、`devkit` は変更把握、局所抽出、patch 診断の出力契約を安定化し、AI や後続ツールが扱いやすい形に揃えます。

### なぜ patch diagnose が必要なのか

patch の失敗を適用時に初めて知ると、全文再読込や diff のやり直しに戻りがちです。`devkit patch diagnose` を先に通すと、次のように失敗を局所化できます。

- invalid patch input: patch ファイルではなく本文や別ファイルを渡していないかを確認する
- target missing: まず `devkit tree` で対象ファイルの存在を確認する
- context mismatch: `devkit search` や `devkit block context` で現行文脈を再取得して patch を作り直す
- already applied or reversed: すでに反映済みか、逆向き patch を適用しようとしていないかを確認する

重要なのは、失敗時に「とりあえず全文を読み直す」ではなく、「分類に応じた次の一手」に進めることです。`--brief` なら 1 行、`--json` なら後続ツールへ渡せる構造で取れます。

### なぜ全文読込を避けるのか

長いファイルを毎回全文読むと、token 消費だけでなく、関係ない文脈が混ざって編集範囲が膨らみやすくなります。

### 人間だけでも使う価値はあるか

あります。`devkit` は AI 専用ではなく、長いファイルや大きな repo を局所的に扱いたい人間にも有効です。

### AI エージェントと組み合わせると何が変わるか

エージェントが shell の雑な全文読込に逃げにくくなり、`--brief` や JSON を使って次の処理へ渡しやすくなります。

### Windows で `Get-Content` や `Get-ChildItem` より何が良いのか

`devkit` は最初から局所化と機械可読性を前提にしているため、過剰読込とノイズを減らしやすくなります。
