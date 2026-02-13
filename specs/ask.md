# Open Questions

## Resolved

- [x] Q10. 設定（環境変数/ワークスペース/グローバル）に変更があった場合、`App Host` や `Workspace Session` は即時再評価して適用すべきですか、それとも再起動/再読み込みを要求すべきですか？
  - Answer: `App Host` はファイル変更を 5 秒以内に検知して再評価し、`Workspace Session` には `ConfigChangeEvent` を配信する。`hot_reloadable=false` のキーは再起動が完了するまで旧値を保持し、再起動を要求する通知を出す。環境変数変更は再起動必須である。 `spec-nue.md` Sec.5 を参照。

- [x] Q1. `MCP Router` はどの粒度で認可を強制しますか？
  - Answer: `Authorization Policy` は `policy_id`/`domain`+`tool`/`argument_constraints`/`execution_context`/`approval_state`/`effect`/`priority` を持つエントリから評価され、`deny` があれば即時拒否、なければ最も特化した `allow` を選択する。`requires_user_consent`/`auto_allow`/`blocked` の承認状態ごとの UI 連携と `Audit Event` 記録を `spec-nue.md` Sec.4.1.1–4.1.3 に RFC 2119 形式で記載しており、未定義の呼び出しは暗黙的に拒否して理由を出す。

- [x] Q3. `Accept` 以外の操作を初期リリースに含めますか？
  - Answer: 初期リリースは `Accept` のみ。将来は `Reject` + `Revert` + `Partial Accept` を追加する。

- [x] Q4. `Galaxy View` は初期リリースでどの位置づけですか？
  - Answer: 段階導入（`v1.0` 以降）とする。

- [x] Q5. ターミナルエミュレーターの最小要件をどこまで必須化しますか？
  - Answer: 初期リリースで PTY + スクロールバック + 文字幅/Unicode 整合を必須化する。

- [x] Q6. 監査イベント (`Audit Event`) の保持方針はどうしますか？
  - Answer: 初期はメモリ保持のみ。`v1.0` 以降でユーザーグローバル保存（`*.jsonl`）を追加する。

- [x] Q2. 初期リリースの `Approval Unit` は何に固定しますか？
  - 選択肢A: 一括 `Accept` のみ（`Workspace Session` 単位）
  - 選択肢B: ファイル単位 `Accept`
  - 選択肢C: ハンク単位 `Accept`
  - Answer: Workspace単位の一括 Accept / ファイル単位 Accept の両方

- [x] Q7. `MCP Router` のポリシー衝突時、優先規則をどう定義しますか？
  - 選択肢A: 常に `deny` 優先
  - 選択肢B: スコープ狭いルール優先（同順位は `deny`）
  - 選択肢C: 最新ルール優先（同順位は `deny`）
  - Answer: 常に `deny` 優先。原因をわかりやすく返すこと。

- [x] Q8. `Audit Event` の保持期間をどうしますか？
  - 選択肢A: 期間制限なし
  - 選択肢B: 30日保持
  - 選択肢C: 90日保持
  - Answer: ファイルに残す場合は保持期限 `30日` をデフォルトとして、日数を設定できるようにする

- [x] Q9. `Audit Event` の匿名化レベルをどうしますか？
  - 選択肢A: 匿名化なし（完全記録）
  - 選択肢B: 引数の機微情報のみマスク
  - 選択肢C: 引数全体をハッシュ化
  - Answer: 引数全体をハッシュ化を行う

- [x] Q11. Galaxy View で 500 ノードを超える状態になった際の詳細度低下は自動のみとし、ユーザーが制御できない仕様でよいですか？あるいは手動でフォーカス/展開できる仕組みが必要ですか？
  - Answer: ユーザー制御ができない仕様でいい

- [x] Q12. `Audit Event` 再送キューの上限 `audit.queue.max` は何件（単位）とし、上限到達または削除した場合のユーザーへの通知/運用挙動をどう定めますか？
  - Answer: `Audit Event` については、必須ではないので後回しにする。

- [x] Q13. `Command Hub` の候補更新に 16ms 以内という目標があるが、`nue-semantic` 感染再スコアリングで遅延が発生する場合の UX をどう扱うか？
  - Answer: `Command Hub` は 16ms を超える遅延時に直前候補を維持して Backoff 状態を表示し、`Action Mode`/`Navigation Mode` への移行案を提示しつつ `nue-semantic` の再スコアリングを続け、再評価完了後に最新候補へ置換する。 (`spec-nue.md` Sec.6.1.1)

- [x] Q14. `nue-semantic` が内部リソースで解決できない場合の外部エージェント提案の制御ポリシーは？
  - Answer: 外部エージェント提案は最終手段とし、`requires_user_consent` + `Audit Event` でプロファイル（例: `external_agent_profile=cloud_lambda_v2`）を限定する。提案先がない場合は「解決不能」メッセージを返す。 (`spec-nue.md` Sec.6.1.2)

- [x] Q16. `ConfigChangeEvent` の `hot_reload_scope` は複数のコンポーネント（例: `router`/`terminal`/`agent`）を含む可能性がありますが、許容される列挙値と、複数 scope を含む変更時の再初期化順序や優先度が未定義です。`hot_reload_scope` の命名規則と、未定義の scope を受け取った場合のフェールセーフ動作を決め、その後の `Workspace Session`/`UI View` の再構成ルールを明確にする必要があります。 (`spec-nue.md` Sec.5.3)
  - Answer: Sec.5.3.1 に `hot_reload_scope` の許容値と、「Dependency-Aware Re-init Sequence」（`app`→`router`→`terminal`→`editor`→`semantic`→`agent`）、未知 scope を `hot_reloadable=false` 扱いで再起動要求し `Audit Event` に `config.reload.unknown_scope` を記録する仕様を追加した。

- [x] Q25. `Minimap`/`Structure Path`/`Smart Gutter`/`SessionSnapshot` が共通で参照する `focus_id` の粒度（行/ハンク/差分）と一意性、`Shadow Buffer` エントリとの対応方法が未定義です。各 UI が同一 `focus_id` を参照し続ける運用を可能にする識別子の生成・更新ルールと、重複が発生した場合の整合性確保策を決める必要があります。 (`spec-nue.md` Sec.3.5.1/3.6/3.7/7.1)
  - Answer: Sec.3.5.1 に `focus_id` をハンク単位の ULID として定義し、`file_path_hash`/`hunk_start_line`/`hunk_checksum`/`timestamp` を基底とした生成・`Shadow Buffer` でのライフサイクル管理、`Audit Event`/`SessionSnapshot`/`Minimap`/`Structure Path`/`Smart Gutter` が同一 `focus_id` を共有する同期要件を規定した。

## Open

- [ ] Q15. 外部エージェント（例: `cloud_lambda_v2`）への問い合わせを `requires_user_consent` かつ `Audit Event` でのみ提案する場合、どのようなプロファイル名/リソースを許可し、誰がその一覧を管理するのか未定義です。提案可能な外部エージェントの最小限の分類や管理者承認フローを決める必要があります。 (`spec-nue.md` Sec.6.1.2)
  - Answer: 外部エージェントへの問い合わせはNue経由では行わない。あくまで、Nueのターミナルを経由して指示はユーザーが直接エージェントに出す。

- [ ] Q17. `Agent Status` や差分/承認の可視化 (`spec-nue.md` Sec.3.1.2) が現在は色とアニメーション（例: `Solar Flare` の点滅、`Cyber Magenta` の明滅）に依存しており、色覚多様性・ハイコントラストモード時の識別方法が未定義です。色以外のパターンや形状、テキストを併用する必要がありますか？
  - Answer:
    - ワークスペース・レイルのバッジやパレット内の状態表示には、色の変化に加えて固有の**「シンボル（Glyph）」**を付与します。
      - 状態 (Status),色 (Color),シンボル (Glyph),動き (Motion)
      - Idle,Dimmed,○ (Empty Circle),静止
      - Busy,Neon Cyan,◈ (Diamond),回転 / パルス（緩やか）
      - Waiting,Solar Flare,▲ (Triangle),点滅（1Hz）
      - Error,Cyber Magenta,✖ (Cross),振動（高周波）
    - 差分（Gutter）の可視化
      - 未承認 (AIによる追加): Solar Flare（イエロー）かつ 「太い実線」。
      - 承認済み / Git管理下: Midnight Glass（グレー）または透明かつ 「細い実線」。
      - 削除箇所: 該当行のガターに 「小さな三角形のマーカー（◀）」 を表示。
    - ハイコントラストは今のところ趣旨と合わないので採用しません。
    - アクセシビリティ対応についても基本的に不要（対象外）とします。
    - あくまで私個人が気持ちよく使うためのツールです。

- [ ] Q18. `Galaxy View` が 500 ノード超で詳細度を下げる際、どのノード/エッジを折りたたみ・簡略化するかや、ユーザーのフォーカスをどう扱うか、具体的なヒューリスティックや操作仕様が未定義です。表示維持/削減の基準、あるいは手動制御の可否を決める必要があります。 (`spec-nue.md` Sec.3.3)
  - Answer: ユーザーの「現在の関心事」を軸に、表示対象を自動的に選別して欲しい。
    - 各ノードに対して以下のスコアを算出し、スコアの高い順に描画を維持します。
      - P0: Active,AI Activity,現在エージェントが編集中のファイル（彗星）およびその直近の依存先。
      - P1: Focus,User Focus,現在エディタで開いているタブ、およびカーソルがあるファイル。
      - P2: Impact,Dependency Hub,依存関係（エッジ）が集中している中心的なモジュール。
      - P3: Context,Distance,P0/P1 からのグラフ距離が 1 以内のノード。
    - ディレクトリ・クラスタリング: 同一ディレクトリ内の低スコアファイル群を一つの「星雲（Nebula）」として集約し、個別のノードを表示せず、ディレクトリ単位の大きな発光体として描画します。
    - 外部ライブラリの非表示: node_modules や vendor などの外部依存は、明示的に修正対象にしない限り描画しません
    - パフォーマンス維持のための工夫
      - インクリメンタル・レイアウト: 全ノードの物理シミュレーションを毎フレーム行うのではなく、フォーカス周辺のノードのみを計算対象とし、遠方のノードは座標を固定（フリーズ）します。
      - GPUインスタンシング: 同一形状のノード（星）やエッジ（光の線）はGPUIのインスタンス描画を利用し、描画コール数を最小限に抑えます。

- [ ] Q19. `Shadow Buffer` の承認操作（`Accept`/`Reject`/`Partial Accept`/`Revert`）を Sec.4.2.1/4.2.2 で定義したが、それぞれが許容する `Approval Unit` の粒度（Workspace Session/ファイル/連続ハンク/任意行）や、`Reject` 後のエージェント再試行義務、`Revert` で生成される逆方向差分の再承認フローを明確にする必要があります。特に `Partial Accept` で非連続行を許すか、`Revert` で旧イベントを参照した `policy_id` をどう扱うかを決定いただけますか？
  - Answer: <未回答>

- [ ] Q20. `spec-nue.md` Sec.3.4 で JetBrainsMono ファミリの Bold/Italic を埋め込むことを決めたが、日本語・絵文字・右起動の記号などをカバーするフォントはどのように提供するか明確ではない。`FontContext` の `fallback_fonts` や追加バンドルによってどこまでカバーすべきか、OS フォント依存でも許容されるのかを決定してください。
  - Answer: 多言語（日本語）や特殊記号をどう扱うか、フォントフォールバックする。
    - UI（メニュー・サイドバーなど）: そもそも日本語を許容しない。UIは英語のみで構成し、`JetBrainsMono` を採用する
    - Editor（コード領域）: Menlo, Monaco, Courier NewなどのOS標準の等幅フォントを使用する
    - JetBrains Monoと日本語を混ぜた時に「日本語だけ浮いて見える」現象を防ぐため、JetBrains Monoに対して日本語を少し小さく、かつベースラインを下げる補正を入れる

- [ ] Q21. `Nebula` が複数ノードをまとめる挙動（Sec.3.3.1）はスコアベースだが、ユーザーが Nebula を展開・縮小したり、集約されたノードをフォーカス/ジェスチャーで追跡したりする操作仕様が定義されていない。自動選別のみで問題ないのか、あるいは視覚的なケアやインタラクション（例: ダブルクリックで展開）を追加すべきかを教えてください。
  - Answer: Nebulaについては先の仕様なので、今は考慮しない

- [ ] Q22. `Smart Gutter`（Sec.3.7）と `Minimap`/`Command Hub` のオーバーレイ（Sec.3.5/6.1）で描画対象が重複する場合、どちらの表示が優先されるべきか、また視覚的に整合を取るために `Audit Event` `resolution_hint` でどのような追加情報を添える必要があるか未定義です。確認すべき優先順位や補助的なメタデータの構成を教えてください。
  - Answer: 複数のオーバーレイが重なる場合、**「ユーザーの現在の操作対象」**を最前面に出し、それ以外を透過または背景へ退避させる「スタッキング・コンテキスト」を適用します。
    - 1,Command Hub,不透明度 1.0。,ユーザーの思考・入力が最優先なので、出すときは常に最前面に。
    - 2,Smart Gutter,不透明度 1.0（アクティブ行）。,どの行を承認/却下するかの判断材料。
    - 3,Editor Decoration,ゴーストテキスト、インラインDiff。,書き換え内容そのもの。
    - 4,Minimap,不透明度 0.6~0.8（フローティング）。,ファイル全体の鳥瞰図であり、背景に近い。

- [ ] Q23. `Global Search` のフィルター切り替え（`workspace.search.exclude` を含む範囲/ディレクトリ選択）は在来の検索設定（グローバル/ワークスペース）とどのように同期すべきか、またこの状態を永続化する必要があるか未定義です。切り替えの範囲や持続性をどこで管理すべきか教えてください。 (`spec-nue.md` Sec.6.2.1)
  - Answer: 「揮発的なセッション・スコープ」 と 「永続的な設定・スコープ」 に分離して管理したい。
    - 揮発的なセッション・スコープ：検索時に、除外するディレクトリを正規表現パターンで入力・指定できる
    - 永続的な設定・スコープ：グローバル・ワークスペースごとに設定として持たせることができる

- [ ] Q24. `Semantic Match` 結果のうち `Command Hub` へ送る `Relevance Intent` の定義と構造（`Approval State` との組み合わせや `Audit Event` への記録フィールド）が未定義です。この `Relevance Intent` をどこで生成し、どのように `Intent Request` にマッピングすべきか教えてください。 (`spec-nue.md` Sec.6.2.2)
  - Answer: 生成フローは次のとおり
    - Generation (nue-semantic): ユーザーの自然言語入力を Phi-3/Qwen が解析し、候補となる ツールと引数（Action）を複数生成します。
    - Internal Validation (Command Hub): 生成された intent が、現在のコンテキスト（エディタがフォーカスされているか等）で実行可能かをチェック。
    - Candidate Presentation (UI): パレットに「アクションの候補」として表示。AIの確信度が高い場合は、決定打としてハイライト。
    - Local Execution (nue-app): ユーザーが選択した瞬間、システム内の Dispatcher が該当する Rust 関数を直接実行。

- [ ] Q26. `Sleep Mode` で `ConfigChangeEvent` を保留している間、そのイベントをどの順序・頻度で再送するか、保留キューの上限や破棄判断、復帰後の適用タイミング（先頭から順に、最新のみ、優先度付きなど）を定義していません。`App Host` が Sleep 中に得た変更をどのように再評価し、失敗時にどうユーザーへ通知するかを明確にしてください。 (`spec-nue.md` Sec.5.2/5.3/7.2)
  - Answer: セッションが Sleep 中に発生した設定変更は、「最新状態への収束」 を優先し、無駄な再計算を排除します。
    - 保留キューと更新ルール
      - 戦略: 「LWW (Last Write Wins) + スコープ合算」。
      - Sleep 中、App Host は変更された hot_reload_scope を累積します。例えば、editor 設定が3回変更されても、保持するのは 最新の revision と 変更されたキーの和集合 のみです。
      - キュー上限: 無制限（単一の最新状態に集約されるため、メモリを圧迫しません）。
    - 復帰後の適用タイミング
      - タイミング: セッションの Wake Up 処理（リソースロード）の直後、かつユーザーの入力を受け付ける 前 に一括適用。
      - 再評価: 復帰時に現在のファイルツリーと設定の整合性をチェックし、失敗した場合は Notification System を通じて「一部の設定（例: 削除されたディレクトリの除外設定）を適用できませんでした」と Cyber Magenta（エラー色）のバナーで通知します。
- [ ] Q27. Intent/Smart Search の候補が複数ファイル・複数ハンクにまたがる場合、どの `Approval Unit` を選び、`Shadow Buffer` の差分と `Audit Event` の `related_event_id`/`focus_id` をどう更新するのか、分割すべきか一括で扱うべきかといったポリシーが未定義です。適正なハンドリングを決める必要があります。 (`spec-nue.md` Sec.4.2.1/6.1.1/6.2.2)
  - Answer:
    - Approval Unit の分割ポリシー
      - 原則: 「論理的最小単位（Atomic Intent）」 で一括管理します。
      - 一つの自然言語入力（Intent）から派生した複数のファイル変更は、同一の parent_intent_id を共有します。
      - UI 提示: Command Hub では一括して「5箇所の変更を適用」と表示しますが、Shadow Buffer 上では個別の focus_id を持ち、ユーザーは「一括 Accept」または「個別の Hunk 単位の Reject」を選択可能です。
    - 識別子の更新ルール
      - 関連性: Audit Event には parent_intent_id と、ぶら下がる全ての focus_id のリストを記録します。
      - 部分的承認時の挙動: 5つの変更のうち3つだけ Accept した場合、残りの2つは Shadow Buffer に残り続け、focus_id は維持されます。Audit Event には result: partial_accepted として記録され、related_event_id を通じて元のインテントと紐付けられます。
