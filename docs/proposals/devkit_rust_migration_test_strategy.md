# xpotato-devkit Rust移行判断のためのテスト方針書

## 1. 目的
本書の目的は、`xpotato-devkit` を今後 Rust に移行すべきかどうかを、感覚ではなく計測結果に基づいて判断できるようにすることである。

現状の `xpotato-devkit` は Python 3.12 を前提とした CLI であり、`typer` と `rich` を中心に構成されている。 fileciteturn21file0  
また、README 上でも、差分集計・構造化 block 操作・outline/context・tree・Markdown 操作・記録テンプレ生成・patch 診断など、AI 支援開発の文脈圧縮に寄与する複数コマンドが提供されている。 fileciteturn20file0

そのため、Rust 移行の是非は単純な「Python は遅い」という一般論ではなく、次の問いに対して答える形で判断する必要がある。

- 実運用で支配的な遅さは本当に Python 由来か
- git / I/O / subprocess 待ちが支配的ではないか
- どのコマンドが高頻度か
- その高頻度コマンドにおいて、速度改善が体感や総工数に効くか
- Rust 移行コストに見合う改善幅があるか

---

## 2. 判断原則
Rust 移行の判断は、次の原則に従う。

### 2.1 Python 全面否定から始めない
Python 実装であっても、主要コマンドの大半が git subprocess やファイル I/O 待ちであれば、Rust へ移しても改善幅は限定的である。  
その場合は移行よりも、出力削減、アルゴリズム改善、コマンド設計の見直しを優先すべきである。

### 2.2 高頻度コマンドを優先して見る
開発体験を左右するのは、絶対的に最も遅いコマンドではなく、**頻繁に繰り返されるコマンド**である。  
1回 20ms の差でも、何百回も叩くなら十分意味がある。  
逆に 1回 800ms でも、ほぼ使わないコマンドなら移行優先度は低い。

### 2.3 単体性能より運用シナリオで測る
CLI は起動コスト、ファイル探索、標準出力、subprocess 呼び出しを含むため、関数単体ベンチだけでは実態を誤る。  
判断は「実際に人間や AI が叩くシナリオ」で行うべきである。

### 2.4 Rust 移行は段階的に判断する
全面移行か維持かの二択ではなく、次の三段階で考える。

1. Python 維持で十分
2. 一部ホットパスのみ Rust 化
3. CLI 全体の Rust 移行が妥当

---

## 3. テストの目的
本テストで明らかにしたいことは以下である。

1. 各主要コマンドの wall-clock time
2. 高頻度コマンドの平均時間と p95
3. 遅さの主因
   - Python 起動
   - Python 内部処理
   - git subprocess
   - ファイル I/O
4. `--brief` など省出力モードによる改善幅
5. リポジトリ規模やファイルサイズによる悪化傾向
6. Rust 移行によって改善余地の大きいコマンドの特定

---

## 4. 対象コマンド
README 記載の機能群を踏まえ、少なくとも以下を対象とする。 fileciteturn20file0

### 4.1 高頻度候補
- `devkit block outline`
- `devkit block context`
- `devkit block extract`
- `devkit diff summarize`
- `devkit tree`
- `devkit md append-section`
- `devkit patch diagnose`

### 4.2 中頻度候補
- `devkit block replace`
- `devkit md replace-section`
- `devkit doc impl-note`
- `devkit doc benchmark-note`
- `devkit git commit-message`
- `devkit git pr-body`

### 4.3 低頻度候補
- `devkit patch apply`
- `devkit git safe-push`
- `devkit encoding check`
- `devkit encoding normalize`

---

## 5. テストシナリオ
テストはコマンド単体ではなく、実運用を模したシナリオ単位で定義する。

## 5.1 シナリオA: repo 構造把握
目的:
- AI や人が対象 repo の構造を把握する最初のステップの性能測定

実行例:
- `devkit tree --max-depth 3`
- `devkit tree --max-depth 5 --ext rs`
- `devkit tree --dirs-only`

観点:
- 起動時間
- ディレクトリ数増加時の悪化
- `.gitignore` / `devkit.toml` 無視設定適用の影響

## 5.2 シナリオB: 対象シンボル探索
目的:
- ファイル全体を読まずに API 構造や対象シンボルを見つける操作の性能測定

実行例:
- `devkit block outline src/devkit/core/block.py`
- `devkit block outline src/devkit/core/block.py --imports`
- `devkit block context src/devkit/core/block.py --symbol replace_block --margin 20`

観点:
- outline の軽さ
- context の対象位置探索時間
- ファイルサイズ増加時の悪化

## 5.3 シナリオC: 局所編集
目的:
- extract / replace / dry-run の往復速度測定

実行例:
- `devkit block extract ... --symbol ...`
- `devkit block replace ... --symbol ... --with-file ... --dry-run`
- `devkit block replace ... --heading-exact ... --with-file ...`

観点:
- symbol 解決コスト
- dry-run diff 生成コスト
- 実書き込みあり/なし差

## 5.4 シナリオD: 実装記録更新
目的:
- Gale 的な「毎実装ごとに記録も更新する」運用の性能測定

実行例:
- `devkit doc impl-note --base origin/main --head HEAD`
- `devkit md ensure-section ...`
- `devkit md append-bullet ...`

観点:
- diff summarize 連携のコスト
- frontmatter を含む長い Markdown の更新性能
- 小さな更新を何度も行うときの累積コスト

## 5.5 シナリオE: patch-first 運用
目的:
- AI が patch を返し、人間または別 AI が診断・適用する流れの測定

実行例:
- `devkit patch diagnose --patch-file ...`
- `devkit patch apply --patch-file ... --dry-run`
- `devkit patch apply --patch-file ... --reject`

観点:
- hunk 数に対するスケーリング
- 失敗パッチ時の診断コスト
- 成功時/失敗時の差

---

## 6. 測定項目
各テストで最低限、以下を記録する。

### 6.1 必須
- command
- scenario
- repo_size_class
- file_size_class
- success / fail
- total_ms

### 6.2 推奨
- startup_ms
- subprocess_ms
- io_ms
- render_ms
- output_bytes
- brief_mode
- timestamp

### 6.3 集計指標
- 平均
- median
- p95
- 最大値
- 成功率
- 1日あたり推定累積時間

---

## 7. テストデータ設計
偏りを避けるため、少なくとも 3 種類の条件を用意する。

### 7.1 小規模
- 数十ファイル
- 小さめの Markdown / コードファイル
- 日常的なローカル util repo 相当

### 7.2 中規模
- 数百ファイル
- 数千行級ファイルを含む
- Gale や類似の継続開発 repo 相当

### 7.3 大規模
- 数千ファイル規模
- vendor / docs / tests を多く含む
- ignore 設定の影響が大きいケース

また、block 系については次のようなファイルも含める。

- Python の大きい class / def を含むファイル
- Rust の `impl` / `enum` / `struct` を多く含むファイル
- Go / JS / C 系で波括弧ネストがあるファイル
- frontmatter 付き長大 Markdown
- patch 対象として hunk 数の異なる diff

---

## 8. 計測手法
## 8.1 基本方針
- 1回だけの測定ではなく複数回測る
- cold / warm を分ける
- stdout 汚染を避けるため timing は stderr または JSON に出す
- 可能なら `--time` と `--time-json` を導入して計測する

## 8.2 回数
最低:
- warm-up 3回
- 本計測 20回

推奨:
- 本計測 30〜50回

## 8.3 比較軸
- `--brief` あり / なし
- 小 / 中 / 大 repo
- 成功ケース / 失敗ケース
- patch 小 / 中 / 大
- Markdown 小 / 長大

---

## 9. 分析観点
計測結果は、単に「速い / 遅い」でなく次の観点で解釈する。

### 9.1 Python 起動支配か
コマンド内容に対して `startup_ms` の割合が大きい場合、CLI のたびに Python を起動する構造そのものがボトルネックになっている。  
この場合、Rust 単体バイナリ化の効果は見込みやすい。

### 9.2 subprocess 支配か
`diff summarize`、`git commit-message`、`pr-body` などで git 呼び出しが支配的なら、Rust 化しても改善幅は限定的である。  
この場合は git 呼び出し回数削減やキャッシュの方が効く。

### 9.3 I/O 支配か
長大ファイルや tree 走査で I/O が支配的なら、言語変更より探索戦略や ignore 最適化が重要になる。

### 9.4 Python 内部処理支配か
outline / context / block 構造解析 / Markdown 更新 / patch diagnose などで Python 側の文字列処理が支配的なら、Rust 移行候補として有力である。

---

## 10. Rust移行の判断基準
以下を満たす場合、Rust 移行を積極検討する。

### 10.1 部分移行を検討すべき条件
- 高頻度コマンドで Python 内部処理が支配的
- p95 が体感上ストレスになる
- `--brief` を使ってもまだ遅い
- 文字列処理や構造解析の比率が大きい

例:
- `block outline`
- `block context`
- `block extract / replace`
- `md ...`
- `patch diagnose`

### 10.2 全面移行を検討すべき条件
- 高頻度コマンド群の多くで Python 起動コストが支配的
- 部分移行では効果が分散しすぎる
- 配布面でも単体バイナリの価値が大きい
- CLI の使用回数が非常に多い

### 10.3 Python 維持が妥当な条件
- 遅さの主因が git / I/O
- Python 内部処理の比率が低い
- 体感差が小さい
- 移行コストに対し改善幅が小さい

---

## 11. 推奨する結論の出し方
最終判断は次の 3 区分で行う。

### A. Python 維持
- 速度面の不満はあるが、主因は Python ではない
- 現時点では設計改善や観測強化の方が効果的

### B. ハイブリッド移行
- CLI 全体は Python 維持
- hot path だけ Rust 実装
- 例:
  - block parser
  - Markdown section 操作
  - patch diagnose
  - tree walk

### C. Rust 本格移行
- 高頻度コマンドの多くで Python 起動または内部処理が支配的
- 単体バイナリ配布の利点も大きい
- 中長期的保守も Rust の方が有利

---

## 12. 当面の推奨アクション
現時点では、いきなり Rust に全面移行するより、次の順で進めるのが合理的である。

1. `--time` を主要コマンドに導入
2. 可能なら `--time-json` と簡易内訳を追加
3. 本書のシナリオに従って測定を実施
4. 高頻度かつ Python 内部処理支配のコマンドを特定
5. その箇所のみ Rust で試作
6. 再測定し、改善幅と実装コストを比較
7. 全面移行の要否を再判断

---

## 13. 最終結論
`xpotato-devkit` の Rust 移行は、思想上は十分あり得る。  
特に、配布性、起動速度、文字列処理性能、継続運用コストの観点では Rust は魅力的である。

ただし、現段階で最も重要なのは「Rust にしたいから Rust にする」ことではなく、  
**どの遅さが Python 由来で、どこに投資すれば最も開発体験が改善するかを測ること**である。

したがって、判断の出発点は移行そのものではなく、  
**実運用シナリオに基づく観測と比較測定**に置くべきである。
