# devkit Windows Installer 実装ドキュメント

## 1. 目的

本ドキュメントは、`devkit` の Windows 向け配布において、**ユーザー領域へのインストール**、**専用アンインストーラーの同梱**、および **PATH 追加をオプション扱い** とするインストール方式の実装方針を定義する。

本方式の狙いは以下である。

* PowerShell スクリプト依存を避け、`ps1` 実行ポリシー問題を回避する。
* 配布導線を単純化し、`installer.exe` 実行だけで利用可能な状態にする。
* `devkit` 本体、アンインストーラー、メタデータを専用ディレクトリで管理し、将来の更新や削除を安全にする。
* PATH 追加は利便性向上機能として提供するが、インストール成立条件には含めない。

---

## 2. スコープ

### 2.1 本ドキュメントの対象

* Windows 向け `installer.exe` の実装
* ユーザー領域へのファイル配置
* `uninstall.exe` の配置と削除設計
* インストール情報の manifest 管理
* オプションの PATH 追加機能

### 2.2 本ドキュメントの対象外

* MSI パッケージ化
* コード署名
* 自動更新機構
* winget / Chocolatey / Scoop 対応
* macOS / Linux 向けインストーラー

---

## 3. 設計方針

### 3.1 基本方針

インストーラーは単一のネイティブ実行ファイル `devkit-installer.exe` とし、ユーザーがこれを実行すると、`LocalAppData` 配下の専用ディレクトリへ `devkit` 関連ファイルを配置する。

インストール成功の要件は以下とする。

* `devkit.exe` が専用ディレクトリへ配置されていること
* `uninstall.exe` が同ディレクトリへ配置されていること
* `install-manifest.json` が生成されていること

PATH 追加はデフォルト必須ではなく、ユーザーが明示的に選択した場合のみ行う。

### 3.2 採用理由

本方針により、以下の利点が得られる。

* PowerShell 実行ポリシーを回避できる。
* PATH 追加失敗がインストール失敗と直結しない。
* アンインストール対象を installer 管理下のファイルに限定できる。
* 将来 `self update` や Add/Remove Programs 対応を追加しやすい。

---

## 4. インストールレイアウト

### 4.1 インストール先

インストール先はユーザー領域固定とする。

```text
%LOCALAPPDATA%\Xpotato\devkit\
```

実体パス例:

```text
C:\Users\<User>\AppData\Local\Xpotato\devkit\
```

### 4.2 配置ファイル

```text
%LOCALAPPDATA%\Xpotato\devkit\
├─ devkit.exe
├─ uninstall.exe
├─ install-manifest.json
├─ version.txt                # 任意
└─ logs\                     # 任意
```

### 4.3 PATH 追加時の扱い

PATH 追加は、原則としてインストール先ディレクトリそのものをユーザー PATH に追加する。

追加候補:

```text
%LOCALAPPDATA%\Xpotato\devkit\
```

`bin` サブディレクトリを別に切る案もあるが、現時点ではファイル点数が少なく、構成を単純化するため直下配置を優先する。

---

## 5. コンポーネント構成

### 5.1 配布物

初期段階の配布物は以下とする。

* `devkit-installer.exe`

installer は内部に以下の payload を保持する。

* `devkit.exe`
* `uninstall.exe`
* 必要に応じてテンプレート manifest

### 5.2 実装案

実装方式は以下のいずれかを採用する。

#### 案A: installer 専用バイナリ

* `devkit-installer.exe` を Rust で別ターゲットとして実装
* 内部に payload を埋め込み、展開して配置

#### 案B: ブートストラップ + 同梱ファイル

* `devkit-installer.exe` と `devkit.exe` / `uninstall.exe` を同梱配布
* installer が同階層のファイルをコピー

初期リリースでは **案Bの方が実装容易** だが、配布の一貫性・事故防止・UX の観点では **最終的に案Aが望ましい**。

本ドキュメントでは **将来案Aへ移行可能な前提で、まず案Bでも成立する設計** を採用する。

---

## 6. インストーラー要件

### 6.1 必須要件

installer は以下を満たすこと。

1. インストール先ディレクトリを作成できる。
2. `devkit.exe` を配置できる。
3. `uninstall.exe` を配置できる。
4. `install-manifest.json` を生成できる。
5. 既存インストールがある場合、上書きまたは更新として処理できる。
6. PATH 追加をオプションとして提供できる。

### 6.2 失敗時要件

installer は途中失敗時に、可能な範囲で中途半端な状態を残さないこと。

最低限、以下を満たす。

* `devkit.exe` コピー失敗時は install 完了扱いにしない。
* manifest 書き込み失敗時は install 完了扱いにしない。
* PATH 追加失敗時は、PATH 追加のみ失敗として扱い、本体配置が成功していれば install 自体は成功とみなしてよい。ただし警告を表示する。

### 6.3 管理者権限

ユーザー領域のみを扱うため、**管理者権限は不要** とする。

---

## 7. アンインストーラー要件

### 7.1 必須要件

`uninstall.exe` は以下を満たすこと。

1. `install-manifest.json` を読める。
2. installer が配置したファイルのみを削除できる。
3. PATH 追加済みであれば、自分が追加したエントリのみを削除できる。
4. 空になったインストールディレクトリを削除できる。

### 7.2 削除対象

削除対象は manifest 管理下のものに限定する。

例:

* `devkit.exe`
* `uninstall.exe`
* `install-manifest.json`
* installer が生成した補助ファイル
* installer が作成したショートカット（将来対応時）

### 7.3 削除対象外

以下は勝手に削除しない。

* ユーザー設定ファイル
* ユーザー作成データ
* プロジェクト成果物
* installer 管理下にない任意ファイル

### 7.4 自己削除

`uninstall.exe` は自身を実行中に削除できない可能性があるため、以下のいずれかで対応する。

* 削除予約を行う
* 一時バッチまたは一時プロセスに削除を委譲する
* 最終的に空ディレクトリ削除を試みる

実装は OS 依存性が高いため、**初期版では「主要ファイル削除成功」を優先し、自己削除は後段で改善してもよい**。

---

## 8. PATH 追加要件

### 8.1 基本方針

PATH 追加はオプションとする。

デフォルト挙動は以下とする。

* インストーラー UI または CLI フラグで選択可能
* 未選択でも install 成功
* 失敗しても本体配置成功なら install 成功

### 8.2 追加先

ユーザー環境変数 `PATH` に対して追加する。

追加値:

```text
%LOCALAPPDATA%\Xpotato\devkit\
```

### 8.3 重複防止

PATH 追加時は以下を考慮する。

* 既存 PATH に同じディレクトリが含まれていれば追加しない
* 大文字小文字差異、末尾バックスラッシュ差異を吸収して比較する

### 8.4 アンインストール時の挙動

uninstaller は、自分が追加した PATH エントリだけを削除する。

このため manifest に以下の情報を持たせる。

* PATH を追加したか
* 追加した正規化済みパス

---

## 9. Manifest 仕様

### 9.1 目的

manifest は以下のために必要である。

* アンインストール対象の明確化
* PATH 操作内容の復元
* 将来の更新互換性確保

### 9.2 ファイル名

```text
install-manifest.json
```

### 9.3 推奨フィールド

```json
{
  "product": "devkit",
  "version": "0.1.0",
  "install_dir": "C:\\Users\\<User>\\AppData\\Local\\Xpotato\\devkit",
  "installed_at": "2026-04-08T12:00:00+09:00",
  "installed_files": [
    "devkit.exe",
    "uninstall.exe",
    "install-manifest.json"
  ],
  "path_added": true,
  "path_value": "C:\\Users\\<User>\\AppData\\Local\\Xpotato\\devkit",
  "installer_version": "0.1.0"
}
```

### 9.4 設計上の注意

* 将来互換性のため、未知フィールドは無視可能にする。
* `installed_files` は相対パスで保持してもよい。
* `path_added` が `false` の場合、`path_value` は空でもよい。

---

## 10. インストールフロー

### 10.1 標準フロー

1. installer 起動
2. インストール先決定（固定または確認表示）
3. 既存インストール有無確認
4. 必要ディレクトリ作成
5. 一時ファイルへ展開
6. `devkit.exe` 配置
7. `uninstall.exe` 配置
8. manifest 生成
9. オプションで PATH 追加
10. 完了表示

### 10.2 上書き更新フロー

既存インストールがある場合は以下とする。

* 同一ディレクトリへ上書き配置
* manifest を新バージョンで更新
* PATH は既存状態を尊重しつつ、必要なら再確認

削除してから再配置ではなく、**安全な上書き更新** を優先する。

### 10.3 ロールバック方針

初期版では完全ロールバックまでは必須としないが、最低限以下を守る。

* `devkit.exe` の書き換え前に一時名で配置し、最後に rename する
* 失敗時に broken state を極力減らす

---

## 11. 実装詳細

### 11.1 推奨技術

Rust 実装を前提とする。

候補:

* `std::fs` によるコピー・ディレクトリ作成
* `directories` 系クレートで `LocalAppData` 解決
* `serde` / `serde_json` による manifest 管理
* 必要に応じて Windows API 呼び出しで PATH 更新

### 11.2 PATH 更新方法

PowerShell を介さず、Rust から直接ユーザー環境変数を更新する。

必要な処理:

* ユーザー PATH の取得
* 正規化比較
* 重複回避付き追記
* 必要に応じた環境変更通知

### 11.3 ファイル更新の安全性

コピーではなく、以下を推奨する。

1. 一時ファイルへ書き込み
2. flush
3. rename で置換

これにより途中失敗時の破損リスクを減らす。

---

## 12. CLI / UI 仕様案

### 12.1 最小 CLI 例

```text
devkit-installer.exe --install
devkit-installer.exe --install --add-to-path
devkit-installer.exe --unpack-only
```

ただし、一般配布ではダブルクリック実行を基本導線とし、CLI は開発・検証向けでもよい。

### 12.2 完了メッセージ例

* インストール先
* PATH を追加したかどうか
* 実行ファイルの場所
* アンインストール方法

例:

```text
Installed devkit to:
C:\Users\<User>\AppData\Local\Xpotato\devkit

PATH added: Yes
Uninstall: C:\Users\<User>\AppData\Local\Xpotato\devkit\uninstall.exe
```

---

## 13. 受け入れ条件

以下を満たしたとき、本機能は受け入れ可能とする。

### 13.1 基本

* `devkit-installer.exe` 実行で user 領域へ `devkit.exe` が配置される。
* `uninstall.exe` が同ディレクトリに配置される。
* `install-manifest.json` が生成される。

### 13.2 PATH オプション

* PATH 未追加でも install 成功となる。
* PATH 追加を選択した場合、ユーザー PATH に重複なく追加される。
* uninstall 時に追加済み PATH が除去される。

### 13.3 アンインストール

* `uninstall.exe` 実行で installer 管理下ファイルのみ削除される。
* ユーザーデータが削除されない。
* 失敗時も何が残ったか判別できる。

### 13.4 既存更新

* 既存インストールに対して再実行すると安全に上書きできる。
* manifest が新しい内容で更新される。

---

## 14. 実装タスク分解

### Phase 1: 基礎

1. インストール先解決
2. ディレクトリ作成
3. `devkit.exe` 配置
4. `uninstall.exe` 配置
5. manifest 出力

### Phase 2: PATH オプション

6. ユーザー PATH 取得
7. 正規化比較
8. 重複防止付き追加
9. manifest 反映

### Phase 3: uninstall

10. manifest 読み込み
11. 管理対象ファイル削除
12. PATH 除去
13. 空ディレクトリ削除

### Phase 4: 品質強化

14. 一時ファイル経由更新
15. 上書き更新テスト
16. 失敗時メッセージ整備
17. 自己削除改善

---

## 15. 非推奨案

以下は本設計では採用しない。

### 15.1 `ps1` ベース installer

理由:

* 実行ポリシーの問題がある
* 配布体験が悪化しやすい
* セキュリティ上の警戒を招きやすい

### 15.2 PATH 必須化

理由:

* 失敗要因が増える
* 本来成立するインストールが PATH 問題で失敗扱いになる
* 企業環境や制限環境との相性が悪い

### 15.3 uninstall でユーザー設定まで削除

理由:

* 事故リスクが高い
* 削除範囲が曖昧になる
* 将来互換性を損ないやすい

---

## 16. 推奨結論

現時点での最適な実装方針は以下である。

* Windows 向けに `devkit-installer.exe` を用意する
* `LocalAppData` 配下の専用ディレクトリへ `devkit.exe` と `uninstall.exe` を配置する
* `install-manifest.json` により状態管理する
* PATH 追加はオプションとし、失敗しても install 自体は成立させる
* uninstall は installer 管理下ファイルのみに責務を限定する

この方針により、配布体験、実装難易度、将来拡張性のバランスを最もよく取ることができる。
