# Backlog

## Resolved

- [x] P2: `Galaxy View` の成立条件を定義する
  - Before: レイアウト・更新頻度・性能上限・アクセシビリティ基準が未定義。
  - After: `v1.0` 以降での導入タイミング・500 ノード時の 45fps 性能 budget・アクセシビリティ要件などを `spec-nue.md` に記載した。

- [x] P2: `Audit Event` のライフサイクル要件を定義する
  - Before: 初期メモリ保持と `v1.0` 以降の `*.jsonl` 保存方針はあるが、保持期間・匿名化・オフライン再送が未定義。
  - After: 保持期間・匿名化ルール・永続化パス・キュー再送制御（`audit.*` 設定）を `spec-nue.md` Sec.4.4 に RFC 2119 で規定。

- [x] P2: `Configuration` の再評価/ホットリロード要件を定義する
  - Before: 設定のマージ優先度は記述されたが、環境変数・ワークスペース・グローバル設定の変更をどのタイミングで各コンポーネントが再評価するか未定義。
  - After: `ConfigChangeEvent` による再評価サイクル、ホットリロード可能/不可キーの扱い、再起動要求時の監査と `audit.queue.max` 扱いを `spec-nue.md` Sec.5 で定義した。

- [x] P1: `MCP Router` の認可モデルを定義する
  - Before: `MCP Tool` 実行可否と承認粒度が未定義で、セキュリティ境界が曖昧であった。
  - After: `spec-nue.md` Sec.4.1 に Authorization Policy エントリ構造（`policy_id`/`argument_constraints`/`execution_context`/`approval_state`/`effect`/`priority`）、引数ハッシュマッチ、`deny` 優先の評価順、`requires_user_consent`/`auto_allow`/`blocked` の承認フロー、`Audit Event` 連携を RFC 2119 スタイルで記載した。

- [x] P1: Command Hub/Intent候補フローを定義する
  - Before: `Command Hub`/Intent候補のモード、ローカルAI、実行計画との連携が未定義で、MCP Router および Shadow Buffer との整合性が曖昧であった。
  - After: `spec-nue.md` Sec.6.1 でコマンドパレットのモード・候補更新要件、`nue-semantic`（Intent Resolver/Local RAG/Policy-Aware Scoring）の構成と `Audit Event`/Shadow Buffer との連携を RFC 2119 形式で明記した。

- [x] P1: MCPプロバイダの「ドメイン境界」とライフサイクル
  - Before: エージェントの `run_command` 等が workspace の境界を越えて実行される可能性と、複数 `run_command` の同時起動による衝突時のキューイングが未定義で、CI/セキュリティ境界が不明瞭であった。
  - After: `spec-nue.md` Sec.4.5 で `workspace_root` に基づく `execution_context` の強制、FSツールの Canonical 化、`ToolExecutionState`/`ToolRequestQueue` による `run_command` のライフサイクルとキュー上限、逸脱時の `Audit Event` 記録を RFC 2119 で明記した。

- [x] P1: エージェントの「認可エラー」時のリカバリフロー
  - Before: 認可拒否（deny）された際、エージェントがループ（再試行の繰り返し）に陥るのを防ぐ仕様が必要で、ユーザーに次の対応を示す指示が未定義であった。
  - After: `spec-nue.md` Sec.4.1.4 に `resolution_hint` を含む `Audit Event`、`DeniedRequestHistory`/`retry_delay_seconds` による再試行抑制、`Feedback Loop` バナーと `ConfigChangeEvent` 連携、`blocked` ポリシーへの 24 時間抑制を RFC 2119 で規定した。

- [x] P1: `ConfigChangeEvent` の `hot_reload_scope` の列挙値と依存順序
  - Before: `hot_reload_scope` の列挙値と複数スコープにまたがる再初期化順序が未定義で、依存関係を考慮した再評価エラー時の挙動が不明確であった。
  - After: `spec-nue.md` Sec.5.3.1 に許容列挙値（`app`/`router`/`terminal`/`editor`/`semantic`/`agent`）と「Dependency-Aware Re-init Sequence」順序を RFC 2119 で明記し、未知 scope を `hot_reloadable=false` で再起動要求、`Audit Event` に `config.reload.unknown_scope` を記録するフェールセーフを追加した。

- [x] P1: デザイン・カラーの具体化
  - Before: カラーパレット、エージェント状態・差分表現ともに未定義で、アクセシビリティや承認結果の視覚化方針が明確でなかった。
  - After: `spec-nue.md` Sec.3.1.1/3.1.2 に WCAG AA 準拠のパレット（Deep Abyss ～ Dusty Grey）と `Agent Status` の状態別エフェクト、差分/承認インジケーターの色規約を RFC 2119 スタイルで記載し、`nue-ui` GPUI への反映を要求した。

- [x] P1: Command Hubの候補更新遅延と外部エージェント提案の挙動
  - Before: 16ms を超える候補更新遅延時の UI 表示と `Action/Navigation Mode` への案内、`nue-semantic` の遅延中の再スコアリング状態、外部エージェント提案の制御ポリシーが未定義であった。
  - After: `spec-nue.md` Sec.6.1.1 で `Backoff` 状態が遅延中の候補を保持し進捗ラベルを表示すること、`Action Mode`/`Navigation Mode` への移行ヒントを明示し、`nue-semantic` の再スコアリング中も UI が状態を伝えることを RFC 2119 で定義した。Sec.6.1.1-6.1.2 では外部エージェント提案を `requires_user_consent` + `Audit Event` で限定プロファイル（例: `external_agent_profile=cloud_lambda_v2`）のみに許可し、候補が存在しない場合に「現在のコンテキストでは解決不能」レスポンスを返すと記述した。

## ToDo

- [ ] P1: 使用可能フォントの具体化と埋め込み
  - OSSフォントを使用可能フォントに設定、バイナリに埋め込み
    - 標準コード / UI,JetBrainsMono-Regular.ttf,高い可読性とリガチャー。
    - キーワード / 強調,JetBrainsMono-Bold.ttf,ネオンカラーと組み合わせた際の見栄え。
    - メタ情報 / コメント,JetBrainsMono-Italic.ttf,コメントやMuted Textの区別。
  - UIの初期化プロセス内で埋め込んだバイナリを `FontContext` に登録


- [ ] P1: `Shadow Buffer` の承認モデルを定義する
  - Before: `Accept` 以外（Reject/Revert/Partial Accept）が未定義で、運用手順が確立できない。
  - After: 承認操作セット、`Approval Unit`、競合時挙動を定義し、UI 操作と Core 反映ルールを一致させる。

- [ ] P1: `MCP Router` ポリシー衝突時の優先順位を定義する
  - Before: 認可粒度（ツール/引数/実行コンテキスト）は確定したが、`allow` と `deny` の衝突解決規則が未定義。
  - After: 衝突解決規則（例: `deny` 優先）と評価順序を RFC 2119 で規定し、拒否時の `Audit Event` 記録要件を固定する。

- [ ] P1: 初期リリースの `Approval Unit` を確定する
  - Before: 初期リリースは `Accept` のみと決定済みだが、承認粒度（一括固定/ファイル単位）が未定義。
  - After: 初期 `Approval Unit` を `一括` に固定し、将来のハンク単位承認への拡張条件を仕様化する。
