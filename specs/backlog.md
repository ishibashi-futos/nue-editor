# Backlog

## Resolved

- [x] P2: `Galaxy View` の成立条件を定義する
  - Before: レイアウト・更新頻度・性能上限・アクセシビリティ基準が未定義。
  - After: `v1.0` 以降での導入タイミング・500 ノード時の 45fps 性能 budget・アクセシビリティ要件などを `spec-nue.md` に記載した。

- [x] P2: `Audit Event` のライフサイクル要件を定義する
  - Before: 初期メモリ保持と `v1.0` 以降の `*.jsonl` 保存方針はあるが、保持期間・匿名化・オフライン再送が未定義。
  - After: 保持期間・匿名化ルール・永続化パス・キュー再送制御（`audit.*` 設定）を `spec-nue.md` Sec.4.4 に RFC 2119 で規定。

- [x] P2: `Configuration` の再評価/ホットリロード要件を定義する
  - Before: 設定のマージ優先度は記述されたが、環境変数・ワークスペース・グローバル設定の変更をどのタイミングで各コンポーネントが再評価するか未定義。
  - After: `ConfigChangeEvent` による再評価サイクル、ホットリロード可能/不可キーの扱い、再起動要求時の監査と `audit.queue.max` 扱いを `spec-nue.md` Sec.5 で定義した。

- [x] P1: `MCP Router` の認可モデルを定義する
  - Before: `MCP Tool` 実行可否と承認粒度が未定義で、セキュリティ境界が曖昧であった。
  - After: `spec-nue.md` Sec.4.1 に Authorization Policy エントリ構造（`policy_id`/`argument_constraints`/`execution_context`/`approval_state`/`effect`/`priority`）、引数ハッシュマッチ、`deny` 優先の評価順、`requires_user_consent`/`auto_allow`/`blocked` の承認フロー、`Audit Event` 連携を RFC 2119 スタイルで記載した。

## ToDo

- [ ] P1: デザイン・カラーの具体化を進める
  - カテゴリ,色名,カラーコード,主な使用箇所 / 役割
    - Background,Deep Abyss, #0B0E14 ,エディタのメイン背景。最も暗いレイヤー。
    - Surface,Space Grey, #1A1D23 ,サイドバー、タブ、パネルの背景。Baseより一段明るい。
    - Border,Midnight Glass, #2D323C ,パネルの境界線、セパレーター。
    - Primary (AI),Neon Cyan, #00F5FF ,エージェント活動中、AI提案のハイライト、ミニマップのAI位置。
    - Success / Accept,Electric Lime, #32FF7E ,一括承認ボタン、正常終了通知、保存済みインジケーター。
    - Warning / Wait,Solar Flare, #FFF200 ,ユーザー入力待ち、未承認の差分ガター、警告アイコン。
    - Error / Alert,Cyber Magenta, #FF006E ,ビルドエラー、認可拒否、Galaxy Viewでのノード異常振動。
    - Information,Ether Purple, #BF5AF2 ,LSPの型情報、シンボル定義、Galaxy Viewの接続線（依存関係）。
    - Text (Main),Cloud White, #E4E7EB ,標準テキスト、コード文字。
    - Text (Muted),Dusty Grey, #717984 ,コメント、無効なUI要素、パンくずリスト。
  - エージェントの状態表現
    - Busy: Neon Cyan(#00F5FF) がパルス状に発光（Opacity 0.4 ↔ 1.0）。
    - Waiting: Solar Flare(#FFF200) が低速で点滅。
    - Error: Cyber Magenta(FF006E) が鋭く明滅。
  - 差分（Diff）と承認の視覚化
    - 未承認の行 (Gutter): Solar Flare（イエロー）の縦線。
    - 承認済み / 確定: インジケーターが消滅し、テキストが Cloud White に馴染む。
    - 承認ボタン: Electric Lime（グリーン）のグロー効果。

- [ ] P1: 使用可能フォントの具体化と埋め込み
  - OSSフォントを使用可能フォントに設定、バイナリに埋め込み
    - 標準コード / UI,JetBrainsMono-Regular.ttf,高い可読性とリガチャー。
    - キーワード / 強調,JetBrainsMono-Bold.ttf,ネオンカラーと組み合わせた際の見栄え。
    - メタ情報 / コメント,JetBrainsMono-Italic.ttf,コメントやMuted Textの区別。
  - UIの初期化プロセス内で埋め込んだバイナリを `FontContext` に登録

- [ ] P1: `Shadow Buffer` の承認モデルを定義する
  - Before: `Accept` 以外（Reject/Revert/Partial Accept）が未定義で、運用手順が確立できない。
  - After: 承認操作セット、`Approval Unit`、競合時挙動を定義し、UI 操作と Core 反映ルールを一致させる。

- [ ] P1: `MCP Router` ポリシー衝突時の優先順位を定義する
  - Before: 認可粒度（ツール/引数/実行コンテキスト）は確定したが、`allow` と `deny` の衝突解決規則が未定義。
  - After: 衝突解決規則（例: `deny` 優先）と評価順序を RFC 2119 で規定し、拒否時の `Audit Event` 記録要件を固定する。

- [ ] P1: 初期リリースの `Approval Unit` を確定する
  - Before: 初期リリースは `Accept` のみと決定済みだが、承認粒度（一括固定/ファイル単位）が未定義。
  - After: 初期 `Approval Unit` を 1 つに固定し、将来のハンク単位承認への拡張条件を仕様化する。
