# nue-ui ドメイン再編計画（2段階: Move-only Green → Behavior-preserving Refactor）

## 概要
`nue-ui/src/` を機能ドメイン単位へ再編し、責務分解と認知負荷低減を進める。
ただし変更リスクを下げるため、実装は次の 2 段階で行う。

- Phase 1: Move-only Green（ファイル移動とモジュール配線のみ）
- Phase 2: Behavior-preserving Refactor（振る舞い不変の小分割）

この順序により、Tidy First とトレーサビリティを維持しつつ、レビュー容易性を高める。

## 前提・既定値（確定）

- 公開パス互換は維持しない。`pub use` 互換レイヤは作らない。（破壊的変更を許容）。
- 参照側更新は同一 PR で完了させる。
- `legacy_` は「古い機能」扱いしない。命名維持で再配置のみ行う。
- 並列実行は行わず、全工程を逐次実行する。
- 開発スループットよりも、確実性とトレーサビリティを優先する。

## 現状スコープ（2026-02-24 時点）

対象のフラット配置ファイル:

- `nue-ui/src/command_hub.rs`
- `nue-ui/src/design_system.rs`
- `nue-ui/src/design_system_gpui.rs`
- `nue-ui/src/editor_events.rs`
- `nue-ui/src/editor_input.rs`
- `nue-ui/src/legacy_explorer.rs`
- `nue-ui/src/tab_bar.rs`
- `nue-ui/src/terminal.rs`
- `nue-ui/src/terminal_display.rs`
- `nue-ui/src/ui_style.rs`
- `nue-ui/src/workspace_rail.rs`
- `nue-ui/src/workspace_session_layout.rs`

備考:

- `nue-ui/src/lib.rs` は crate root のため維持する。ただし Phase 1 後は「公開モジュール配線 + アプリ起動導線」に責務を限定する。
- `nue-ui::UiLaunchRequest` / `nue_ui::run_app` は本計画では root 維持（Phase 1 で公開パスを増やさない）。

## 公開API/モジュール変更（破壊的）

- `nue_ui::command_hub::*` → `nue_ui::command::hub::*`
- `nue_ui::editor_events::*` → `nue_ui::editor::events::*`
- `nue_ui::editor_input::*` → `nue_ui::editor::input::*`
- `nue_ui::terminal::*` → `nue_ui::terminal::controller::*`
- `nue_ui::terminal_display::*` → `nue_ui::terminal::display::*`
- `nue_ui::workspace_rail::*` → `nue_ui::workspace::rail::*`
- `nue_ui::legacy_explorer::*` → `nue_ui::workspace::legacy_explorer::*`
- `nue_ui::tab_bar::*` → `nue_ui::layout::tab_bar::*`
- `nue_ui::workspace_session_layout::*` → `nue_ui::layout::workspace_session_layout::*`
- `nue_ui::design_system::*` → `nue_ui::design::system::*`
- `nue_ui::design_system_gpui::*` → `nue_ui::design::gpui::*`
- `nue_ui::ui_style::*` → `nue_ui::design::ui_style::*`

外部参照の更新対象（同一 PR）:

- `nue-app/src/main.rs`（`nue_ui::UiLaunchRequest` / `nue_ui::run_app` は据え置き想定）
- `nue-app/src/workspace_session_factory.rs`
  - `CommandHubUiController`
  - `EditorEventSubscriber`
  - `LegacyExplorerModel`
  - `TabBarUiController`
  - `TerminalUiController`

## 目標ディレクトリ構成

- `nue-ui/src/lib.rs`
- `nue-ui/src/command/`
  - `mod.rs`
  - `hub.rs`
- `nue-ui/src/editor/`
  - `mod.rs`
  - `events.rs`
  - `input.rs`
- `nue-ui/src/terminal/`
  - `mod.rs`
  - `controller.rs`
  - `display.rs`
- `nue-ui/src/workspace/`
  - `mod.rs`
  - `rail.rs`
  - `legacy_explorer.rs`
- `nue-ui/src/layout/`
  - `mod.rs`
  - `tab_bar.rs`
  - `workspace_session_layout.rs`
- `nue-ui/src/design/`
  - `mod.rs`
  - `system.rs`
  - `gpui.rs`
  - `ui_style.rs`

## 実装手順（逐次・Tidy First）

### Phase 1: Move-only Green

目的は「構造変更のみ」で常時グリーンを維持すること。ロジック改変は禁止する。

基本ルール:

- `git mv` ベースでファイルを新ディレクトリへ移動する。
- `mod.rs` / `lib.rs` / `use` / `pub mod` の配線だけを更新する。
- 関数本体・条件分岐・アルゴリズムには手を入れない。
- 移動時にファイル名は責務が明確になる範囲でのみ変更する（例: `terminal.rs` → `terminal/controller.rs`）。

推奨実施順（逐次）:

1. `design` ドメインを移動
   - `design_system.rs` → `design/system.rs`
   - `design_system_gpui.rs` → `design/gpui.rs`
   - `ui_style.rs` → `design/ui_style.rs`
   - `lib.rs` / `workspace_rail.rs` / `workspace_session_layout.rs` などの `crate::design_*` 参照を更新
2. `editor` ドメインを移動
   - `editor_events.rs` → `editor/events.rs`
   - `editor_input.rs` → `editor/input.rs`
3. `layout` ドメインを移動
   - `tab_bar.rs` → `layout/tab_bar.rs`
   - `workspace_session_layout.rs` → `layout/workspace_session_layout.rs`
4. `terminal` ドメインを移動
   - `terminal.rs` → `terminal/controller.rs`
   - `terminal_display.rs` → `terminal/display.rs`
5. `workspace` ドメインを移動
   - `workspace_rail.rs` → `workspace/rail.rs`
   - `legacy_explorer.rs` → `workspace/legacy_explorer.rs`
6. `command` ドメインを移動
   - `command_hub.rs` → `command/hub.rs`
7. `nue-app` 側の `use nue_ui::...` を新パスへ更新し、残存旧参照を除去
8. `nue-ui/src/lib.rs` の `pub mod` 宣言をドメイン配下公開へ整理

Phase 1 の禁止事項:

- 関数本体の意味変更
- 制御フロー変更
- アルゴリズム変更
- 新規機能追加
- 互換 `pub use` の追加

Phase 1 のチェックポイント:

- 各ドメイン移動後に `cargo test -p nue-ui`
- 外部参照更新後に `cargo test -p nue-app`
- Phase 1 完了時に `scripts/sanity.sh` を実行し、すべての成功を確認する
- コミットを行う
  - 変更したファイルのみ明示的に `git add` する
  - コミットメッセージは `<type>: <short title>` + 日本語説明のフォーマットに従う

### Phase 2: Behavior-preserving Refactor

目的は巨大ファイルの内部分割だが、振る舞いは不変とする。1 責務ずつ小さく実施する。

実施優先順:

1. `workspace/rail.rs`（旧 `workspace_rail.rs`）
2. `design/system.rs`（旧 `design_system.rs`）
3. `command/hub.rs`（旧 `command_hub.rs`）
4. `terminal/controller.rs`（旧 `terminal.rs`）

各対象での進め方:

- 抽出単位を 1 つ定義してから分割する（状態、変換、表示、連携のいずれか 1 つ）
- 小分割を実施
- 直後に `cargo test -p nue-ui`
- 必要に応じて既存テストを補強し、振る舞いを固定する

分割ガイド（初回候補）:

- `workspace/rail.rs`
  - `view_state`（`WorkspaceRailItemViewModel`, `WorkspaceRailViewState`）
  - `actions`（`WorkspaceRailUiAction`, `WorkspaceRailActionQueue`）
  - `connector`（`WorkspaceRailConnector`, validation message 変換）
  - `view`（`WorkspaceRailView`, `Render` 実装）
- `design/system.rs`
  - `palette_and_colors`（色定義と `ColorPalette`）
  - `overlay_layers`（`Overlay*` と z-index 解決）
  - `fonts`（`FontContext`, `BundledFont`, font catalog）
  - `theme_neon_night`（`neon_night_*` 生成関数群）
  - `style_resolvers`（`glow_spec_for`, `diff_*`, `transition_timing_for`）
- `command/hub.rs`
  - `transition`（`CommandHubUiTransition`, 通知生成）
  - `ui_state`（`CommandHubOverlayState`, `CommandHubUiState`）
  - `controller`（`CommandHubUiController` 本体）
  - `messages`（`message_for_error` などの表示文言変換）
- `terminal/controller.rs`
  - `audit_message`（`TerminalAuditMessage` と event→UI message 変換）
  - `formatters`（`summarize_audit_payload`, rejection/interruption 表示文言）
  - `controller`（`TerminalUiController` を薄く保つ）

Phase 2 の禁止事項:

- API 仕様変更（Phase 1 で決めた公開面の再変更）
- エラーハンドリング方針の変更
- 外部クレート追加
- 表示文言変更（翻訳修正含む）を構造変更と混在させること

Phase 2 完了時のチェック:

- `scripts/sanity.sh` を実行し、すべての成功を確認する
- コミットを行う
  - 変更したファイルのみ明示的に `git add` する
  - コミットメッセージは `<type>: <short title>` + 日本語説明のフォーマットに従う

## 受け入れ条件

- `nue-ui/src/lib.rs` がフラットな UI モジュール列挙ではなく、ドメイン公開面（`command` / `editor` / `terminal` / `workspace` / `layout` / `design`）を持つこと
- `nue-ui` / `nue-app` 内で旧フラット公開パス（例: `nue_ui::command_hub`, `nue_ui::workspace_rail`）の参照が残っていないこと
- `legacy_explorer` の命名が維持されたまま `workspace` ドメインへ再配置されていること
- Phase 1 / Phase 2 の各コミットが「構造変更」と「振る舞い変更」を混在させていないこと
- `cargo test` と `scripts/sanity.sh` が成功すること

## リスクと緩和策

- リスク: モジュール移動時の `crate::` / `super::` 参照修正漏れ
  - 緩和策: ドメイン単位で移動し、その都度 `cargo test -p nue-ui`
- リスク: `nue-app` 側の公開パス変更追従漏れ
  - 緩和策: Phase 1 完了条件に `rg -n "nue_ui::(command_hub|editor_events|editor_input|terminal|terminal_display|workspace_rail|legacy_explorer|tab_bar|workspace_session_layout|design_system|design_system_gpui|ui_style)" .` を含める
- リスク: Phase 2 で構造変更と文言/仕様変更が混ざる
  - 緩和策: 1 PR / 1 責務抽出 / 直後テストの運用を徹底する
