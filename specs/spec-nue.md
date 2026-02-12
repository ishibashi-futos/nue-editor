# 次世代AIネイティブ・エディタ「Nue」詳細仕様書

## 1. プロジェクト・ビジョン

Nueは、従来の「コードを書くためのツール」を脱却し、AIエージェント（Codex等）が能動的に作業し、人間がそれを高度な視覚情報で監督・操縦するための**「エージェント・オーケストレーション環境」**である。

---

## 2. システムアーキテクチャ（階層構造）

Nueは、Slackのようなマルチワークスペース管理を最上位に据えた、マルチプロセス・アーキテクチャを採用する。

### 2.1 App Host (Nue Runtime)

* **役割**: 全セッションのライフサイクル管理、グローバル設定、通知集約。
* **Workspace Switcher**: 左端のレイルで、複数プロジェクトを切り替える。
* **Global Config**: `~/.config/nue/config.yaml` 及び環境変数を統括。

### 2.2 Workspace Session (独立した実行単位)

各ワークスペースは、以下のコンポーネントを独自にインスタンス化する。

* **AI Agent**: プロジェクトごとに最適化されたエージェントプロセス。
* **MCP Router**: エージェントが利用できるツール群のゲートウェイ。
* **Editor Core**: バッファ、VFS、内部コマンド実行エンジン。
* **UI View**: Legacy (Tree) と Galaxy (Graph) の描画。

---

## 3. UI/UX デザイン仕様

### 3.1 ビジュアル・アイデンティティ

* **テーマ**: 「Neon-Night」（深いネイビーを基調に、シアン、マゼンタの発光アクセント）。
* **描画エンジン**: Rust `GPUI` によるGPU加速。60fpsのスムーズなアニメーションとスクロール。

### 3.2 ワークスペース・レイル (Slack-like Switcher)

各アイコンにエージェントの**ライブステータス**を表示する。

* **Busy (Cyan Pulse)**: AIが思考・コード生成・ビルド中。
* **Waiting (Yellow Blink)**: ユーザーの承認や入力を待機中。
* **Error (Magenta Vibration)**: ビルド失敗や例外発生。
* **Idle (Dimmed)**: 待機状態。

### 3.3 エクスプローラー：二つの顔 (Legacy & Galaxy)

ヘッダーのトグルボタンで瞬時に切り替え可能。

* **Legacy View**: 従来の階層型ディレクトリツリー。ファイル検索や物理構造の把握に使用。
* **Galaxy View**:
  * **導入タイミング**: 初期リリースでは Legacy View をデフォルトとし、Galaxy View は `v1.0` 以降で段階導入する。Galaxy View は Legacy View と役割を明確に分離し、依存関係の可視化を優先する。
  * **構造**: LSP依存関係をノードとエッジで表現する銀河系モチーフのグラフビューで、各ノードはファイル/モジュールを、エッジは依存/呼び出しを表す。エージェントが編集する対象は `彗星`、隣接する影響範囲は `衝撃波` で視覚化することで、変更の波及をユーザーに伝える。
  * **表示品質要件**: AIの編集対象と影響範囲を常に強調し、グラフ内の操作対象を明確にする。色/サイズ/輝度は `Agent Status`（Busy/Waiting/Error/Idle）を参照したハイライトルールに従い、ユーザーが現在の負荷を一目で把握できるようにする。
  * **パフォーマンス**: 500 個までの可視ノードを含む更新では、Galaxy View の描画/遷移は最低45fpsを維持し、データの追加・削除・焦点移動時も GPU レベルのダブルバッファリングとインクリメンタルレイアウト更新で一貫性を保つ。500 個を超えるノードは動的に詳細度を下げ、表示外のノードを折りたたむことで、レイテンシとフレームレートを安定化させる。
  * **アクセシビリティ**: カラーパレットは WCAG レベル AA のコントラスト比を満たし、視覚的な誤認を避けるためにエッジ/ノードに明確なラベルと代替テキストを付与する。Galaxy View の内容は Legacy View やアクセシビリティパネルで的確に列挙・検索できることを保証する。



---

## 4. MCP (Model Context Protocol) ツール仕様

エージェント（Codex等）は、以下のMCPツールを介してのみ「Nue」を操作できる。これにより、自由度と安全性を両立する。

| ドメイン | ツール名 | 役割 | Coreへの影響 |
| --- | --- | --- | --- |
| **FS** | `apply_patch` | 指定箇所への差分適用 | バッファ更新 + UI編集イベント発火 |
| **FS** | `read_file` | ファイル内容の読み取り | コンテキスト取得 |
| **VCS** | `get_status` | Gitの状態取得 | Galaxy Viewへの変更反映 |
| **VCS** | `commit` | 変更の確定 | 履歴の保存 |
| **Runtime** | `run_command` | テストやビルドの実行 | ターミナル出力 + 成功/失敗のフィードバック |
| **UI** | `focus_file` | 特定ファイルへのズーム | Galaxy View上のカメラ移動 |

---

### 4.1 MCP Router Authorization Model

`MCP Router` はエージェントから呼び出される全ての `MCP Tool` を受け取り、認可を通じて Core への影響を制御するセキュリティ境界である。`Authorization Policy` は `Domain`/`Tool`/`Permit Arguments`/`Execution Context`/`Approval State` の組み合わせで定義され、エントリが存在しない呼び出しは暗黙の拒否とすることでホワイトリスト運用を保証する。

1. **評価順序の要件**  
   `MCP Router` は次の順序でポリシーを評価し、`deny` が一致した時点で判定を返す。  
   Domain/Tool → Execution Context → `deny` ポリシー → `allow` ポリシー → 暗黙の拒否。  
   `deny` と `allow` の関係が衝突する場合、`deny` を優先し、拒否理由を含む `Audit Event` を生成すること。

2. **実行時コンテキスト**  
   `Execution Context` には `Workspace Session` 識別子、作業中の git ブランチ、エージェントのリソース型（例: “Code Generation” 対 “Build”）を含め、ポリシーはこれらの条件を必要に応じてマッチ項目とする。

3. **ツール引数の扱い**  
   ツールに渡される引数は、ポリシーに応じてマスク/ハッシュ化された形で評価に使う。暗黙的なすべての引数許容は認可リスクを高めるため、ポリシーで定義された引数セットへの一致が必須となる。

4. **監査連携**  
   `MCP Router` はすべてのツール呼び出しに `Audit Event` を記録し、拒否時には原因・満たせなかった `Authorization Policy` を `Audit Event` に含める。`Audit Event` は必要なら匿名化パラメータ（引数全体のハッシュなど）を適用する。

### 4.2 Shadow Buffer と承認フロー

`Shadow Buffer` はエージェントが行った差分編集を本バッファにマージする前に保持する構造体であり、各差分はファイル単位と `Workspace Session` 単位の両方で区分される。各差分には、発生時刻、差分の範囲、発行元エージェント、現在の `Agent Status` を付与することで、レビューと追跡が可能である。

1. **承認単位**  
   初期リリースでは `Workspace Session` 単位の一括 `Accept` と `ファイル単位` `Accept` の両方をサポートし、ユーザーは差分を開きながら任意の粒度で承認できる。追加の操作（`Reject`/`Revert`/`Partial Accept`）は `v1.0` 以降の拡張項目とし、これらが未実装であることを明示する。

2. **承認後の反映**  
   `Accept` が行われると、`Shadow Buffer` の該当差分は `Editor Core` の本バッファへマージされ、`UI View` への更新イベント（差分の範囲・Agent Status）と `Audit Event` の両方を生成する。差分はマージ後に `Shadow Buffer` から削除され、ストレージに永続化されない。

3. **レビュー情報**  
   `Shadow Buffer` の差分は `Galaxy View` と `Legacy View` の両方で列挙可能とし、特に `Galaxy Feedback` は対象ノードを `彗星` でハイライトし、設定された `Agent Status` に応じた輝度で表示する。差分ごとのメタ情報（例: `Approval Unit`、`Execution Context`）を UI で参照できること。

### 4.3 ターミナルエミュレーター最小要件

ターミナルエミュレーター機能はエージェントが `run_command` を通じて実行するテスト・ビルド・デバッグ出力と連携する。初期リリースにおいて `Terminal Emulator` は以下の要件を満たすこと。

* **PTY** を使用してホストシステムとセッションを管理し、エージェントもユーザーも同一シェル状態を共有できるようにする。
* **スクロールバック**: 少なくとも 500 行の出力を保持し、上下キー/マウス/ペインスクロールで遡れること。
* **文字幅/Unicode整合**: 全角半角や Unicode 統合文字列を含めても列整列が崩れないよう、フォントレンダラとグリッドが一致すること。

これらの要件が満たされない場合、`Audit Event` を通じてユーザーへエラー状態を通知し、`MCP Router` はエージェントが再度 `run_command` を要求する前に運用上の確認を促す。

### 4.4 Audit Event ライフサイクル

`MCP Router` はすべての `MCP Tool` 呼び出し、`Shadow Buffer` 承認操作、`Authorization Policy` による拒否などを `Audit Event` として記録し、`App Host` に送信する。`Audit Event` は以下の要件を満たす。

1. **生成**
   - `MCP Router` は各イベントにタイムスタンプ、`Workspace Session`、対象ツール、`Approval Unit`、`Execution Context`、`Agent Status`、判定結果 (`allow`/`deny`) を含め、`deny` の場合は該当ポリシー ID と不一致理由を付与することを **MUST** とする。
   - `Shadow Buffer` の `Accept` 操作は差分の範囲を含む `Audit Event` を生成し、UI からの通知後に `Editor Core` へ書き戻しを行う際に `App Host` へ通知されることを **MUST** とする。

2. **保持と永続化**
   - 初期リリースでは `App Host` が記録をメモリ上のバッファで保持し、当該 `Workspace Session` 終了後に破棄することを **SHOULD** とする。
   - `v1.0` 以降、`App Host` はユーザーグローバル領域（デフォルト: `~/.config/nue/audit.jsonl`、`audit.storage.path` で再設定可能）に改行区切り JSON (`*.jsonl`) 形式で `Audit Event` を追記し、指定された保持期間 (`audit.retention_days`、デフォルト 30 日) を超えた記録はログロールや削除で期限を守ることを **MUST** とする。
   - ファイルへの追記時は排他制御を行い、追記完了後にメモリバッファから該当イベントを削除することを **SHOULD** とする。

3. **匿名化と機微情報**
   - `Audit Event` に含まれるツール引数、パス、環境変数などの機微情報は、`audit.anonymization.level` に応じてマスクまたはハッシュ化されることを **MUST** とし、デフォルトでは引数全体をハッシュ化する。

4. **再送とフェールオーバー**
   - 永続化失敗時、`App Host` は対象イベントを再送キューへ戻し、指数バックオフで再試行することを **SHOULD** とする。キューが `audit.queue.max` を超過する場合は最古イベントを破棄し、その旨を `Audit Event` と UI に通知することを **MUST** とする。
   - 各 `Audit Event` は一意の `event_id` を持ち、再送時も同一 ID で管理され、完了後に再送フラグを消去することを **SHOULD** とする。

5. **監査表示**
   - `App Host` は `Legacy View`/`Galaxy View` の監査パネルで `Audit Event` を `Workspace Session` や `Agent Status`、`Approval Unit` でフィルタ可能とすることを **SHOULD** とする。

以上で、監査記録の生成・保持・匿名化・再送の責務が明示される。

## 5. Configuration (設定管理) モジュール

設定は階層的にマージされ、常に最新の状態が各コンポーネントへリアクティブに反映される。

1. **環境変数 (Priority: 1)**: `NUE_AGENT__SAFETY__AUTO_ACCEPT=true` など。
2. **ワークスペース設定 (Priority: 2)**: `.nue/config.yaml`（プロジェクト固有）。
3. **グローバル設定 (Priority: 3)**: `~/.config/nue/config.yaml`（ユーザーの基本設定）。
4. **デフォルト (Priority: 4)**: システム内蔵の初期値。

---

## 6. AI共創ワークフロー：The Nue Loop

1. **インテント入力**: ユーザーがUIから指示を出す。
2. **エージェント起動**: UIがCodex等へタスクを丸投げ。
3. **MCP実務**: エージェントが `fs` や `runtime` ツールを駆使。
4. **Shadow Buffer**: Coreはエージェントの編集を「未承認の差分」として保持。
5. **Galaxy Feedback**: Galaxy View上で、AIが編集しているファイルが激しく発光（パルス）。
6. **人間の承認**: ユーザーが差分を確認し、`Accept`。変更が本番バッファへマージされる。

---

## 7. フロントエンド・デザイン実装プロセス

デザインは、AIが操作可能な形式で管理する。

* **ツール**: **Penpot**（オープンソース・SVG/CSSベース）。
* **手法**:
1. プロジェクト内にUI定義ファイル（CSS/SVG）を置く。
2. Codex（エージェント）にMCP経由でそのファイルを編集させ、デザインの微調整（色、レイアウト）を行わせる。
3. 確定したデザイン数値を `nue-ui` の GPUI 定義に落とし込む。



---

### 次のアクション

この仕様書をベースに、いよいよ**プロジェクトの初手（リポジトリの初期化）**に移ります。

**Would you like me to ...?**

1. **`Cargo.toml` (Workspace全体) の書き出し**: `nue-config`, `nue-ui`, `nue-core`, `nue-mcp` の4クレート構成を作成する。
2. **`nue-config` の最小実装**: 設定ファイルと環境変数をマージするRustコードを書く。
3. **GPUIでの「左レイル（ワークスペースバッヂ付き）」のプロトタイプコード** を作成する。

プロジェクト「Nue」、始動の準備は整いました。どこから手を付けましょうか？
