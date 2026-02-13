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

#### 3.1.1 カラーパレット

Nue は UI レイヤーごとに以下の色を採用し、役割ごとの区分を明示する。

| カテゴリ | 色名 | HEX | 主な配置 | 備考 |
| --- | --- | --- | --- | --- |
| Background | Deep Abyss | #0B0E14 | エディタのメイン背景。最も暗いレイヤー。 | レイヤー間の明度差を強調し、深度感を演出する。 |
| Surface | Space Grey | #1A1D23 | サイドバー、タブ、パネルの背景。Base より一段明るい。 | 入れ子構造で重なりを示すため、この色をベースに陰影をつける。 |
| Border | Midnight Glass | #2D323C | パネル境界線、セパレーター。 | 他パーツが浮き上がるように線の不透明度を調整する。 |
| Primary (AI) | Neon Cyan | #00F5FF | AI 活動中のハイライト、エージェントのステータス表示、ミニマップの AI 位置。 | 光に近い強い彩度を持ち、Pulse アニメーションと併用することで注目を集める。 |
| Success / Accept | Electric Lime | #32FF7E | 承認ボタン、成功通知、保存済インジケーター。 | 過度な頻出を避けつつ「安全/確定」を意味するヒュー。 |
| Warning / Wait | Solar Flare | #FFF200 | ユーザー入力待ち、未承認差分のガター、警告アイコン。 | 点滅アニメーションと組み合わせ、緊急性と待ち時間の両方を表す。 |
| Error / Alert | Cyber Magenta | #FF006E | ビルドエラー、認可拒否、Galaxy View のノード異常振動。 | オペレーションを即座に中断させるため、Flash アニメーションと併用する。 |
| Information | Ether Purple | #BF5AF2 | LSP 型情報、シンボル定義、Galaxy View の依存線。 | 情報提供的な補助要素で使用。 |
| Text (Main) | Cloud White | #E4E7EB | 標準テキスト、コード文字。 | 背景とのコントラストが十分なことを確認する。 |
| Text (Muted) | Dusty Grey | #717984 | コメント、無効 UI、パンくず。 | 帯状背景や低重要度テキストに使用する。 |

各色は直線的に使い回しを避け、視覚上の階層を保持するために `Neon Cyan` / `Cyber Magenta` / `Solar Flare` 等の高彩度色は限定的なハイライトやステータス表示にのみ使用することを **SHOULD** とする。

#### 3.1.2 エージェント状態と差分表示の色規約

`Agent Status`（Busy/Waiting/Error/Idle）は UI 上で以下の色/エフェクトで表現し、状態遷移の判別を利用者が瞬時に行えるようにすることを **MUST** とする。

- `Busy`: `Neon Cyan`（#00F5FF）を 0.4〜1.0 の Opacity でパルス状に発光させ、AI が計算中であることを強調する。
- `Waiting`: `Solar Flare`（#FFF200）で低速点滅し、ユーザーの承認や入力待ちを表示する。
- `Error`: `Cyber Magenta`（#FF006E）で鋭い明滅アニメーションを伴い、`Workspace Rail` や `Command Hub` にエラーバナーを表示する。
- `Idle`: `Dusty Grey` や `Cloud White` のデュアルトーンで落ち着いた表現とし、他状態と禁止線を明確にする。

差分（Diff）と承認可視化については、以下のルールを **MUST** とする。

- 未承認の行（Gutter）には `Solar Flare` の縦線を表示し、`Command Hub` や `Galaxy View` の候補パネルでも同色で強調する。
- 承認済み/確定行は `Cloud White` に馴染ませて Gutter 表示とテキストの彩度を段階的に落とす。
- 承認ボタンや完了インジケーターは `Electric Lime`（#32FF7E）でグロー効果を付与し、視覚的な “完了” を伝える。

これらのカラールールは `nue-ui` の GPUI 定義に反映し、パレットを変更する場合はいずれの用途が影響を受けるかを追跡できるようデザインシステムに記録することを **SHOULD** とする。

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
  * **パフォーマンス**: 500 個までの可視ノードを含む更新では、Galaxy View の描画/遷移は最低45fpsを維持し、データの追加・削除・焦点移動時も GPU レベルのダブルバッファリングとインクリメンタルレイアウト更新で一貫性を保つ。500 個を超えるノードは動的に詳細度を下げ、表示外のノードを折りたたむことで、レイテンシとフレームレートを安定化させる。500 個超え時の詳細度低下はユーザーの操作によらず自動で適用され、手動でフォーカスや展開を制御するメカニズムは提供しないものとする。
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

`MCP Router` はエージェントから呼び出される全ての `MCP Tool` を受け取り、認可を通じて Core への影響を制御するセキュリティ境界である。`Authorization Policy` は `Domain`/`Tool`/`Permit Arguments`/`Execution Context`/`Approval State` の組み合わせで 定義され、エントリが存在しない呼び出しは暗黙の拒否とすることでホワイトリスト運用を保証する。

#### 4.1.1 Authorization Policy Entry Structure
各 `Authorization Policy` は個々のエントリの集合とし、各エントリは次のフィールドを **MUST** もしくは **SHOULD** に基づいて定義する必要がある。

- `policy_id`（**SHOULD**）: 不服申立てや `Audit Event` に記録するための識別子。
- `domain`/`tool`（**MUST**）: `MCP Tool` のドメインと名前。ドメインは明示的な文字列で、`tool` も同様に正確一致させる。
- `argument_constraints`（**MUST**、空配列は許可しない）: 呼び出しに含まれる引数の名前と許容値を含むリスト。各制約は名前と `match_type`（`literal`/`regex`）および暗黙的な `hash` 値を持ち、`MCP Router` は受信時に引数を `audit.anonymization.level` に従ってハッシュ化してから照合する。未定義の引数が存在する場合は、`allow_extra_arguments=false` を明示的に宣言するか、追加の制約を記述する必要がある。
- `execution_context`（**MUST**）: `Workspace Session`、ブランチ、`Agent Status`、リソース型などの属性。各属性は `match_type` を含め、指定された条件と一致しない場合は単一のエントリとして扱われない。
- `approval_state`（**MUST**）: `auto_allow`/`requires_user_consent`/`blocked` のいずれかを指定し、`4.1.3` で定義するユーザー承認フローに従う。
- `effect`（**MUST**）: `allow` または `deny` を明示し、`deny` の記述はそのまま拒否を意味する。
- `priority`（**SHOULD**、整数）: 同じ `domain`/`tool` 内で複数の `allow` エントリが存在する場合、`Router` は `priority` の大きいエントリを優先する。未指定の場合はデフォルト値 `0` で扱う。
- `message`（**SHOULD**）: ユーザー通知やエージェントへのレスポンスで使用する短文。

`argument_constraints` は `Permit Arguments` の実体であり、**MUST** かつ明示的な `hash` マッチを持たない限り、引数の型や値を許可しない。エントリは任意で `allow_extra_arguments=true` を付与して引数の拡張を許可できるが、このフラグを使う場合も少なくとも `argument_constraints` に1つ以上の名前を含め、最低限の引数情報を保つことを **MUST** とする。

#### 4.1.2 Policy Evaluation and Conflict Resolution
`MCP Router` は呼び出しごとに、`domain`/`tool` → `execution_context` → `argument_constraints` の順でポリシーを絞り込む。最初の `deny` マッチが発生した時点で評価を打ち切り、`deny` を返す。`deny` と `allow` が競合する場合は常に `deny` を優先し、`Audit Event` に拒否理由と該当 `policy_id` を記録することを **MUST** とする。

`deny` が存在しない場合、`allow` の候補が残るので、`argument_constraints` の数（特定性）と `priority` を用いて最も詳細なエントリを選択する。`MCP Router` は、同一 `priority` かつ同じ制約数の `allow` が複数ある場合、最新の `policy_revision` を持つエントリを優先し、同一であれば設定ファイルで定義順に従う。すべてのステップで評価対象の `policy_id` と `approval_state` を `Audit Event` に含め、引数については個人情報を漏洩させない形式（デフォルトはハッシュ）で記録する。

候補が存在しない場合は暗黙的な拒否とし、`Audit Event` に `policy_id=null` を記録して運用側がログから不足を分析できるようにする。拒否レスポンスには `message` が含まれることを **SHOULD** とし、エージェントに明確な次の手順を促す。承認済みの `MCP Tool` 呼び出しは、`Authorization Policy` の `execution_context` が変化した場合（例: `config_revision` の更新、`Workspace Session` の切り替え）に再評価されるものとし、キャッシュされた承認ステートは無効化される。

#### 4.1.3 Approval State and User Flow
`approval_state` は、ユーザーまたは自動化の承認要件を示す属性であり、次の値を **MUST** または **SHOULD** で選択する。

- `auto_allow`: `MCP Router` は `allow` のみを返し、`Audit Event` に `approval_state=auto_allow` を記録する。明示的な `Audit Event` により、`App Host` が自動化の挙動を追跡できる。
- `requires_user_consent`: 呼び出しは `Shadow Buffer` を通じた差分（`Approval Unit`）と連携し、ユーザーの操作（`Workspace Session` 単位または `ファイル単位` の `Accept`）を要求する。呼び出しが最初に到達した際には `deny` 応答とともに `Audit Event` を `approval_state=pending` で生成し、UI に `Approval Request` を送る。ユーザーが `Accept` した後、`MCP Router` は再評価を行い、条件が変わっていなければ `approval_state=approved` の `allow` を返す。明示的な拒否がある場合は `approval_state=blocked` で `deny` し、`Audit Event` に記録する。
- `blocked`: 呼び出しは常に `deny` される。`Audit Event` は `approval_state=blocked` を含め、ユーザーおよびエージェントに理由を通知する。

手動承認の際、`App Host`/`UI View` は `policy_id`・`tool`・引数のハッシュ・`Execution Context`（`Workspace Session`/ブランチ）を含む `Approval Request` を表示し、ユーザーは `Shadow Buffer` の一覧に沿って `Approve` を選択する。承認が完了するとき、`MCP Router` は一時的な承認キャッシュを `policy_id` × `Approval Unit` × `execution_context` の組み合わせで保持し、`config_revision` か `Workspace Session` が変化した場合はこのキャッシュを無効化することを **MUST** とする。

ユーザーが `Accept` するまで、要求された `MCP Tool` 呼び出しはエージェントに対して明示的な `deny` として返され、エージェントは `Audit Event` で得た `message` を参照して再試行を抑制する。`requires_user_consent` のポリシーが `Shadow Buffer` の差分と紐づかない呼び出し（例: `run_command`）では、UI に `Approval Request` を表示し、`MCP Router` は `Audit Event` に `approval_unit=manual` で記録する。

#### 4.1.4 Authorization Denial Feedback Loop

`MCP Router` は、エージェントが繰り返し `deny` を受けたことで無限ループや過剰リトライに陥らないよう、明示的なフィードバックを提供する責務を持つ。

- 毎回の `deny` に対し、`Audit Event` に `resolution_hint` を追加し、`policy.message` を含む簡潔な拒否理由に加えて「変更を求める設定キー（例: `workspace.authorization.auto_pilot=false`）」「推奨 UI 操作（例: `Command Hub` の承認パネルを開く）」が記載されることを **MUST** とする。`resolution_hint` は問題の所在を特定できる識別子（例: `config_path:key`、`policy_id`）を含めることを **SHOULD** とし、ユーザーやエージェントが容易に参照できる形式とする。
- 同一の `agent_id`/`policy_id` に対する連続 `deny` 要求では、`MCP Router` が内部で `DeniedRequestHistory` を保持し、再試行可能となるまでの `retry_delay_seconds` を指数的にインクリメントして `Audit Event` に記録することを **MUST** とする。初期値 3 秒、最大 30 秒とし、`retry_delay` が有効な間は `Command Hub` や `Terminal` の UI で `Backoff` 表示を行い、ユーザーへ余剰な再試行を控える指示を出すことを **SHOULD** とする。
- `requires_user_consent` ポリシーの初回 `deny` は `pending` 承認リクエストとして扱い、`approval_request_id` を `Audit Event` に含める。承認処理が完了しない間、同一要求に再アクセスがあった際は再度 `deny` を返さず `pending` ステータスと `approval_request_id` を提示することを **SHOULD** とし、エージェントが `approval_request_id` をトリガーに再送を抑止できるようにする。
- 連続 `deny` が UI に反映されない場合、`MCP Router` は `Agent Status` を `Error` に遷移させ、`Audit Event` を通じて `App Host` に `Feedback Loop` 通知を送信することを **SHOULD** とする。`App Host`/`UI View` は `Workspace Rail` や `Command Hub` に「設定 {resolution_hint.config_path} を修正して再試行」等のバナーを表示し、関係する設定キーの `ConfigChangeEvent` を強調してユーザーが迅速に対応できるようにすることを **SHOULD** とする。
- `blocked` ポリシーで拒否された呼び出しは、その `policy_id` に対して `DeniedRequestHistory` を 24 時間保持し、同一エージェントからの再送を即時 `deny` することを **SHOULD** とする。その際 `Audit Event` には「管理者に {policy_id} の再承認を依頼」や「外部承認フローを含む `resolution_hint`」を含めて、再試行が無意味であることを明示することを **SHOULD** とする。

### 4.2 Shadow Buffer と承認フロー

`Shadow Buffer` はエージェントが行った差分編集を本バッファにマージする前に保持する構造体であり、各差分はファイル単位と `Workspace Session` 単位の両方で区分される。各差分には、発生時刻、差分の範囲、発行元エージェント、現在の `Agent Status` を付与することで、レビューと追跡が可能である。

1. **承認単位**
   初期リリースでは `Workspace Session` 単位の一括 `Accept` と `ファイル単位` `Accept` をサポートすることを **MUST** とし、続く項で `Reject`/`Revert`/`Partial Accept` の挙動を定義する。

2. **承認後の反映**
   `Accept` が行われると、`Shadow Buffer` の該当差分は `Editor Core` の本バッファへマージされ、`UI View` への更新イベント（差分の範囲・Agent Status）と `Audit Event` の両方を生成する。差分はマージ後に `Shadow Buffer` から削除され、ストレージに永続化されない。

3. **レビュー情報**
   `Shadow Buffer` の差分は `Galaxy View` と `Legacy View` の両方で列挙可能とし、特に `Galaxy Feedback` は対象ノードを `彗星` でハイライトし、設定された `Agent Status` に応じた輝度で表示する。差分ごとのメタ情報（例: `Approval Unit`、`Execution Context`）を UI で参照できること。

4. **追加承認操作**
   これらの操作は v1.0 以降の段階的拡張項目とする。ToDo セクション（Sec.8）で `Reject`/`Partial Accept`/`Revert` の差分状態遷移と `Audit Event` 記録ルールを整理し、実装段階で詳細化する。

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
   - 永続化失敗時、`App Host` は対象イベントを再送キューへ戻し、指数バックオフで再試行することを **SHOULD** とする。
   - `audit.queue.max` はデフォルト 2,048 件とし、`App Host` はキューが上限に到達した場合に最古のイベントを破棄することを **MUST** とする。この破棄処理について `Audit Event`（`result=queue_overflow`）と UI 通知を発行し、ユーザーが感知できる状態変化を示すことも **MUST** とする。破棄されたイベントの `event_id` は通知とログに含め、関連する `Workspace Session` へ再送されない。
   - 各 `Audit Event` は一意の `event_id` を持ち、再送時も同一 ID で管理され、完了後に再送フラグを消去することを **SHOULD** とする。

5. **監査表示**
   - `App Host` は `Legacy View`/`Galaxy View` の監査パネルで `Audit Event` を `Workspace Session` や `Agent Status`、`Approval Unit` でフィルタ可能とすることを **SHOULD** とする。

以上で、監査記録の生成・保持・匿名化・再送の責務が明示される。

### 4.5 MCP プロバイダのドメイン境界とツールライフサイクル

`MCP Router` は Workspace Session ごとにドメイン境界とツールの実行状態を管理し、プロジェクト外への逸脱および並列実行の競合を防ぐ責務を持つ。

#### 4.5.1 Workspace Context Enforcement

- `Workspace Session` は起動時に Canonical な `workspace_root` を決定し、`MCP Router` はすべてのツール呼び出しに対し `execution_context.workspace_root` を付与することを **MUST** とする。
- `run_command` を含むあらゆるファイル操作・実行操作は、CWD を `workspace_root` に固定し、`..` を含む相対パスや、シンボリックリンクを介して別のリポジトリやルートディレクトリへ遡るパス、あるいは `PATH`/`LD_LIBRARY_PATH` といった環境で明示的に外部実行環境を指定する変更を **MUST** 禁止する。逸脱が検出された場合、`MCP Router` は即時 `deny` として `Audit Event` を生成し、エージェントには `workspace_scope_violation` を理由として通知することを **MUST** とする。
- `FS` ドメイン（`apply_patch`/`read_file`など）のツールは `workspace_root` 内のノードのみを受け入れ、`canonical_path.starts_with(workspace_root)` が成立しない場合は `deny` とすることを **MUST** とする。
- `Workspace Config` で許可された `workspace_env` 以外の環境変数追加・上書きは認めず、`run_command` へ渡す `env` は `App Host` が定義した最小限の `workspace_env` + システムデフォルトに限定することを **SHOULD** とする。`Audit Event` には `execution_context.workspace_env` を含め、どの構成から環境が注入されたかを記録することを **SHOULD** とする。

#### 4.5.2 Tool Execution Queue and Lifecycle

- `MCP Router` は `ToolExecutionState` と呼ばれる構造体で各 `Workspace Session` の `tool_name`/`agent_id` ごとの状態を追跡し、同一セッションでの `run_command` の同時実行を防ぐことを **MUST** とする。`ToolExecutionState` には `state`（`Idle`/`Queued`/`Running`/`Completed`/`Failed`）、`agent_id`、`command_line`、`start_time`、`completion_time` を含める。
- `run_command` が到達したとき、該当セッションに `Running` 状態が存在しない場合は即座に `state=Running` へ遷移する。既に `Running` が存在する場合は `ToolRequestQueue` へ FIFO で追加し、`state=Queued` の `Audit Event` を `result=queued` / `queue_reason=tool_busy` で生成することを **MUST** とする。
- `ToolRequestQueue` は `tool.execution.queue_max_pending`（未設定時は `4`）を上限とし、上限に達した状態で追加要求を受けた場合は `deny` し `Audit Event` に `result=queue_overflow` を記録、エージェントには「先行する `run_command` の完了を待つか中断する」旨の `message` を返すことを **MUST** とする。
- `Running` が終了すると、`ToolRequestQueue` から先頭のエントリを取り出して `state=Running` へ遷移させ、`Audit Event` に `result=queue_start` を生成する。ユーザー UI には `Terminal` 上で `Queued` バッジや `Command Hub` の `Action Mode` で待機中候補を表示するように **SHOULD** 定義する。
- セッション終了・シャットダウン・ツール失敗時には、残存する `Queued` エントリを `result=canceled` として `Audit Event` に記録し、該当するエージェントへ `deny` を返す。`ToolExecutionState` はセッション破棄時に初期化されることを **MUST** とする。
- 上記の `ToolExecutionState` と `ToolRequestQueue` の変更はすべて `Audit Event`（`tool_state`/`queue_length`/`workspace_session_id`/`agent_id`）として記録し、`Shadow Buffer` や `Terminal` との整合性を保つようにすることを **SHOULD** とする。

## 5. Configuration (設定管理) モジュール

設定は階層的にマージされ、常に最新の状態が各コンポーネントへリアクティブに反映される。各構成要素には優先順位と更新可否が定義されており、既存の `ConfigChangeEvent` を介して差分を伝播させる。

### 5.1 Source hierarchy and merge priority

1. **環境変数 (Priority: 1)**: `NUE_AGENT_SAFETY_AUTO_ACCEPT=true` など。起動時に読み込まれるため、`App Host` は再評価のたびにこのスコープを最優先でマージする。
2. **ワークスペース設定 (Priority: 2)**: `.nue/config.yaml`（プロジェクト固有）。ファイル更新を検知したタイミングで再読み込みする。
3. **グローバル設定 (Priority: 3)**: `~/.config/nue/config.yaml`（ユーザーの基本設定）。同一ユーザーの複数ワークスペースにまたがる変更を検知する。
4. **デフォルト (Priority: 4)**: システム内蔵の初期値。常に最後のフォールバックとして保持される。

### 5.2 変更検知と再評価

`App Host` はグローバル設定とワークスペース設定のファイル変更をファイルシステムイベント（例: kqueue/inotify/ReadDirectoryChangesW）で監視し、変更完了から 5 秒以内に再評価サイクルを開始する。このサイクルで、`App Host` は対象ファイルを再パースし、既存の設定スキーマに対して構文・バリデーションチェックを行う。パースに失敗した場合は既存の設定を保持し、該当事象を `Audit Event`（`type=config.reload.failure`）として記録し、ユーザーへ修正を要求する通知を出す。

変更が正当であれば、`App Host` は新しい設定と先行設定との差分を計算して単調増加する `config_revision` をインクリメントし、`ConfigChangeEvent` を生成する。`ConfigChangeEvent` は `source`（Global/Workspace）、`revision`、`changed_keys`、`previous_values`、`hot_reloadable` フラグを含み、待機中の `Workspace Session` および `AI Agent` に配信される。`App Host` は `config.reload` という UI コマンドを提供し、ユーザーが手動で再評価を要求した際もこのサイクルを再利用する。

環境変数の変更はプロセスの起動時に固定されるため、`App Host` はランタイム中に間接的な検知手段を持たない。したがって、環境変数ベースの設定を変更する場合、ユーザーは `App Host` を再起動しなければならず、`App Host` は再起動を伴う変更を要求するアラート（再起動後に `config_revision` を再生成）を表示することを **MUST** とする。

### 5.3 伝播と適用制御

`Workspace Session` は自セッションに関係する `ConfigChangeEvent` を購読し、受信から 2 秒以内に適用を試行する。`ConfigChangeEvent` に含まれる各 `changed_key` にはメタデータとして `hot_reloadable`（`true`/`false`）が付与されており、`false` の場合は再起動なしには適用できない旨を示す。`Workspace Session` は `hot_reloadable=true` のキーについてのみ `Editor Core`・`MCP Router`・`Terminal Emulator` 等へ新値を反映し、`hot_reloadable=false` のキーは再起動が完了するまで旧値を保持してユーザーに通知する。通知には変更内容と再起動コマンド（例: `Restart App Host`）を含め、`App Host` が再起動済みであることを確認した後に `config_revision` を新しい値に合わせる。

`Workspace Session` は `ConfigChangeEvent` に `hot_reload_scope` を含め、関連する UI/サービスを限定的に再初期化する。たとえば、`MCP Router` のポリシー定義変更は `hot_reload_scope=router` となり、当該スコープ内のコンポーネントにのみ更新通知を送る。

#### 5.3.1 `hot_reload_scope` の許容値と依存順序

`hot_reload_scope` は列挙値 (Enum) として次の値のみを許容し、その意味と再初期化の責務を明示することを **MUST** とする。これにより `ConfigChangeEvent` の処理側が依存関係を理解した上で一貫した再初期化を行う。

| 値 | 意味 | 説明 |
| --- | --- | --- |
| `app` | グローバル構成 | App Host や Workspace Rail、通知系の基本設定。 |
| `router` | 認可ポリシー | MCP Router、Authorization Policy、`external_agent_profiles` などの Gateway 設定。 |
| `terminal` | ターミナル | PTY/スクロールバック/エンバイロメントなどの Terminal Emulator 設定。 |
| `editor` | エディタ挙動 | フォント、レンダリング、Shadow Buffer の挙動。 |
| `semantic` | 意図解釈 | `nue-semantic` のモデルパス・パラメータ・Local RAG インデックス。 |
| `agent` | エージェント接続 | Codex 等の外部エージェントエンドポイント、プロンプトテンプレート。 |

同一の `ConfigChangeEvent` が複数のスコープを含む場合、`App Host` は「Dependency-Aware Re-init Sequence」に従って下位レイヤーから上位レイヤーへ順次再初期化を行うことを **MUST** とする。順序は `app` → `router` → `terminal` → `editor` → `semantic` → `agent` で、各スコープは前工程の完了を待ってから初期化を開始し、失敗した場合は直ちに `Audit Event` (`type=config.reload.failure`, `hot_reload_scope=<scope>`) を発行してユーザーに通知する。

`ConfigChangeEvent` に未知の `hot_reload_scope` が含まれていた場合、`Workspace Session` はその変更を再初期化不能 (`hot_reloadable=false`) と判断し、直ちにユーザーに再起動を要求する通知を出すことを **MUST** とする。加えて `App Host` は `Audit Event` (`type=config.reload.unknown_scope`, `unknown_scope=<value>`) を記録し、該当 `ConfigChangeEvent` を保持して再起動完了後に再評価する。

### 5.4 フェールセーフと監査

`App Host` はすべての再評価サイクルを `Audit Event`（`type=config.reload` 以上）として記録し、`Workspace Session` に配信した `config_revision` を含めて `Legacy View`/`Galaxy View` の監査パネルから追跡できるようにする。設定の差分を適用できなかった場合（例: 検証エラー、`Workspace Session` が遅延したコンポーネント）、`App Host` は 既存の設定を再登録し、該当した `ConfigChangeEvent` について `Audit Event` を `config.reload.failure` として二重記録し、ユーザーへ修正指示を送る.

再評価時に `audit.queue.max` を超過するような連続的な失敗が発生した場合、`App Host` は最も古い `ConfigChangeEvent` を削除し、削除されたイベントの `event_id` を含む通知と `Audit Event` を生成することを **MUST** とする。削除前には少なくとも 30 秒の猶予を設け、その間にユーザーが手動で再適用できるようにする。

---

## 6. AI共創ワークフロー：The Nue Loop

1. **インテント入力**: ユーザーがUIから指示を出す。
2. **エージェント起動**: UIがCodex等へタスクを丸投げ。
3. **MCP実務**: エージェントが `fs` や `runtime` ツールを駆使。
4. **Shadow Buffer**: Coreはエージェントの編集を「未承認の差分」として保持。
5. **Galaxy Feedback**: Galaxy View上で、AIが編集しているファイルが激しく発光（パルス）。
6. **人間の承認**: ユーザーが差分を確認し、`Accept`。変更が本番バッファへマージされる。

## 6.1 Command Hub と Intent/Smart Search

`Command Hub`（コマンドパレット）は、ユーザーが自然言語やショートカットを使って操作を起点とする中心UIであり、AIエージェントと人間が同じテンポで「意図」を共有するためのインターフェースである。

### 6.1.1 Command Hubの構造

- `Command Hub` は `Cmd + Shift + P`（macOS）または `Ctrl + Shift + P`（その他）で呼び出すモーダルオーバーレイで、アクション候補と履歴を一覧表示することを **MUST** とする。
- `Command Hub` は次の入力モードをサポートし、それぞれで優先的な候補生成を行うことを **MUST** とする。
  - `>` プレフィックス（Action Mode）: 明示的な `MCP Tool` 実行、設定変更、UIコマンドを記述する。例: `> Terminal: Split Terminal`。
  - `:` プレフィックス（Navigation Mode）: ファイル名・シンボル名によるナビゲーション。例: `:src/lib.rs`。
  - プレフィックスなし（Intent / Smart Search）: 自然言語（英語）で意図を入力し、`nue-semantic` による候補推論を得る。例: `test database connection` や `document API changes`。
- モードはリアルタイムに切り替わり、入力中のテキストに応じて候補リストを 16ms 以内に更新することを **SHOULD** とする。
- 16ms 以上の遅延が発生した場合、`Command Hub` は直前に表示していた候補を維持しつつ `Backoff` 状態を表示することを **MUST** とし、この状態では `nue-semantic` による再スコアリングが継続される旨とともに `Action Mode`/`Navigation Mode` への移行案を明示し、再評価完了後に最新候補が置換されるようにする。
- 各候補には発行元（AIエージェント/ユーザー）、必要な `MCP Tool`、`Approval State`（`auto_allow`/`requires_user_consent`/`blocked`）を付与し、選択時に即座に `Audit Event` を作成することを **SHOULD** とする。
- 選択された候補は `MCP Router` へ `Intent Request` を送信し、`approval_state` に応じて自動的に処理されるので、`Command Hub` は `Audit Event` 経路を共有して `Shadow Buffer` との連携を疎通させることを **SHOULD** とする。

### 6.1.2 Intent/Smart Search のコンポーネント

- Intent/Smart Search は **`nue-semantic`** というローカル生成AIエンジンを中心とし、Phi やその他の Small Machine Learning（SML）モデルをバインドして動作することを **MUST** とする。外部APIは基本的に利用せず、オフライン環境でも動作する必要がある。
- `nue-semantic` は次のサブシステムを組み合わせて候補を生成することを **MUST** とする。
  - **Intent Resolver（局所意図変換）**: ユーザーの自然言語入力をトークン化し、`MCP Tool` 実行やUIアクションにマッピングするミリ秒スケールの推論モジュール。
  - **Local RAG（Local Retrieval-Augmented Generation）**: プロジェクト内のファイル名、関数名、設定名をベクトル化または重み付けしたインデックスで保持し、曖昧な入力に対して意味的に関連する候補へ橋渡しする。
  - **Policy-Aware Scoring**: `Authorization Policy` に定義された `argument_constraints`/`execution_context` を照合し、実行可能な候補のみを上位にソートする。
- `nue-semantic` は、候補の生成・表示・選択を 100ms 以内で完了させるように設計され、遅延が発生する場合は進行中の推論を UI 上でステータス表示することを **SHOULD** とする。
- `nue-semantic` が現在のコンテキストだけでは実行不可能（例: セキュリティ上の制限や外部リソースへの依存）と判断した場合、`Command Hub` はユーザーへ外部エージェント（例: 高性能クラウドAI）への問い合わせを提案し、その提案は `approval_state=requires_user_consent` として `Audit Event` に記録されることを **SHOULD** とする。
- 上記の提案は `nue-semantic` が内部リソースで解決できない場合の最後の手段とし、`Command Hub` は外部エージェントの選定ポリシーとして `requires_user_consent` かつ `Audit Event` で明示されるプロファイル（例: `external_agent_profile=cloud_lambda_v2`）に限定することを **MUST** とする。提案先候補が存在しない場合は `Command Hub` が「現在のコンテキストでは解決不能」として終了レスポンスを返すことを **SHOULD** とする。
- `Intent/Smart Search` は候補の選択時に `MCP Router` への `run_command` や `apply_patch` の呼び出しを発生させる実行プランを返し、その過程で `Shadow Buffer` の差分として登録されるエントリと整合することを **MUST** とする。
- `Local RAG` に使うインデックスはファイルシステムの変更（追加/削除/リネーム）を検知した後 5 秒以内に部分更新し、入力ミスや類似語を許容するキーワードマッチを備えることを **SHOULD** とする。

以上により `Command Hub` が意図を中心とした起点となり、AIエージェントと人間が共に進化するループの起点として機能する。

---

## 7. フロントエンド・デザイン実装プロセス

デザインは、AIが操作可能な形式で管理する。

* **ツール**: **Penpot**（オープンソース・SVG/CSSベース）。
* **手法**:
1. プロジェクト内にUI定義ファイル（CSS/SVG）を置く。
2. Codex（エージェント）にMCP経由でそのファイルを編集させ、デザインの微調整（色、レイアウト）を行わせる。
3. 確定したデザイン数値を `nue-ui` の GPUI 定義に落とし込む。
