# Windows Installer GUI/PATH/署名 Follow-up 計画

Date: 2026-04-08

## 概要

今回の実機確認で、Windows installer は実行自体は完了したが、GUI からの通常実行時に PATH が通らず、既存の `C:\Users\miyut\.local\bin\devkit.exe` と二重管理になることが確認された。

このドキュメントは、現状整理、実装案、残タスク、受け入れ条件をチェックリスト形式で管理するためのもの。

---

## 現状整理

- [x] `devkit-installer.exe` は GUI installer ではなく CLI 実装である
- [x] 既定動作では `%LOCALAPPDATA%\Xpotato\devkit` に `devkit.exe` / `uninstall.exe` / `install-manifest.json` を配置する
- [x] PATH 更新は `--add-to-path` 指定時のみ行われる
- [x] GUI から通常実行しただけでは `path_added: false` のままインストールが完了しうる
- [x] 実機では `%LOCALAPPDATA%\Xpotato\devkit\devkit.exe` が存在している
- [x] 実機では `Get-Command devkit` が `C:\Users\miyut\.local\bin\devkit.exe` を拾っている
- [x] そのため、installer 配置物と既存 user-local install が競合しうる
- [x] SmartScreen の警告は回避できておらず、実行時に保護画面が表示される

---

## 問題点

- [x] GUI から通常実行した利用者にとって、インストール成功と PATH 利用可能状態が一致していない
- [x] installer が管理する `devkit.exe` と既存の `C:\Users\miyut\.local\bin\devkit.exe` のどちらが使われるかが不透明
- [x] `devkit-installer.exe` 単体ではインストールできず、同じディレクトリの `devkit.exe` を前提にしている
- [x] install 完了後に「どの `devkit` が優先されるか」を利用者へ示していない
- [ ] SmartScreen 警告の回避手段が release 運用に組み込まれていない

---

## 実装方針

### 1. GUI 実行時でも PATH を通す

- [x] installer の既定動作を「インストール + PATH 追加」に変更する
- [x] `--unpack-only` を opt-out として維持し、PATH 更新を抑止できるようにする
- [x] install 結果に `PATH added: Yes/No` だけでなく、失敗時の理由を明示する
- [x] PATH 更新後に新規 shell で反映されることを前提に、再起動不要の案内文を出す

### 2. 既存 `devkit.exe` 競合を可視化する

- [x] install 前に PATH 上の既存 `devkit.exe` を探索する
- [x] `%LOCALAPPDATA%\Xpotato\devkit\devkit.exe` 以外が先に見つかる場合は警告を出す
- [x] 警告には想定される競合先パスを表示する
- [x] install 完了後に「現在優先される `devkit`」または「新規 shell で優先される想定パス」を表示する

### 3. installer 単体配布へ寄せる

- [x] `devkit-installer.exe` 単体で動作できるよう payload 内包方式を検討する
- [x] release workflow で `devkit.exe` を installer に埋め込む build 手順を設計する
- [ ] 単体 installer 方式と bundle 方式の差分を比較する
- [x] まずは Windows 向け release asset として単体 installer を主導線にする案をまとめる

### 4. SmartScreen 回避のための署名運用を定義する

- [ ] Authenticode 署名が必要であることを docs に明記する
- [ ] `devkit-installer.exe` と `devkit.exe` の両方を署名対象に含める
- [ ] timestamp 付与を release 手順に含める
- [ ] OV / EV コードサイニング証明書の選択肢を整理する
- [ ] SmartScreen は署名だけで即時に完全解消するとは限らず、reputation に依存することを明記する

---

## 実装案

### 案A: 既存 CLI installer を改善する

- [x] `--add-to-path` なしでも PATH を更新する
- [x] `--unpack-only` のみ明示 opt-out にする
- [x] 既存 `devkit.exe` 競合検出を追加する
- [x] install 完了時のメッセージを強化する

利点:

- [ ] 現行実装との差分が小さい
- [ ] 先に PATH 問題を解消できる

欠点:

- [ ] `devkit-installer.exe` 単体配布はまだ解決しない
- [ ] GUI installer の見た目は得られない

### 案B: 単体自己展開 installer を導入する

- [x] installer binary に payload を埋め込む
- [x] 実行時に `%LOCALAPPDATA%\Xpotato\devkit` へ展開する
- [x] uninstall 用 payload も installer から生成する

利点:

- [x] `devkit-installer.exe` 単体配布が可能
- [x] 利用者導線が単純になる

欠点:

- [ ] release workflow の変更量が大きい
- [ ] binary size と build 手順が複雑化する

### 推奨

- [x] まず案Aで PATH と競合可視化を解消する
- [x] その後、案Bを別タスクとして導入する

---

## 残タスク

### 実装

- [x] installer の既定動作を PATH 追加ありへ変更
- [x] `--unpack-only` の意味を docs と実装で揃える
- [x] PATH 上の既存 `devkit.exe` 探索処理を追加
- [x] install 結果に競合警告と優先パス表示を追加
- [ ] `%LOCALAPPDATA%\Xpotato\devkit` が PATH に入っているかを install 後に再確認する
- [ ] manifest に PATH 更新の成否と競合検出情報を記録するか検討

### 配布

- [x] 単体 installer 化の設計を決める
- [x] release asset を `devkit-installer.exe` 主導線に寄せるか決める
- [x] README / `docs/install/windows-installation.md` / release docs を更新する

### 署名

- [ ] コードサイニング証明書の調達方針を決める
- [ ] GitHub Actions 上での署名方法を決める
- [ ] 秘密鍵保管方式を決める
- [ ] timestamp server 利用方針を決める
- [ ] SmartScreen 回避に関する利用者向け説明文を追加する

---

## 受け入れ条件

### インストール体験

- [x] 利用者が `devkit-installer.exe` を通常実行しただけで、既定では `%LOCALAPPDATA%\Xpotato\devkit` が PATH に追加される
- [x] install 完了後のメッセージで、インストール先、PATH 更新結果、現在または次回 shell で優先される `devkit.exe` が分かる
- [x] 既存の `C:\Users\miyut\.local\bin\devkit.exe` のような競合先がある場合、警告が表示される

### 安全性

- [x] `--unpack-only` では PATH を変更しない
- [x] uninstall は manifest 記録に従って削除し、他の `devkit.exe` には影響しない
- [x] PATH 削除は installer が追加したエントリのみを対象にする

### 配布

- [x] Windows release 導線として installer の使い方が明確に docs 化されている
- [x] 単体 installer 化を採る場合、同梱 `devkit.exe` に依存せず動作する

### 署名

- [ ] release 手順に署名工程が追加されている
- [ ] `devkit-installer.exe` と `devkit.exe` の両方に署名する方針が定義されている
- [ ] SmartScreen は reputation 依存であることが docs に明記されている

---

## 直近の推奨順序

- [x] Step 1: installer 既定動作を PATH 追加ありに変更
- [x] Step 2: 既存 `devkit.exe` 競合検出を追加
- [x] Step 3: install 完了メッセージと docs を更新
- [x] Step 4: 単体 installer 化の設計を確定
- [ ] Step 5: 署名運用を release workflow と docs に落とし込む
