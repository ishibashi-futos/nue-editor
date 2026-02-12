# Open Questions

## Resolved

- [x] Q10. 設定（環境変数/ワークスペース/グローバル）に変更があった場合、`App Host` や `Workspace Session` は即時再評価して適用すべきですか、それとも再起動/再読み込みを要求すべきですか？
  - Answer: `App Host` はファイル変更を 5 秒以内に検知して再評価し、`Workspace Session` には `ConfigChangeEvent` を配信する。`hot_reloadable=false` のキーは再起動が完了するまで旧値を保持し、再起動を要求する通知を出す。環境変数変更は再起動必須である。 `spec-nue.md` Sec.5 を参照。

## Open

- [ ] Q1. `MCP Router` はどの粒度で認可を強制しますか？

Answer:  ツール + 引数 + 実行コンテキスト（`Workspace Session`/ブランチ）で評価する。

- [ ] Q3. `Accept` 以外の操作を初期リリースに含めますか？

Answer:  初期リリースは `Accept` のみ。将来は `Reject` + `Revert` + `Partial Accept` を追加する。

- [ ] Q4. `Galaxy View` は初期リリースでどの位置づけですか？

Answer:  段階導入（`v1.0` 以降）とする。

- [ ] Q5. ターミナルエミュレーターの最小要件をどこまで必須化しますか？

Answer:  初期リリースで PTY + スクロールバック + 文字幅/Unicode 整合を必須化する。

- [ ] Q6. 監査イベント (`Audit Event`) の保持方針はどうしますか？

Answer: 初期はメモリ保持のみ。`v1.0` 以降でユーザーグローバル保存（`*.jsonl`）を追加する。

- [ ] Q2. 初期リリースの `Approval Unit` は何に固定しますか？
  - 選択肢A: 一括 `Accept` のみ（`Workspace Session` 単位）
  - 選択肢B: ファイル単位 `Accept`
  - 選択肢C: ハンク単位 `Accept`

Answer: Workspace単位の一括 Accept / ファイル単位 Accept の両方

- [ ] Q7. `MCP Router` のポリシー衝突時、優先規則をどう定義しますか？
  - 選択肢A: 常に `deny` 優先
  - 選択肢B: スコープ狭いルール優先（同順位は `deny`）
  - 選択肢C: 最新ルール優先（同順位は `deny`）

Answer: 常に `deny` 優先。原因をわかりやすく返すこと。

- [ ] Q8. `Audit Event` の保持期間をどうしますか？
  - 選択肢A: 期間制限なし
  - 選択肢B: 30日保持
  - 選択肢C: 90日保持

Answer: ファイルに残す場合は保持期限 `30日` をデフォルトとして、日数を設定できるようにする

- [ ] Q9. `Audit Event` の匿名化レベルをどうしますか？
  - 選択肢A: 匿名化なし（完全記録）
  - 選択肢B: 引数の機微情報のみマスク
  - 選択肢C: 引数全体をハッシュ化

Answer: 引数全体をハッシュ化を行う

- [ ] Q11. Galaxy View で 500 ノードを超える状態になった際の詳細度低下は自動のみとし、ユーザーが制御できない仕様でよいですか？あるいは手動でフォーカス/展開できる仕組みが必要ですか？

Answer: ユーザー制御ができない仕様でいい

- [ ] Q12. `Audit Event` 再送キューの上限 `audit.queue.max` は何件（単位）とし、上限到達または削除した場合のユーザーへの通知/運用挙動をどう定めますか？

Answer: `Audit Event` については、必須ではないので後回しにする。
