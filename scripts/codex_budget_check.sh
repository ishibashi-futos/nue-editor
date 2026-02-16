#!/bin/zsh

# codex exec --json を使って、使用状況（5h / weekly）の残量を定期表示する。

set -euo pipefail

INTERVAL_SEC=60
COUNT=0

usage() {
  cat <<'USAGE'
Usage:
  scripts/codex_status_watch.sh [--interval SEC] [--count N]

Options:
  --interval SEC   取得間隔（秒）。デフォルト: 60
  --count N        実行回数。0 は無限ループ（デフォルト）
  -h, --help       ヘルプを表示

Examples:
  scripts/codex_status_watch.sh --interval 60
  scripts/codex_status_watch.sh --interval 30 --count 10
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --interval)
      INTERVAL_SEC="${2:-}"
      shift 2
      ;;
    --count)
      COUNT="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "不明な引数です: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! [[ "$INTERVAL_SEC" =~ ^[0-9]+$ ]] || [[ "$INTERVAL_SEC" -lt 1 ]]; then
  echo "--interval には 1 以上の整数を指定してください" >&2
  exit 1
fi

if ! [[ "$COUNT" =~ ^[0-9]+$ ]]; then
  echo "--count には 0 以上の整数を指定してください" >&2
  exit 1
fi

if ! command -v codex >/dev/null 2>&1; then
  echo "codex コマンドが見つかりません" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq コマンドが見つかりません" >&2
  exit 1
fi

format_epoch() {
  local epoch="$1"

  if [[ -z "$epoch" || "$epoch" == "null" ]]; then
    echo "N/A"
    return
  fi

  date -r "$epoch" "+%Y-%m-%d %H:%M:%S %Z"
}

run_once() {
  local tmp_json
  tmp_json="$(mktemp)"

  # 追加のツール呼び出しを避けるため、最小の応答を返すプロンプトを送る。
  codex exec --json --cd /tmp --skip-git-repo-check \
    "Do not run any commands. Reply with exactly: OK" \
    >"$tmp_json" 2>/dev/null

  local thread_id
  thread_id="$(jq -r 'select(.type == "thread.started") | .thread_id' "$tmp_json" | tail -n 1)"
  rm -f "$tmp_json"

  if [[ -z "$thread_id" || "$thread_id" == "null" ]]; then
    echo "[ERROR] thread_id を取得できませんでした" >&2
    return 1
  fi

  local session_file
  session_file="$(find "$HOME/.codex/sessions" -type f -name "*${thread_id}.jsonl" | tail -n 1)"

  if [[ -z "$session_file" ]]; then
    echo "[ERROR] セッションログが見つかりません: thread_id=$thread_id" >&2
    return 1
  fi

  local rate_json
  rate_json="$(jq -c '
    select(
      .type == "event_msg"
      and .payload.type == "token_count"
      and .payload.rate_limits != null
    )
    | .payload.rate_limits
  ' "$session_file" | tail -n 1)"

  if [[ -z "$rate_json" || "$rate_json" == "null" ]]; then
    echo "[ERROR] rate_limits を取得できませんでした: $session_file" >&2
    return 1
  fi

  local primary_used primary_left primary_reset
  local secondary_used secondary_left secondary_reset

  primary_used="$(echo "$rate_json" | jq -r '.primary.used_percent // 0')"
  secondary_used="$(echo "$rate_json" | jq -r '.secondary.used_percent // 0')"

  primary_left="$(awk "BEGIN { printf \"%.1f\", 100 - ${primary_used} }")"
  secondary_left="$(awk "BEGIN { printf \"%.1f\", 100 - ${secondary_used} }")"

  primary_reset="$(echo "$rate_json" | jq -r '.primary.resets_at // "null"')"
  secondary_reset="$(echo "$rate_json" | jq -r '.secondary.resets_at // "null"')"

  echo "[$(date '+%Y-%m-%d %H:%M:%S %Z')]"
  echo "  5h limit    : ${primary_left}% left (used ${primary_used}%, resets $(format_epoch "$primary_reset"))"
  echo "  Weekly limit: ${secondary_left}% left (used ${secondary_used}%, resets $(format_epoch "$secondary_reset"))"
  echo
}

main() {
  local iteration=0

  while :; do
    iteration=$((iteration + 1))

    if ! run_once; then
      echo "取得に失敗しました。${INTERVAL_SEC}秒後に再試行します。" >&2
    fi

    if [[ "$COUNT" -gt 0 && "$iteration" -ge "$COUNT" ]]; then
      break
    fi

    sleep "$INTERVAL_SEC"
  done
}

main
