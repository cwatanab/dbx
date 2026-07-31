# SSH Agent 認証のサポート仕様と修復記録

DBX における SSH Agent 認証の動作仕様、不具合原因、および修復内容を記録する。

## 概要

SSH Agent 認証は、秘密鍵をメモリ上のエージェントプロセスで安全に保持し、暗号化キーを渡さずに SSH トンネル接続を確立する認証方式である。
`v0.5.46` から `v0.5.71` にかけての認証構造リファクタリングに伴い、フロントエンド UI とバックエンド（Rust）の条件判定に不整合が生じ、SSH Agent 認証が正常に動作しない問題が発生していた。

## 不具合の原因

本不具合の原因は、以下の 2 点である。

- **フロントエンド UI での非活性化**：[ConnectionDialog.vue](file:///D:/Develop/dbx/apps/desktop/src/components/connection/ConnectionDialog.vue#L6895) の認証方式ドロップダウンにおいて、`auth_method === "agent"` がレガシー項目として非活性化（`disabled`）されていた。
- **バックエンドでの評価優先順位**：[ssh_tunnel.rs](file:///D:/Develop/dbx/crates/dbx-core/src/db/ssh_tunnel.rs#L315) において、`auth_method == "key"` が指定された場合、`use_ssh_agent == true` であっても鍵ファイル検証のみが評価され、Agent 認証処理 (`try_authenticate_with_agent`) へ到達しなかった。

## 対応と修正内容

### 1. バックエンド判定ロジックの修正

[crates/dbx-core/src/db/ssh_tunnel.rs](file:///D:/Develop/dbx/crates/dbx-core/src/db/ssh_tunnel.rs#L317-L358) を修復した。
`auth_method` が `"key"` の場合であっても、鍵パスが未指定であるか鍵認証が完了しない場合に `use_ssh_agent` が有効（`true`）であれば `try_authenticate_with_agent` を試みるフォールバック構造へ変更した。

### 2. フロントエンド UI と設定項目の復元

[apps/desktop/src/components/connection/ConnectionDialog.vue](file:///D:/Develop/dbx/apps/desktop/src/components/connection/ConnectionDialog.vue#L6895-L6936) を修復した。
ドロップダウンの `SelectItem` から `disabled` を除外し、「SSH Agent」を直接選択可能にした。
また、`auth_method` が `agent` の場合および `use_ssh_agent` が有効な場合に Agent ソケットパス指定欄が表示されるようテンプレートロジックを調整した。

### 3. プラットフォーム別の制約事項

- **Unix / macOS**：標準の UNIX ドメインソケット (`$SSH_AUTH_SOCK`) およびカスタムソケットパスの指定に対応する。
- **Windows**：PuTTY Pageant (`pageant::PageantStream`) 経由の認証に対応する（Windows 標準 OpenSSH Agent パイプは未対応）。
