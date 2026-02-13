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
| Focus Score | UI メトリクス | Galaxy View がノードの優先描画を決めるために算出する段階的スコア。P0（AI Activity）〜P3（Context）で構成し、スコアの低いノードは集約対象となる。 | `spec-nue.md` Sec.3.3.1 で 500 ノード超時の描画維持に使用される。 |
| Focus ID | データ | `Minimap`/`Structure Path`/`Smart Gutter`/`SessionSnapshot` で同じ差分や行を強調するために共有される識別子。現行では粒度・一意性・更新タイミングが未定義なため、`specs/ask.md` Q25 の確定を待つ必要がある。 | `spec-nue.md` Sec.3.5.1/3.6/3.7/7.1 に関連記述があり、決定後は各ビューのフォーカス同期ルールと `Audit Event` の `focus_id` 設定を含めて更新する。 |
| Nebula | UI | 同一ディレクトリ内の低優先度ノード群をまとめた高レベル集合体。 | Galaxy View の 500 ノード超時に `Focus Score` で選別されたノードを代替表示する「星雲」クラスタ。 |
| Workspace Rail | UI | ワークスペース切替と状態表示を担う左レイル UI。 | Slack-like は説明語で非用語。 |
| Command Hub | UI | `Cmd + Shift + P` / `Ctrl + Shift + P` で開くモーダル型コマンドパレット。AIと人間が意図を共有し、MCP Tool呼び出しを起点としてショートカット/履歴/自然言語候補を表示する。 | `spec-nue.md` Sec.6.1 で構造とモードを定義する。 |
| Minimap | UI | 編集中バッファの行構造・差分・検索・ビルド結果を 1px スケールで右側に可視化するヒートマップ状プレビュー。`Shadow Buffer`/`Command Hub`/`Audit Event` と同期し、クリックで該当差分の承認に移る。 | `spec-nue.md` Sec.3.5 で 60fps 表示、オーバーレイ色分け、`focus_id` 連携、`Audit Event` 連携を要求している。 |
| Global Search | 検索 | 全ワークスペースを横断するオーバーレイ検索パネル。`Legacy View`/`Command Hub` からの遷移、`workspace.search.exclude` 設定との同期、`Shadow Buffer` 差分と `Audit Event` へのバッジ/記録を備える。 | `spec-nue.md` Sec.6.2.1 で呼び出し・フィルター・同期要件と `Audit Event` の記録を定義している。 |
| Semantic Search | 検索 | `nue-semantic` が生成する意味ベースの検索結果。`Global Search` UI に統合され、`Local RAG` インデックスの `semantic_score` をハイライトして `Smart Gutter`/`Structure Path`/`Command Hub` へ `Relevance Intent` を送る。 | `spec-nue.md` Sec.6.2.2 で `Local RAG` 更新ルール、再スコアリング中ラベル、外部エージェント提案との連携を定義する。 |
| Structure Path | UI | エディタ上部に階層的な Project > Folder > File > Class/Module > Method パスを常時表示し、`Shadow Buffer` の差分・承認状態・`Command Hub` 候補・`Minimap` の `focus_id` を同期させるナビゲーションバー。 | `spec-nue.md` Sec.3.6 で更新トリガー・差分マーカー・相互作用・アクセシビリティ要件を RFC 2119 で定義した。 |
| Relevance Intent | 機能 | `Semantic Search` 結果に紐づけられた意図トークンで、`Command Hub` への `Intent Request` と `Approval Unit` の橋渡しを担う。 `semantic_score` とマッチ対象を内包し、`Audit Event` にも注記される。 | `spec-nue.md` Sec.6.2.2 で `Semantic Match` から `Command Hub` へ送る流れと `Relevance Intent` の役割を記述している。 |
| Smart Gutter | UI | 行番号の隣に `Shadow Buffer`/`Git`/`Audit Event` の情報を表示し、`Command Hub`/`Minimap`/`Structure Path` と同期して `Approval Request` の焦点や `Agent Status` を可視化する情報レイヤー。 | `spec-nue.md` Sec.3.7 で `AI Pulse Indicator`/`Git Delta` 表示、`Audit Event` との整合、`Glyph` によるアクセシビリティ対応を RFC 2119 形式で規定した。 |
| Backoff State | UI | `Command Hub` が候補更新遅延（16ms を超過）を検知した際に表示する遷移状態。遅延中は直前候補を保持し、`Re-scoring…` / `awaiting nue-semantic` 等の進捗ラベルと `Action Mode`/`Navigation Mode` への移行ヒントを併せて出す。 | `spec-nue.md` Sec.6.1.1 で再スコアリング中の挙動を定義している。 |
| Agent Status | 状態 | エージェント実行状態を示す列挙値。 | `Busy`/`Waiting`/`Error`/`Idle`。 `spec-nue.md` Sec.3.1.2 では各状態を `Neon Cyan`/`Solar Flare`/`Cyber Magenta`/`Dusty Grey` などで色/アニメーション表現するルールを定義し、`Workspace Rail`・`Command Hub` などに一貫して反映することを要求している。 |
| Intent / Smart Search | 機能 | `Command Hub` の自然言語入力モードで、`nue-semantic` を中心とした候補推論により `MCP Tool` や UI アクションを提案する。 | 100ms以内の候補生成と、発行元・Approval Stateを付与するプロセスを含む（Sec.6.1.1）。 |
| nue-semantic | コンポーネント | Intent/Smart Search を構成するローカル生成AIエンジン。Phi 等の SML モデルをバインドし、Intent Resolver・Local RAG・Policy-Aware Scoring を組み合わせて候補を出す。 | 外部 API には依存せずオフライン実行を想定（Sec.6.1.2）。 |
| SemanticContextManager | コンポーネント | `App Host` が `nue-semantic` の単一インスタンスを管理するファサードで、`SemanticContextHandle` を通じてワークスペースごとのコンテキストを分離する。 | `spec-nue.md` Sec.6.1.3 で共有インスタンス・スロット制御・再初期化時の `semantic_context_state` 登録を RFC 2119 で規定。 |
| SemanticContextHandle | データ | `SemanticContextManager` が生成するハンドルで、`workspace_session_id`/`local_rag_revision`/`semantic_context_id` を `nue-semantic` のリクエストに付加することで他セッションとの汚染を防ぐ。 | `spec-nue.md` Sec.6.1.3 では `Local RAG` 更新・`Relevance Intent` の Pending 状態管理・`session_snapshot` への状態保存を義務付けている。 |
| semantic_context_state | データ | `SessionSnapshot`（Sec.7.1）が記録する `nue-semantic` 側の状態情報（`intent_history_id`/`pending_relevance_intents` など）で、再起動・Wake up 後に `SemanticContextHandle` へ再登録するために利用される。 | `spec-nue.md` Sec.6.1.3 で `App Host` に一般化された再登録要件を記載。 |
| Intent Resolver | コンポーネント | `nue-semantic` のサブモジュールで、自然言語入力をミリ秒スケールでトークナイズし、MCP Tool や UI アクションにマッピングする推論エンジン。 | `approval_state` や context を含む実行プランを返す必要がある。 |
| Local RAG | コンポーネント | `nue-semantic` が保持する、プロジェクトファイル名/関数名/設定名のベクトル/重み付きインデックス。曖昧な入力を意味的に関連する候補に橋渡しする。 | ファイルシステム変更時に 5 秒以内で更新。 |
| Policy-Aware Scoring | 機能 | `nue-semantic` が `Authorization Policy` の `argument_constraints`/`execution_context` を評価し、実行可能な候補を上位にソートする評価機構。 | 実行可能性と `approval_state` を加味した優先順位付けを実現する（Sec.6.1.2）。 |
| Session Snapshot | データ構造 | `Workspace Session` の状態を記録する `SessionSnapshot` で、開いているタブ/スプリット/カーソル位置/Shadow Buffer の差分/`ToolExecutionState` や `Command Hub` の未処理候補などを含むこと。 | `spec-nue.md` Sec.7.1 で更新タイミング・保存パス・復元要件を RFC 2119 で定義し、タブ・レイアウト管理および `Sleep Mode` 復元の基盤とする。 |
| Sleep Mode | 機能 | 非アクティブな `Workspace Session` を `Workspace Rail` から Sleep させ、リソースを解放しながら `SessionSnapshot` で瞬時に復元できる休止/復帰メカニズム。 | `spec-nue.md` Sec.7.2 でトリガー、`Audit Event` 記録、`ConfigChangeEvent` 保留、復元後の差分復旧ルールを定義している。 |
| Sleep Config Change Aggregator | 機能 | `Sleep Mode` 中に `App Host` が維持する、`hot_reload_scope` ごとに `config_revision` が最大の変更と `LWW` マージ済み `changed_keys` を保持するキャッシュ。復帰時にはこの Aggregator をフラッシュして `ConfigChangeEvent` を再構成し、`Dependency-Aware Re-init Sequence` で順次再適用する。 | `spec-nue.md` Sec.7.2 で `hot_reload_scope` に紐づく LWW マージと `Audit Event` 記録の要件を追加した。 |
| Shadow Buffer | データモデル | エージェント変更を承認前に保持する一時差分領域。各差分には発生時刻・発行元・対象ファイル・`Agent Status` を含み、`Galaxy View` と `Legacy View` で列挙/レビューできる。初期リリースでは `Accept` のみを提供し、`Workspace Session` 単位と `ファイル単位` の承認粒度をサポートする。 | 承認後に本バッファへ反映し、永続化されない。 |
| Accept | 操作 | Shadow Buffer の差分をユーザーが承認し、確定反映する操作。 | 初期リリースは `Accept` のみ提供。 |
| Reject | 操作 | Shadow Buffer 上の差分をユーザーが却下し、該当変更を破棄する操作。 | `spec-nue.md` Sec.4.2.1 で `Audit Event` 確認後にエージェントへ再生成を促すフローを定義。 |
| Revert | 操作 | 過去に `Accept` した差分を取り消し、逆向きの差分を Shadow Buffer へ生成する操作。 | `spec-nue.md` Sec.4.2.1 で `result=revert` の `Audit Event` と再承認ループを規定。 |
| Approval Unit | 操作 | 変更承認の粒度（例: 一括、ファイル単位、ハンク単位）。初期リリースは `Workspace Session` 単位の一括 `Accept` と `ファイル単位 Accept` を提供し、`Partial Accept`/`Reject`/`Revert` は未実装。 | 将来的にハンク単位など細分化できる。 |
| Approval Request | UI | `MCP Router` が `requires_user_consent` の呼び出しに対して `policy_id`/`tool`/`argument`（ハッシュ）/`execution_context` をまとめて `Shadow Buffer` に提示する UI 操作で、ユーザーが `Accept` を押すことで再評価が触発される。 | `spec-nue.md` Sec.4.1.3/4.2.2 で `Shadow Buffer` との整合と `Audit Event` の `approval_state=pending` 対応を定義しており、`Command Hub`/`Smart Gutter`/`Structure Path` が該当 `focus_id` を引用してハイライトする。 |
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
| FontContext | コンポーネント | `nue-ui`（GPUI）に実装されたフォント登録レジストリで、`nue-font::text`/`emphasis`/`meta` のキーに `FontHandle` を紐づけ、描画レイヤーが一貫した字形を参照できるようにする。 | `spec-nue.md` Sec.3.4.2 でフォント埋め込みと初期化順序を定め、フォントのオーバーライドやフォールバック順序もこのコンテキスト経由で制御することを要件化。 |
| JetBrains Mono | タイポグラフィ | JetBrains が公開する SIL Open Font License 下の等幅フォントファミリ。`Regular`/`Bold`/`Italic` を Nue の標準コード文字・強調・メタ情報に割り当てる。 | `spec-nue.md` Sec.3.4.1 で各用途と `FontContext` への埋め込み要件を明記。 |
