## 用語集

本ドキュメントは `Nue` 内で使用する用語を定義する。
本文では本書の定義を前提とし、同義語の使用は禁止する。

| 用語 | 分類 | 定義(description) | note |
| :- | :- | :- | :- |
| Nue | プロダクト | Rust/GPUI ベースのマルチプラットフォーム対応エディタ兼ターミナルエミュレーター。 | 本仕様の対象プロダクト。 |
| App Host | アーキテクチャ | 全ワークスペースのライフサイクル管理、グローバル設定管理、通知集約を行う上位実行主体。 | Runtime と同義で扱わない。 |
| Workspace Session | アーキテクチャ | ワークスペース単位で独立動作する実行単位。 | 障害分離の最小単位。 |
| AI Agent | 実行主体 | ユーザー意図に基づき MCP ツール経由で作業を実行するエージェント。 | 主体はユーザーと区別する。 |
| MCP Router | アーキテクチャ | AI Agent からのツール呼び出しを受け付け、実行制御する境界コンポーネント。 | 認可判定は `Authorization Policy` の Domain/Tool/Execution Context/Approval State/引数を評価し、Domain/Tool → Execution Context → `deny` → `allow` → 暗黙の拒否の順で判定する。 |
| Editor Core | アーキテクチャ | バッファ管理、差分適用、内部コマンド実行を担う中核。 | UI とは責務分離する。 |
| UI View | UI | ユーザー操作と状態可視化を担当する表示層。 | Legacy/Galaxy を含む。 |
| Legacy View | UI | ディレクトリツリー中心の階層表示ビュー。 | 物理構造把握向け。 |
| Galaxy View | UI | `v1.0` 以降で導入される依存関係ベースのグラフ表示ビュー。LSP 依存関係をノード/エッジで表現し、AI が触れたノードは `彗星` で強調し、影響範囲は `衝撃波` で伝播を可視化する。 | Legacy View とは役割を明確に分離し、最大 500 ノードまで描画した状態で 45fps 以上を維持し、WCAG AA 相当のアクセシビリティを満たす。 |
| Workspace Rail | UI | ワークスペース切替と状態表示を担う左レイル UI。 | Slack-like は説明語で非用語。 |
| Agent Status | 状態 | エージェント実行状態を示す列挙値。 | `Busy`/`Waiting`/`Error`/`Idle`。 |
| Shadow Buffer | データモデル | エージェント変更を承認前に保持する一時差分領域。各差分には発生時刻・発行元・対象ファイル・`Agent Status` を含み、`Galaxy View` と `Legacy View` で列挙/レビューできる。初期リリースでは `Accept` のみを提供し、`Workspace Session` 単位と `ファイル単位` の承認粒度をサポートする。 | 承認後に本バッファへ反映し、永続化されない。 |
| Accept | 操作 | Shadow Buffer の差分をユーザーが承認し、確定反映する操作。 | 初期リリースは `Accept` のみ提供。 |
| Approval Unit | 操作 | 変更承認の粒度（例: 一括、ファイル単位、ハンク単位）。初期リリースは `Workspace Session` 単位の一括 `Accept` と `ファイル単位 Accept` を提供し、`Partial Accept`/`Reject`/`Revert` は未実装。 | 将来的にハンク単位など細分化できる。 |
| Approval State | 状態 | `MCP Router` のポリシーが指す承認条件。 | `auto_allow`/`requires_user_consent`/`blocked` のいずれか。`requires_user_consent` は Shadow Buffer 承認の進行と連動し、`blocked` は常時拒否。 |
| Global Config | 設定 | ユーザー全体に適用される設定。 | `~/.config/nue/config.yaml`。 |
| Workspace Config | 設定 | ワークスペース固有設定。 | `.nue/config.yaml`。 |
| Config Merge Priority | 設定 | 複数設定ソース衝突時の優先順序。 | 環境変数 > Workspace > Global > Default。 |
| Terminal Emulator | 機能 | エージェントの `run_command` 実行や対話型シェルを表示する端末機能。 | 初期リリースでは PTY、500 行以上のスクロールバック、文字幅/Unicode 整合を保証する。 |
| MCP Tool | インターフェース | Agent が Router 経由で利用する操作 API 単位。 | 具体ツール集合は仕様側で規定。 |
| Execution Context | 実行文脈 | `MCP Router` が認可判定に用いる実行条件。 | 例: `Workspace Session`、ブランチ。 |
| Audit Event | データ | MCP 操作・Shadow Buffer承認・認可拒否などを `event_id`/タイムスタンプ付きで記録し、`Workspace Session`/`Execution Context`/`Agent Status`/引数ハッシュを含む。初期はメモリ保持、`v1.0` 以降で JSONL へ追記する。 | ファイル保存は`audit.storage.path`に追記、`audit.retention_days` (デフォルト 30 日) で古い記録を破棄、`audit.anonymization.level` で引数/機微情報を制御、`audit.queue.max` で再送キュー上限と破棄ルールを調整。 |
| Authorization Policy | ルール | `MCP Router` がドメイン・ツール・許可引数・実行コンテキスト・承認状態・ポリシーバージョンの組み合わせを宣言的に表現し、呼び出しごとの `deny`/`allow` を決定するルールセット。 | 各エントリは `policy_id`/`argument_constraints`/`execution_context`/`approval_state`/`effect`/`priority` を持ち、未定義呼び出しは暗黙の拒否。 |
| Policy Evaluation Order | ルール | `MCP Router` が `Authorization Policy` を評価する決定順序。 | ドメイン/ツール一致 -> 実行コンテキスト一致 -> `deny` 評価 -> `allow` 評価 -> 暗黙拒否。 |
| Policy Revision | 管理 | Authorization Policy が再定義されるたびにインクリメントされる数値。 | `policy_id` とセットで `Audit Event` に記録され、最新の `policy_revision` を持つエントリが評価優先される。 |
| Config Change Event | イベント | 設定変更の差分を表す構造体で、`source`/`revision`/`changed_keys`/`previous_values`/`hot_reloadable` を含み `App Host` → `Workspace Session` 間で伝播される。 | `ConfigChangeEvent` を起点に再評価サイクルが展開する。 |
| Config Revision | シーケンス | 設定の再評価ごとに単調増加する番号。 | `ConfigChangeEvent` には `config_revision` を含め再起動や再適用の状態を判別可能にする。 |
| Hot Reload Scope | 範囲 | 設定変更が `Workspace Session` 内のどのコンポーネント（例: `router`）に影響するかを示す列挙値。 | `hot_reloadable=false` の変更は再起動が完了しない限り適用されない。 |
