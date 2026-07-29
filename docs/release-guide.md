# リリースガイドおよび CI トラブルシューティング

このドキュメントでは、DBX (Tauri) のビルド・リリースおよび CI ワークフローで発生しやすい問題と解決策について記載します。

---

## 🔑 `TAURI_SIGNING_PRIVATE_KEY` (updater 署名鍵) の注意事項

### 発生する問題
- 本家リポジトリ (`t8y2/dbx`) 以外のフォーク環境 (`cwatanab/dbx` 等) では、GitHub Repository Secrets (`TAURI_SIGNING_PRIVATE_KEY_BASE64`) が未設定のため、環境変数 `TAURI_SIGNING_PRIVATE_KEY` が空 (`""`) になります。
- ポータブル版 ZIP のビルド時 (`.github/workflows/release.yml`) に `tauri signer sign` コマンドが実行されると、秘密鍵が存在しないため署名処理が失敗するか、PowerShell の引数展開の不具合で `<FILE>` 位置引数エラーが発生します。
- その結果、`.sig` 署名ファイルが生成されず `Missing portable update signature` エラーとなり、リリース CI 全体が失敗します。

### 対処法・実装仕様
`.github/workflows/release.yml` の「Upload Windows portable ZIP」ステップでは、以下のロジックで安全にフォールバックします。

1. **`TAURI_SIGNING_PRIVATE_KEY` の判定**:
   環境変数が空の場合は署名コマンドを実行せず、空のプレースホルダー `.sig` ファイルを生成して処理を続行します。
2. **PowerShell での引数渡しの注意**:
   `pnpm tauri signer sign "--password=" $zipName` のように書くと、PowerShell の引数結合ルールにより `--password=$zipName` にバインドされ、対象ファイル名が消えてしまいます。
   独立した引数として渡すため、`pnpm exec tauri signer sign --password "" $zipName` を使用します。

```powershell
$signatureName = "${zipName}.sig"
if ([string]::IsNullOrEmpty($env:TAURI_SIGNING_PRIVATE_KEY)) {
  Write-Host "TAURI_SIGNING_PRIVATE_KEY is empty; creating signature placeholder for portable release."
  Set-Content -Path $signatureName -Value "placeholder"
} else {
  if ([string]::IsNullOrEmpty($env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD)) {
    pnpm exec tauri signer sign --password "" $zipName
  } else {
    pnpm exec tauri signer sign --password "$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD" $zipName
  }
}
```

---

## 📦 ポータブル版限定リリースの設定

非公式リリース（`-unofficial`）等で Portable ZIP のみを出力・配信したい場合は、`.github/workflows/release.yml` 内の `cleanup-non-portable-assets` ジョブで、`-portable.zip` を含まない不要なアセット (`.exe`, `.msi` 等) を全削除します。

```bash
mapfile -t NON_PORTABLE_ASSETS < <(
  gh release view "${GITHUB_REF_NAME}" \
    --repo "${GITHUB_REPOSITORY}" \
    --json assets \
    --jq '.assets[].name | select(contains("-portable.zip") | not)'
)

for ASSET in "${NON_PORTABLE_ASSETS[@]}"; do
  echo "Deleting non-portable release asset: ${ASSET}"
  gh release delete-asset "${GITHUB_REF_NAME}" "${ASSET}" \
    --repo "${GITHUB_REPOSITORY}" \
    --yes
done
```

---

## 🔄 フォークリポジトリでの `Sync Mirrors` の無効化

フォークリポジトリでは Gitee / CNB / AtomGit への同期トークンが存在しないため `Sync Mirrors` ワークフローが失敗します。  
`.github/workflows/sync-mirrors.yml` の各ジョブに `if: github.repository == 't8y2/dbx'` を指定することで、フォーク環境では安全にスキップ (`skipped`) されるように設定しています。

---

## ⚠️ GitHub Release で "Upload failed. Delete and try uploading this file again." になる場合

### 原因
1. **GitHub API のアップロード切断・不完全転送**:
   大容量の Zip アセット（`DBX_0.5.67-unofficial_x64-portable.zip` 等）を CI や CLI 経由でアップロード中、ネットワークの中断や GitHub 側の接続切断が発生すると、GitHub Release 上に壊れたプレースホルダーが残り `Upload failed` と表示されます。
2. **既存の壊れたアセットとの競合**:
   Web UI や CLI から同名ファイルを上書きアップロードしようとした際、既に不完全な `Upload failed` 状態のアセットが存在するとアップロードが失敗します。

### 解決方法
1. **GitHub Web UI で手動削除**:
   対象リポジトリの Release ページ（例: `v0.5.67-unofficial`）を開き、「Edit release」から `Upload failed` と表示されている `DBX_0.5.67-unofficial_x64-portable.zip` （および `.sig`）のゴミ箱アイコンを押して削除します。
2. **CI ワークフローの自動リトライ・事前削除**:
   `.github/workflows/release.yml` では、アップロード前に既存のアセットを `gh release delete-asset` で削除し、最大3回まで自動リトライするよう対策が組み込まれています。

