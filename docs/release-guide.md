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
  Set-Content -Path $signatureName -Value "" -NoNewline
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
