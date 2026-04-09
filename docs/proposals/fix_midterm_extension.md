# devkit 次回修正・中期拡張 要件定義
**目的:**  
次回の修正では、以下の4点を同時に前進させる。

1. `devkit` の価値を、初見の開発者にも短時間で理解できるようにする  
2. `winget` 配布に向けた不足要件を埋め、公開準備を進める  
3. AI や人間が `git diff` / `rg` / `tree` / `Get-Content` / `Get-ChildItem` に逃げる理由を減らす  
4. 今後の拡張候補を要件として先に明文化し、実装方針のブレや取りこぼしを防ぐ

---

# 1. 背景

現状の `devkit` は、Rust 製ネイティブ CLI として主要コマンド群、Windows 向けインストーラ、アンインストーラ、GitHub Releases によるタグ駆動配布がすでに整備されている。  
一方で、現時点では以下の課題が残っている。

- 初見ユーザーにとって、`devkit` が「何を解決するツールか」は読み取れるが、まだ直感的ではない
- `devkit` が、単なる既存コマンド代替ではなく、**AI 時代向けの局所探索・局所編集・安全な patch 適用ワークフロー**であることをより明確に伝える必要がある
- `winget` 配布に必要な installer 契約、asset 形態、manifest 前提情報が未整理または未明文化である
- 一部では `devkit` の方が高付加価値であるにもかかわらず、実利用上の不足により `git diff` / `rg` / `tree` / PowerShell の `Get-Content` / `Get-ChildItem` へ戻ってしまう可能性がある
- Windows 環境では特に、`grep` の代わりに `Get-Content` や `Get-ChildItem` を乱用しやすく、AI もそれに引っ張られやすい

---

# 2. この修正のゴール

## ゴールA: 価値の伝達
ユーザーが README の冒頭と導線だけで、`devkit` の価値を以下の粒度で理解できること。

- `devkit` は何のためのツールか
- どのような失敗や無駄を減らすのか
- どのようなユーザーに向いているのか
- どのような流れで使うのか
- 普通の全文読込・全文再生成型 AI 編集と何が違うのか

## ゴールB: winget 準備
少なくとも、`winget` 公開に向けて以下が揃っている状態にする。

- Windows 配布物の形が `winget` から扱いやすい
- installer の挙動が明文化されている
- `winget manifest` を作成できる情報が揃っている
- 今後 `winget-pkgs` に出すための下準備ができている

## ゴールC: 既存コマンド逃避の抑制
`devkit` の利用を阻害している主要な逃避先コマンドについて、少なくとも高頻度用途を `devkit` 内で吸収できるようにする。

対象は次の通り。

- `git diff`
- `rg`
- `tree`
- `Get-Content`
- `Get-ChildItem`

## ゴールD: 中期拡張方針の固定
今後の実装候補を先に文書化し、将来の追加開発において以下を避ける。

- 同じ議論の再発
- 価値の薄い互換機能の無秩序な追加
- 中核価値を損なう実装の肥大化
- 「便利そうだから追加する」だけの判断

---

# 3. スコープ

## 対象
- README
- 価値説明ドキュメント
- インストール関連ドキュメント
- リリース関連ドキュメント
- Windows installer の CLI 契約
- GitHub Release asset 構成
- `winget` 用メタデータの準備
- `diff summarize`
- 探索系コマンドの強化または新設
- `tree` の最低限の実用オプション
- 将来拡張候補の優先順位付け

## 対象外
- Microsoft Store 申請対応の完了
- コード署名の完全解決
- GUI の追加
- 既存コマンドとの完全互換
- Linux/macOS 配布方式の全面再設計
- `ripgrep` や `tree` の完全再実装

---

# 4. 要件詳細

---

## 4.1 価値伝達の要件

### 要件 4.1.1 README 冒頭の再設計
README 冒頭に、現状の機能列挙より前または直後に、以下を短く明示する。

#### 必須要素
- `devkit` が解決する問題
- 対象ユーザー
- 典型ユースケース
- 他の一般的な手法との差分

#### 期待する説明の方向性
`devkit` は単なる便利 CLI 集ではなく、以下を実現するツールであることを明示する。

- 巨大リポジトリを毎回全文読まない
- 必要な部分だけ取り出す
- patch を診断してから当てる
- AI の無駄な token 消費を抑える
- AI の破壊的編集を減らす

### 要件 4.1.2 「何ができるか」ではなく「何を防ぐか」を書く
README または専用ドキュメントに、次のような失敗例を明示する。

#### 例
- AI が長いファイルを毎回全文読む
- AI が関数1個の修正なのにファイル全体を書き直す
- patch が外れているのにそのまま適用しようとする
- Markdown の1セクション修正なのに全文置換する
- 差分の全体像を見ずに変更を進める
- Windows 環境で `Get-Content` による全文読込を乱発する
- `Get-ChildItem -Recurse` による雑な探索でノイズを増やす

### 要件 4.1.3 最短の利用導線を作る
README から 3 分以内に、次の流れを理解できるようにする。

1. `diff summarize`
2. `block outline` / `block context` / `block extract`
3. `patch diagnose`
4. `patch apply`

この導線には、実際の短いサンプルを含める。

### 要件 4.1.4 ユースケースを明文化する
以下の 3 系統を最低限示す。

- **AI エージェント支援開発**
- **人間による大規模リポジトリの局所編集**
- **Windows 環境で PowerShell の全文読込・再帰列挙を減らす利用**

### 要件 4.1.5 README の情報設計を整理する
README の順序を次の優先度に寄せる。

1. 何のツールか
2. どんな問題を解決するか
3. 最短ユースケース
4. 主なコマンド群
5. インストール
6. 詳細ドキュメント

---

## 4.2 価値説明用ドキュメントの要件

### 要件 4.2.1 専用の価値説明ドキュメントを追加
新規ドキュメントを追加する。

#### 候補名
- `docs/concepts/why-devkit.md`
- `docs/concepts/value-proposition.md`

#### 内容
- なぜ `devkit` が必要か
- 既存ツールだけでは何がつらいか
- なぜ AI 時代に局所編集が重要か
- `inspect -> edit -> verify` の意味
- `diff summarize -> block extract -> patch diagnose -> patch apply` の一連の価値
- Windows で `Get-Content` / `Get-ChildItem` を乱用すると何が起きるか
- なぜ PowerShell ワンライナー乱用より専用 CLI の方が再現性・機械可読性に優れるか

### 要件 4.2.2 FAQ 追加
最低限以下の FAQ を用意する。

- grep や git diff だけではだめなのか
- なぜ patch diagnose が必要なのか
- なぜ全文読込を避けるのか
- 人間だけでも使う価値はあるか
- AI エージェントと組み合わせると何が変わるか
- Windows で `Get-Content` や `Get-ChildItem` を使うより何が良いのか

---

## 4.3 winget 準備の要件

### 要件 4.3.1 Windows Release asset の見直し
`winget` で扱いやすいよう、Windows Release asset に **単体の `devkit-installer.exe`** を追加できる構成を検討・実装する。

#### 現状の課題
- docs 上は Windows asset が zip 中の `devkit-installer.exe` 前提
- `winget` では単体 installer URL の方が manifest を書きやすい

#### 目標
- Release asset として `devkit-installer.exe` を直接取得できること
- 既存 zip asset を残すかどうかは任意だが、少なくとも単体 installer を用意すること

### 要件 4.3.2 silent install 対応
Windows installer に、`winget` 経由で無人実行可能なオプションを用意する。

#### 必須項目
- silent install
- silent uninstall
- 非対話モードでの exit code 定義
- 失敗時の終了コード整理

### 要件 4.3.3 upgrade/install 挙動の明文化
次の挙動を docs に明示する。

- 未インストール時の install
- 既存バージョンがある場合の再 install
- 上書き upgrade の可否
- PATH 更新の扱い
- uninstall 後の状態

### 要件 4.3.4 SHA256 取得フローの整備
`winget manifest` 作成に必要な SHA256 を確実に取得できるようにする。

#### 最低要件
- リリース後にハッシュを取得する手順があること

#### 望ましい要件
- GitHub Actions で自動算出し、成果物として残すこと

### 要件 4.3.5 winget メタデータ下書きの追加
repo 内に `winget` 用の下書きを置く。

#### 候補
- `packaging/winget/`
- `docs/packaging/winget/`

#### 含めるもの
- PackageIdentifier 案
- PackageName
- Publisher
- Moniker
- ShortDescription
- Description
- Tags
- Commands
- Installer URL 設定方針
- InstallerType の整理
- Scope の整理

### 要件 4.3.6 commands の明示
`winget` 導入後に使えるコマンドとして、少なくとも `devkit` を docs 上で明示する。

### 要件 4.3.7 user-scope install の明示
現在の `%LOCALAPPDATA%\Xpotato\devkit` へのインストールは `user-scope` 前提として扱いやすい。  
この点を `winget` 前提の文脈で明文化する。

---

## 4.4 ドキュメント整備要件

### 要件 4.4.1 `winget` 向けドキュメント追加
新規ドキュメントを追加する。

#### 候補名
- `docs/install/windows-winget-prep.md`
- `docs/packaging/winget.md`

#### 内容
- 現状の対応状況
- 未対応項目
- manifest に必要な情報
- installer 契約
- リリース運用との関係
- 今後の公開手順

### 要件 4.4.2 README からリンクする
以下を README から辿れるようにする。

- 価値説明ドキュメント
- AI workflow
- Windows installation
- winget 準備ドキュメント

---

## 4.5 既存コマンド互換・穴埋め要件

### 目的
`devkit` は `tree` や `ripgrep` や PowerShell の完全互換を目指すものではない。  
ただし、現状で `devkit` の方が高い価値を持つにもかかわらず、日常利用上の不足により既存コマンドへ戻ってしまうケースがあるなら、それを解消する。

本修正では、**「高付加価値ツールとしての立ち位置を維持したまま、実利用を阻害する最低限の不足を埋める」** ことを目的とする。

### 要件 4.5.1 ギャップ分析ドキュメントの追加
以下に対して、`devkit` の優位性と不足点を整理したドキュメントを追加する。

- `git diff`
- `rg`
- `tree`
- `Get-Content`
- `Get-ChildItem`

#### 候補名
- `docs/design/command-gap-analysis.md`
- `docs/concepts/devkit-vs-existing-tools.md`

#### 含める内容
- 既存コマンドでできること
- `devkit` の方が優れている点
- 現状 `devkit` で不足している点
- その不足が「実利用阻害」か「非本質」かの分類
- 今後埋めるべき対象の優先順位

### 要件 4.5.2 優先ギャップの選定
互換・拡張対象は、以下の条件を満たすものに限定する。

#### 採用条件
- 日常利用頻度が高い
- 既存コマンドでは当然に可能
- 現状の `devkit` 利用を阻害している
- 実装コストが過大でない
- `devkit` の中核価値（局所化・安全性・AI向け出力）と整合する

#### 非採用条件
- 完全互換のためだけの実装
- 低頻度オプションの網羅
- 挙動再現コストが大きいもの
- `devkit` 独自価値に寄与しないもの

---

## 4.6 高優先の機能拡張要件

### 要件 4.6.1 `diff summarize` 強化
`git diff` に逃げる理由を減らすため、`diff summarize` を変更把握の第一選択として成立させる。

#### 必須候補
- `--files-only`
- `--name-status`
- `--staged`
- `--unstaged`
- `--base <ref>`
- `--head <ref>`
- `--stat`
- `--limit N`
- `--json` 強化
- `--brief` 強化

#### 期待挙動
- 変更ファイル一覧だけ見たい用途を満たす
- 変更規模を短く把握できる
- AI が次にどのファイルを読むべきか決めやすい
- `git diff` を直接叩かなくてもよいケースを増やす

### 要件 4.6.2 探索系コマンドの新設または強化
`rg` / `Get-Content` / `Get-ChildItem` に逃げる理由を減らすため、探索入口を `devkit` 内に持つ。

#### 候補コマンド
- `devkit search text <pattern>`
- `devkit search symbol <name>`

または既存 `block` 系への統合でもよい。

#### 最低限欲しいオプション
- `--glob`
- `--type`
- `--ignore-case`
- `--fixed-strings`
- `--context N`
- `--files-with-matches`
- `--count`
- `--limit N`
- `--json`
- `--brief`

#### PowerShell 乱用対策として期待すること
- `Get-ChildItem -Recurse | Select-String ...` 的な雑な探索の代替になること
- `Get-Content` による全文読込の代わりに、ヒット周辺や対象ブロックのみ取得できること
- Windows でも shell 依存の重いワンライナーに逃げずに済むこと

### 要件 4.6.3 `tree` の最低限の実用強化
`tree` や `Get-ChildItem -Recurse` に逃げる理由を減らすため、`devkit tree` に実用オプションを追加する。

#### 最低限候補
- `--max-depth`
- `--dirs-only`
- `--files-only`
- `--hidden`
- `--glob`
- `--json`
- `--limit N`

#### 期待挙動
- AI が浅い構造把握だけしたい時に使える
- 人間がノイズを抑えて構造把握できる
- Windows で `Get-ChildItem -Recurse` を雑に叩く必要が減る

### 要件 4.6.4 検索から block 系への接続強化
検索後に AI が別ツールに飛ばないように、検索結果から `block context` / `block extract` へ自然に繋げられるようにする。

#### 候補
- 検索結果にシンボル名を含める
- ヒット箇所から `block context` を取りやすくする
- ヒット箇所から `block extract` を取りやすくする
- `--json` の出力を後続処理しやすくする

#### 理想
`search -> block context -> block extract -> patch diagnose`
の導線が `devkit` 内で閉じること。

### 要件 4.6.5 `patch diagnose` の説明力強化
patch 失敗時に AI が全文再読込や `git diff` 再実行へ逃げないよう、診断情報を強化する。

#### 改善候補
- どの hunk が失敗したか
- 失敗理由の分類
- 前後近傍の表示
- ズレ・内容変更・対象欠落の区別
- JSON 出力の構造化

---

## 4.7 Windows / PowerShell 特有の要件

### 要件 4.7.1 PowerShell 代替導線の明文化
docs において、Windows 環境で次のような置換方針を示す。

#### 例
- `Get-ChildItem -Recurse` の代わりに `devkit tree`
- `Get-Content` の全文読込の代わりに `devkit block context` / `block extract`
- `git diff` の概要確認の代わりに `devkit diff summarize`
- `Select-String` 相当の探索の代わりに `devkit search`

### 要件 4.7.2 AI への推奨行動を docs 化
AI 向け workflow docs または SKILL に、以下を明記する。

- Windows でも `Get-Content` の全文読込を常用しない
- `Get-ChildItem -Recurse` による雑な探索を避ける
- まず `devkit` の構造化出力を優先する
- shell ワンライナーよりも `devkit --brief` / `--json` を優先する

### 要件 4.7.3 再現性優先
PowerShell ワンライナーは柔軟だが、出力安定性・可搬性・機械可読性に難がある。  
`devkit` はそれを置き換える**安定した契約面**を持つことを重視する。

---

# 5. 非機能要件

## 5.1 ドキュメント品質
- README 冒頭 30 秒で価値が伝わること
- 初見の開発者が「自分向けかどうか」を判断できること
- 機能列挙だけで終わらず、利用の流れが見えること

## 5.2 配布品質
- Windows installer が手動配布でも壊れないこと
- `winget` 対応のために既存手動配布 UX を大きく損なわないこと

## 5.3 後方互換性
- 既存の GitHub Releases 利用者の導線を大きく壊さない
- `devkit` コマンド名は維持する

## 5.4 中核価値の維持
- 既存コマンドの完全再実装は目指さない
- 追加実装は `devkit` の中核価値を強める範囲に限定する
- 局所化・安全性・機械可読性・AI 向け出力契約を優先する

---

# 6. 受け入れ条件

以下を満たしたら、この修正は完了とみなす。

## 受け入れ条件A: 価値伝達
- README 冒頭に価値説明が追加されている
- 最短の利用例が追加されている
- 「なぜ必要か」を説明する専用ドキュメントが追加されている
- `inspect -> edit -> verify` の流れが README またはリンク先で明確である

## 受け入れ条件B: winget 準備
- `winget` 用メタデータ下書きが repo 内に存在する
- installer の silent / uninstall / upgrade 契約が docs に整理されている
- Windows Release asset の単体 `devkit-installer.exe` 提供方針が決まっている
- SHA256 の取得方法が確立されている

## 受け入れ条件C: 既存コマンド逃避の抑制
- `git diff` / `rg` / `tree` / `Get-Content` / `Get-ChildItem` とのギャップ分析が文書化されている
- 高優先の不足機能が優先順位付きで整理されている
- 少なくとも高優先の不足 1〜3 件について、実装または issue 化が行われている

## 受け入れ条件D: 開発者体験
- 次回以降、`winget` manifest を実際に書くために必要な情報を repo 内だけで追える
- 初見ユーザーが README だけで `devkit` の存在意義を理解しやすくなっている
- Windows 環境でも、PowerShell の雑な全文読込・再帰列挙を避ける導線が見える

---

# 7. 推奨実装順

## Phase 1: 価値説明
1. README 冒頭再設計
2. 最短ユースケース追加
3. `why-devkit` 系ドキュメント追加
4. FAQ 追加

## Phase 2: winget 契約整理
1. installer の silent / uninstall / upgrade 契約整理
2. docs 追加
3. Release asset 構成見直し
4. SHA256 手順整備

## Phase 3: 逃避防止の高優先機能
1. `diff summarize` 強化
2. `search` 系の最小実装
3. `tree --max-depth --json --dirs-only --files-only`

## Phase 4: 接続強化
1. 検索結果から `block context` / `block extract` に繋ぐ
2. `patch diagnose` の説明力強化
3. Windows / PowerShell 利用ガイド追加

## Phase 5: 今後の実装候補の管理
1. command gap analysis 更新
2. 未実装候補の issue 化
3. 優先順位の定期見直し

---

# 8. 今後の実装候補（忘れないための記録）

以下は今回すぐ実装しない場合でも、将来候補として記録しておく。

## 8.1 diff 系
- `--files-only`
- `--name-status`
- `--staged`
- `--unstaged`
- `--base/--head`
- `--stat`
- `--limit`
- 次に見るべき候補の提案

## 8.2 search 系
- text search
- symbol search
- ignore-case
- fixed-string
- context
- files-with-matches
- count
- glob/type filter
- JSON/brief 契約

## 8.3 tree 系
- max-depth
- dirs-only
- files-only
- hidden
- glob
- limit
- JSON

## 8.4 block/search 接続
- 検索ヒットから context 抽出
- 検索ヒットから block 抽出
- シンボル名との接続
- JSON パイプライン最適化

## 8.5 patch 系
- hunk 単位の失敗理由分類
- 近傍表示
- 修正方針の示唆
- JSON 詳細化

## 8.6 Windows / PowerShell 対応
- `Get-Content` 代替の明示
- `Get-ChildItem` 代替の明示
- `Select-String` 相当導線
- Windows 向け AI workflow 最適化

---

# 9. この修正で期待する成果

この修正が完了すると、`devkit` は次の段階に進む。

- **価値が伝わる OSS**
- **Windows CLI 配布が整理された OSS**
- **winget 提出前の準備が済んだ OSS**
- **AI や人間が既存コマンドへ逃げにくい OSS**
- **今後の拡張方針が文書化された OSS**

すなわち、単に「機能がある」状態から、
**「初見ユーザーが価値を理解でき、Windows 配布チャネル拡張にも着手でき、将来の拡張方針も見失わない状態」**
へ進める。