#!/bin/zsh

# 1. 今日の日付を取得（パスの形式 yyyy/MM/dd に合わせる）
TODAY=$(date "+%Y/%m/%d")
SESSION_DIR="$HOME/.codex/sessions/$TODAY"

# 2. ディレクトリの存在確認
if [ ! -d "$SESSION_DIR" ]; then
    echo "No session logs found for today ($TODAY)."
    exit 0
fi

echo "Finding command history for $TODAY..."

# .jsonlファイルを読み取り、jqでフィルタリング
# for log_file in "$SESSION_DIR"/*.jsonl; do
#     [ -e "$log_file" ] || continue

#     echo "--- File: ${log_file:t} ---"
#     jq -r '
#       select(.payload.type=="function_call" and .payload.name=="exec_command")
#       | .payload.arguments
#       | fromjson
#       | "Command: \(.cmd)\nWorkdir: \(.workdir)\n------------------"' "$log_file"
# done

# 1. ログから全ての cmd を抽出し、前後の空白を除去
# 2. 意味のある「コマンドらしい行」を正規表現で抽出
#    - 行頭が英数字、アンダースコア、スラッシュ、ドットのいずれか
#    - ただし、明らかにコード片であるもの（#[ , pub , async 等）を除外
ALL_CMDS=$(find "$SESSION_DIR" -name "*.jsonl" -print0 | xargs -0 jq -r '
  select(.payload.type=="function_call" and .payload.name=="exec_command")
  | .payload.arguments
  | fromjson
  | .cmd' 2>/dev/null \
  | sed 's/^[[:space:]]*//' \
  | grep -E "^[a-zA-Z0-9./][ -~]*$" \
  | grep -Ev "^(#[a-z]|pub |async |fn |let |use |type |mod |where |impl |return |if |else |while |for |static |const )")

# 3. 最初の単語（コマンド名）を抜き出して、主要なグループを自動特定する
# 出現回数が多い上位の「第一単語」を取得
TOP_COMMANDS=($(echo "$ALL_CMDS" | awk '{print $1}' | sort | uniq -c | sort -rn | head -n 10 | awk '{print $2}'))

for cmd_name in "${TOP_COMMANDS[@]}"; do
    # 各主要コマンドごとの詳細集計
    RESULT=$(echo "$ALL_CMDS" | grep -E "^${cmd_name}([[:space:]]|$)" | sort | uniq -c | sort -rn | head -n 5)

    if [ -n "$RESULT" ]; then
        echo "\n[${cmd_name}]"
        echo "$RESULT" | while read count detail; do
            printf "  %2d items: %s\n" "$count" "$detail"
        done
    fi
done
