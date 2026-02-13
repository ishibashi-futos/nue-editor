# 次世代AIネイティブ・エディタ「Nue」仕様書

## 1. Summary

### 1.1 プロジェクト・ビジョン

Nueは、従来の「コードを書くためのツール」を超え、AIエージェントが能動的に作業し、人間が高密度な視覚情報を通じて監督・承認するためのエージェント・オーケストレーション環境である。

### 1.2 プロダクトの提供価値

AIエージェントツール（CodexやClaude Codeなど）と高度に連携し、開発速度と人間の判断品質を向上させるため、エディタ内にAIエージェントツールとの連携を色濃く反映させる。MCPによる接続と、`Shadow Buffer`によって、変更を追従しつつ、「AIによる変更」を確実にレビューできるようにすることで開発品質を高める。

### 1.3 システム全体像（高レベル）

Nueは `App Host` を最上位に持ち、複数の `Workspace Session` を独立実行単位として管理する。各セッションは AI Agent、MCP Router、Editor Core、UI View を内包し、ワークスペースごとの分離と高速な切り替えを保証する。

### 1.4 スコープ / 非スコープ

- スコープ:
  - 高機能エディタとしての機能
    - ワークスペース管理
    - ファイルツリー、VCS連携
    - ターミナル機能
    - コマンドパレット機能とLLMを利用したコマンドサジェスチョン
    - Sleep/復元
    - 検索
  - AI恊働機能: MCP 認可、Shadow Buffer 承認
- 非スコープ
  - 高度なアクセシビリティ対応の全面実装
  - AIエージェント自体の実装
  - 日本語、英語以外の多言語対応

## 2. Core Concepts

### 2.1 実行モデル: App Host と Workspace Session

`App Host` はライフサイクル、設定、監査、通知を統括し、`Workspace Session` は AI 実行・編集・UI 表示を分離して処理する。これにより、複数プロジェクトを安全に並行管理できる。

### 2.2 承認モデル: Approval Unit / Shadow Buffer / Audit Event

AI の変更は直ちに本バッファへ反映せず、`Shadow Buffer` に格納される。ユーザーは `Approval Unit`（workspace/file/hunk）単位で承認操作を行い、すべての操作は `Audit Event` として追跡される。

### 2.3 操作モデル: Command Hub / Intent / Smart Search

`Command Hub` はアクション実行、ナビゲーション、自然言語意図の入口である。`nue-semantic` は意図解釈と検索候補生成を担い、承認状態と連動した実行可能な提案のみを前面に出す。

### 2.4 境界モデル: MCP Router と Provider のドメイン境界

`MCP Router` はツール呼び出しの唯一の入口として認可・拒否・再試行制御を行う。ツールは `workspace_root` 境界内で実行され、境界逸脱は即時 deny として監査記録される。

### 2.5 設定モデル: Source hierarchy / hot reload scope / 伝播制御

設定は環境変数、ワークスペース設定、グローバル設定、デフォルトの優先順でマージされる。`ConfigChangeEvent` は `hot_reload_scope` を持って配信され、依存順序に従って段階的に再初期化される。

### 2.6 継続性モデル: セッション復元 / Sleep / リソース管理

`SessionSnapshot` は開いていたタブ、差分状態、実行キューを保持し、Sleep からの復帰時に UI と実行状態を再構築する。これによりリソース節約と作業継続性を両立する。

## 3. Detailed Spec

### 3.1 UI/UX デザイン仕様

#### 3.1.1 ビジュアル・アイデンティティ

* **テーマ**: 「Neon-Night」（深いネイビーを基調に、シアン、マゼンタの発光アクセント）。
* **描画エンジン**: Rust `GPUI` によるGPU加速。60fpsのスムーズなアニメーションとスクロール。

##### 3.1.1.1 カラーパレット

Nue は UI レイヤーごとに以下の色を採用し、役割ごとの区分を明示する。

| カテゴリ | 色名 | HEX | 主な配置 | 備考 |
| --- | --- | --- | --- | --- |
| Background | Deep Abyss | #0B0E14 | エディタのメイン背景。最も暗いレイヤー。 | レイヤー間の明度差を強調し、深度感を演出する。 |
| Surface | Space Grey | #1A1D23 | サイドバー、タブ、パネルの背景。Base より一段明るい。 | 入れ子構造で重なりを示すため、この色をベースに陰影をつける。 |
| Border | Midnight Glass | #2D323C | パネル境界線、セパレーター。 | 他パーツが浮き上がるように線の不透明度を調整する。 |
| Primary (AI) | Neon Cyan | #00F5FF | AI 活動中のハイライト、エージェントのステータス表示、ミニマップの AI 位置。 | 光に近い強い彩度を持ち、Pulse アニメーションと併用することで注目を集める。 |
| Success / Accept | Electric Lime | #32FF7E | 承認ボタン、成功通知、保存済インジケーター。 | 過度な頻出を避けつつ「安全/確定」を意味するヒュー。 |
| Warning / Wait | Solar Flare | #FFF200 | ユーザー入力待ち、未承認差分のガター、警告アイコン。 | 点滅アニメーションと組み合わせ、緊急性と待ち時間の両方を表す。 |
| Error / Alert | Cyber Magenta | #FF006E | ビルドエラー、認可拒否、UI 上のノードや通知の異常振動表示。 | オペレーションを即座に中断させるため、Flash アニメーションと併用する。 |
| Information | Ether Purple | #BF5AF2 | LSP 型情報、シンボル定義、依存線などの補助的情報。 | 情報提供的な補助要素で使用。 |
| Text (Main) | Cloud White | #E4E7EB | 標準テキスト、コード文字。 | 背景とのコントラストが十分なことを確認する。 |
| Text (Muted) | Dusty Grey | #717984 | コメント、無効 UI、パンくず。 | 帯状背景や低重要度テキストに使用する。 |

各色は直線的に使い回しを避け、視覚上の階層を保持するために `Neon Cyan` / `Cyber Magenta` / `Solar Flare` 等の高彩度色は限定的なハイライトやステータス表示にのみ使用することを **SHOULD** とする。

##### 3.1.1.2 エージェント状態と差分表示の色規約

`Agent Status`（Busy/Waiting/Error/Idle）は UI 上で以下の色/エフェクトで表現し、状態遷移の判別を利用者が瞬時に行えるようにすることを **MUST** とする。

- `Busy`: `Neon Cyan`（#00F5FF）を 0.4〜1.0 の Opacity でパルス状に発光させ、AI が計算中であることを強調する。
- `Waiting`: `Solar Flare`（#FFF200）で低速点滅し、ユーザーの承認や入力待ちを表示する。
- `Error`: `Cyber Magenta`（#FF006E）で鋭い明滅アニメーションを伴い、`Workspace Rail` や `Command Hub` にエラーバナーを表示する。
- `Idle`: `Dusty Grey` や `Cloud White` のデュアルトーンで落ち着いた表現とし、他状態と禁止線を明確にする。

差分（Diff）と承認可視化については、以下のルールを **MUST** とする。

- 未承認の行（Gutter）には `Solar Flare` の縦線を表示し、`Command Hub` や承認パネルなどでも同色で強調する。
- 承認済み/確定行は `Cloud White` に馴染ませて Gutter 表示とテキストの彩度を段階的に落とす。
- 承認ボタンや完了インジケーターは `Electric Lime`（#32FF7E）でグロー効果を付与し、視覚的な “完了” を伝える。

これらのカラールールは `nue-ui` の GPUI 定義に反映し、パレットを変更する場合はいずれの用途が影響を受けるかを追跡できるようデザインシステムに記録することを **SHOULD** とする。

アクセシビリティや高彩度モードといった特別対応は、本アプリケーションが開発者個人の利用を想定しているため、主要仕様としては取り扱わない。ただし、暫定的な検討内容やトリガー候補は `specs/spec_accessibility.md` でまとめており、必要に応じてそちらを参照する。

#### 3.1.2 ワークスペース・レイル

各アイコンにエージェントの**ライブステータス**を表示する。

* **Busy (Cyan Pulse)**: AIが思考・コード生成・ビルド中。
* **Waiting (Yellow Blink)**: ユーザーの承認や入力を待機中。
* **Error (Magenta Vibration)**: ビルド失敗や例外発生。
* **Idle (Dimmed)**: 待機状態。

#### 3.1.3 エクスプローラー（Legacy View）

ヘッダーのトグルボタンにより複数モードを準備する設計ではあるが、初期リリースでは Legacy View を中心に据え、ディレクトリツリーを活用したファイル検索と構造把握を重視する。Legacy View は従来型の階層表示として、ファイル/フォルダの展開・折りたたみ・フルテキスト検索を低遅延で提供し、ユーザーが物理構造と論理構造を素早く行き来できるようにする。

- **Legacy View**: 階層的なディレクトリツリーを基盤とし、ファイルの開閉・パス表示・差分のマーカー付与を行う。Agent Status に応じたハイライトや承認済みラインの彩度調整などはこのビューで完結する。
- **Galaxy/Nebula View**: `Nebula`/`Galaxy View` に関する機能は将来的に段階的導入される計画であり、以降のバージョンで必要な要件とヒューリスティックを別途 `specs/spec_galaxy_view.md` にまとめている。本仕様では意図的に、このビューの詳細な描画要件やパフォーマンスルールを除外する。

#### 3.1.4 タイポグラフィ / フォント埋め込み / FontContext

Nue の UI は背景のダークトーンとネオン系アクセントのコントラストの中で文字の可読性を確保することが不可欠である。フォントの種類・埋め込み・利用パターンは以下の要件を満たすことを **MUST** とする。

##### 3.1.4.1 標準フォントと利用領域

- `JetBrainsMono-Regular`（標準コード用）: エディタ本文、差分ラベル、コマンドリストなど主要テキストはこのフォントを優先的に使用することを **MUST** とする。等幅かつリガチャに偏りがない字形を選び、高彩度背景でも文字の輪郭が鮮明に見える存在感を維持する。
- `JetBrainsMono-Bold`（キーワード・強調）: AIの提案や `Command Hub` の操作候補、`Shadow Buffer` のヘッダーには太字を用いて視線を誘導することを **SHOULD** とする。強調項目が枠線や色に埋もれないよう、太さと間隔のバランスが保たれるように設定する。
- `JetBrainsMono-Italic`（メタ情報・コメント）: ステータス注釈、コメント、控えめな説明文には斜体を使い、ノイズ感を下げることを **SHOULD** とする。`Dusty Grey` との組み合わせで、「補足」や「参照」を明示する。
- これらフォントがカバーしきれないスクリプト（CJK/右から左など）に対しては、`FontContext` のフォールバック設定を通じて OS 由来の信頼できるフォントを利用することを **SHOULD** とする。フォールバック順序は `JetBrainsMono` 系列 → グローバル設定の `fallback_fonts` → OS の等幅フォントの順とし、どのフォントが実際に選ばれたかを `FontContext` がトレースできるよう属性を記録する。
- メニューやサイドバーなど UI レイヤーのテキストは英語のみを前提とし、`JetBrainsMono` 系フォントの埋め込みだけで描画することを **SHOULD** とする。日本語や絵文字を含むテキストは UI で避け、どうしても必要なときは OS フォントに頼らず `FontContext` が埋め込んだ JetBrains Mono を再利用して表示トーンを揃えることで視覚的一貫性を守る。
- エディタ本文では JetBrains Mono と混在する日本語や特殊文字を `fallback_fonts`（例: Menlo, Monaco, Courier New）で補完し、`FontContext` がそれらを優先的に選択するよう設定することを **SHOULD** とする。日本語グリフは若干縮小（0.5〜0.9 倍）し、ベースラインを -2〜-4 px 程度下げる補正を施すことで「日本語だけが浮く」印象を軽減し、Emoji/記号は OS のカラーフォントに任せる。この混在状態でも `FontContext` が実際に選択したフォント名をログやデバッガで参照可能にしておくことを **SHOULD** とする。

##### 3.1.4.2 埋め込みと `FontContext` 登録

- `Nue` は `nue-ui`（GPUI）の初期化時点で `JetBrainsMono-Regular`/`Bold`/`Italic` をバイナリに埋め込み、そのバイト列を `FontContext` に登録することで、オフライン環境や制御されたランタイムでも同一の字形を保証することを **MUST** とする。
- 埋め込みフォントにはライセンスパッケージ（例: JetBrains Mono の SIL Open Font License）を同梱し、バイナリ内で `FontContext` Bundled Font Catalog にメタ情報（ファイル名・ライセンス）を付与することを **SHOULD** とする。
- `FontContext` への登録は、UI 初期化の最初のフレームより前（`nue-ui` の `FontContext::register` 呼び出しの完了前）に終えておくことを **MUST** とする。フォント名のキーは `nue-font::text`, `nue-font::emphasis`, `nue-font::meta` のように命名し、描画レイヤーがキーを参照して一覧できるようにする。
- 上記フォントを置換したい場合は、`FontContext` の `override` API を通じて別の `FontHandle` を挿入し、`Shadow Buffer` や関連する UI レイヤーはフォントキーを変えずに差し替えられる仕組みを維持することを **SHOULD** とする。

#### 3.1.5 エディタ補助パネル（Minimap / Structure Path / Smart Gutter）

`Minimap` はエディタ右側に配置される 1px スケールの高解像度ファイルプレビューであり、`Editor Core` のバッファ・`Shadow Buffer`・`Git` 差分・`Command Hub` 検索結果・ビルド/テスト出力を統合して「ファイル全体の健康状態」を常に表現することを **MUST** とする。

##### 3.1.5.1 Minimap

- `Minimap` は GPU 補完描画（`GPUI` インスタンシング）を用いて 60fps を維持しつつ、1 フレームの間にすべての行/スコープを縮小表示し、`Editor Core` の変更・`Shadow Buffer` の差分・`Command Hub` の候補・`Audit Event` による検出をリアルタイムに反映することを **MUST** とする。描画対象は現在アクティブなバッファで、更新は同一 API でイベント駆動される。
- マウス/タッチによるスクロール同期（`scroll_sync=true`）を提供し、ビュー移動時に `Command Hub` の `Navigation Mode` へのヒントを出しながらラグが生じた場合は直近位置を保持する `Backoff` 表示を行うことを **SHOULD** とする。ドラッグ操作のリリース時には `Editor Core` の `line_offset` にジャンプし、`Shadow Buffer` の `focus_id` を光らせることで次の差分位置へフォーカスを提供する。
- 表示領域には次のオーバーレイを必ず重畳することを **MUST** とする。
  - `Cyber Magenta` の帯でコンパイル/ビルド・LSP エラー（`Agent Status=Error`）を示し、`Audit Event` の `result=deny` に同期させて点滅し、ユーザーに即時の修正を促す。
  - `Solar Flare` の点またはストロークで `Command Hub` や `Legacy View` の検索・シンボルハイライト結果を示し、モード（ファイル内/グローバル）に応じて点の密度・強度を動的に変化させる。
  - `Git` 差分と `Shadow Buffer` 差分を区別するため、`Midnight Glass` 系の細線で Git 差分を、`Neon Cyan`/`Electric Lime` でAI 由来差分（`Shadow Buffer` エントリ）を描き、AI 差分には `Agent Status=Busy` のパルスアニメーションを付与して選択中の `Approval Unit` を浮き立たせる。
  - `Partial Accept` や `Revert` 中の行は `Solar Flare` でフラッシュし、`Shadow Buffer` 側の `focus_id` とリンクした `Audit Event`（`result=partial_accept`/`revert`）の生成と同時に色が減衰することを **SHOULD** とする。
- `Minimap` のクリック/タップ操作は `Command Hub` の `Approval Requests` と `Legacy View` のツリーを連動させ、該当差分を開いて確認・承認できるようにすることを **MUST** とする。
- `Minimap` の表示更新は `Audit Event` (`type=minimap.overlay` など) を発行し、検索/ビルド/差分イベントが UI へ提示されたことを記録して監査できるようにすることを **SHOULD** とする。

##### 3.1.5.2 `focus_id` の生成と同期

`focus_id` は `Minimap`、`Structure Path`、`Smart Gutter`、`SessionSnapshot`、および `Command Hub` が共有する差分追跡の統一識別子であり、`Shadow Buffer` に差分エントリ（ハンク）が登録されたタイミングで生成される **MUST** とする。焦点となる差分はハンク単位（差分塊）で記録し、**MUST** で一意な値を割り当てる。

- `focus_id` の生成は次の構成要素を含む ULID 形式とし、`Shadow Buffer` はエントリ保存時に以下をハッシュ化して基底値とすることを **MUST** とする。
  - `file_path_hash`: ファイルパスの安定ハッシュ（64bit）により同じファイルを示す。
  - `hunk_start_line` / `hunk_end_line`: 差分の開始行・終了行。
  - `hunk_checksum`: 変更前後のスニペットの CRC32 や SHA1 を使ったダイジェスト。
  - `timestamp`: 差分登録時のタイムスタンプ（UTC）。

- 上記要素の組み合わせで生成される ULID を `focus_id` とし、`Shadow Buffer` は同じハンクを示す限りは同じ `focus_id` を再利用し、行番号や内容が変化して新たな差分塊が生まれた場合は新しい `focus_id` を再発行することを **MUST** とする。`focus_id` の更新は `Audit Event` の `focus_id` フィールドにも反映し、変更前後のマッピングを `related_event_id` で追跡できるようにすることを **SHOULD** とする。

- 各 UI コンポーネント（`Minimap`/`Structure Path`/`Smart Gutter`/`Command Hub`）は、該当差分を描画・ハイライトする際に上記 `focus_id` を参照し、同一の `focus_id` を持つ差分は同じ視覚的状態になるよう同期を取ることを **MUST** とする。差分が承認/拒否された場合は 50ms 以内に関連する UI すべてで強調表示を解除し、`SessionSnapshot` もその `focus_id` を削除することを **SHOULD** とする。

- `SessionSnapshot` は `focus_id` と `Approval Unit` のペアを記録し、ワークスペース再開時には該当 `focus_id` を `Shadow Buffer` 内で再解決して UI を再構築することを **MUST** とする。`focus_id` が見つからない場合はその差分が既に処理済みと判断し、保持していた `SessionSnapshot` エントリを破棄することを **SHOULD** とする。

- `focus_id` は `Shadow Buffer` エントリが完全に `Accept`/`Reject` された時点で無効化され、同じロジックの差分が再度生成された場合は新しい `focus_id` を再作成することを **MUST** とする。このライフサイクルを厳密に管理することで、`Audit Event` やログが古い差分と紐づき続けるのを防ぎ、UI からの参照整合性を維持することができる。

##### 3.1.5.3 Structure Path

`Structure Path` はエディタ上部に常設される階層ナビゲーションバーであり、「Project > Folder > File > Class/Module > Method/Function」のパスを常に表示し、変更のコンテキストと承認状態を一目で把握できることを **MUST** とする。

- **更新トリガー**: エディタのアクティブバッファ、カーソル位置、`Shadow Buffer` 内の差分および `Command Hub` の現在候補が変更された際、`Structure Path` は 100ms 以内に再評価され、存在する差分・承認待ち・実行中のエージェントフローを反映した状態にすることを **SHOULD** とする。
- **表示/マーキング**: 各セグメントには `Agent Status` に応じた `Glyph` と `Color` を付与する。AI 差分を含むセグメントは `Neon Cyan` のパルス、承認待ちは `Solar Flare` のドット、Git 差分のみは `Midnight Glass` の下線、エラー状態は `Cyber Magenta` のバッジ、といった視覚的強調を付ける。高彩度が困難なモードでは `Glyph` とラベルで同じ意味を伝えることを **MUST** とする。
- **相互作用**: セグメントクリックで `Legacy View`/`Command Hub` の `Navigation Mode` が対応箇所へジャンプし、Shift+クリックなどの複数選択操作で `Command Hub` に `Intent Request` を出して範囲選択状態を生成することを **SHOULD** とする。キーボードショートカット（`Alt+1` ～ `Alt+5` など）も提供する。
- **Shadow Buffer との連携**: `Structure Path` は `Approval Unit`（ファイル/ハンク）ごとの `Audit Event` を参照し、未承認のパスに `Solar Flare` の点滅、承認済のパスには `Electric Lime` のチェックマークを表示する。`Partial Accept` で分割された範囲にはサブセグメントを展開して `related_event_id` を追跡できる形にすることを **SHOULD** とする。
- **Command Hub / Minimap との同期**: `Command Hub` が `Backoff State` 中は `Structure Path` が最終確定候補を保持しつつ `Re-scoring…` ラベルを表示し、`Minimap` の `focus_id` と連動して現在の差分セグメントを強調表示することを **SHOULD** とする。差分の承認/拒否後は 50ms 以内にパス上の状態を更新し、UI の整合性を維持することを **MUST** とする。

`Structure Path` は `Legacy View`・`Minimap`・`Command Hub` のいずれのモードにおいても現在位置と未処理差分をつなぎとめる役割を果たし、ユーザーが自身の作業対象を見失わないよう一貫したナビゲーション体験を提供することを **MUST** とする。

##### 3.1.5.4 Smart Gutter

`Smart Gutter` は、エディタの行番号領域の隣に配置されたコンテキスト情報表示領域であり、`Shadow Buffer` 上の AI 差分、`Git` 差分、`Agent Status`、および `Audit Event` の状態を一目で識別できるようにすることを **MUST** とする。`Smart Gutter` は `Structure Path`/`Command Hub`/`Minimap` と双方向で同期し、線の左右どちらに差分が存在するかや承認リクエストの焦点がどこにあるかを明確に伝える。

- `Smart Gutter` は、該当ラインに関連付けられた `Shadow Buffer` エントリが存在する場合、ライン左に縦に伸びる `Solar Flare` の `AI Pulse Indicator` を表示し、`Agent Status` に応じてパルスの `Opacity`/`Motion` を変化させることで、AI のアクティビティを可視化することを **MUST** とする。`Solar Flare` の点滅は `Agent Status=Waiting` の `Approval Request` を、持続的な `Neon Cyan` の発光は `Busy` な生成処理を、それぞれ示す。
- `Smart Gutter` は、`Shadow Buffer` の差分が `accept` されると `Electric Lime` の細いグロー表示へ滑らかに遷移し、`Audit Event` の `result=accepted` を反映することで、承認済みの行を行番号と共に淡色化して視覚的に完了を伝えることを **MUST** とする。`result=denied` あるいは `partial_accept` 状態では `Solar Flare` と `Midnight Glass` の二重線を用いて、再レビューが必要な行として強調することを **SHOULD** とする。
- `Smart Gutter` は、`Git` 差分（追加/変更/削除）がある行に対して `Midnight Glass` の図形（追加: 上向き三角、削除: 下向き三角、変更: 横のバー）を描き、`Shadow Buffer` の AI 差分表示と重なった場合は `AI` 表示を優先しつつ `Git` 残差として細い輪郭を並列表示することで、同一行の両要素を区別できるようにすることを **SHOULD** とする。
- `Smart Gutter` のインジケーターをユーザーがクリックまたはキーボードフォーカスした際、`Command Hub` の該当 `Approval Request` を呼び出し、`Shadow Buffer` エントリの差分詳細（変更前後のスニペット・`Audit Event` 参照）と共に `Command Hub` 内で `Accept` を実行できる操作フローを **MUST** 提供する。`Smart Gutter` が `Command Hub` へ渡す `approval_unit`/`focus_id` は `Audit Event` の `related_event_id` と一致させ、クリック後 50ms 以内に `Command Hub`/`Minimap`/`Structure Path` のハイライト状態を更新することを **SHOULD** とする。
- `Smart Gutter` は `Audit Event` の `result=pending` / `result=queued` など、まだ `Approval` が完了していない行について `Neon Cyan` の進捗リング（1行につき最大 2 本）を添えて `Minimap` の `focus_id` を追跡し、`focus_id` が変更された際は該当行の `Smart Gutter` 表示を再描画して `Command Hub` の `Backoff State` を反映することを **SHOULD** とする。`Audit Event` で `resolution_hint` が更新された場合は、`Smart Gutter` のツールチップで具体的な修正設定キー/ポリシー ID を表示することを **SHOULD** とする。

`Smart Gutter` の仕様は `specs/glossary.md` に新しい用語として定義し、`specs/backlog.md` に `Smart Gutter` タスクの解決と `Command Hub`/`Shadow Buffer` との依存関係を追記しておくことを **MUST** とする。

###### 3.1.5.4.1 オーバーレイのスタッキング・コンテキスト

`Command Hub`/`Smart Gutter`/`Editor Decoration`/`Minimap` などのオーバーレイが重なる場合、ユーザーの現在の操作対象を最前面に出す「スタッキング・コンテキスト」を維持することを **MUST** とする。優先順位は以下のとおり定義し、各レイヤーは `focus_id` との一致で自動的に透明度・インタラクション可否を切り替える。

1. `Command Hub`（不透明度 1.0）: 入力・承認候補が現在フォーカスされている場合は常に最前面とし、背後のオーバーレイを無効化する。`focus_id` が `Audit Event` と合致しない場合も、最前面の選択状態を保持する。
2. `Smart Gutter`（アクティブ行）: 現在レビュー対象の `focus_id` が属する行は `Command Hub` に次ぐ優先度で保持し、非該当時は半透明にして背景へ退避する。
3. `Editor Decoration`（ゴーストテキストやインライン Diff）: 差分自体を編集領域に差し込むため中間レイヤーとなり、`Command Hub`/`Smart Gutter` と競合する場合は透過処理で `Command Hub` のラベル・ボタンを表示したままにする。
4. `Minimap`（フローティング・バックグラウンド、不透明度 0.6～0.8）: 鳥瞰図に徹し、他レイヤーがフォーカスされるたびにグローを弱める。ユーザー選択が `focus_id` と一致しないときは自動的に半露出状態となる。

各オーバーレイが `Audit Event` の `resolution_hint`/`overlay_hint` を参照し、表示優先度や遮蔽理由を追跡できるようにすることを **SHOULD** とする。`overlay_hint` には `focus_id`・`policy_id`・`config_path` を含め、被覆されているレイヤーを再描画するタイミングを `App Host` に通知できるようにする。必要に応じて `Command Hub` は `Backoff` ステータスのヒントを出してユーザーに次の操作を促すことを **SHOULD** とする。

---

### 3.2 MCP ツール仕様

エージェント（Codex等）は、以下のMCPツールを介してのみ「Nue」を操作できる。これにより、自由度と安全性を両立する。

| ドメイン | ツール名 | 役割 | Coreへの影響 |
| --- | --- | --- | --- |
| **FS** | `apply_patch` | 指定箇所への差分適用 | バッファ更新 + UI編集イベント発火 |
| **FS** | `read_file` | ファイル内容の読み取り | コンテキスト取得 |
| **VCS** | `get_status` | Gitの状態取得 | Legacy View 等の監査パネル／差分ビューへの変更反映 |
| **VCS** | `commit` | 変更の確定 | 履歴の保存 |
| **Runtime** | `run_command` | テストやビルドの実行 | ターミナル出力 + 成功/失敗のフィードバック |
| **UI** | `focus_file` | ユーザー選択中のファイル・選択箇所（行）の取得 | コンテキスト取得 |

---

#### 3.2.1 Router Authorization Model

`MCP Router` はエージェントから呼び出される全ての `MCP Tool` を受け取り、認可を通じて Core への影響を制御するセキュリティ境界である。`Authorization Policy` は `Domain`/`Tool`/`Permit Arguments`/`Execution Context`/`Approval State` の組み合わせで 定義され、エントリが存在しない呼び出しは暗黙の拒否とすることでホワイトリスト運用を保証する。

##### 3.2.1.1 Authorization Policy Entry Structure
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

##### 3.2.1.2 Policy Evaluation and Conflict Resolution
`MCP Router` は呼び出しごとに次のパイプラインで候補ポリシーを絞り込むことを **MUST** とする。

1. `domain`/`tool` 項目と一致するエントリを抽出する。
2. 抽出されたエントリを `execution_context`（例: `Workspace Session`、ブランチ、`Agent Status`）でフィルタリングする。
3. `argument_constraints` で引数の名前・値・マッチング方式を照合し、条件を満たすエントリを残す。
4. 残ったエントリを `effect`（`deny`/`allow`）で分類する。

このパイプラインの過程で `deny` が1件でもマッチした場合、評価を直ちに打ち切って `deny` を返すことを **MUST** とする。`deny` のレスポンスには該当 `policy_id`、`message`、`resolution_hint` を含め、`Audit Event` には拒否理由・`policy_id`・`execution_context`・匿名化された引数値を記録することを **MUST** とする。`deny` は `allow` に優先し、`deny` / `allow` が競合する順序付けや `Audit Event` 上でのトレーサビリティを損なわないようにすることを **SHOULD** とする。

`deny` が存在しない場合、`allow` 候補が残るので以下の順序で最も特化したエントリを選定することを **MUST** とする。

- `argument_constraints` の件数（より多数＝より詳細な引数制約）を降順で評価し、特異性の高い候補を優先する。
- 同一件数の場合は `priority`（整数、未指定は `0`）を降順で比較する。
- それでも複数残る場合は `policy_revision`（大きい方が新しい）を優先し、さらに同値であれば設定ファイル内の定義順に従う。

選ばれた `allow` エントリは `Audit Event` に `policy_id`、`approval_state`、`priority`、`argument_constraints` の照合結果を含め、引数は `audit.anonymization.level` に従ってハッシュまたはマスクした形式で記録することを **MUST** とする。

フィルタリングの結果、`deny` も `allow` も残らなかった場合は暗黙的な拒否とし、`Audit Event` に `policy_id=null` を記録して不足を運用側が分析できるようにし、エージェントには `message` を添えて次に取るべきアクションを示すことを **SHOULD** とする。

`Authorization Policy` の定義に変化（`config_revision` の更新、`Workspace Session` の切り替えなど）があった場合、既存の承認ステートは無効化され、対象の `MCP Tool` 呼び出しは再評価されることを **MUST** とする。

##### 3.2.1.3 Approval State and User Flow
`approval_state` は、ユーザーまたは自動化の承認要件を示す属性であり、次の値を **MUST** または **SHOULD** で選択する。

- `auto_allow`: `MCP Router` は `allow` のみを返し、`Audit Event` に `approval_state=auto_allow` を記録する。明示的な `Audit Event` により、`App Host` が自動化の挙動を追跡できる。
- `requires_user_consent`: 呼び出しは `Shadow Buffer` を通じた差分（`Approval Unit`）と連携し、ユーザーの操作（`Workspace Session` 単位または `ファイル単位` の `Accept`）を要求する。呼び出しが最初に到達した際には `deny` 応答とともに `Audit Event` を `approval_state=pending` で生成し、UI に `Approval Request` を送る。ユーザーが `Accept` した後、`MCP Router` は再評価を行い、条件が変わっていなければ `approval_state=approved` の `allow` を返す。明示的な拒否がある場合は `approval_state=blocked` で `deny` し、`Audit Event` に記録する。
- `blocked`: 呼び出しは常に `deny` される。`Audit Event` は `approval_state=blocked` を含め、ユーザーおよびエージェントに理由を通知する。

手動承認の際、`App Host`/`UI View` は `policy_id`・`tool`・引数のハッシュ・`Execution Context`（`Workspace Session`/ブランチ）を含む `Approval Request` を表示し、ユーザーは `Shadow Buffer` の一覧に沿って `Approve` を選択する。承認が完了するとき、`MCP Router` は一時的な承認キャッシュを `policy_id` × `Approval Unit` × `execution_context` の組み合わせで保持し、`config_revision` か `Workspace Session` が変化した場合はこのキャッシュを無効化することを **MUST** とする。

ユーザーが `Accept` するまで、要求された `MCP Tool` 呼び出しはエージェントに対して明示的な `deny` として返され、エージェントは `Audit Event` で得た `message` を参照して再試行を抑制する。`requires_user_consent` のポリシーが `Shadow Buffer` の差分と紐づかない呼び出し（例: `run_command`）では、UI に `Approval Request` を表示し、`MCP Router` は `Audit Event` に `approval_unit=manual` で記録する。

##### 3.2.1.4 Authorization Denial Feedback Loop

`MCP Router` は、エージェントが繰り返し `deny` を受けたことで無限ループや過剰リトライに陥らないよう、明示的なフィードバックを提供する責務を持つ。

- 毎回の `deny` に対し、`Audit Event` に `resolution_hint` を追加し、`policy.message` を含む簡潔な拒否理由に加えて「変更を求める設定キー（例: `workspace.authorization.auto_pilot=false`）」「推奨 UI 操作（例: `Command Hub` の承認パネルを開く）」が記載されることを **MUST** とする。`resolution_hint` は問題の所在を特定できる識別子（例: `config_path:key`、`policy_id`）を含めることを **SHOULD** とし、ユーザーやエージェントが容易に参照できる形式とする。
- 同一の `agent_id`/`policy_id` に対する連続 `deny` 要求では、`MCP Router` が内部で `DeniedRequestHistory` を保持し、再試行可能となるまでの `retry_delay_seconds` を指数的にインクリメントして `Audit Event` に記録することを **MUST** とする。初期値 3 秒、最大 30 秒とし、`retry_delay` が有効な間は `Command Hub` や `Terminal` の UI で `Backoff` 表示を行い、ユーザーへ余剰な再試行を控える指示を出すことを **SHOULD** とする。
- `requires_user_consent` ポリシーの初回 `deny` は `pending` 承認リクエストとして扱い、`approval_request_id` を `Audit Event` に含める。承認処理が完了しない間、同一要求に再アクセスがあった際は再度 `deny` を返さず `pending` ステータスと `approval_request_id` を提示することを **SHOULD** とし、エージェントが `approval_request_id` をトリガーに再送を抑止できるようにする。
- 連続 `deny` が UI に反映されない場合、`MCP Router` は `Agent Status` を `Error` に遷移させ、`Audit Event` を通じて `App Host` に `Feedback Loop` 通知を送信することを **SHOULD** とする。`App Host`/`UI View` は `Workspace Rail` や `Command Hub` に「設定 {resolution_hint.config_path} を修正して再試行」等のバナーを表示し、関係する設定キーの `ConfigChangeEvent` を強調してユーザーが迅速に対応できるようにすることを **SHOULD** とする。
- `blocked` ポリシーで拒否された呼び出しは、その `policy_id` に対して `DeniedRequestHistory` を 24 時間保持し、同一エージェントからの再送を即時 `deny` することを **SHOULD** とする。その際 `Audit Event` には「管理者に {policy_id} の再承認を依頼」や「外部承認フローを含む `resolution_hint`」を含めて、再試行が無意味であることを明示することを **SHOULD** とする。

#### 3.2.2 Shadow Buffer と承認フロー

`Shadow Buffer` はエージェントが行った差分編集を本バッファにマージする前に保持する構造体であり、各差分はファイル単位と `Workspace Session` 単位の両方で区分される。各差分には、発生時刻、差分の範囲、発行元エージェント、現在の `Agent Status` を付与することで、レビューと追跡が可能である。

1. **承認単位**
   初期リリースでは `Workspace Session` 単位の一括 `Accept` と `ファイル単位` `Accept` をサポートすることを **MUST** とし、続く項で `Reject`/`Revert`/`Partial Accept` の挙動を定義する。

2. **承認後の反映**
   `Accept` が行われると、`Shadow Buffer` の該当差分は `Editor Core` の本バッファへマージされ、`UI View` への更新イベント（差分の範囲・Agent Status）と `Audit Event` の両方を生成する。差分はマージ後に `Shadow Buffer` から削除され、ストレージに永続化されない。

3. **レビュー情報**
`Shadow Buffer` の差分は Legacy View や `Command Hub` の `Approval Requests` パネルなど、既存 UI 上で列挙・レビューできること。差分ごとのメタ情報（例: `Approval Unit`、`Execution Context`）を UI で参照し、ユーザーが編集対象と承認ステータスを確認できるようにする。将来的な `Galaxy Feedback` や `Nebula` 連携は `specs/spec_galaxy_view.md` にて追加で定義する。

4. **追加承認操作**
   これらの操作は v1.0 以降の段階的拡張項目とする。ToDo セクション（Sec.8）で `Reject`/`Partial Accept`/`Revert` の差分状態遷移と `Audit Event` 記録ルールを整理し、実装段階で詳細化する。

##### 3.2.2.1 承認操作セットと Approval Unit

`Shadow Buffer` は `Accept` に加えて `Reject`・`Partial Accept`・`Revert` の操作セットを提供し、各操作は明示的な `Approval Unit`（`workspace`/`file`/`hunk`）と結びつけて UI に提示されることを **MUST** とする。すべての操作は `Audit Event` に `result` を含め、ユーザーや監査システムが操作の種類と影響範囲を辿れるようにする。

- `Accept`（**MUST**）: 選択された `Approval Unit` の差分を `Editor Core` に適用し、該当エントリを `Shadow Buffer` から削除する。`Audit Event` には `result=accepted`、`approval_unit`、`focus_id`、`file_path`、`changed_range`、`agent_id`、`config_revision` を含め、`UI View` では完了マーカーと輝度の低減によって「処理済み」を表現する。`Accept` は `workspace`/`file` の両粒度を **MUST** でサポートし、`Partial Accept` によって `hunk` 粒度も `SHOULD` で扱えるようにする。
- `Reject`（**SHOULD**）: `Shadow Buffer` の差分を破棄し、`Editor Core` には何も反映しない。`Audit Event` には `result=rejected`、`reason=manual_reject`、`approval_unit`、`focus_id`（ある場合）、`related_event_id`（再トライ前の Accept イベント）、`agent_id` を含める。`MCP Router` は該当差分に対して新しい `Approval Request`（`requires_user_consent`）を生成し、エージェントには「差分は拒否されたため再生成してください」という `message` を返すことを **MUST** とする。拒否された差分を再採用する場合は、`Command Hub` から新たな意図を発行させることを **SHOULD** とする。
- `Partial Accept`（**SHOULD**）: 大粒度差分の一部を複数の `hunk`/`line_range` として承認し、残像を新たな `focus_id` で保持する。`Shadow Buffer` は承認済み範囲（連続・非連続を問わず）を `Editor Core` にマージし、残余行は再生成された差分として保持し続ける。`Audit Event` には `result=partial_accept`、`approval_unit=hunk`、`approved_ranges`（開始/終了行のリスト）、`remaining_ranges`、`focus_id`（残差分）、`related_event_id`（元イベント）を含め、`Command Hub`/`Structure Path`/`Minimap` が継続追跡できるようにする。UI ではチェックボックス・ドラッグ選択・`Solar Flare` ハイライトを用いて承認済みセクションを薄く表示し、複数 `focus_id` を明示することを **SHOULD** とする。
- `Revert`（**SHOULD**）: 過去の `Accept` を取り消し、逆向き差分を `Shadow Buffer` に再登録する操作。`Audit Event` には `result=revert`、`reverted_event_id`（元 Accept イベントの `event_id`）、`approval_unit`、`focus_id`（逆差分）、`file_path`、`agent_id`、`resolution_hint` を含め、再承認が必要な場合は `approval_state=pending` で再度 `requires_user_consent` 承認を展開することを **MUST** とする。`Shadow Buffer` は `related_event_id` を付与して元の差分と関連づけ、`Command Hub`/`Smart Gutter`/`Structure Path` が経路を追跡できるようにする。`UI View` は `Solar Flare` でマーキングされた行に `Esc`/`Undo` 操作を提供し、誤操作をキャンセルできるようにすることを **SHOULD** とする。

##### 3.2.2.2 UI / Core 整合と `Audit Event`

`Shadow Buffer` での承認操作は `UI View` の差分一覧（`Command Hub` の `Approval Requests` パネル含む）と `Editor Core` の状態を常に一致させることを **MUST** とする。具体的には、`UI View` 上の操作が発火したとき、同じ `approval_unit` を含む `Audit Event` が生成され、それが `Editor Core` のマージ/削除/再生成（`Revert`）とトリガー同期すること。`Partial Accept` では `Audit Event` に `approved_ranges`/`remaining_ranges` を記録し、`focus_id` の再生成と `related_event_id` で元イベントの追跡ができるようにし、`Revert` では `related_event_id`/`reverted_event_id` を用いて UI が逆方向差分と元許可経路を結びつけることを **SHOULD** とする。`Shadow Buffer` は、`Partial Accept` や `Revert` により差分の行番号が変化した場合にも `focus_id` を更新し、対象ノードを再ハイライトすることで UI 側の整合性を保つことを **SHOULD** とする。

すべての承認操作について、`Audit Event` には `result`（`accepted`/`rejected`/`partial_accept`/`revert`）と併せて `approval_unit`（`workspace`/`file`/`hunk`）を必ず含めることを **MUST** とし、その情報により `App Host` の監査パネルが絞り込み可能になる。`Partial Accept` により細分化された差分は `related_event_id` で親イベントと関連づけることを **SHOULD** とし、`Revert` の再承認では同一 `policy_id` を参照して過去のキャッシュを破棄する処理を **MUST** とする。

#### 3.2.3 ターミナルエミュレーター最小要件

ターミナルエミュレーター機能はエージェントが `run_command` を通じて実行するテスト・ビルド・デバッグ出力と連携する。初期リリースにおいて `Terminal Emulator` は以下の要件を満たすこと。

* **PTY** を使用してホストシステムとセッションを管理し、エージェントもユーザーも同一シェル状態を共有できるようにする。
* **スクロールバック**: 少なくとも 500 行の出力を保持し、上下キー/マウス/ペインスクロールで遡れること。
* **文字幅/Unicode整合**: 全角半角や Unicode 統合文字列を含めても列整列が崩れないよう、フォントレンダラとグリッドが一致すること。

これらの要件が満たされない場合、`Audit Event` を通じてユーザーへエラー状態を通知し、`MCP Router` はエージェントが再度 `run_command` を要求する前に運用上の確認を促す。

#### 3.2.4 Audit Event ライフサイクル

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
- `App Host` は Legacy View の監査パネルから `Audit Event` を `Workspace Session` や `Agent Status`、`Approval Unit` でフィルタ可能とし、将来の Galaxy View 統合は `specs/spec_galaxy_view.md` にて詳細を定義することを **SHOULD** とする。

以上で、監査記録の生成・保持・匿名化・再送の責務が明示される。

#### 3.2.5 Provider のドメイン境界とツール実行ライフサイクル

`MCP Router` は Workspace Session ごとにドメイン境界とツールの実行状態を管理し、プロジェクト外への逸脱および並列実行の競合を防ぐ責務を持つ。

##### 3.2.5.1 Workspace Context Enforcement

- `Workspace Session` は起動時に Canonical な `workspace_root` を決定し、`MCP Router` はすべてのツール呼び出しに対し `execution_context.workspace_root` を付与することを **MUST** とする。
- `run_command` を含むあらゆるファイル操作・実行操作は、CWD を `workspace_root` に固定し、`..` を含む相対パスや、シンボリックリンクを介して別のリポジトリやルートディレクトリへ遡るパス、あるいは `PATH`/`LD_LIBRARY_PATH` といった環境で明示的に外部実行環境を指定する変更を **MUST** 禁止する。逸脱が検出された場合、`MCP Router` は即時 `deny` として `Audit Event` を生成し、エージェントには `workspace_scope_violation` を理由として通知することを **MUST** とする。
- `FS` ドメイン（`apply_patch`/`read_file`など）のツールは `workspace_root` 内のノードのみを受け入れ、`canonical_path.starts_with(workspace_root)` が成立しない場合は `deny` とすることを **MUST** とする。
- `Workspace Config` で許可された `workspace_env` 以外の環境変数追加・上書きは認めず、`run_command` へ渡す `env` は `App Host` が定義した最小限の `workspace_env` + システムデフォルトに限定することを **SHOULD** とする。`Audit Event` には `execution_context.workspace_env` を含め、どの構成から環境が注入されたかを記録することを **SHOULD** とする。

##### 3.2.5.2 Tool Execution Queue and Lifecycle

- `MCP Router` は `ToolExecutionState` と呼ばれる構造体で各 `Workspace Session` の `tool_name`/`agent_id` ごとの状態を追跡し、同一セッションでの `run_command` の同時実行を防ぐことを **MUST** とする。`ToolExecutionState` には `state`（`Idle`/`Queued`/`Running`/`Completed`/`Failed`）、`agent_id`、`command_line`、`start_time`、`completion_time` を含める。
- `run_command` が到達したとき、該当セッションに `Running` 状態が存在しない場合は即座に `state=Running` へ遷移する。既に `Running` が存在する場合は `ToolRequestQueue` へ FIFO で追加し、`state=Queued` の `Audit Event` を `result=queued` / `queue_reason=tool_busy` で生成することを **MUST** とする。
- `ToolRequestQueue` は `tool.execution.queue_max_pending`（未設定時は `4`）を上限とし、上限に達した状態で追加要求を受けた場合は `deny` し `Audit Event` に `result=queue_overflow` を記録、エージェントには「先行する `run_command` の完了を待つか中断する」旨の `message` を返すことを **MUST** とする。
- `Running` が終了すると、`ToolRequestQueue` から先頭のエントリを取り出して `state=Running` へ遷移させ、`Audit Event` に `result=queue_start` を生成する。ユーザー UI には `Terminal` 上で `Queued` バッジや `Command Hub` の `Action Mode` で待機中候補を表示するように **SHOULD** 定義する。
- セッション終了・シャットダウン・ツール失敗時には、残存する `Queued` エントリを `result=canceled` として `Audit Event` に記録し、該当するエージェントへ `deny` を返す。`ToolExecutionState` はセッション破棄時に初期化されることを **MUST** とする。
- 上記の `ToolExecutionState` と `ToolRequestQueue` の変更はすべて `Audit Event`（`tool_state`/`queue_length`/`workspace_session_id`/`agent_id`）として記録し、`Shadow Buffer` や `Terminal` との整合性を保つようにすることを **SHOULD** とする。

### 3.3 Configuration（設定管理）仕様

設定は階層的にマージされ、常に最新の状態が各コンポーネントへリアクティブに反映される。各構成要素には優先順位と更新可否が定義されており、既存の `ConfigChangeEvent` を介して差分を伝播させる。

#### 3.3.1 Source hierarchy と merge priority

1. **環境変数 (Priority: 1)**: `NUE_AGENT_SAFETY_AUTO_ACCEPT=true` など。起動時に読み込まれるため、`App Host` は再評価のたびにこのスコープを最優先でマージする。
2. **ワークスペース設定 (Priority: 2)**: `.nue/config.yaml`（プロジェクト固有）。ファイル更新を検知したタイミングで再読み込みする。
3. **グローバル設定 (Priority: 3)**: `~/.config/nue/config.yaml`（ユーザーの基本設定）。同一ユーザーの複数ワークスペースにまたがる変更を検知する。
4. **デフォルト (Priority: 4)**: システム内蔵の初期値。常に最後のフォールバックとして保持される。

#### 3.3.2 変更検知と再評価

`App Host` はグローバル設定とワークスペース設定のファイル変更をファイルシステムイベント（例: kqueue/inotify/ReadDirectoryChangesW）で監視し、変更完了から 5 秒以内に再評価サイクルを開始する。このサイクルで、`App Host` は対象ファイルを再パースし、既存の設定スキーマに対して構文・バリデーションチェックを行う。パースに失敗した場合は既存の設定を保持し、該当事象を `Audit Event`（`type=config.reload.failure`）として記録し、ユーザーへ修正を要求する通知を出す。

変更が正当であれば、`App Host` は新しい設定と先行設定との差分を計算して単調増加する `config_revision` をインクリメントし、`ConfigChangeEvent` を生成する。`ConfigChangeEvent` は `source`（Global/Workspace）、`revision`、`changed_keys`、`previous_values`、`hot_reloadable` フラグを含み、待機中の `Workspace Session` および `AI Agent` に配信される。`App Host` は `config.reload` という UI コマンドを提供し、ユーザーが手動で再評価を要求した際もこのサイクルを再利用する。

環境変数の変更はプロセスの起動時に固定されるため、`App Host` はランタイム中に間接的な検知手段を持たない。したがって、環境変数ベースの設定を変更する場合、ユーザーは `App Host` を再起動しなければならず、`App Host` は再起動を伴う変更を要求するアラート（再起動後に `config_revision` を再生成）を表示することを **MUST** とする。

#### 3.3.3 伝播と適用制御

`Workspace Session` は自セッションに関係する `ConfigChangeEvent` を購読し、受信から 2 秒以内に適用を試行する。`ConfigChangeEvent` に含まれる各 `changed_key` にはメタデータとして `hot_reloadable`（`true`/`false`）が付与されており、`false` の場合は再起動なしには適用できない旨を示す。`Workspace Session` は `hot_reloadable=true` のキーについてのみ `Editor Core`・`MCP Router`・`Terminal Emulator` 等へ新値を反映し、`hot_reloadable=false` のキーは再起動が完了するまで旧値を保持してユーザーに通知する。通知には変更内容と再起動コマンド（例: `Restart App Host`）を含め、`App Host` が再起動済みであることを確認した後に `config_revision` を新しい値に合わせる。

`Workspace Session` は `ConfigChangeEvent` に `hot_reload_scope` を含め、関連する UI/サービスを限定的に再初期化する。たとえば、`MCP Router` のポリシー定義変更は `hot_reload_scope=router` となり、当該スコープ内のコンポーネントにのみ更新通知を送る。

##### 3.3.3.1 `hot_reload_scope` の許容値と依存順序

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

#### 3.3.4 フェールセーフと監査

`App Host` はすべての再評価サイクルを `Audit Event`（`type=config.reload` 以上）として記録し、`Workspace Session` に配信した `config_revision` を含めて Legacy View の監査パネルから追跡できるようにする（Galaxy View への拡張は `specs/spec_galaxy_view.md` を参照）。設定の差分を適用できなかった場合（例: 検証エラー、`Workspace Session` が遅延したコンポーネント）、`App Host` は既存の設定を再登録し、該当した `ConfigChangeEvent` について `Audit Event` を `config.reload.failure` として二重記録し、ユーザーへ修正指示を送る.

再評価時に `audit.queue.max` を超過するような連続的な失敗が発生した場合、`App Host` は最も古い `ConfigChangeEvent` を削除し、削除されたイベントの `event_id` を含む通知と `Audit Event` を生成することを **MUST** とする。削除前には少なくとも 30 秒の猶予を設け、その間にユーザーが手動で再適用できるようにする。

---

### 3.4 AI 共創ワークフロー詳細（The Nue Loop）

1. **インテント入力**: ユーザーがUIから指示を出す。
2. **エージェント起動**: UIがCodex等へタスクを丸投げ。
3. **MCP実務**: エージェントが `fs` や `runtime` ツールを駆使。
4. **Shadow Buffer**: Coreはエージェントの編集を「未承認の差分」として保持。
5. **UI Feedback**: AIが編集しているファイルやモジュールは UI 上で強調され、状態に応じた光度やアニメーション（例: パルス）でユーザーへ進行中の変更を伝える。Galaxy/Nebula 表示の具体的な振る舞いは `specs/spec_galaxy_view.md` で定義する。
6. **人間の承認**: ユーザーが差分を確認し、`Accept`。変更が本番バッファへマージされる。

#### 3.4.1 Command Hub の構造とインテント候補

`Command Hub`（コマンドパレット）は、ユーザーが自然言語やショートカットを使って操作を起点とする中心UIであり、AIエージェントと人間が同じテンポで「意図」を共有するためのインターフェースである。

##### 3.4.1.1 Command Hub の構造

- `Command Hub` は `Cmd + Shift + P`（macOS）または `Ctrl + Shift + P`（その他）で呼び出すモーダルオーバーレイで、アクション候補と履歴を一覧表示することを **MUST** とする。
- `Command Hub` は次の入力モードをサポートし、それぞれで優先的な候補生成を行うことを **MUST** とする。
  - `>` プレフィックス（Action Mode）: 明示的な `MCP Tool` 実行、設定変更、UIコマンドを記述する。例: `> Terminal: Split Terminal`。
  - `:` プレフィックス（Navigation Mode）: ファイル名・シンボル名によるナビゲーション。例: `:src/lib.rs`。
  - プレフィックスなし（Intent / Smart Search）: 自然言語（英語）で意図を入力し、`nue-semantic` による候補推論を得る。例: `test database connection` や `document API changes`。
- モードはリアルタイムに切り替わり、入力中のテキストに応じて候補リストを 16ms 以内に更新することを **SHOULD** とする。
- 16ms 以上の遅延が発生した場合、`Command Hub` は直前に表示していた候補を維持しつつ `Backoff` 状態を表示することを **MUST** とし、この状態では `nue-semantic` による再スコアリングが継続される旨とともに `Action Mode`/`Navigation Mode` への移行案を明示し、再評価完了後に最新候補が置換されるようにする。
- `Backoff` 状態では、候補一覧に `Re-scoring…` もしくは `awaiting nue-semantic` などの明示的なラベルを付与し、最後に生成された候補と所要時間・進捗を表示することで遅延の因果をユーザーへ伝えることを **SHOULD** とする。
- 同時に、`Command Hub` は `Action Mode` か `Navigation Mode` への移行手順（例: `>` でアクションを記述、`:` で候補を絞る）をヒントとして表示し、遅延が長引く場合はユーザーが明示的なコマンド入力へフォーカスを移せるよう案内することを **SHOULD** とする。
- 各候補には発行元（AIエージェント/ユーザー）、必要な `MCP Tool`、`Approval State`（`auto_allow`/`requires_user_consent`/`blocked`）を付与し、選択時に即座に `Audit Event` を作成することを **SHOULD** とする。

- 選択された候補は `MCP Router` へ `Intent Request` を送信し、`approval_state` に応じて自動的に処理されるので、`Command Hub` は `Audit Event` 経路を共有して `Shadow Buffer` との連携を疎通させることを **SHOULD** とする。

##### 3.4.1.2 インテント候補の Approval Unit

`nue-semantic` が自然言語入力から複数のファイル/ハンクを含む `Intent Request` を生成した場合、`Command Hub` はそれらを一つの **Atomic Intent**（用語集参照）として取り扱い、その集合を識別する `parent_intent_id` を付与することを **MUST** とする。Atomic Intent は「自然言語で表現された一連の変更提案を最小の実行可能なかたまり（論理最小単位）でまとめたもの」であり、`Command Hub` は候補一覧上で「5 箇所の変更」や「Rust ファイル 3 件」などの集約表現を提示しつつ、同一 `parent_intent_id` を共有する個別の `focus_id` を参照/展開できるようにすることを **SHOULD** とする。

各候補に紐づいて `Shadow Buffer` 内に登録される差分エントリは、原則として `focus_id` ごとに `Approval Unit`（ハンク単位）を定義しつつ、`parent_intent_id` によって関連付けることを **MUST** とする。`Shadow Buffer` は変更元のファイルパス・行範囲・diff 内容に応じて `approval_unit`（ハンク／ファイルなど）を設定し、`Audit Event` には `parent_intent_id` と `focus_id` の一覧を含めて、同一意図とその分割されたチャンクが追跡可能であることを **SHOULD** とする。`Command Hub` から候補を展開した際は、ユーザーが一覧内の任意の `focus_id` にフォーカスして差分を確認・部分承認できる UI を提供し、選択されなかった `focus_id` は `Shadow Buffer` に残したままにすることを **MUST** とする。

Atomic Intent の承認操作については、`Command Hub` が「すべて承認」アクションを受け取ったときに、集合内の各 `focus_id` に対して順次 `Accept` を実行し、`Audit Event` 側では `related_event_id=parent_intent_id` として `result=accepted` を記録することを **SHOULD** とする。部分的に (例: 5 個のうち 3 個) の `focus_id` だけを `Accept` した場合、残り 2 個は `Shadow Buffer` に新しい `focus_id` を保持したままで残され、`Audit Event` に `result=partial_accept`・`approved_ranges`・`related_event_id=parent_intent_id` を含めてトレースできるようにすることを **MUST** とする。これにより `SessionSnapshot` や `Structure Path` などが `parent_intent_id` をキーに UI 状態を再構築できるようにし、残差分の `focus_id` が `Audit Event` の `related_event_id` で親インテントと連結されることを **SHOULD** とする。

`Shadow Buffer` が `Accept` 以外の操作（`Reject`/`Partial Accept`/`Revert`）によって差分を変化させた場合も、変更済みの `focus_id` と `parent_intent_id` は引き続き `Audit Event` に記録し、`result=revert`/`result=reject` などで Operation の種類を明示することを **SHOULD** とする。これによって `Command Hub` の候補リストと `Audit Event` の監査パネルが同一の `Atomic Intent` を軸に同期し、単一の自然言語入力に対する承認状態・差分一覧・再試行履歴を一貫して追跡できるようにすることを **MUST** とする。

#### 3.4.2 Intent/Smart Search コンポーネント

- Intent/Smart Search は **`nue-semantic`** というローカル生成AIエンジンを中心とし、Phi やその他の Small Machine Learning（SML）モデルをバインドして動作することを **MUST** とする。外部APIは基本的に利用せず、オフライン環境でも動作する必要がある。
- `nue-semantic` は次のサブシステムを組み合わせて候補を生成することを **MUST** とする。
  - **Intent Resolver（局所意図変換）**: ユーザーの自然言語入力をトークン化し、`MCP Tool` 実行やUIアクションにマッピングするミリ秒スケールの推論モジュール。
  - **Local RAG（Local Retrieval-Augmented Generation）**: プロジェクト内のファイル名、関数名、設定名をベクトル化または重み付けしたインデックスで保持し、曖昧な入力に対して意味的に関連する候補へ橋渡しする。
  - **Policy-Aware Scoring**: `Authorization Policy` に定義された `argument_constraints`/`execution_context` を照合し、実行可能な候補のみを上位にソートする。
- `nue-semantic` は、候補の生成・表示・選択を 100ms 以内で完了させるように設計され、遅延が発生する場合は進行中の推論を UI 上でステータス表示することを **SHOULD** とする。
- `Intent/Smart Search` は候補の選択時に `MCP Router` への `run_command` や `apply_patch` の呼び出しを発生させる実行プランを返し、その過程で `Shadow Buffer` の差分として登録されるエントリと整合することを **MUST** とする。
- `Local RAG` に使うインデックスはファイルシステムの変更（追加/削除/リネーム）を検知した後 5 秒以内に部分更新し、入力ミスや類似語を許容するキーワードマッチを備えることを **SHOULD** とする。
- `nue-semantic` が内部リソースで解決できないと判断した場合、`Command Hub` は「この意図は現行リソースで解決できない」旨を明示し、ユーザーが `Terminal` などの `run_command` 経由で外部エージェント（例: 高性能クラウドAI）を呼び出すためのサンプルコマンドや必要な入力情報をガイドすることを **SHOULD** とする。`Command Hub`/`MCP Router` は `external_agent_profile` の一覧や `requires_user_consent` の承認ループを保持せず、外部エージェントへの問い合わせはあくまでユーザーが手動で行う運用とすることで Q15 を解決し、Nue 本体は `Audit Event` の `approval_state` に `requires_user_consent` を記録しないようにする。

##### 3.4.2.1 `nue-semantic` のインスタンスとワークスペース分離

`nue-semantic` は 450MB 以上に及ぶモデル本体を含むため、`App Host` はアプリ全体で **ただ一つのインスタンス** を起動し、すべての `Workspace Session` がこの共有インスタンスを参照することを **MUST** とする。複数のインスタンスを同一プロセス内で生成しようとする試みはリソース制限違反として拒否され、その事象は `Audit Event` (`type=semantic.instance_violation`) に記録することを **SHOULD** とする。

共有インスタンスは `SemanticContextManager`（名前は実装自由）を介して `Workspace Session` ごとに `SemanticContextHandle` を発行し、次の責務を果たすものとする。

- 各 `Workspace Session` は `SemanticContextHandle` を通じて自身に紐づく `Local RAG` インデックスのバージョン、プロンプトヒストリ、`Relevance Intent` の Pending 状態を保持し、**他のセッションと一切の情報を共有しない**ことを **MUST** とする。
- `SemanticContextHandle` は `workspace_session_id` をキーとし、`nue-semantic` へ渡すリクエストに必ず `semantic_context_id` と `local_rag_revision` を付与する。これにより `nue-semantic` はワークスペース単位のセッションを識別し、コンテキストの汚染を防ぐことを **SHOULD** とする。
- `App Host` は `nue-semantic` へのリクエストをスロット制御（例: Worker Pool/ファイバー）で調停し、同時実行スロットを `semantic.concurrent.requests` などの設定で制限する（デフォルト 4）。`Workspace Session` は `SemanticContextHandle` を介して順次リクエストを送信し、スロットが満杯の場合は `Command Hub`/`Smart Gutter` へ `Backoff` ステータスを提示することを **SHOULD** とする。

`nue-semantic` のインスタンスは `hot_reload_scope=semantic` を含む `ConfigChangeEvent` 時、または `App Host` の再起動時に再初期化される。再起動前の `SemanticContextHandle` に関連する `local_rag_revision`、`intent_history_id`、`pending_relevance_intents` は `Command Hub` の `SessionSnapshot`（Sec.7.1）へ `semantic_context_state` として記録し、復元後に `nue-semantic` へ再登録することを **SHOULD** とする。`SessionSnapshot` がこの情報を持たない場合でも、`App Host` は `nue-semantic` に空のコンテキストを再利用させ、少なくとも `workspace_session_id` で一意にトレースできるようにすることを **MUST** とする。

以上により `Command Hub` が意図を中心とした起点となり、AIエージェントと人間が共に進化するループの起点として機能する。

#### 3.4.3 グローバル検索とセマンティック検索統合

##### 3.4.3.1 グローバル検索

`Global Search` パネルは `Cmd + Shift + F` / `Ctrl + Shift + F` で呼び出されるオーバーレイとし、`Legacy View` の検索バーや `Command Hub` の `Navigation Mode`（`:` プレフィックス）からも遷移できるようにすることを **MUST** とする。ファイル内検索（`Cmd + F` / `Ctrl + F`）からの切り替えでは、現在のクエリ・正規表現/大文字小文字/単語単位のスイッチ・一時的な範囲（選択テキスト）を保持し、ユーザーが同じキーワードを再入力することなくワークスペース全体へスケールアップできるようにすることを **MUST** とする。

パネルは一致したファイルパス・行番号・スニペットを一覧表示し、`Shadow Buffer` の差分エントリか永続ファイルかを示すバッジと、`Audit Event` に必要な `workspace_session_id`/`agent_id` を含めたメタ情報のハイライトを付与することを **SHOULD** とする。結果の選択時には対象ファイルを `Editor Core` で開き、`Structure Path`/`Minimap`/`Smart Gutter` に該当行をハイライトしつつ検索語をフォーカスすることを **MUST** とする。

パネルには常時次のスコープ/フィルターを提供し、ユーザーが必要に応じて範囲を括る/広げる操作を行えるようにすることを **SHOULD** とする。

- `workspace_root`（デフォルト）
- 開いているバッファ・タブのみ
- 特定ディレクトリ（例: `src/`, `crates/`）や拡張子（例: `.rs`, `.md`）
- `workspace.search.exclude` / `.gitignore` に基づくディレクトリ（`node_modules`, `target`, `.venv`, `.cache`）の除外
- 検索対象を `Shadow Buffer` の差分に限定

`workspace.search.exclude` はデフォルトで除外対象に含めるディレクトリ群を定義し、ユーザーが「隠しディレクトリ/外部依存を含む」トグルで一時的にオーバーライドできるようにすることを **SHOULD** とする。スコープの変更は `Audit Event` (`type=search.scope_change`) に記録され、`App Host` が `workspace_session_id` ごとに追跡できるようにすることを **SHOULD** とする。

検索スコープのフィルターは `SearchFilterState` として `session_filters`（現在の `Global Search` パネルが保持する揮発的な正規表現/ディレクトリ選択）と `persistent_filters`（`workspace.search.exclude` 等の設定ファイルに紐づく永続的な値）に明確に分離することを **MUST** とする。`Session_filters` は `Command Hub`/`Global Search` の UI 状態で保持され、セッションをまたがず再起動時に消える。一方 `persistent_filters` は `Workspace` または `Global` の設定ファイルに書き戻され、同期時に `Workspace Session` が再評価して `search_scope_revision` を増分する。その際、`SearchFilterState` の更新は `Audit Event` (`type=search.scope_change`, `scope_type=session|persistent`, `filters=...`) に記録され、ユーザーがどのようにフィルターを変更したかを監査できるようにすることを **SHOULD** とする。UI は `Reset to defaults` 操作を提供し、`session_filters` をクリアすると同時に現在の `persistent_filters` を参照して `Global Search` リストを再描画することを **SHOULD** とする。

永続的なフィルター変更は `workspace.search.exclude` などのキーを含む構成ファイルに逐次保存され、`App Host` は変更を検知した後 5 秒以内に再評価を行い、新旧フィルター値を `ConfigChangeEvent` の `changed_keys` に含めることで `Workspace Session` へ `Hot Reload` を通知する。こうした構成変更によって `session_filters` が影響を受ける場合（例: 永続対象ディレクトリの追加）、`Global Search` UI は強調バナーでユーザーへ通知し、`Audit Event` に `resolution_hint=filter.sync` を付与して `Minimap`/`Structure Path` との整合を取ることを **SHOULD** とする。

`Global Search` は結果をストリーミング表示し、ファイル構造の更新・差分生成に伴って真新しい一致が発見された際には「再計算中」ラベルを出しつつ直前の一覧を保持することを **SHOULD** とする。検索処理が重くなる場合はパネル上に処理済ファイル数/残件数の進捗を表示し、必要に応じてユーザーが計算をキャンセルしたり新たなフィルターを適用したりできるようにすることを **SHOULD** とする。

##### 3.4.3.2 セマンティック検索統合

`Semantic Search` は `Global Search` の結果リストと同一 UI に統合され、`Command Hub` の自然言語モード（プレフィックスなし）で入力された意図を `nue-semantic` の `Intent Resolver` へ渡すことで意味ベースのマッチを生成することを **MUST** とする。`Local RAG` のベクトルインデックスはファイル名・関数名・コメント・設定名などの要素を保持し、`semantic_score` を計算して `Global Search` の一覧内に `Semantic Match` バッジとスコアを表示することを **SHOULD** とする。

`Semantic Search` の結果は目的語の意味的関連性を優先し、高スコアファイルは `Neon Cyan` のハイライトと専用 `Glyph` で表示することを **SHOULD** とする。これらの結果は `Smart Gutter`・`Structure Path` にも反映し、該当する行にハイライトを付与すると同時に `Command Hub` に `Relevance Intent` を起票して `Intent Request` へ連携できることを **MUST** とする。

###### 3.4.3.2.1 Relevance Intent の構造と連携
`Relevance Intent` は `nue-semantic` の `Semantic Search` が生成する意図トークンであり、`Command Hub` に渡された際に以下の情報を **MUST** で保持する。

- `semantic_score`: 候補の意味的一致度（0.0〜1.0）で、UI はスコアが高いものを優先表示し、`Audit Event` にも保存して後からの分析に利用する。
- `focus_ids`: `Shadow Buffer` に登録された各差分ハンクの `focus_id` リスト。`Command Hub` はこの一覧を使って候補を展開し、ユーザーが任意の `focus_id` を選べる UI を提供することを **SHOULD** とする。
- `parent_intent_id` / `related_event_id`: Atomic Intent を表す一意の識別子で、複数の `focus_id` を含む場合はそれぞれをこの ID で関連づけ、`Audit Event` の `related_event_id` および `Command Hub` の候補集約視点に一致させる。
- `approval_state`: `nue-semantic` が `Authorization Policy` を参照して計算した `auto_allow`/`requires_user_consent`/`blocked` の値。`Command Hub` はこの値を `Intent Request` に引き継ぎ、`MCP Router` が同じ承認ルールで最終判定できるようにする。
- `related_audit_event_id`: 直前の `Audit Event`（例: 기존の差分生成や再スコアリング結果）を指す ID。`Command Hub` は選択時にこの ID を `Intent Request` へ付与して、`MCP Router` と `Shadow Buffer` が同じトレーサビリティを維持する。

`Command Hub` は `Relevance Intent` を受け取ったとき、上記の `focus_ids` を元に `Shadow Buffer` の該当差分をハイライトしつつ、ユーザーが複数 `focus_id` をまとめて `Accept` する操作を行った場合は `MCP Router` に `Intent Request` として `approval_unit=parent_intent_id` を付加することを **SHOULD** とする。`Intent Request` の `Audit Event` にも `Relevance Intent` の `semantic_score`/`focus_ids`/`parent_intent_id` を含め、`Command Hub` の `Backoff` 状態でも `related_audit_event_id` で進行中の再スコアリングを追跡できるようにすることを **MUST** とする。

`Relevance Intent` から派生した `Intent Request` によって `Shadow Buffer` に差分が登録された場合、各 `focus_id` には `Audit Event` の `result=queued`/`pending`/`accepted` などのステータスとともに、元の `parent_intent_id` を `related_event_id` で記録することを **SHOULD** とする。これにより `SessionSnapshot` が `Relevance Intent` を再開時に再構築しやすくなるほか、`Command Hub` の UI が `Audit Event` による監査と整合しつつ再試行や `Partial Accept` などの分岐を追跡できる。

`Semantic Search` の再評価は `Local RAG` の 5 秒ルール（ファイルシステムイベントからの部分更新）に従って実行し、再スコアリングの進行中は `Global Search` パネルに `Re-scoring…` ラベルを表示して直前の結果を保持することを **MUST** とする。`nue-semantic` が意味的解決に至らない場合は `Command Hub` で外部エージェント提案を `requires_user_consent` で行い、結果が存在しなければ「この意図は現行コンテキストで解決不能」と表示することを **SHOULD** とする。

上記により `Global Search` は文字列一致をベースとするクラシックな検索と、`Semantic Search` による意味的ハイライトを同一のループで扱い、`Command Hub` が意図 → 検索 → 承認の流れを一貫して担保することを **MUST** とする。

### 3.5 ワークスペース継続性とリソース管理

`App Host` は複数のワークスペースを高速に切り替えながら、必要なときにリソースを解放することで全体メモリの圧縮とユーザー操作の継続性を両立させる責務を持つ。これを実現するために、各 `Workspace Session` について**セッションスナップショット**（`SessionSnapshot`）を整備し、スリープ/再開のトリガーやレイアウト復元を統制することを **MUST** とする。

#### 3.5.1 セッションスナップショットとレイアウト復元

`SessionSnapshot` は `Workspace Session` の現在状態を表す構造体であり、以下の項目を最低限含めることを **MUST** とする。

1. `workspace_session_id`/`snapshot_revision`/`captured_at` といった識別情報。
2. `open_tabs` の一覧（`tab_id`/ファイルパス/バッファ ID/カーソル位置/ビューポートオフセット/スプリット位置）。
3. `Shadow Buffer` にある未承認差分とそれに紐づく `Audit Event` の `related_event_id`・`approval_unit`・`focus_id`。
4. `ToolExecutionState`/`ToolRequestQueue` のキュー状態と `Approval State`。
5. `Terminal Emulator` のセッション（`working_dir`/`scrollback` の直近範囲）と、`Command Hub` の未処理候補および `sets`（`Action Mode`/`Navigation Mode`）のフォーカス。

`App Host` は `SessionSnapshot` を次のタイミングで更新することを **SHOULD** とする。

- 開いているタブやスプリット構成が変化したとき。
- ユーザーがカーソル位置・アクティブタブを切り替えたとき。
- `Workspace Session` が `Sleep Mode` に移行する直前。

スナップショットは `~/.config/nue/sleep/session_snapshots/<workspace_id>/<snapshot_id>.json` など恒久的なストレージへ原子書き込みされ、`sleep.snapshot.max_per_workspace`（デフォルト `3`）を超えると最も古いスナップショットを削除して `Audit Event`（`type=sleep.snapshot.prune`）を発行することを **MUST** とする。 `SessionSnapshot` は `App Host` が保持する `Workspace Rail` の表示や将来的なタブ/レイアウト復元機構のベースとなり、Tab/レイアウト管理の最終仕様は本スナップショットの拡張を通じて実現することを **SHOULD** とする。

#### 3.5.2 Sleep モード遷移と復元フロー

##### トリガー

`Workspace Session` は次のいずれかの条件を満たすと `Sleep Mode` に移行することを **MUST** とする。

- `workspace.sleep.timeout_seconds`（デフォルト 600 秒）以上、各種 UI/入力・エージェント操作がない状態が続いた。
- `Workspace Rail` 上の対象アイコンを右クリックした `Sleep Workspace` コマンド、もしくは `Command Hub` からの `Sleep` 操作が明示的に発行された。

##### Sleep への移行

1. `App Host` は移行前に `SessionSnapshot` を更新し、`snapshot_id` をロックする。
2. `SessionSnapshot` をストレージへ書き出し、`Audit Event`（`type=sleep.enter`, `workspace_session_id`, `snapshot_id`, `trigger`）を生成することを **MUST** とする。
3. `Workspace Session` は `MCP Router`、`Editor Core`、`Terminal Emulator`、`nue-semantic` などの実稼働コンポーネントを停止し、`ToolExecutionState`/`ToolRequestQueue` を `Paused` 状態に移行させる。`MCP Router` は `run_command` を拒否し、差分の生成を停止することを **MUST** とする。
4. `App Host` は当該 `Workspace Session` を `State=Sleep` としてマークし、`Workspace Rail` 上に Sleep バッジ（`Sleeping`）を表示する。睡眠中の `Command Hub` は `Away` バックドロップを表示し、`Audit Event` への `state=sleeping` 属性を持たせることを **SHOULD** とする。
5. Sleep 中は当該セッションに対する `ConfigChangeEvent` の配信を保留し、`hot_reload_scope` ごとにまとめて再適用することを **SHOULD** とする。このとき `App Host` は `Sleep Config Change Aggregator` を維持し、各 `hot_reload_scope` について `config_revision` が最大となる最新の値を保持しつつ、変更されたキーの和集合・最終値・`hot_reloadable` フラグを上書き（Last Write Wins）マージすることを **MUST** とする。

   Aggregator は `hot_reload_scope` の数に限定されたエントリを持ち、すべての変更が `LWW (Last Write Wins)` セマンティクスで退避されるため、明示的なキュー長の上限を設ける必要がない。各イベント到着時には改めて `config_revision` を比較し、同一キーについては最新の値で置き換えて `changed_keys` を更新し、`source`/`timestamp`/`config_path` など監査に必要なメタデータも併せて保持することを **SHOULD** とする。`hot_reloadable=false` の変更が含まれる場合は、Aggregator がそのスコープを「再起動要求」状態としてマークし、Wake up 時に `Audit Event`（`type=config.reload.sleep.apply`, `scope`, `config_revision`）でその事実を伝えるようにすることを **SHOULD** とする。

   Aggregator の状態は `App Host` の Sleep ステータスに紐づき、複数セッションで共有されないことを **MUST** とする。再開後の適用に失敗した場合は、`Audit Event`（`type=config.reload.sleep.failure` / `reason=invalid_path` など）とともに Notification System で `Cyber Magenta` バナーを表示し、どの設定キーが適用できなかったのかを明示してユーザーが対処できるようにすることを **MUST** とする。

##### 復元

1. ユーザーが対象ワークスペースを再アクティブにすると `App Host` は最新の `SessionSnapshot` を読み込み、`Audit Event`（`type=sleep.resume`, `snapshot_id`）を生成することを **MUST** とする。
2. `Workspace Session` を再生成し、`workspace_session_id` を再利用した上で `MCP Router`・`Editor Core`・`Terminal Emulator`・`nue-semantic` を再起動する。`ToolExecutionState`/`ToolRequestQueue` はスナップショットと整合するようにキュー状態を再構築し、`Approval Request` は `Shadow Buffer` 内の `related_event_id` に戻す。

   再起動後、`App Host` は Sleep 中に蓄積した `Sleep Config Change Aggregator` をフラッシュし、`Dependency-Aware Re-init Sequence`（`app`→`router`→`terminal`→`editor`→`semantic`→`agent`）の順で永続化済みのスコープを再適用することを **MUST** とする。各スコープについては、Aggregator の `changed_keys` の和集合と最終値をそのまま `ConfigChangeEvent` として再構成し、元の `config_revision` も含めて `Workspace Session` に配信することを **MUST** とする。適用成功時には `Audit Event`（`type=config.reload.sleep.apply` / `scope` / `config_revision`）を記録し、失敗時には前述の `Audit Event` と `Cyber Magenta` バナーでユーザーへ通知することを **MUST** とする。

   再適用後、Aggregator の状態はクリアされ、復帰完了まで同じ Sleep セッションに再利用しないことを **SHOULD** とする。復帰処理の最後で `Command Hub` が `Approval Requests` を再表示する前にすべての設定差分が確実に反映されていることを確認することを **SHOULD** とする。
3. 開いていたタブは `SessionSnapshot` に従って順番・スプリット構成・カーソル位置・ビューポートを復元し、`Minimap`/`Structure Path`/`Smart Gutter` にも `focus_id` を通知する。復元完了後、`Command Hub` は自動で `Approval Requests` を再表示することを **SHOULD** とする。
4. `App Host` は Sleep 復元後、直前の `SessionSnapshot` と異なる `cursor`/`focus` 状態を `focus_discrepancy` として `Audit Event`（`type=sleep.focus-discrepancy`）に記録し、ユーザーへ差分があることを通知することを **SHOULD** とする。
5. 復元時に未処理差分がある `Shadow Buffer` については `Command Hub` が `Solar Flare` を点滅させて `Approval Request` を強調し、sleep 解除直後でも承認が継続できる状態とする。

`Sleep Mode` はメモリ・CPU を解放しながらも、`Workspace Session` を再び選択した瞬間に 1 秒以内（`workspace.sleep.resume_budget_ms`）で作業状態を復元するよう設計することを **SHOULD** とする。 `App Host` は最大 `sleep.concurrent.max`（デフォルト 2）の Sleep セッションを同時に保持し、上限を超える場合最も古いセッションを `sleep.terminate_on_overflow=true` で終了して `Audit Event`（`type=sleep.terminate`, `reason=overflow`）を生成することを **SHOULD** とする。

上記を満たすことで、`Sleep Mode` は `Workspace Session` の `Cursor`/`承認状態`/`Audit Event` 関連の整合性を維持しつつ、未使用のワークスペースを積極的に休止させるしくみとして機能する。

### 3.6 未解決課題 / ToDo

本仕様では、`specs/backlog.md` に ToDo 形式で追跡している項目を逐次列挙し、Sec.4.2.1 で言及した `Reject`/`Partial Accept`/`Revert` のような拡張を忘れないように管理することを **MUST** とする。

#### 3.6.1 承認フロー拡張

`Shadow Buffer` の `Reject`/`Partial Accept`/`Revert` に関して、承認単位（ハンク/行/ファイル/セッション）の組み合わせ、`Audit Event` に含めるフィールド、UI での差分再表示・再承認の制御を未定義のままにしないことを **MUST** とする。詳細は `specs/ask.md` の Q19 に追跡しており、該当項目が具体化するまでは本仕様の該当節を再レビューして不足がないか確認することを **SHOULD** とする。

#### 3.6.2 補助 UI 強化

Minimap や Smart Gutter、Structure Path のような新規ビューは、`Shadow Buffer` や差分データ、`Agent Status` との整合性を明示しないまま構築を進めてはならない。これらの仕様は `specs/backlog.md` の該当 ToDo（`Minimap` / `Smart Gutter` / `Structure Path`）に目標と依存関係を残し、実装検討時に再度 `Command Hub`/`Legacy View` とのデータ連携を文書化することを **SHOULD** とする。

#### 3.6.3 検索 / セッション / リソース管理の未完項目

Global Search、Semantic Search Integration、タブ・レイアウト管理、Sleep 機能、`nue-semantic` のシングルトン運用など、ワークスペースや AI リソースに関わる機能要望は `specs/backlog.md` の該当 ToDo に記録しておき、仕様化に着手する際は `App Host`/`Workspace Session` の構成と整合する形で取り込む必要がある。これらの項目は `nue-semantic` の応答性や `App Host` のメモリ制御戦略に影響するため、再設計時には関連する `ConfigChangeEvent` の `hot_reload_scope` と整合性を取ることを **SHOULD** とする。

#### 3.6.4 外部エージェント連携 / フォント周りの未解決

外部エージェントへの問い合わせ先のプロファイルや管理フローについては `specs/ask.md` Q15 で確認中であり、承認制御や `Audit Event` 連携の仕様が固まるまでは `nue-semantic` による提案を自動化しない運用を **MUST** とする。また、JetBrains Mono と日本語/特殊記号フォントの混在に関する要件は Q20 で再確認する予定で、`FontContext` のフォールバック順序に変更が生じた場合は Sec.3.4 の記述を即座に更新することを **SHOULD** とする。
