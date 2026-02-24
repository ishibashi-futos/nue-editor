# nue-editor ローカル起動到達プラン（2026-02-24 時点）

## ローカル起動スコープ固定（T-001 / L1-1 / SS-1-1）

このセクションは、本マイルストーンの完成判定と実装判断のぶれを防ぐための固定条件を定義する。詳細設計や後続セクションに記述が分散している場合でも、ローカル起動到達の完成扱いは本セクションを優先して判断する。

### 完成扱い（本マイルストーンの DoD 判定基準）

次のすべてを満たした時点を「ローカル起動到達」の完成扱いとする。

- `macOS` 上で `cargo run -p nue-app -- --workspace <ABSOLUTE_PATH>` により GPUI ウィンドウが起動し、ワークスペース画面へ到達できる
- 空引数起動（`cargo run -p nue-app --`）でもクラッシュせず、ワークスペース未選択画面（プレースホルダ可）が表示される
- 本セクションの「必須画面」「必須操作」が成立する
- 本セクションの「手動スモーク手順」を実行し、期待結果を確認できる

### 必須画面（最低限の表示要件）

- 未選択画面（空引数起動時）
  - ワークスペース未選択であることが視認できるプレースホルダ表示でよい
- ワークスペースセッション画面（`--workspace` 指定時）
  - `Workspace Rail`
  - 左パネル（初期表示は `Explorer`）
  - 中央エディタ領域（テキスト表示/編集可能）
  - タブバー（最低 1 タブ表示）
  - `Command Hub` オーバーレイ（開閉可能）
  - 下部ターミナル領域（状態表示 + スクロールバック表示）
- パネル切替表示
  - `Explorer` / `Search` / `VCS` の切替が視認できる
  - `Search` / `VCS` はプレースホルダ表示でよい

### 必須操作（最低限の到達確認）

- Explorer からファイルを選択し、エディタ表示が切り替わる
- エディタで編集し、保存操作でローカルファイルへ反映される
- `Command Hub` を開閉でき、最低 1 つのコマンド実行結果が画面状態へ反映される
  - 少なくとも `panel focus` 系コマンドを対象に含める
- パネル切替（`Explorer` / `Search` / `VCS`）が動作する
- ターミナル領域で `run_command` を実行でき、状態遷移（`Queued` / `Running` / `Completed` または `Failed`）と出力が表示される

### 許容モック範囲（本マイルストーンで未実装を許容する範囲）

- `nue-mcp`, `nue-semantic` は no-op / stub のままでよい
- `nue-config` は CLI 起動に必要な最小設定のみでよい（設定ファイル・環境変数マージ・ホットリロードは対象外）
- 空引数起動時のワークスペース追加/選択導線 UI は不要（未選択画面表示のみ必須）
- `Search` / `VCS` パネルの詳細 UI・実データ接続は不要（プレースホルダ可）
- Minimap / SmartGutter / StructurePath の高度描画は不要（状態表示・件数表示で可）
- エディタ外のファイル変更検知と自動リロードは不要
- ターミナル PTY 実装は `run_command` 到達を優先し、`alacritty_terminal` の利用を許容する

### 手動スモーク手順（固定）

1. `cargo run -p nue-app -- --workspace <ABSOLUTE_PATH>` を実行し、ワークスペースセッション画面が表示されることを確認する
2. `Explorer` から既存ファイルを 1 つ選択し、エディタに内容が表示されることを確認する
3. エディタで文字を編集して保存し、対象ファイルの内容が実際に更新されることを確認する
4. `Command Hub` を開閉し、少なくとも 1 つの `panel focus` 系コマンドを実行して表示パネルが切り替わることを確認する
5. `Search` / `VCS` パネルへ切り替え、プレースホルダ表示でもクラッシュせず遷移できることを確認する
6. ターミナル領域で `run_command`（例: `pwd`, `echo nue-smoke`）を実行し、状態遷移と出力のスクロールバック反映を確認する
7. `cargo run -p nue-app --` を実行し、未選択画面が表示されることを確認する

## 0. 目的とスコープ

本ドキュメントは、`nue-editor` の現状コード/バックログ/リポジトリ状態を踏まえ、`nue-app` をローカル環境で実際に起動し、基本的な操作確認ができる状態まで到達するための実装プランを定義する。

対象は「エントリーポイント・画面遷移の繋ぎこみ」を中心とし、既存の `nue-core` / `nue-ui` のロジック資産を活かして統合アプリとして成立させることを目的とする。

本マイルストーンの受け入れ対象 OS は `macOS` に限定する。起動主導線は `--workspace` 指定の CLI とし、空引数起動時は未選択画面を表示する（UI からのワークスペース追加/選択導線は本マイルストーンの必須要件に含めない）。ターミナルの PTY 実装は `run_command` 到達を優先し、`alacritty_terminal` の利用を許容する。

## 1. Phase1: 現状分析

### 1.1 リポジトリ状態（確認結果）

- ブランチ: `feat/v1.0`
- ワークツリー: 変更なし（クリーン）
- 実行確認:
  - `cargo run -p nue-app --quiet` は成功するが、出力は `Hello, world!` のみ
  - `cargo check --all-targets` は成功
  - `cargo test --quiet` は成功（ワークスペースのユニットテスト群は通過）

### 1.2 `specs/backlog.md` のステータス（確認結果）

- `0 基盤的な仕組みの整備` 〜 `7 Command Hub` は全項目が完了（`[x]`）
- `## ToDo` セクションも全項目が完了（`[x]`）
- `8 UX/性能指標` はコメントアウトされており未着手扱い

### 1.3 実装済みコンポーネント（コードから確認できる範囲）

- `nue-core`
  - `EditorCore`（編集、ショートカット、Markdown、検索、Minimap/SmartGutterイベント生成）
  - `LegacyWorkspaceEditor`（ワークスペース配下ファイルの選択・保存）
  - `WorkspaceRegistry` / `WorkspaceRailModel`（登録・除外・ステータス更新）
  - `PaneManager` / `TabManager` / `SessionState`（ペイン・タブ・復元）
  - `CommandHubActionModel` / `CommandHubStateSink`（コマンド実行と上位状態同期）
  - `TerminalSession`（キュー/監査/スクロールバック/コンテキスト検証）
  - `SearchService`, `RepositoryService`, `commit_log` 取得
- `nue-ui`
  - `CommandHubUiController`
  - `LegacyExplorerModel`
  - `EditorInputController`
  - `EditorEventSubscriber`（Minimap/SmartGutterイベント購読）
  - `TerminalUiController`, `TerminalScrollbackRenderer`
  - `TabBarUiController`
  - `DesignSystem`, `UiStyleGuide`, フォント資産（JetBrainsMono埋め込みバイト列）

### 1.4 未実装/未接続の主要ギャップ

- `nue-app`
  - エントリーポイントが `Hello, world!` のみで、`nue-core`/`nue-ui` を一切初期化していない
- `nue-ui`
  - `gpui` 依存はあるが、GPUI ウィンドウ/ビュー/イベントループの実装が存在しない
  - 実画面コンポーネント（アプリルート、ワークスペース画面、パネル切替 UI 等）が未実装
- `nue-config`, `nue-mcp`, `nue-semantic`
  - `add()` のみの雛形で、起動に使う型/初期化 API が未整備

### 1.5 画面構成・ルーティング・状態管理方針（現コードから推定）

- 画面遷移（ルーティング）専用の型は未存在
- 状態管理は以下の分散型設計がベースになっている
  - `nue-core`: ドメインモデル + イベントキュー/スナップショット
  - `nue-ui`: UIコントローラ（状態遷移・イベント変換）
  - `nue-app`（未実装）: これらを束ねる統合状態・画面遷移の責務を持つ想定
- `CommandHubActionModel` は `CommandHubStateSink` により「上位アプリ状態同期」のための接続点を既に提供している

### 1.6 想定されるアプリのエントリーポイント

- 実行バイナリ: `nue-app`
- 入口ファイル: `nue-app/src/main.rs`
- 将来の責務（必要）:
  - 起動引数/初期設定の解決
  - `App Host` 相当の初期化
  - GPUI アプリ起動
  - ワークスペースセッション/画面ルートの生成

### 1.7 未接続・未使用コンポーネント/ユーティリティ

- `nue-core::command::actions::CommandHubApplicationState(Sink)`（アプリ統合向けだが未使用）
- `nue-ui::editor_events::EditorEventSubscriber`（UIイベント購読はあるが実画面未接続）
- `nue-ui::terminal_display::TerminalScrollbackRenderer`（描画用整形器だが実表示未接続）
- `nue-core::workspace::repository` / `commit_log` / `search`（モデルはあるが専用 UI 未接続）

## 2. Phase2: ゴール状態の定義（ローカル起動・基本動作確認）

### 2.1 ビルド・起動コマンド（DoD 観点）

- 受け入れ対象 OS（本マイルストーン）:
  - `macOS` のみ
- 必須:
  - `cargo run -p nue-app -- --workspace <ABSOLUTE_PATH>`
- 推奨（補助スクリプト）:
  - `scripts/run_nue.sh --workspace <ABSOLUTE_PATH>`
- 空引数時の挙動:
  - 起動は成功し、ワークスペース未選択画面（プレースホルダ可）を表示する

### 2.2 最低限表示されるべき画面・UI

- アプリ起動直後に GPUI ウィンドウが表示される
- ワークスペースセッション画面（最小構成）
  - `Workspace Rail`
  - 左パネル（`Legacy Explorer`）
  - 中央エディタ領域（テキスト編集可能）
  - タブバー（最低 1 タブ表示）
  - Command Hub オーバーレイ（開閉可能）
  - 下部ターミナル領域（最低限の状態/ログ表示）
- パネル切替（Explorer / Search / VCS）は少なくとも画面遷移として動作する
  - Search/VCS は初期段階ではプレースホルダ表示でも可

### 2.3 ユーザーが行える基本操作

- ワークスペースの選択（主導線は起動引数 `--workspace`。空引数起動時は未選択画面表示のみ）
- `Legacy Explorer` からファイル選択
- ファイル内容の表示・編集
- 保存操作（ショートカットまたは UI 操作）
- Command Hub の開閉と最低限のコマンド実行
  - `panel focus`
  - `pane` / `tab` の一部操作（閉じる/移動/再配置など）
- ターミナル領域での `run_command` 実コマンド実行（PTY）と状態表示

### 2.4 必須の初期化処理

- App 起動設定の解決（CLI 引数、将来の `nue-config` 接続余地）
- `tokio` ランタイム初期化（UI スレッドとの責務分離を含む）
- DesignSystem/フォント資産の GPUI 登録
- アプリ統合状態（ワークスペース一覧、アクティブセッション、パネル種別、通知）の初期化
- ワークスペースセッション生成時のコントローラ初期化
  - `LegacyWorkspaceEditor`
  - `LegacyExplorerModel`
  - `CommandHubUiController`
  - `TabBarUiController`
  - `TerminalUiController`
  - `EditorEventSubscriber`
- `EditorCore` のデフォルトショートカット登録
- 画面イベントからコア/コントローラへのディスパッチと `drain_events` 反映ループ

## 3. Phase3: ギャップ分析

### 3.1 足りていないファイル・エントリーポイント・インタフェース

- `nue-app` の実エントリーポイント（起動引数処理、アプリ初期化）
- `nue-ui` の GPUI ルート（ウィンドウ作成、View 構築、入力イベント接続）
- `nue-app` もしくは `nue-ui` の統合状態（App Host / Workspace Session / 画面遷移状態）
- `LegacyWorkspaceEditor` と `EditorCore` の統合利用に必要な公開委譲 API / 読み取りアクセサ
  - 現状は `EditorCore` のイベント排出・高度機能をアプリ層から扱いにくい
  - 本プランでは `EditorCore` の生 `&mut` 公開ではなく、`LegacyWorkspaceEditor` 経由の委譲を基本方針とする
- パネル遷移 UI（Explorer/Search/VCS）とプレースホルダ View

### 3.2 繋がっていないコンポーネント

- `WorkspaceRegistry` ↔ 実際の `Workspace Session` 起動
- `LegacyExplorerModel` ↔ 実画面クリック入力 ↔ `LegacyWorkspaceEditor`
- `EditorInputController` ↔ 実キーボード/コンテキストメニュー入力
- `EditorCore::drain_events` ↔ `EditorEventSubscriber` ↔ 実 UI 表示更新
- `CommandHubUiController` ↔ 実オーバーレイ UI ↔ 実行結果のアプリ状態反映
- `TabBarUiController` / `PaneManager` ↔ 実タブ・ペイン UI
- `TerminalUiController` / `TerminalScrollbackRenderer` ↔ 実ターミナル表示 UI

### 3.3 ビルド・起動時に想定されるエラー/難所

- GPUI 0.2.2 の実 API に合わせた初期化・ウィンドウ生成の実装コスト
- macOS 上の UI スレッド制約と `tokio` ランタイム共存の設計
- `macOS` 上の PTY 実行統合（`alacritty_terminal` 利用を含む）における終了処理・入出力読取・状態反映の実装コスト
- Rust の所有権制約による統合状態（複数コントローラと UI 更新）の持ち方
- `LegacyWorkspaceEditor` の委譲 API 不足により、アプリ層で別 `EditorCore` を持ってしまう二重状態化リスク
- Path/非UTF-8表示・絶対パス入力の UI ハンドリング

### 3.4 暫定実装/モックで許容できる箇所と本実装必須箇所

#### 暫定実装/モックで許容（本マイルストーン）

- `nue-mcp`, `nue-semantic` の no-op 実装（未使用のままで可）
- `nue-config` の最小設定（CLI 主導で可）
- 空引数起動時のワークスペース追加/選択 UI 導線（未選択画面表示のみで可）
- Search/VCS パネルの詳細 UI（プレースホルダで可）
- Minimap/SmartGutter/StructurePath の高度描画（まずは状態表示/件数表示）

#### 本実装必須（本マイルストーン）

- `nue-app` の実エントリーポイント
- `macOS` での GPUI ウィンドウ起動とルート画面表示
- ワークスペース選択から `Legacy Explorer` / エディタへの導線
- 編集と保存の動作確認
- Command Hub 開閉と最低限の画面遷移/状態更新
- `macOS` でのターミナル `run_command` 実コマンド実行（PTY）と結果表示
- ローカル起動手順の固定（コマンド/スクリプト/手順書）

## 4. Phase4: 実装プラン（Layer1-3）

## Layer1: 大きな塊（全体の流れ）

### Step L1-1: 起動基盤と統合アプリ骨格を作る

- 目的:
  - `nue-app` を `Hello, world!` から実アプリ起動に置き換える
  - GPUI ウィンドウを開くための最低限の統合骨格を用意する
- 論点:
  - GPUI と `tokio` の共存方式
  - 統合スパイク（技術検証）を独立ゲートとして先に通すか
  - GPUI 0.2.2 実 API 差分があった場合の「まず繋ぐ」優先の実装順
  - 統合状態を `nue-app` に置くか `nue-ui` に置くか
- 完了条件(DoD):
  - `cargo run -p nue-app` で GPUI ウィンドウが開く
  - クラッシュせず、空画面またはプレースホルダ画面が描画される
  - `macOS` 上で `GPUI + tokio` 共存とダミー非同期イベント反映の統合スパイクが通過している

### Step L1-2: ワークスペースセッション初期化と基本画面構成を接続する

- 目的:
  - `WorkspaceRegistry` / `LegacyWorkspaceEditor` / `LegacyExplorerModel` を統合し、ワークスペースを開けるようにする
  - 画面の主要レイアウト（Rail / Explorer / Editor / Terminal）を表示する
- 論点:
  - `LegacyWorkspaceEditor` 委譲 API の公開範囲（生アクセサを避ける）
  - アクティブワークスペース選択状態をどこで保持するか
- 完了条件(DoD):
  - 指定ワークスペースを開いた状態で Explorer と Editor 領域が表示される
  - Explorer のファイル選択でエディタに内容が表示される

### Step L1-3: 編集入力・保存・イベント反映の導線を完成させる

- 目的:
  - `EditorInputController` を `LegacyWorkspaceEditor` の委譲 API に接続し、基本編集を可能にする
  - `EditorCore` イベントを UI 状態へ反映する最小ループを作る
- 論点:
  - UI イベント → コア呼び出し → `drain_events` → 再描画の順序
  - `EditorEventSubscriber` をどの範囲まで使うか
- 完了条件(DoD):
  - キーボード入力で編集できる
  - 保存操作でファイルが更新される
  - 編集後に UI 状態（タイトル/dirty 表示等）が更新される

### Step L1-4: 画面遷移（パネル/ペイン/タブ/Command Hub）を繋ぎ込む

- 目的:
  - Command Hub を起点にパネル切替・ペイン/タブ操作を画面へ反映する
  - 「エントリーポイント・画面遷移の繋ぎこみ」を完成させる
- 論点:
  - `CommandHubActionModel` の内部状態と実 UI 状態の二重管理
  - `CommandActionEvent` を UI に反映する責務分割
- 完了条件(DoD):
  - Command Hub の開閉と最低限のコマンド実行が動く
  - Explorer/Search/VCS のパネル切替が目視確認できる
  - タブ/ペイン操作が UI に反映される

### Step L1-5: ローカル起動導線・初期化スクリプト・検証手順を固定する

- 目的:
  - 再現可能なローカル起動方法を整備し、他作業者も動作確認できるようにする
- 論点:
  - 依存前提（Rust バージョン、起動引数、ワークスペースパス）
  - GUI アプリの自動テスト範囲と手動確認範囲の切り分け
- 完了条件(DoD):
  - 起動コマンド/スクリプトが文書化される
  - 手動スモークテスト手順が揃う
  - `cargo check` / `cargo test` / 必要最小限の sanity が通る

## Layer2: Layer1 をサブステップに分解

### L1-1 のサブステップ

#### SS-1-1

- 目的:
  - 本マイルストーンの起動仕様と最低画面要件をコード側前提に固定する
- 概要:
  - 起動引数（`--workspace`）の扱い方針を決める
  - 空起動時の画面状態（未選択画面）を定義する
  - スモーク確認シナリオを先に列挙する
- 受け入れ条件:
  - 実装者が迷わず起動フローを実装できる仕様メモが存在する

#### SS-1-2

- 目的:
  - `nue-app` から実際に UI 起動を呼び出せる入口を作る
- 概要:
  - `main` の責務を「引数解決」「起動設定生成」「UI 起動」に分割する
  - エラー時の標準エラー出力/終了コードを定義する
- 受け入れ条件:
  - `Hello, world!` が除去され、起動処理が新しい入口に置き換わっている

#### SS-1-3

- 目的:
  - GPUI 用のルートビュー/最小描画を `nue-ui` に用意する
- 概要:
  - ウィンドウ生成とルート画面表示を行う
  - DesignSystem/フォント資産を起動時に登録する導線を追加する
- 受け入れ条件:
  - 空状態のウィンドウが起動し、プレースホルダUIが見える

#### SS-1-4

- 目的:
  - `GPUI + tokio` 共存と UI スレッドへの非同期イベント反映の最小導線を、機能実装前の独立ゲートとして検証する
- 概要:
  - `tokio` ランタイム所有者と UI 更新経路（ダミーイベント）を最小実装で接続する
  - シャットダウン時にハング/クラッシュしない終了順を確認する
  - 検証結果を後続タスク（特に `T-010`, `T-030`）の前提として固定する
- 受け入れ条件:
  - `macOS` 上で GPUI ウィンドウ起動中にダミー非同期イベントが UI 表示へ反映される
  - ウィンドウクローズ時にプロセスが異常終了/ハングしない

### L1-2 のサブステップ

#### SS-2-1

- 目的:
  - アプリ統合状態（App Host / Session / パネル状態）を設計し、保持できるようにする
- 概要:
  - ワークスペース一覧・アクティブワークスペース・通知を保持する
  - 画面種別/パネル種別の状態を持つ
  - セッションごとのコントローラ束を保持する
- 受け入れ条件:
  - 起動時に空状態/単一ワークスペース状態のどちらも表現できる

#### SS-2-2

- 目的:
  - ワークスペース選択後に `LegacyWorkspaceEditor` などのコアを初期化する導線を作る
- 概要:
  - `WorkspaceRegistry` 登録情報からセッションを生成する
  - `LegacyExplorerModel` / `TabBar` / `Terminal` / `CommandHub` を初期化する
  - 初期タブ/ペイン/フォーカスを設定する
- 受け入れ条件:
  - ワークスペースを開く操作でセッション状態が生成される

#### SS-2-3

- 目的:
  - 主要画面レイアウトを描画し、状態に応じて表示を切り替える
- 概要:
  - Workspace Rail / 左パネル / Editor / 下部Terminal / Overlayを配置する
  - パネル未実装箇所はプレースホルダで埋める
- 受け入れ条件:
  - セッション画面全体が表示され、構造が視認できる

### L1-3 のサブステップ

#### SS-3-1

- 目的:
  - Explorer から Editor へのファイルオープン導線を接続する
- 概要:
  - Explorer ノード選択を `LegacyWorkspaceEditor::select_file` に接続する
  - 開いたファイルの内容・パス・タブ名を UI に反映する
- 受け入れ条件:
  - Explorer クリックでエディタ内容が更新される

#### SS-3-2

- 目的:
  - エディタ入力と保存導線を `EditorCore` ベースで動作させる
- 概要:
  - キーボード入力/ショートカット/保存を接続する
  - dirty 状態や保存結果を UI に表示する
- 受け入れ条件:
  - 編集と保存がローカルファイルへ反映される

#### SS-3-3

- 目的:
  - `EditorCore` イベントを UI 表示更新に反映する最小ループを作る
- 概要:
  - `drain_events` を購読し `EditorEventSubscriber` を更新する
  - Minimap/SmartGutter の状態件数やフォーカス情報を画面へ反映する
- 受け入れ条件:
  - 編集/差分イベントに応じて関連 UI 表示が更新される

### L1-4 のサブステップ

#### SS-4-1

- 目的:
  - Command Hub オーバーレイの開閉と入力処理を実画面に接続する
- 概要:
  - 開閉ショートカット/ESC/クリック外しを処理する
  - 候補一覧・確認状態・通知表示を反映する
- 受け入れ条件:
  - Command Hub が UI 上で開閉し、候補が表示される

#### SS-4-2

- 目的:
  - `CommandActionEvent` をアプリ状態へ反映し、画面遷移を成立させる
- 概要:
  - `panel focus` による Explorer/Search/VCS 切替を実装する
  - `pane` / `tab` 操作を実 UI 状態へ同期する
  - 必要に応じて `CommandHubStateSink` を接続する
- 受け入れ条件:
  - Command Hub 実行結果で画面表示と内部状態が一致して変化する

#### SS-4-3

- 目的:
  - Terminal UI とタブ/ペインの操作導線を整え、`run_command` の実コマンド実行（PTY）まで確認可能にする
- 概要:
  - ターミナルキュー/監査メッセージ/スクロールバックを表示する
  - `TerminalSession` と実 PTY 実行を接続し、状態遷移を UI に反映する
  - タブ/ペイン操作を UI ボタン/キーボード/Command Hub から試せるようにする
- 受け入れ条件:
  - 実コマンド実行の状態遷移（Queued/Running/Completed/Failed）とタブ/ペインの変化を画面で確認できる

### L1-5 のサブステップ

#### SS-5-1

- 目的:
  - ローカル起動の再現手順をスクリプト化する
- 概要:
  - `cargo run -p nue-app` を包む補助スクリプトを追加する
  - ワークスペースパス検証/ヘルプ表示を用意する
- 受け入れ条件:
  - 初見の作業者でも一発で起動コマンドを実行できる

#### SS-5-2

- 目的:
  - 自動確認と手動確認のチェックリストを固定する
- 概要:
  - `cargo check` / `cargo test` / 必要なら `scripts/sanity.sh` の運用手順を明記する
  - GUI のスモークテスト手順を文書化する
- 受け入れ条件:
  - 「何を確認すれば DoD か」が手順として再現可能

#### SS-5-3

- 目的:
  - リリース前の自己レビュー観点を残し、次フェーズへ引き継げる状態にする
- 概要:
  - 既知制約（未実装、モック、パフォーマンス未対応）を列挙する
  - 次フェーズの拡張ポイントを整理する
- 受け入れ条件:
  - 本マイルストーンの達成範囲と残課題が明確になっている

## Layer3: タスク単位の粒度（Issue/ToDo 登録可能レベル）

Layer3 のタスク一覧は `specs/plans/v1-release-backlog.md` に分離した。

- Issue/ToDo 登録、進捗チェック、受け入れ条件の確認は `specs/plans/v1-release-backlog.md` を参照する
- 本ファイルは Phase1/2/3/4/5 の計画・設計決定・前提条件の管理に集中する

## 5. Phase5: 自己レビュー・設計決定・Open Questions

### 5.1 自己レビュー（プラン整合性チェック）

- Phase1（現状分析）で確認した事実を前提に、Phase2（ゴール）を「起動可能・基本操作可能」に限定している
- `nue-core`/`nue-ui` の既存資産を最大限利用し、未実装の GPUI 画面と統合状態に集中する構成になっている
- `nue-mcp`/`nue-semantic` を no-op 扱いにし、マイルストーンの達成を阻害しないように切り分けている
- Layer3（`specs/plans/v1-release-backlog.md`）は逐次実行しやすい依存順で記述している（Tidy First で境界整理→接続→検証）

### 5.2 設計決定（反映済み）

- D-1: `LegacyWorkspaceEditor` に `EditorCore` の委譲 API / 読み取りアクセサを追加する方針を採用する
  - `EditorCore` の生 `&mut` アクセサは原則公開しない
  - アプリ層で別 `EditorCore` を保持しない
  - ワークスペース境界（ファイル選択・保存の検証）は `LegacyWorkspaceEditor` に残す
  - 影響タスク: `T-009`, `T-018`, `T-019`, `T-020`

- D-2: GPUI 0.2.2 の実 API 差分に対しては、先に実際の起動・入力接続を成立させてからモジュール分割を判断する
  - 先行分割より実接続を優先する
  - `T-004`〜`T-006` は「まず繋ぐ」前提で実装し、分割見直しは後追いで行う

- D-3: 本マイルストーンにターミナルの実コマンド実行（PTY）を含める
  - `TerminalSession` のキュー/監査/スクロールバック表示に加えて、実行プロセスとの接続まで行う
  - PTY/端末制御の基盤として、Zed で採用実績のある `alacritty_terminal` の利用を許容する（将来の `Windows` 対応を見据える）
  - 影響タスク: `T-010`, `T-029`, `T-030`

- D-4: Search/VCS パネルの最低限要件はプレースホルダ表示を許容する
  - 画面遷移の成立を優先し、詳細 UI 接続は後続フェーズへ送る
  - 影響タスク: `T-014`, `T-026`

- D-5: 本マイルストーンで `tokio` ランタイムを導入する
  - UI スレッド責務と非同期実行責務を分離する
  - `tokio` ランタイムは専用バックグラウンドスレッドで起動し、UI スレッドとは GPUI の `AsyncAppContext`（または同等の導線）経由で通信する
  - `GPUI + tokio` 共存とダミー非同期イベント反映の統合スパイクを、機能実装前の必須ゲートとして通す
  - 影響タスク: `T-002`, `T-004`, `T-004A`, `T-010`, `T-030`

- D-6: 本マイルストーンの受け入れ対象 OS は `macOS` に限定する
  - GUI 起動確認・PTY 実行確認・手動スモークテストは `macOS` のみを DoD 対象とする
  - 影響タスク: `T-005`, `T-030`, `T-035`, `T-036`

- D-7: 空引数起動時の UX は「未選択画面表示のみ」を本マイルストーンの要件とする
  - 起動主導線は `--workspace` 指定（CLI）
  - 空引数時に UI からワークスペース追加/選択してセッション起動する導線は必須要件に含めない
  - 影響タスク: `T-001`, `T-013`, `T-033`, `T-035`

- D-8: `nue-config` の実装スコープは本マイルストーンでは CLI 引数からの最小設定値注入に限定する
  - 設定ファイル読み込み・環境変数マージ・ホットリロードはスコープ外（スタブ許容）
  - 影響タスク: `T-003`

- D-9: エディタ外でのファイル変更検知と自動リロードは本マイルストーンの対象外とする
  - 保存要件はエディタ内操作による書き込み成立に限定し、File System Watcher 連携は後続フェーズへ送る
  - 影響タスク: `T-018`, `T-020`

- D-10: `DesignSystem` フォントは `include_bytes!` によるバイナリ埋め込み方式で登録する
  - 外部パス依存を避け、ローカル起動確認時の再現性を優先する
  - 影響タスク: `T-006`
