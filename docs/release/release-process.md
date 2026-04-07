# リリース手順 (Release Process)

本プロジェクトでは GitHub Actions を利用して、マルチプラットフォーム向けバイナリ（Windows, macOS, Linux）のビルドと GitHub Releases へのアップロードを自動化しています。

## 手順

新しいバージョンをリリースする際の手順は以下の通りです。

### 1. バージョン番号の更新
`rust/crates/devkit-cli/Cargo.toml` および必要に応じて他のクレートの `version` を更新します。
その後、`cargo build` などを実行して `Cargo.lock` を更新し、変更を commit & push します。

### 2. タグの作成とプッシュ
対象となるコミット（通常は `main` の最新）に、`v` プレフィックスから始まる SemVer 形式のタグを作成し、プッシュします。

```bash
git checkout main
git pull

# 例: 0.1.0 をリリースする場合
git tag "v0.1.0"
git push origin "v0.1.0"
```

### 3. GitHub Actions の確認
タグが `origin` にプッシュされると、`.github/workflows/release.yml` が自動で発火します。
[Actions タブ](https://github.com/Xpotato1024/xpotato-devkit/actions) からビルドジョブの進行状況を確認してください。

### 4. GitHub Releases ページでの調整
ジョブが完了すると、[Releases ページ](https://github.com/Xpotato1024/xpotato-devkit/releases) に対象タグのリリースが作成され、バイナリアセットがアタッチされます。

最後に、必要に応じてリリースの「Edit」ボタンを押し、Release notes (Changelog) を追記・生成して公開設定を完了させてください。

## 対象プラットフォーム

現状、以下のプラットフォーム向けのバイナリが生成されます。
- `x86_64-unknown-linux-gnu` (Linux)
- `x86_64-pc-windows-msvc` (Windows)
- `aarch64-apple-darwin` (macOS Apple Silicon)
