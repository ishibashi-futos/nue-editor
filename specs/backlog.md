# v1 Backlog

## 0 基盤的な仕組みの整備
- [x] B-1: ワークスペースのメタデータと `WorkspaceRail` 状態 (Busy/Waiting/Error/Idle) を保持する状態遷移モデルを `nue-core` 側で定義し、UI ステータスと双方向に同期できるようにする。
- [x] B-2: ペイン・タブ・Command Hub・ターミナルなどを横断するフォーカスと `focus_id` の統一的な管理レイヤーを用意し、ステータス同期処理とイベントキューを通じて 50ms 以内反映を目指せるようにする。
- [x] B-3: `Command Hub` の構文解析器と Picker UI の基礎 (Domain/Verb/Target の正規化、一覧表示/キャンセルパターン、確認フロー) を用意し、Action/Navigation 両モードで共通利用できるようにする。
- [x] B-4: デザイントークン (Neon-Night-Glass カラー/Glass マテリアル) を `nue-ui` で定義し、カラーパレット・フォントコンテキスト・アニメーション規約を `DesignSystem` コンポーネントが参照できるようにする。
- [x] B-5: Terminal の `run_command` キュー/ステータス/通知を管理する `TerminalSession` モデルと、UI 側が `Queued/Running/Completed/Failed` を視覚化するためのイベント API を整備する。
- [x] B-6: Markdown/Editor の基本バッファ・UndoRedo・保存フロー・ショートカット登録を担う `EditorCore` モジュール (ファイル開く、カーソル保持、`Cmd/Ctrl + S` 等) を定義し、上位機能での Hook を取り付けやすくする。

## 1 Workspaces
- [x] WS-1: `+` ボタンからワークスペース登録ダイアログを開く UI を実装し、パス入力→検証→追加のフローを `WorkspaceRail` に反映できるようにする。
- [x] WS-2: `WorkspaceRail` 上のワークスペース項目を右クリックすると除外コンテキストメニューが出る仕組みを構築し、除外後に表示更新を行う。
- [x] WS-3: `WorkspaceRail` で Busy/Waiting/Error/Idle ステータスを色+アニメーションで表現し、基盤サーバ状態と連携してライブ更新できるようにする。

## 2 Design System
- [x] DS-1: Neon-Night-Glass カラーパレットと `FontContext` (JetBrainsMono の Regular/Bold/Italic 埋め込み) を `nue-ui` で登録する。
- [x] DS-2: Glass 質感（Backdrop Blur / Fine Grain / Specular Edge）の共通スタイルを `Command Hub` や主要パネルのコンポーネントに適用し、再利用可能な CSS/StyleObject を用意する。
- [x] DS-3: Agent Status（Busy/Waiting/Error/Idle）のカラーとアニメーション規約 (遷移タイミング・Glow) を定義し、`Command Hub`/`Structure Path`/`Workspace Rail` などで参照するスタイル定義を整備する。
- [x] DS-4: 未承認/承認済み差分用の Solar Flare / Electric Lime / Cloud White を `SmartGutter`/Approval UI/Gutter のスタイルにマッピングする。
- [x] DS-5: UI オーバーレイのスタッキング規約 (`Command Hub` > `Smart Gutter` > `Decoration` > `Minimap`) を `focus_id` と連動させた z-index マネージャーで管理する。

## 3 Editor
- [x] ED-1: Legacy View ファイルツリーコンポーネントを実装し、ワークスペース内のファイル一覧をツリー構造でレンダリングする。
- [x] ED-2: ファイル選択時に `EditorCore` でバッファを開き、表示・編集・保存・Undo/Redo など基本操作を可能にする。
- [x] ED-3: ショートカット収集と `Cmd/Ctrl + S/Z/P/F/Shift+F` 等の登録機構を整備し、各ショートカットで `EditorCore` のコマンドを実行できるようにする。
- [x] ED-4: 右クリックメニューの基盤を作り、`Editor Core` API を呼ぶメニューアイテム (保存/コピー/Markdown メニュー) を追加できるようにする。
- [x] ED-5: Markdown リッチ機能（構文強調・リスト補完・ペア補完・リンク開く・プレビュー同期など）の各サブ機能を `MarkdownService` に分割し、差分検知/同期を観測するイベントを追加する。
- [x] ED-6: `Minimap` コンポーネントと `Shadow Buffer` オーバーレイの基礎を作成し、検索結果や AI/Git 差分をライン/色で描画できるようにする。`focus_id` 更新で `Command Hub`/`Structure Path` に同期するイベントを出す。
- [x] ED-7: `Smart Gutter` の差分タイプ判別 (git/AI)・色アニメーション・クリックジャンプ・Approval Request 連携を分解し、`Structure Path`/`Command Hub` の状態と連動するイベントを定義する。
- [x] ED-8: ビュー側で Markdown Preview を右クリックメニューに統合し、プレビューとのスクロール同期や見出しクリックで本文にジャンプする機能を追加する。
- [ ] ED-9: ペインの再配置・Split/Close/Open to Side/Move Tab 機能などを制御する `PaneManager` を実装し、ペイン状態と Command Hub の設定と連携させる。
- [ ] ED-10: タブ操作（ドラッグ並び替え、ピン止め、Close Others/Close to Right/Reopen Closed Tab）の各操作フローを `TabManager` でモデル化し、UI/キーボード/Command Hub いずれからも呼べるようにする。
- [x] ED-11: ペインごとの「直前ファイルへ戻る」履歴スタックを `PaneHistory` で保持する仕組みを作成する。
- [x] ED-12: セッション復元機能の骨格（分割構成・アクティブタブ・タブ順序/ピン状態）を `SessionState` に格納し、保存/復元フローを実装。失敗した場合は UI で理由付き通知。
- [x] ED-13: `Structure Path` コンポーネントを追加し、セグメントに Agent Status/差分マークを表示。クリック・`Alt+1`〜`Alt+5`・Command Hub Backoff と同期。

## 4 Search
- [x] SR-1: グローバル検索インターフェースと `SearchService` を構築し、正規表現・`.gitignore` 除外トグル・条件指定 (フォルダ/ファイル名/正規表現) をサポート。
- [x] SR-2: 検索結果一覧の Enter/F4/Shift+F4 キーボード操作でカーソルジャンプや前後移動を行うトラッキングを用意する。

## 5 VCS
- [x] VC-1: ファイルツリー上で git 変更ステータス (Untracked/Modified/Deleted) をラベル表示するフックを追加し、VS Command Hub などとステータスを共有。
- [x] VC-2: ワークスペース内のリポジトリを検出する `RepositoryService` と、変更一覧 (ステージ・差分) 表示 UI を作成。
- [x] VC-3: コミットログビューを追加し、選択リポジトリベースでログを取得・表示できるようにする。

## 6 Terminal
- [ ] TM-1: Terminal セッション起動 UI と `PTY` 管理を実装し、Workspace Session に紐づく run_command を PTY で実行。
- [ ] TM-2: スクロールバック（500行以上）と Unicode/全角対応のレンダリングコンポーネントを整備。
- [ ] TM-3: 同一 Workspace Session 内のコマンドキュー (Running/Queued/Completed/Failed) を FIFO で制御し、`tool.execution.queue_max_pending` 超過時に拒否・UI通知。
- [ ] TM-4: 環境変数注入の制御 (workspace_env、未許可の追加禁止) と CWD 固定化。
- [ ] TM-5: run_command の結果と通知を連動させ、ターミナル UI 上のステータスを同期。
- [ ] TM-6: キュー遷移/拒否/中断を Audit Event として記録し、Terminal UI の表示に反映。

## 7 Command Hub
- [x] CH-1: `Cmd+Shift+P` (Ctrl+Shift+P) でコマンドパレットを開くキーボード/クリック制御と、別要素クリック/ESC で閉じる仕組みを作る。
- [x] CH-2: Action Mode (`>` 系コマンド) と Navigation Mode (`:` 系コマンド) の切り替え・ドメイン/構文の正規化を行い、Picker 連動で選択/実行/キャンセルを実現。
- [x] CH-3: Workspace/Panes/Panel/Terminal/Selected 系コマンドに必要なリスト取得と実行 (Add/List/Remove/Split/Next/Prev/Close/Open など) を実装。
- [x] CH-4: 破壊的操作の確認フロー (ダイアログ or 2段階) を入れ、Command Hub からの実行時に失敗やキャンセルを伝える。
- [x] CH-5: `target` の Literal バイパス (`"..."` / `--literal`) と大文字小文字無視の受理をパーサでサポートし、内部での正規化結果をログ/実行に渡す。

<!--
## 8 UX/性能指標
- [ ] UX-1: 空ワークスペース起動時間測定用のベンチマーク構成と `nue-app` での初回描画時間を 2 秒以内に抑えるための遅延ロード管理を導入。
- [ ] UX-2: 10,000 行級ファイルのスクロール連続性 (Canvas/Virtualization) と 60fps 相当の UI 更新ポリシー (requestAnimationFrame サイクル制御) を調整。
- [ ] UX-3: グローバル検索初回応答 500ms 以内を目指し、検索インデックス/キャッシュ・表示更新間の非同期遅延を計測するフックを整備。
- [ ] UX-4: 大量検索結果表示中の入力操作遅延防止のための debounced/paginated 表示とバックプレッシャー付きレンダリングを試案。
-->

## ToDo

- [x] `nue-ui` の Legacy Explorer を実装し、`LegacyFileTree` 購読によるツリー描画と `LegacyWorkspaceEditor::select_file` へのファイル選択導線を接続する。
- [ ] `nue-ui` の Editor 入力導線を実装し、ショートカット入力を `EditorCore::dispatch_shortcut`、右クリック操作を `open_context_menu` / `execute_context_menu_item`、Markdown 操作を `execute_markdown_feature` へ接続する。
- [x] `EditorCoreEvent::Minimap*` / `EditorCoreEvent::SmartGutter*` を `nue-ui` で購読し、Minimap・Smart Gutter の描画、クリックジャンプ、Approval Request、Command Hub/Structure Path 同期を反映する。
- [x] `WorkspaceRail` 実コンポーメントを `WorkspaceRegistry` に接続し、`drain_status_events` のライブ更新、右クリック除外、`WorkspacePathValidation` の文言マッピングを実装する。
- [x] `DesignSystem` の `glass_style_for_surface` / `status_style_for` / `diff_style_for` / `OverlayZIndexManager::resolve` を実 UI（Command Hub/Workspace Rail/Structure Path/Smart Gutter/Minimap）へ適用する。
- [ ] `CommandHubSession` と `CommandHubActionModel` を UI 入力と接続し、Action/Navigation 切替、Picker 実行、破壊的操作確認フローを `nue-ui` 側で完結させる。
- [ ] `CommandHubDispatchOutcome`（`Closed` / `BackToListing` / `Failed` / `Canceled` 相当）を UI 状態遷移と通知表示へ明示マッピングする。
- [ ] `CommandHubActionModel` の実行結果を Workspace/Pane/Terminal の実体状態更新へ接続し、モデル内完結から上位レイヤー連携へ置き換える。
- [x] `EditorCore` 公開型の `PathBuf` を UI 表示へ変換する文字列化ポリシー（非UTF-8時の扱い含む）を仕様化し、上位層で統一適用する。
- [ ] `cargo clippy --all-targets -- -D warnings` で検出される既存警告（`pane_history` / `search_navigator` / `structure_path` 付近）を解消し、`scripts/sanity.sh` 完了条件を満たす。
