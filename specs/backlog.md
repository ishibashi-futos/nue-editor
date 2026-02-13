# Backlog

## Resolved

- [x] P1: `nue-semantic` のインスタンス管理を明文化
  - Before: `nue-semantic` のシングルトン運用が ToDo で残っており、実装規約やセッション分離の方法が未定義だった。
  - After: `spec-nue.md` Sec.6.1.3 で `App Host` が共有インスタンスを起動し、`SemanticContextManager`/`SemanticContextHandle` を経由してワークスペース単位の `Local RAG` と `Relevance Intent` を隔離することを RFC 2119 で記載し、`SessionSnapshot` への `semantic_context_state` 登録も要求した。

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

- [x] P1: `MCP Router` ポリシー衝突時の優先順位を定義する
  - Before: 認可粒度（ツール/引数/実行コンテキスト）は確定したが、`allow` と `deny` の衝突解決規則が未定義で、評価順序・`Audit Event` への記録粒度が不明瞭であった。
  - After: `spec-nue.md` Sec.4.1.2 にポリシー評価パイプライン（`domain`/`tool` → `execution_context` → `argument_constraints` → `effect`）と `deny`/`allow` の優先付け、`priority`/`policy_revision` による選定順序、および `Audit Event` で必要なトレーサビリティフィールドを RFC 2119 で規定した。

- [x] P1: 初期リリースの `Approval Unit` を確定する
  - Before: 初期リリースは `Accept` のみと決定済みだが、承認粒度（一括固定/ファイル単位）が未定義で、`Shadow Buffer` 操作との整合も未記述であった。
  - After: `spec-nue.md` Sec.4.2 において、ワークスペース単位とファイル単位の `Accept` を `MUST` とし、`Partial Accept` によるハンク単位の拡張や `Reject`/`Revert` の監査要件は現行 ToDo として整理した。

- [x] P1: `Shadow Buffer` の承認モデルを定義する
  - Before: `Accept` 以外（Reject/Revert/Partial Accept）が未定義で、運用手順が確立できない。
  - After: `spec-nue.md` Sec.4.2.1/4.2.2 で `Shadow Buffer` が提供すべき `Accept`/`Reject`/`Partial Accept`/`Revert` の承認操作、それぞれの `Approval Unit`・`Audit Event` フィールド・Editor/UI との整合ルールを RFC 2119 形式で定義し、差分の分割・拒否・逆方向承認の更新手順と通知要件を固めた。

- [x] P1: Global search / Navigation integration
  - Before: 全ワークスペース横断検索と Legacy/Command Hub からの遷移ルート、`Shadow Buffer` との同期が未定義だった。
  - After: `spec-nue.md` Sec.6.2.1 で Global Search パネルの呼び出し経路、フィルター/スコープ、検索結果の Legacy View/Smart Gutter/Structure Path との同期、`Audit Event` 記録要件を RFC 2119 で規定した。

- [x] P1: Semantic Search Integration
  - Before: 意味ベース検索の表示・スコアリング、`Command Hub` との連携、`Local RAG` インデックス更新、外部エージェント提案の制御が曖昧だった。
  - After: `spec-nue.md` Sec.6.2.2 で Semantic Search を Global Search UI と Command Hub に統合する流れ、`semantic_score` 表示、Smart Gutter/Structure Path/Command Hub へ `Relevance Intent` を伝播するルール、`Local RAG` の 5 秒更新と再スコアリング中ラベル、`requires_user_consent` の外部エージェント提案について RFC 2119 で規定した。

- [x] P1: 使用可能フォントの具体化と埋め込み
  - Before: UI で利用する等幅フォント・強調フォント・メタフォントやそれらの埋め込み手順が未定義で、マルチプラットフォームでの字形の一貫性が確保できなかった。
  - After: `spec-nue.md` Sec.3.4 に JetBrainsMono ファミリ（Regular/Bold/Italic）の用途割当を RFC 2119 で定義し、`nue-ui` の `FontContext` へのバイナリ埋め込み・初期化順序・フォールバック・オーバーライド要件を明記した。
- [x] P2: Minimap の可視化/連携仕様
  - Before: エディタ右側に配置される高解像度プレビューの要件や `Shadow Buffer`/`Command Hub`/`Audit Event` との同期方法が曖昧で、CI 出力や差分の可視化方針が未定義。
  - After: `spec-nue.md` Sec.3.5 に Minimap の 60fps 表示要件、エラー/検索/差分オーバーレイ、`Shadow Buffer` との `focus_id` 連携、クリック操作の `Command Hub` への橋渡しおよび `Audit Event` 記録について RFC 2119 で定義した。

- [x] P2: Structure Path 連携
  - Before: エディタ上部に階層パスを表示する構造/動作要件が未定義で、Command Hub・Shadow Buffer・Legacy View との同期と操作仕様が曖昧であった。
  - After: `spec-nue.md` Sec.3.6 に Structure Path の表示トリガー、差分/承認マーカー、Shadow Buffer/Command Hub/Minimap との同期ルール、相互作用・アクセシビリティ要件を RFC 2119 で文書化した。

- [x] P2: Smart Gutter 表示
  - Before: 行番号領域の差分表示や AI 活動インジケーター、`Audit Event` 連携の UI 側要件が未定義で、Command Hub/Minimap との同期方法も未整備。
  - After: `spec-nue.md` Sec.3.7 に Smart Gutter の `AI Pulse Indicator`/`Git Delta` 表示、`Audit Event` の `result`・`resolution_hint` 連携、`Glyph` によるアクセシビリティ表現、Command Hub/Minimap/Structure Path へのフォーカス同期を RFC 2119 で定義した。

- [x] P1: タブ・レイアウト管理
  - Before: ワークスペースを切り替えた際、それぞれのタブの開き具合やスクロール位置などを完全に復元する機構が存在せず、作業継続性が確保できていなかった。
  - After: `spec-nue.md` Sec.7.1 に `SessionSnapshot` による開いているタブ/スプリット/ビュー表示/Shadow Buffer 差分/ツールキューの記録・復元を定義し、切り替え後の元の状態への復帰要件を RFC 2119 で定義した。

- [x] P1: Sleep 機能
  - Before: 非アクティブなワークスペースを休止させる仕組みがなく、メモリとエージェントリソースの浪費を招いていた。
  - After: `spec-nue.md` Sec.7.2 で Sleep モードのトリガー、スナップショット保存・復元、`MCP Router`/`Editor Core` の停止と再開、`ConfigChangeEvent` の保留と `Audit Event` 記録を RFC 2119 で規定した。


## ToDo

- [ ] P2: `focus_id` の生成/伝播ルールを確定する
  - Before: `Minimap`/`Structure Path`/`Smart Gutter`/`SessionSnapshot` が `focus_id` を参照するが、粒度や一意性・更新タイミングが未定義で UI 整合の担保ができない。
  - After: `specs/spec-nue.md` Sec.3.5.1/3.6/3.7/7.1 に `focus_id` の定義を追加し、`specs/ask.md` Q25 の回答にもとづいて参照実装が共有できる。

- [ ] P2: Sleep 中の `ConfigChangeEvent` 保留の振る舞いを仕様化する
  - Before: `Sleep Mode` では一括保留しているが、蓄積されたイベントのキュー順序・上限・復帰時の適用順序が未定義で再起動後の状態差異が生じる可能性がある。
  - After: `specs/spec-nue.md` Sec.7.2 に保留キューの耐性・適用順序・破棄条件を明記し、`specs/ask.md` Q26 の決定に従って `App Host` の再評価フローを統一する。

- [ ] P2: Intent 候補からの `Approval Unit` 選択ポリシーを確定する
  - Before: 自然言語モードで複数ファイル/ハンクを含む候補が生成される場合の `Approval Unit` や `Audit Event` への紐づけ、`Shadow Buffer` への差分記録粒度が未定義で整合性が取れていない。
  - After: `specs/spec-nue.md` Sec.6.1.1/6.2.2 に `Approval Unit` ポリシーを追加し、`specs/ask.md` Q27 の解に基づいて UI と `Audit Event` の整合性が保証される。
