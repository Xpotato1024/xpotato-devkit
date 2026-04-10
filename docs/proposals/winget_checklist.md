# xpotato-devkit 0.1.5 winget 公開直前チェックリスト

対象バージョン: `v0.1.5`

判定:
- `v0.1.5` の release asset / manifest / docs 整合は確認できた
- ただし reinstall 後の uninstall で PATH 管理が崩れる実バグを branch 上で修正したため、`winget` 提出対象は `v0.1.5` ではなく次の release tag に繰り上げる
- version を上げる理由は manifest 作業ではなく機能修正であり、方針上は patch release なので候補は `v0.1.6`

目的:
- `winget-pkgs` 提出直前に、manifest・release asset・installer 契約・docs の不整合を潰す
- 提出後の差し戻しや修正往復を減らす
- repo 内の source-of-truth と実際の release を一致させる

---

## 1. 対象 release の固定

- [x] 対象 release が `v0.1.5` で確定している
- [x] GitHub Release が公開済みである
- [x] Windows 向け standalone installer asset が存在する
- [x] checksum asset が存在する

### 想定 asset
- [x] `devkit-installer-v0.1.5-x86_64-pc-windows-msvc.exe`
- [x] `devkit-v0.1.5-sha256.txt`

補足:
Windows release では standalone installer asset を `winget` に使う方針になっている :contentReference[oaicite:2]{index=2}。

---

## 2. winget メタデータ確認

repo 内 draft と実提出内容が一致していることを確認する。

- [x] `PackageIdentifier` = `Xpotato.devkit`
- [x] `PackageName` = `devkit`
- [x] `Publisher` = `Xpotato`
- [x] `Moniker` = `devkit`
- [x] `Commands` に `devkit` を含む
- [x] `InstallerType` = `exe`
- [x] `Scope` = `user`

これらは `docs/install/windows-winget-prep.md` および `packaging/winget/README.md` の固定方針と一致していること :contentReference[oaicite:3]{index=3} :contentReference[oaicite:4]{index=4}。

---

## 3. version / URL / SHA256 の更新

`packaging/winget/` の draft を実提出値に更新する。

- [x] `PackageVersion` を `0.1.5` に更新した
- [x] `InstallerUrl` を `devkit-installer-v0.1.5-x86_64-pc-windows-msvc.exe` の URL に更新した
- [x] `InstallerSha256` を `devkit-v0.1.5-sha256.txt` から転記した
- [x] URL がブラウザや curl 相当で取得可能である
- [x] SHA256 の値が installer 本体と一致する

注記:
- この 3 点は `v0.1.5` の検証としては成立した
- ただし branch で機能修正が発生したため、最終提出時は次の release tag に対して同じ手順をやり直す

repo 内 draft metadata の更新手順もこの3点を前提にしている :contentReference[oaicite:5]{index=5}。

---

## 4. installer 契約確認

`winget` から扱う前提の installer 契約が実物と一致していることを確認する。

### サポートオプション
- [x] `--silent`
- [x] `--uninstall`
- [x] `--unpack-only`
- [x] `--install-dir <path>`
- [x] `--add-to-path`

### 動作確認
- [x] `.\devkit-installer.exe --silent` が成功する
- [x] `.\devkit-installer.exe --uninstall --silent` が成功する
- [x] `.\devkit-installer.exe --unpack-only` が成功する
- [x] 既定 install が `%LOCALAPPDATA%\Xpotato\devkit` に入る
- [x] user-scope install である
- [x] install 後に `devkit` が解決される

installer contract は docs に明記されている :contentReference[oaicite:6]{index=6}。

---

## 5. exit code の確認

- [x] 成功時 `0`
- [x] operational failure で `1`
- [x] argument / usage error で `2`

さらに以下も確認する。

- [x] `--silent` で成功時 stdout が抑制される
- [x] エラーは stderr に残る
- [x] `winget` 実行時に失敗判定しやすい

exit code 契約も docs に整理済み :contentReference[oaicite:7]{index=7}。

---

## 6. install / upgrade / uninstall の実地確認

### install
- [x] 未インストール環境で install 成功
- [x] PATH 追加が期待通り
- [x] 新しい shell で `devkit --help` が通る

### reinstall / upgrade
- [x] 同一 install 先への再実行で壊れない
- [ ] `v0.1.4` → `v0.1.5` 上書きで問題ない
- [x] 古い binary が PATH 上に残って誤解決しない
- [x] README の `where devkit` 案内と整合する

注記:
- `v0.1.4` の Windows release は standalone installer ではなく zip のみで、現行と asset 形態が異なる
- 今回の gate として重要だったのは「同一 install 先への再実行で PATH 管理が壊れないこと」で、これは branch 上で修正し E2E で確認した

### uninstall
- [x] uninstall 後に管理下ファイルのみ削除される
- [x] `install-manifest.json` 基準の削除が機能する
- [x] PATH の扱いが docs と矛盾しない

この install / upgrade / uninstall 方針は docs 側で定義されている :contentReference[oaicite:8]{index=8}。

---

## 7. docs 整合性確認

### README
- [x] Windows asset 名が実 release と一致
- [x] standalone installer 名が一致
- [x] `--silent` の説明が一致
- [x] `windows-winget-prep.md` へのリンクが生きている
- [x] `packaging/winget/` への言及が現状と一致

### winget prep docs
- [x] PackageIdentifier など固定値が README と矛盾しない
- [x] install 先、scope、silent switch が一致
- [x] SHA256 の説明が release 実態と一致

### packaging/winget/
- [x] README の更新手順が今回の release と一致
- [x] draft metadata が放置された古い version のままではない

現状、README / prep docs / draft metadata の3点で source-of-truth を揃える設計になっている :contentReference[oaicite:9]{index=9} :contentReference[oaicite:10]{index=10} :contentReference[oaicite:11]{index=11}。

---

## 8. manifest 品質確認

- [x] package 名や publisher 名に揺れがない
- [x] version が SemVer と release tag に整合する
- [x] installer URL が tag 固定 URL になっている
- [x] command と moniker が実 CLI と整合する
- [x] user scope を前提とする説明が揃っている
- [x] short description / description が現状の README の価値説明と整合する

---

## 9. 実機検証

少なくとも Windows 環境で以下を確認する。

- [x] 未インストール環境で install
- [x] install 後に `devkit --help`
- [x] install 後に `devkit search text "foo"` のような新コマンドが動く
- [ ] upgrade install
- [x] uninstall
- [x] reinstall

可能なら別ユーザーまたはクリーン環境でも確認する。

---

## 10. 提出前の最終判断

以下をすべて満たしたら提出してよい。

- [x] standalone installer URL が安定している
- [x] SHA256 が一致している
- [x] silent install / uninstall が成功する
- [x] user-scope install 契約が docs と一致する
- [x] README / prep docs / draft metadata の不整合がない
- [x] 0.1.5 の実 asset と manifest が一致している
- [ ] reinstall / uninstall 後に PATH 管理が崩れない release が公開済みである

最終判断:
- `v0.1.5` の公開済み asset と manifest は一致している
- しかし reinstall / uninstall の PATH 管理バグを branch 上で修正したため、`winget` 提出は `v0.1.5` では止める
- 次の提出対象はこの修正を含む新しい patch release (`v0.1.6` 想定)

---

## 11. 今回は後回しでよいもの

以下は重要だが、`winget` 提出の最小条件とは切り分けて扱う。

- [ ] コード署名の完全対応
- [ ] Microsoft Store 提出
- [ ] GUI 導入
- [ ] `rg` / `tree` 完全互換
- [ ] Linux / macOS 配布方式の再設計

---

## 12. 提出後に見るべき項目

- [ ] install 成功報告 / 失敗報告の収集
- [ ] PATH 競合の報告有無
- [ ] silent install の想定外ケース
- [ ] uninstall の不整合
- [ ] 次 version 向けの manifest 更新フローの簡素化
