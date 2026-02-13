# Galaxy View / Nebula 仕様（将来機能）

## 1. カラーパレット（Galaxy 表示向けのハイライト）

Galaxy View 上ではノード・エッジ・ステータスの強調表示が重要となるため、以下の高彩度カラーを限定的に利用します。

| カテゴリ | 色名 | HEX | 主な配置 | 備考 |
| --- | --- | --- | --- | --- |
| Error / Alert | Cyber Magenta | #FF006E | ノード異常、依存チェーン割断、認可拒否通知 | ノード/エッジに Flash アニメーションを重ね、視線を即座に誘導する。 |
| Information | Ether Purple | #BF5AF2 | 依存線、補助情報、Galaxy Feedback の補助ラベル | 認知的に補助的なタグ表示に用い、背景とのコントラストを確保する。 |

これらの色は Galaxy View 上の LSP 依存関係や状態表示に限って使用し、GPUI のカラーパレットとしてバンドルされる必要があります。

## 2. Galaxy View の全体像

Galaxy View は、LSP 依存関係を銀河系モチーフのグラフで表現する専用ビューとして設計されます。Legacy View と役割を分離し、依存/呼び出しの伝播、AI の編集対象、影響範囲を一つの視覚化で表現することを目的とします。

- **導入タイミング**: 初期リリースでは Legacy View をデフォルトとし、Galaxy View は `v1.0` 以降で段階導入する予定です。
- **構造**: ノードがファイル/モジュールを、エッジが依存/呼び出しを表し、AI の編集対象は `彗星`、その影響範囲は `衝撃波` で視覚的に表現します。これにより変更の波及を即座に把握できます。
- **表示品質**: `Agent Status` に応じてノードの色/サイズ/輝度を動的に変え、`Busy`・`Waiting`・`Error`・`Idle` を一目で判別できるようにします。
- **パフォーマンス**: 500 個までのノードを含む更新で 45fps を維持し、GPU レベルのダブルバッファリングとインクリメンタルレイアウト更新で滑らかな描画を保証します。500 個超え時は詳細度を自動的に下げることでレイテンシやフレームレートを安定化させます。
- **アクセシビリティ**: WCAG レベル AA を満たすコントラスト、ラベル付きエッジ、代替テキストを必須とし、内容を Legacy View やアクセシビリティパネルで検索・列挙できるようにします。

## 3. スケーリング・ヒューリスティック

Galaxy View は常に 500 個以内のノードを明示的に描画し、超過時には `Focus Score` に基づいて描画対象を絞り込みます。スコアは以下の優先度で評価されます。

- `P0: Active, AI Activity` – エージェントが現在編集/生成中のファイル（`彗星`）およびその直近依存先。最も高い描画優先度を **SHOULD** とします。
- `P1: Focus, User Focus` – ユーザーが開いているタブやカーソル位置にあるファイル。ユーザー操作と整合性を保つため **SHOULD** 描画対象に残します。
- `P2: Impact, Dependency Hub` – 依存線の本数や接続密度が高い中心的モジュール。依存が多いほどスコアを上げ、可視化ルートに残します。
- `P3: Context, Distance` – P0/P1 ノードからのグラフ距離が 1 以内のノード。文脈理解のために低めの優先度で保持しますが、必要に応じて折りたたむことも可能です。

スコアの低いノードは自動的に集約・折りたたまれ、「星雲（Nebula）」として描画することで 500 個超の情報を視覚的に維持しながら操作コストを抑えます。Nebula は同一ディレクトリ内で Focus Score が一定未満の子ノード群をまとめた発光体であり、ユーザーのフォーカスに応じて展開・収束します。

外部依存（`node_modules`/`vendor` など）は、ユーザーが明示的に修正対象としている場合を除き、デフォルトで描画対象から除外し、必要に応じてヒエラルキーから展開するトグルを提供します。遠方ノードは座標を固定し、フォーカス周辺のみを再配置するインクリメンタルレイアウトを **SHOULD** 用います。同一形状のノード/エッジには GPUI の GPU インスタンシングを使いドローコールを削減します。

## 4. Shadow Buffer と Galaxy Feedback の連携

Galaxy View では、`Shadow Buffer` の差分が専用の候補パネルに列挙され、対象ノードが `彗星`/`衝撃波` として輝度変化とパルスで表示されます。差分ごとのメタ情報（`Approval Unit`、`Execution Context` など）は UI で参照でき、ユーザーが編集対象と承認ステータスを追跡できるようにします。この連携は `Command Hub` の `Approval Requests` パネルや専用の差分ビューとも同期します。

## 5. 承認操作と Galaxy Feedback

`Shadow Buffer` は `Accept` に加えて `Reject`・`Partial Accept`・`Revert` の操作を提供し、すべて明示的な `Approval Unit`（Workspace Session / ファイル / ハンク）と併せて UI に提示されます。各操作は `Audit Event` に `result` フィールドを含み、ユーザーや監査システムで差分の種類と影響範囲を追跡可能にします。

- `Accept`（**MUST**）: 選択された `Approval Unit` の差分は `Editor Core` にマージされ、`Shadow Buffer` から削除されます。`Audit Event` には `result=accepted`、`approval_unit`、`changed_range`/`file_path`、`agent_id` を含み、`Galaxy Feedback` では対象ノードに `彗星` のハイライトと `Electric Lime` の完了マーカーを表示するとともに輝度を落とすことで「処理済み」であることを伝えます。
- `Reject`（**SHOULD**）: 差分を却下するとエントリは破棄され、`Editor Core` や `Shadow Buffer` に変更を加えません。`Audit Event` には `result=rejected`、`reason=manual_reject`、`approval_unit`、`agent_id` を含め、`MCP Router` は同一変更への再承認を `requires_user_consent` として再送します。
- `Partial Accept`（**SHOULD**）: 大粒度差分を `ハンク`/`行` に分割し、個別の `Accept` を実行可能にします。受け入れた領域は `Editor Core` に反映され、残差は新たな差分として保持。`Audit Event` は `result=partial_accept`、`approved_ranges` を含み、残差分には `related_event_id` を付与します。UI はチェックボックスやドラッグ選択で承認済みセクションを薄く表示します。
- `Revert`（**SHOULD**）: 過去に `Accept` した差分を取り消すと、`Shadow Buffer` に逆向き差分を生成して再評価します。`Audit Event` は `result=revert`、`reverted_event_id`、`file_path` を含み、必要に応じて `approval_state=pending` として再承認を要求します。`Galaxy Feedback` では `Solar Flare` で該当行をマークし、ユーザーがキャンセルできるように `Esc`/`Undo` を提供します。

## 6. UI / Core の整合

`Shadow Buffer` での承認操作は `Galaxy View` の差分一覧と `Editor Core` を常に一致させることを **MUST** とします。`UI View` の操作により同一 `approval_unit` を含む `Audit Event` が生成され、それが `Editor Core` のマージ/削除/再生成（`Revert`）と同期する必要があります。`Partial Accept` や `Revert` によって行番号が変化した場合でも `focus_id` を更新し、`Galaxy Feedback` で対象ノードを再ハイライトし続けることで視覚整合性を維持します。

## 7. App Host と監査パネル

`App Host` は `Audit Event`（`type=config.reload` 以上を含む）を記録し、Galaxy View の監査パネルから `Workspace Session`・`Agent Status`・`Approval Unit` などでフィルタ可能にする必要があります。`ConfigChangeEvent` の差分適用に失敗した場合、既存設定を再登録し `config.reload.failure` として再度 `Audit Event` を生成し、ユーザーに修正を促します。

## 8. The Nue Loop における Galaxy Feedback

`The Nue Loop` のステップ 5 では Galaxy Feedback が AI の編集中を強調します。対象ノードは `彗星` 的な発光で示され、影響範囲は `衝撃波` で可視化されます。状態に応じて色/輝度/アニメーションを変え、ユーザーが変更の進行度を直感的に把握できるようにします。
