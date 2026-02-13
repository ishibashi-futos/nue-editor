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

## Open

- [ ] Q15. 外部エージェント（例: `cloud_lambda_v2`）への問い合わせを `requires_user_consent` かつ `Audit Event` でのみ提案する場合、どのようなプロファイル名/リソースを許可し、誰がその一覧を管理するのか未定義です。提案可能な外部エージェントの最小限の分類や管理者承認フローを決める必要があります。 (`spec-nue.md` Sec.6.1.2)
  - Answer: 外部エージェントへの問い合わせはNue経由では行わない。あくまで、Nueのターミナルを経由して指示はユーザーが直接エージェントに出す。
