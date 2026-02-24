#!/bin/zsh

set -euo pipefail

BACKLOG_FILE="specs/plans/v1-release-backlog.md"

if [[ ! -f "$BACKLOG_FILE" ]]; then
  echo "backlog file not found: $BACKLOG_FILE" >&2
  exit 1
fi

awk '
function flush_task(    i, line_count, lines, line) {
  if (!in_task) {
    return
  }

  if (task_status == "open") {
    if (selected_num == "" || task_num < selected_num) {
      selected_num = task_num
      selected_title = task_title
      selected_body = task_body
    }
  }

  in_task = 0
  task_status = ""
  task_num = ""
  task_title = ""
  task_body = ""
}

{
  # タスク配下の行は2スペース以上のインデントのみ許可し、それ以外が来たらブロック終了。
  if (in_task && $0 !~ /^  /) {
    flush_task()
  }

  if ($0 ~ /^- \[[ xX]\] T-[0-9][0-9][0-9]/) {
    flush_task()
    in_task = 1
    task_status = (substr($0, 4, 1) == " " ? "open" : "done")
    task_num = substr($0, 9, 3) + 0
    task_title = substr($0, 7)
    next
  }

  if (in_task && $0 ~ /^  /) {
    task_body = task_body $0 ORS
  }
}

END {
  flush_task()

  if (selected_num == "") {
    print "未完了の T-xxx タスクはありません。"
    exit 1
  }

  print "Title: " selected_title

  line_count = split(selected_body, lines, /\n/)
  for (i = 1; i <= line_count; i++) {
    line = lines[i]
    if (line == "") {
      continue
    }
    sub(/^  /, "", line)
    print line
  }
}
' "$BACKLOG_FILE"
