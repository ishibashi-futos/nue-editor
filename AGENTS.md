## 作業ルール

- 日本語で応答する
- ドキュメント作成は日本語で行う
- コメントは日本語でつける
- Kent Beckの"Tidy First"に従う
- コードは常にメンテナンス製とテスト容易性を最も重要視する
- タスクは常に逐次実行する。並列化は行わない
- 時間の節約・開発スループットよりも、確実とトレーサビリティを優先する
- 後方互換性や例外処理のために処理を複雑にせず、シンプルで唯一の正解のために DRY な実装を行う
- GPUI等の高度な特定課題解決ライブラリを除き、原則として外部クレートに頼らず独自実装する。汎用ライブラリによるブラックボックス化を避け、ドメイン知識が反映された制御可能なコードを維持せよ

## プロジェクト概要

Rust・GPUI製エディタアプリケーション

## ディレクトリ構造

- `nue-app`: `App Host` としてワークスペースごとのセッション、ライフサイクル、設定、監査、通知を統括し、AI エージェント・MCP Router・Editor Core・UI View を分離して高速に切り替える最上位のプロセス群を提供する
- `nue-config`: 環境変数→ワークスペース設定→グローバル設定→デフォルトの優先順位で構成をマージし、変更を検知すると ConfigChangeEvent で差分＋hot_reloadable 情報を配信することで各コンポーネントに最新状態をリアクティブに伝える
- `nue-core`: Shadow Buffer と Editor Core を束ね、エージェントが生成した差分を保持してレビュー対象にし、Accept/Reject/Partial Accept/Revert を Approval Unit（workspace/file/hunk）単位で実行しながら Editor Core 本体のマージと Audit Event による UI との整合を保つプロセス
- `nue-mcp`: MCPツールのホスト。MCP Router を具現化し、すべての MCP ツール呼び出しを workspace_root 境界で認可・拒否・再試行制御し、Authorization Policy の argument_constraints や auto_allow/requires_user_consent/blocked を評価して応答するとともに、ToolExecutionState/ToolRequestQueue で並列実行を管理し、Audit Event を生成して App Host へ報告するセキュリティ境界
- `nue-ui`: Nueのユーザー向けフロントエンド
- `nue-semantic`: Intent/Smart Search の基盤として Intent Resolver・Local RAG・Policy-Aware Scoring を使って自然言語入力から候補を生成し、Command Hub に 100ms 以内で返すローカル生成 AI エンジン

## 開発コマンド

- `cargo build`: ビルド
- `cargo test`: テスト実行
- `cargo clippy -- -D warnings`: 静的解析
- `cargo fmt`: コードフォーマット

## Development Workflow

1. Read log: 過去の作業履歴をチェックする
   1. 最新のコミットログ 5件 を確認し、作業履歴から現在の状態を読み取る
2. Red: インタフェースを定義し、テストを書く
   1. 最小限の関数、クラス実装を定義する
   2. 最小実装に対するテストケースを書く
   3. Acceptance Criteria: テスト実行により、期待通り失敗する
3. Green: テストケースを最短でパスする実装コードを書く
   1. 最短の実装で、テストケースをパスさせる
   2. Acceptance Criteria: 全てのテストがパスすること
4. Refactor: リファクタリングでコードを整える
   1. コードを整える: 変数名の整理、重複の削除、関数の分割
   2. Acceptance Criteria:
      1. `scripts/sanity.sh` を実行し、lint error / format error / unit test errorが残っていないこと
      2. Tidyが完了し認知負荷が下がること
      3. Dead codeや不要なコメントが削除されること
5. Sync: Commit&進捗の可視化を行う
   1. 作業のコミットを行う
      1. コミットメッセージは、次のフォーマットに従う `<type>: <short title>\n<description>`
         1. type: fix, feat, docs, chroe のいずれかを使用する
         2. short title: 作業内容を表す短い英語のタイトルをつける
         3. description: 次の内容を、日本語で記述する。変更の理由, 解決アプローチ, 影響範囲, 関連リソース(design doc, 参考にしたurlなど)
      2. `git add .` のような乱暴なステージをしない。明示的に変更したファイルのみステージし、コミットすること。
   2. `.github/dashboard.md` を更新する。ファイルがない場合、 `.github/dashboard_template.md` から作成する
      - 更新日時: 作業完了時点での日時を入力する。いかなる場合でも、JSTで記述する
      - 要対応: ユーザーの判断が必要なアイテムをチェックリスト形式で記述する
      - スキル化候補: 繰り返し行われたコマンド実行や、一定の操作をシンプルに行うためのアイディアがあれば記述する
      - 成果: 次の内容を、日本語で記述する。実施したタスク名, 変更の理由, 解決アプローチ, 影響範囲, 関連リソース(design doc, 参考にしたurlなど), ToDo事項
      - `.github/dashboard.md` は `.gitignore` に指定しているためコミット不要

## 詳細ドキュメント

- 仕様書: @specs/spec-nue.md
- 開発ロードマップ: @specs/roadmap.md
- 作業指示: @specs/backlog.md
- コンポーネント仕様: @specs/features/*.md
