## 用語集

本ドキュメントは `Nue` 内で使用する用語を定義する。
本文では本書の定義を前提とし、同義語の使用は禁止する。

| 用語 | 分類 | 定義(description) | note |
| :- | :- | :- | :- |
| Nue | プロダクト | Rust/GPUI ベースのマルチプラットフォーム対応エディタ兼ターミナルエミュレーター。 | 本仕様の対象プロダクト。 |
| Neon-Night Theme | デザイン | 深いネイビー基調にシアン・マゼンタを配した `Nue` のシステムテーマ。 | `spec-nue.md` Sec.3.1 でカラー/アニメーション規約を定義し、アクセシビリティ基準を満たすことを要求している。 |
| Nue Color Palette | デザイン | WCAG AA を満たす `Deep Abyss` ～ `Dusty Grey` の 9 色セット。 | Sec.3.1.1 で各用途（背景/状態/テキスト）が明記され、`nue-ui` GPUI 定義への反映を義務付ける。 |
| App Host | アーキテクチャ | 全ワークスペースのライフサイクル管理、グローバル設定管理、通知集約を行う上位実行主体。 | Runtime と同義で扱わない。 |
| Workspace Session | アーキテクチャ | ワークスペース単位で独立動作する実行単位。 | 障害分離の最小単位。 |
| AI Agent | 実行主体 | ユーザー意図に基づき MCP ツール経由で作業を実行するエージェント。 | 主体はユーザーと区別する。 |
| MCP Router | アーキテクチャ | AI Agent からのツール呼び出しを受け付け、実行制御する境界コンポーネント。 | 認可判定は `Authorization Policy` の Domain/Tool/Execution Context/Approval State/引数を評価し、Domain/Tool → Execution Context → `deny` → `allow` → 暗黙の拒否の順で判定する。 |
| Editor Core | アーキテクチャ | バッファ管理、差分適用、内部コマンド実行を担う中核。 | UI とは責務分離する。 |
| UI View | UI | ユーザー操作と状態可視化を担当する表示層。 | Legacy/Galaxy を含む。 |
| Legacy View | UI | ディレクトリツリー中心の階層表示ビュー。 | 物理構造把握向け。 |
| Galaxy View | UI | `v1.0` 以降で導入される依存関係ベースのグラフ表示ビュー。LSP 依存関係をノード/エッジで表現し、AI が触れたノードは `彗星` で強調し、影響範囲は `衝撃波` で伝播を可視化する。 | Legacy View とは役割を明確に分離し、最大 500 ノードまで描画した状態で 45fps 以上を維持し、WCAG AA 相当のアクセシビリティを満たす。 |
| Workspace Rail | UI | ワークスペース切替と状態表示を担う左レイル UI。 | Slack-like は説明語で非用語。 |
| Command Hub | UI | `Cmd + Shift + P` / `Ctrl + Shift + P` で開くモーダル型コマンドパレット。AIと人間が意図を共有し、MCP Tool呼び出しを起点としてショートカット/履歴/自然言語候補を表示する。 | `spec-nue.md` Sec.6.1 で構造とモードを定義する。 |
| Backoff State | UI | `Command Hub` が候補更新遅延（16ms を超過）を検知した際に表示する遷移状態。遅延中は直前候補を保持し、`Re-scoring…` / `awaiting nue-semantic` 等の進捗ラベルと `Action Mode`/`Navigation Mode` への移行ヒントを併せて出す。 | `spec-nue.md` Sec.6.1.1 で再スコアリング中の挙動を定義している。 |
| Agent Status | 状態 | エージェント実行状態を示す列挙値。 | `Busy`/`Waiting`/`Error`/`Idle`。 `spec-nue.md` Sec.3.1.2 では各状態を `Neon Cyan`/`Solar Flare`/`Cyber Magenta`/`Dusty Grey` などで色/アニメーション表現するルールを定義し、`Workspace Rail`・`Command Hub` などに一貫して反映することを要求している。 |
| Intent / Smart Search | 機能 | `Command Hub` の自然言語入力モードで、`nue-semantic` を中心とした候補推論により `MCP Tool` や UI アクションを提案する。 | 100ms以内の候補生成と、発行元・Approval Stateを付与するプロセスを含む（Sec.6.1.1）。 |
| nue-semantic | コンポーネント | Intent/Smart Search を構成するローカル生成AIエンジン。Phi 等の SML モデルをバインドし、Intent Resolver・Local RAG・Policy-Aware Scoring を組み合わせて候補を出す。 | 外部 API には依存せずオフライン実行を想定（Sec.6.1.2）。 |
| Intent Resolver | コンポーネント | `nue-semantic` のサブモジュールで、自然言語入力をミリ秒スケールでトークナイズし、MCP Tool や UI アクションにマッピングする推論エンジン。 | `approval_state` や context を含む実行プランを返す必要がある。 |
| Local RAG | コンポーネント | `nue-semantic` が保持する、プロジェクトファイル名/関数名/設定名のベクトル/重み付きインデックス。曖昧な入力を意味的に関連する候補に橋渡しする。 | ファイルシステム変更時に 5 秒以内で更新。 |
| Policy-Aware Scoring | 機能 | `nue-semantic` が `Authorization Policy` の `argument_constraints`/`execution_context` を評価し、実行可能な候補を上位にソートする評価機構。 | 実行可能性と `approval_state` を加味した優先順位付けを実現する（Sec.6.1.2）。 |
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
| Resolution Hint | データ | `MCP Router` が `deny` 応答に含める補助情報で、`policy.message` に加えて修正すべき設定キーや推奨 UI 操作・識別子 (`config_path:key`/`policy_id`) を提供する。 | `spec-nue.md` Sec.4.1.4 で `Audit Event` へ `resolution_hint` を含め、ユーザーが次の手順を把握できるように規定。 |
| DeniedRequestHistory | データ構造 | `MCP Router` が同一 `agent_id`/`policy_id` の連続 `deny` を監視するために保持する履歴レコードで、再試行待機時間(`retry_delay_seconds`)や `approval_request_id` を管理する。 | `spec-nue.md` Sec.4.1.4 に指数的バックオフ (`retry_delay_seconds` 3〜30 秒) と `blocked` ポリシーで 24 時間抑制する運用を規定。 |
| Feedback Loop | UI | `MCP Router` による連続拒否や `resolution_hint` に基づき、`Workspace Rail`/`Command Hub` がエージェント状態を `Error` へ遷移させ、バナーで設定修正を促すアラート表示のパターン。 | `spec-nue.md` Sec.4.1.4 で `App Host` に `Feedback Loop` バナーと `ConfigChangeEvent` の強調を要求。 |
| Authorization Policy | ルール | `MCP Router` がドメイン・ツール・許可引数・実行コンテキスト・承認状態・ポリシーバージョンの組み合わせを宣言的に表現し、呼び出しごとの `deny`/`allow` を決定するルールセット。 | 各エントリは `policy_id`/`argument_constraints`/`execution_context`/`approval_state`/`effect`/`priority` を持ち、未定義呼び出しは暗黙の拒否。 |
| Policy Evaluation Order | ルール | `MCP Router` が `Authorization Policy` を評価する決定順序。 | ドメイン/ツール一致 -> 実行コンテキスト一致 -> `deny` 評価 -> `allow` 評価 -> 暗黙拒否。 |
| Policy Revision | 管理 | Authorization Policy が再定義されるたびにインクリメントされる数値。 | `policy_id` とセットで `Audit Event` に記録され、最新の `policy_revision` を持つエントリが評価優先される。 |
| ToolExecutionState | データ構造 | `MCP Router` が `run_command` ごとに `agent_id`/`workspace_session_id` と共に保持する状態トラッキング。 | `state`（`Idle`/`Queued`/`Running`/`Completed`/`Failed`）、`command_line`、`start_time`、`completion_time` を含み、`ToolRequestQueue` と連携して重複実行を防ぐ。 |
| ToolRequestQueue | データ構造 | `MCP Router` が同一 `Workspace Session` 内の `run_command` 要求を FIFO で待機させるキュー。 | `tool.execution.queue_max_pending` を上限とし、`queue_overflow` 時に `Audit Event` を生成、`result=queue_start` 等で状態を通知する。 |
| Config Change Event | イベント | 設定変更の差分を表す構造体で、`source`/`revision`/`changed_keys`/`previous_values`/`hot_reloadable` を含み `App Host` → `Workspace Session` 間で伝播される。 | `ConfigChangeEvent` を起点に再評価サイクルが展開する。 |
| Config Revision | シーケンス | 設定の再評価ごとに単調増加する番号。 | `ConfigChangeEvent` には `config_revision` を含め再起動や再適用の状態を判別可能にする。 |
| Hot Reload Scope | 範囲 | 設定変更が `Workspace Session` 内のどのコンポーネント（例: `router`）に影響するかを示す列挙値。 | `spec-nue.md` Sec.5.3.1 で `app`/`router`/`terminal`/`editor`/`semantic`/`agent` を明記し、`hot_reloadable=false` の変更は再起動完了まで適用されないとする。 |
| Dependency-Aware Re-init Sequence | プロセス | `ConfigChangeEvent` が複数の `hot_reload_scope` を含む場合に下位レイヤーから上位レイヤーへ順次再初期化する再評価シーケンス。 | `spec-nue.md` Sec.5.3.1 で `app`→`router`→`terminal`→`editor`→`semantic`→`agent` の順序と `Audit Event` の失敗記録を定義。 |
