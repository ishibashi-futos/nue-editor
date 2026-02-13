# Nue 機能一覧

## アーキテクチャとワークスペース管理
- App Host が全セッションのライフサイクルとグローバル設定・通知を統括し、左端レイルで複数ワークスペースを切り替える「Workspace Switcher」を提供。
- 各ワークスペースは独立した Workspace Session を持ち、専用の AI Agent、MCP Router、Editor Core、Legacy/Galaxy UI View を自律的に生成。
- Workspace Session ごとに canonical な workspace_root を設定し、エージェントとユーザー操作はその範囲内でのファイル操作とコマンド実行に限定。

## UI/UX とビジュアル表現
- 深いネイビー基調の「Neon-Night」テーマと指定パレット（Neon Cyan、Cyber Magenta、Solar Flare など）で状態を色とアニメーションで直感的に伝える。
- Workspace Rail 上のアイコンにエージェント状態（Busy/Waiting/Error/Idle）のライブステータス表示を付与し、状態ごとの色・エフェクトで視認性を高める。
- エクスプローラーには従来の Legacy Tree View と依存関係可視化を行う Galaxy View を用意。Galaxy View は 500 個以内のノードに Focus Score による描画優先度を適用し、Nebula 集約や外部依存の除外でパフォーマンスを維持。
- Galaxy View のノード/エッジは Agent Status や差分承認状態ごとに輝度・色・アニメーションを変化させ、補助テキストやラベルでアクセシビリティ要件（WCAG AA）を満たす。

## MCP（Model Context Protocol）と認可
- エージェントは apply_patch/read_file/get_status/commit/run_command/focus_file などの MCP Tool のみで操作し、MCP Router がドメインごとの認可を仲介。
- Authorization Policy は domain/tool/argument_constraints/execution_context/approval_state/effect を持つ明示的なホワイトリスト運用で、deny を最優先とする衝突解決。
- requires_user_consent の呼び出しは Shadow Buffer 差分を Approval Unit として UI に提示し、ユーザー承認後に処理を再評価。自動承認・ブロック・待機中の retry backoff などのフィードバックループを生成。
- 連続拒否には指数バックオフと UI での Backoff 表示・Banner 通知を行い、Blocked ポリシーは 24 時間再試行禁止。

## Shadow Buffer と差分承認ワークフロー
- エージェント編集は Shadow Buffer に差分として蓄積され、Accept（Workspace/File/Hunk）、Partial Accept、Reject、Revert の操作でユーザーが人間の判断で本バッファへ反映・破棄。
- Accept は Electric Lime マーカーで完了表示し、Partial Accept はチェックボックスやドラッグで細分化。Reject はエージェント再生成を促すメッセージを返し、Revert は Solar Flare でマークして取り消し予定を可視化。
- すべての承認操作は Audit Event に result/approval_unit/agent_id などのトレーサブルなメタ情報を含め、Galaxy Feedback で彗星ハイライトとして UI と同期。
- Partial Accept や Revert に伴って差分の行番号や focus_id を更新し、Galaxy View 上で対象ノードを再ハイライト。

## ターミナルと監査（Audit）
- 共有 PTY ベースのターミナルエミュレーターはテストやビルドの出力を最大 500 行のスクロールバック付きで表示し、全角/Unicode を正しく整列。
- MCP Router は ToolExecutionState と ToolRequestQueue で run_command の同時実行を制御し、Queue Overflow/Queued/Running 状態を Audit Event と UI で通知。
- すべての MCP 呼び出し・差分承認・ツール状態の変化を Audit Event として記録し、逐一タイムスタンプ・Workspace Session 情報・Agent Status・判定結果を保持。
- Audit Event は優先的にメモリ保持しつつ v1.0 以降は ~/.config/nue/audit.jsonl へ追記。キュー上限超過時は oldest drop で event_id を通知し、匿名化/再送/フェールオーバーも管理。
- Legacy/Galaxy の監査パネルから Audit Event をセッション・ステータス・Approval Unit でフィルタ閲覧可能。

## 設定管理とホットリロード
- 環境変数 > ワークスペース config > グローバル config > デフォルトの優先順位で設定をマージし、変更はファイル監視で 5 秒以内に再評価。
- ConfigChangeEvent は source/revision/changed_keys/previous_values/hot_reloadable を含み、hot_reload_scope（app/router/terminal/editor/semantic/agent）に応じた依存順で再初期化。
- hot_reloadable=false のキーや未知 scope は再起動を要求する通知と Audit Event を送出し、再起動後に config_revision を更新。
- 設定変更のフェールは Audit Event（config.reload.failure）として記録し、audit.queue.max を超える連続失敗時は最古の ConfigChangeEvent を削除して通知。

## AI 共創ループと Command Hub
- ユーザーの「意図入力」→ Codex 等のエージェント起動→ MCP Tool で差分生成→ Shadow Buffer→ Galaxy Feedback→ ユーザー承認の循環（Nue Loop）。
- Command Hub は Cmd/Ctrl+Shift+P で開くモーダルで、Action/Navigation/Intent モード（`>`/`:`/プレフィックスなし）を持ち、遅延時は Backoff 表示やモード移行ヒントを出す。
- 各候補に発行元・必要ツール・Approval State を付与し、選択時に即座に Audit Event を生成。Intent モードは自然言語入力を nue-semantic で解釈し、Policy-Aware Scoring で実行可能な候補を上位に。
- nue-semantic は Intent Resolver、Local RAG、Policy-Aware Scoring を統合し、100ms 以内で候補を生成。遅延時は UI で進捗表示し、解決不能な場合は外部エージェントの提案（requires_user_consent）を行う。
- Local RAG はファイル変更を 5 秒以内に反映し、類似語や曖昧な入力にも対応。選択した候補は MCP Router への実行プランを返し、Shadow Buffer と整合する差分を生成。
