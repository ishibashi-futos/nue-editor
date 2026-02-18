# nue-core ドメイン再編計画（2段階: Move-only Green → Behavior-preserving Refactor）

## 概要
`nue-core/src/` を機能ドメイン単位へ再編し、責務分解と認知負荷低減を進める。
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

## 公開API/モジュール変更（破壊的）

- `nue_core::command_hub` / `nue_core::command_hub_actions` → `nue_core::command::*`
- `nue_core::editor_core` / `markdown_service` / `minimap_service` / `smart_gutter_service` / `structure_path` / `focus_layer` → `nue_core::editor::*`
- `nue_core::search_service` / `search_navigator` → `nue_core::search::*`
- `nue_core::pane_manager` / `pane_history` / `tab_manager` / `session_state` → `nue_core::layout::*`
- `nue_core::terminal_session` / `terminal_scrollback` → `nue_core::terminal::*`
- `nue_core::workspace_rail` / `workspace_registry` / `repository_service` / `git_status` / `commit_log` / `legacy_file_tree` / `legacy_workspace_editor` → `nue_core::workspace::*`
- `nue_core::path_display` → `nue_core::shared::path_display`

## 目標ディレクトリ構成
- `lib.rs`
- `command/`（`mod.rs`, `parser.rs`, `actions.rs`, `state.rs`）
- `editor/`（`mod.rs`, `core.rs`, `markdown.rs`, `minimap.rs`, `smart_gutter.rs`, `focus.rs`, `structure_path.rs`）
- `search/`（`mod.rs`, `service.rs`, `navigator.rs`）
- `layout/`（`mod.rs`, `pane_manager.rs`, `pane_history.rs`, `tab_manager.rs`, `session_state.rs`）
- `terminal/`（`mod.rs`, `session.rs`, `scrollback.rs`）
- `workspace/`（`mod.rs`, `rail.rs`, `registry.rs`, `repository.rs`, `git_status.rs`, `commit_log.rs`, `legacy_file_tree.rs`, `legacy_workspace_editor.rs`）
- `shared/`（`mod.rs`, `path_display.rs`）

## 実装手順（逐次・Tidy First）

### Phase 1: Move-only Green
目的は「構造変更のみ」で常時グリーンを維持すること。ロジック改変は禁止する。

- `git mv` ベースでファイルを新ディレクトリへ移動する。
- `mod.rs` / `lib.rs` / `use` / `pub mod` の配線だけを更新する。
- `legacy_` は命名を維持し、`workspace/` 配下へ再配置する。
- `nue-ui` 側の `use nue_core::...` を新パスへ置換する。
  - nue-ui/src/editor_input.rs
  - nue-ui/src/command_hub.rs
  - nue-ui/src/terminal_display.rs
  - nue-ui/src/tab_bar.rs
  - nue-ui/src/workspace_rail.rs
  - nue-ui/src/legacy_explorer.rs
  - nue-ui/src/design_system.rs
  - nue-ui/src/editor_events.rs
  - nue-ui/src/terminal.rs
  - nue-ui/src/ui_style.rs

Phase 1 の禁止事項:
- 関数本体の意味変更
- 制御フロー変更
- アルゴリズム変更
- 新規機能追加

Phase 1 のチェックポイント:
- ドメイン単位（`command` / `editor` / `layout` / `terminal` / `workspace` / `search` / `shared`）で移動後に `cargo test`
- `scripts/sanity.sh` を実行し、すべての成功を確認する。
- コミットを行う
  - 変更したファイルのみ明示的に `git add` する。
  - コミットメッセージは `<type>: <short title>` + 日本語説明のフォーマットに従う。

### Phase 2: Behavior-preserving Refactor
目的は巨大ファイル分割だが、振る舞いは不変とする。1 責務ずつ小さく実施する。

実施順:
1. `editor/core.rs`（旧 `editor_core.rs`）
2. `command/actions.rs`（旧 `command_hub_actions.rs`）
3. `terminal/session.rs`（旧 `terminal_session.rs`）

各対象での進め方:
- 抽出単位を 1 つ定義（例: 型群、イベント変換、パース補助）
- 小分割を実施
- 直後に `cargo test`
- 必要に応じてスナップショット/既存単体テストを補強（振る舞い固定のため）

Phase 2 の禁止事項:
- API 仕様変更
- エラーハンドリング方針の変更
- 外部クレート追加

Phase 2 完了時のチェック:
- `scripts/sanity.sh` を実行し、すべての成功を確認する。
- コミットを行う
  - 変更したファイルのみ明示的に `git add` する。
  - コミットメッセージは `<type>: <short title>` + 日本語説明のフォーマットに従う。

## 受け入れ条件

- `nue-ui` の import が旧パスを参照していないこと。
- `nue-core/src/lib.rs` が新ドメイン公開面のみを持つこと。
- 主要機能（Command Hub/Editor/Workspace/Terminal/Search）の既存テストがすべて通過すること。
