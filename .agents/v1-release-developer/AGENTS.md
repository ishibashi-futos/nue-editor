以下は、`nue-editor` の「ローカル起動到達」マイルストーンを進めるための作業コンテキストです。
共通ルール・開発手順・各種制約は `AGENTS.md` を参照し、このプロンプトでは `specs/plans/v1-release-plan.md`（計画/設計決定）と `specs/plans/v1-release-backlog.md`（Layer3タスク）に基づくマイルストーン固有の前提のみを補足します。

## このマイルストーンの目的

- `nue-app` を `Hello, world!` から、実際に起動できる統合アプリ入口へ置き換える
- `macOS` 上で GPUI ウィンドウを起動し、基本画面と基本操作を確認できる状態にする
- 既存の `nue-core` / `nue-ui` のロジック資産を活かして、エントリーポイントと画面遷移の繋ぎこみを成立させる

## 到達ゴール（DoDの中心）

- `cargo run -p nue-app -- --workspace <ABSOLUTE_PATH>` で起動できる
- 空引数起動でもクラッシュせず、ワークスペース未選択画面（プレースホルダ可）が表示される
- 最低限のワークスペース画面（Rail / 左パネル / エディタ / タブ / Command Hub / ターミナル領域）が表示される
- 以下の基本操作が成立する
  - Explorer からファイル選択
  - ファイル表示・編集・保存
  - Command Hub の開閉と最低限のコマンド実行
  - パネル切替（Explorer / Search / VCS、Search/VCS はプレースホルダ可）
  - ターミナル `run_command` 実行と状態表示

## 現状の前提（重要）

- `nue-app` は現状 `Hello, world!` のみで、統合初期化未実装
- `nue-ui` にはコントローラ群はあるが、GPUI のウィンドウ/ルートビュー/イベントループ接続が未実装
- `nue-config`, `nue-mcp`, `nue-semantic` は起動に必要な最低限以外は未整備でよい
- `nue-core` / `nue-ui` には再利用可能な既存資産が多い。新規実装より先に接続を優先する

## 再利用を優先すべき既存資産

- `nue-core`
  - `LegacyWorkspaceEditor`
  - `EditorCore`
  - `WorkspaceRegistry`
  - `PaneManager`, `TabManager`, `SessionState`
  - `CommandHubActionModel`, `CommandHubStateSink`
  - `TerminalSession`
- `nue-ui`
  - `CommandHubUiController`
  - `LegacyExplorerModel`
  - `EditorInputController`
  - `EditorEventSubscriber`
  - `TerminalUiController`, `TerminalScrollbackRenderer`
  - `TabBarUiController`
  - `DesignSystem`, `UiStyleGuide`, フォント資産

## スコープ内 / スコープ外

### スコープ内（必須）

- `nue-app` の実エントリーポイント実装
- GPUI ウィンドウ起動とルート画面表示（`macOS`）
- ワークスペース引数 `--workspace` からのセッション起動
- Explorer → Editor の導線、編集・保存
- Command Hub 開閉と最低限の画面遷移反映
- ターミナル `run_command` の実コマンド実行（PTY）と結果表示

### スコープ外 / 暫定許容

- `nue-mcp`, `nue-semantic` の本実装（no-op/stub で可）
- `nue-config` の本格機能（設定ファイル/環境変数マージ/ホットリロード）
- 空引数起動時のワークスペース追加・選択 UI 導線（未選択画面表示のみ必須）
- Search/VCS パネルの詳細 UI（プレースホルダ可）
- Minimap/SmartGutter/StructurePath の高度描画（状態表示・件数表示で可）
- エディタ外でのファイル変更検知と自動リロード

## 設計決定（作業中に崩さない前提）

- `LegacyWorkspaceEditor` に必要最小限の委譲 API を追加し、`EditorCore` 生 `&mut` をアプリ層へ公開しない
- アプリ層で別 `EditorCore` を持たない（状態の二重化を避ける）
- GPUI 実 API への適合を優先し、先に「まず繋ぐ」。分割の整備は後追いでよい
- `tokio` は本マイルストーンで導入し、UI スレッドとは分離する
- `GPUI + tokio` 共存の統合スパイク（ダミー非同期イベント反映）は、後続機能前の前提ゲートとして扱う
- 受け入れ対象 OS は `macOS` 限定
- `DesignSystem` フォントは `include_bytes!` による埋め込み登録を採用する
- ターミナル実装は `run_command` 到達を優先し、`alacritty_terminal` の利用を許容する（本マイルストーンの設計決定）

## 実装順の指針（高レベル）

1. 起動基盤の置換（`nue-app` 入口、引数、UI起動、エラー処理）
2. GPUI ルート起動とプレースホルダ画面
3. `GPUI + tokio` 統合スパイク（非同期イベント反映と終了順確認）
4. アプリ統合状態 / ワークスペースセッション状態の定義
5. `WorkspaceRegistry` とセッション生成の接続
6. ワークスペース画面レイアウト（Rail / 左パネル / Editor / Terminal / Overlay）
7. Explorer → Editor → 編集/保存 の導線
8. `EditorCore` イベント drain → UI 反映ループ
9. Command Hub / パネル / タブ / ペインの画面遷移接続
10. ターミナル UI と PTY 実コマンド実行接続
11. 起動スクリプト・手動スモーク手順・検証記録の固定

## タスク実行時のトレーサビリティ方針（plan/backlog 準拠）

- 作業対象は `specs/plans/v1-release-backlog.md` の `T-001`〜`T-037` を単位として扱う
- 着手時に、対象タスクIDと対応する受け入れ条件を明示してから実装する
- 実装で前提変更が必要になった場合は、コード先行で曖昧に進めず、`specs/plans/v1-release-plan.md` の設計決定と `specs/plans/v1-release-backlog.md` のタスク依存関係への影響を明文化する
- 既存資産を使い回せる場合は新規抽象化より接続を優先し、必要になった箇所だけ後で整える

## 主要な難所（先回りで注意）

- GPUI 0.2.2 実 API に合わせた初期化・ウィンドウ生成
- `macOS` の UI スレッド制約と `tokio` ランタイムの共存
- 統合状態での所有権整理（複数コントローラ + UI更新）
- `LegacyWorkspaceEditor` の委譲 API 不足による二重状態化リスク
- PTY 実行の終了処理・出力取り込み・状態遷移反映

## 受け入れ時に確認する観点（実装者向けチェック）

- 起動:
  - `cargo run -p nue-app -- --workspace <ABSOLUTE_PATH>` でセッション画面まで到達できる
  - 空引数起動で未選択画面が表示される
- 編集:
  - Explorer 選択でファイルが開く
  - 編集して保存するとローカルファイルが更新される
- 遷移:
  - Command Hub 開閉、`panel focus` が動く
  - タブ/ペイン操作の最低限が UI に反映される
- ターミナル:
  - `run_command` の状態遷移（Queued/Running/Completed/Failed）が確認できる
  - 出力がスクロールバックに反映される

## 依頼の受け方（AI Agentの行動指針）

- ユーザーから曖昧な依頼が来たら、まず `specs/plans/v1-release-backlog.md` のどのタスクID群に相当するかを特定する
- 実装案は「新規設計」より「既存の `nue-core` / `nue-ui` をどう接続するか」で説明する
- マイルストーン外の要求（例: MCP/Semantic本実装、Windows対応、高度描画）は、スコープ外として切り分けたうえで最小の代替案を提示する
- DoD に直結しない複雑化（過剰な互換対応、先回り抽象化）は避ける

## 参照元

- `specs/plans/v1-release-plan.md`（2026-02-24 時点のローカル起動到達プラン、Layer1/2/設計決定）
- `specs/plans/v1-release-backlog.md`（Layer3 タスク単位の粒度、進捗チェック用）
