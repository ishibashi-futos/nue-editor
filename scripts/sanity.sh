#!/bin/zsh

# pipefailを有効にし、パイプライン内のエラーを検知可能にする
set -o pipefail

echo "Running sanity checks..."

run_check() {
  local label=$1
  shift

  echo -n "  - $label... "

  # 実行中に出力を出さないよう、変数にキャプチャ
  # local と代入を分けることで、コマンド自体の終了ステータスを正しく $? に反映させる
  local output
  output=$("$@" 2>&1)
  local exit_code=$?

  if [ $exit_code -eq 0 ]; then
    echo "✅ OK"
  else
    echo "❌ FAILED"
    echo "--------------------------------------------------"
    # 失敗した時だけ、溜めていた出力を表示
    echo "${output}"
    echo "--------------------------------------------------"
    # 失敗した時点でスクリプト全体を異常終了させる
    exit 1
  fi
}

# 各工程を実行
run_check "Formatting" cargo fmt --all
run_check "Type check" cargo check --all-targets
run_check "Linting (Clippy)" cargo clippy --all-targets -- -D warnings
run_check "Unit tests" cargo test --quiet

echo "\n✨ All checks passed! You're good to go."