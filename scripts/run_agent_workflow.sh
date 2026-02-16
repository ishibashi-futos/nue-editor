#!/bin/zsh

# ==================================================
# 開発ワークフローの実行を自動化するスクリプトです
# 1. agentにより、5回の実行でbacklogを実行します
# 2. agentが過去のコミットを読み取り、自己レビューを行います
# ==================================================

set -o pipefail

PROMPT=".agents/developer/AGENTS.md"
MODEL="gpt-5.3-codex"
MODEL_REASONING_EFFORT="medium" # low, medium, high, xhigh

REVIEW_MODEL="gpt-5.3-codex"
REVIEW_MODEL_REASONING_EFFORT="high"
REVIEW_PROMPT=".agents/review/AGENTS.md"

AGENT_LOGS="logs/workflow.log"
if [ -f "$AGENT_LOGS" ]; then
    rm "$AGENT_LOGS"
    echo "Removed existing log file: $AGENT_LOGS"
fi
if [ ! -d "logs/" ]; then
    mkdir -p "logs/"
    echo "Created log directory: logs/"
fi

for i in $(seq 1 5)
do
  echo -n "🤖 $i: running task ... "
  COMMIT_LOG=$(git log -n 5 --pretty=format:"%h %as [%s] %b%n---")
  RAW_AGENT_PROMPT=$(cat $PROMPT)
  codex --dangerously-bypass-approvals-and-sandbox \
    --model $MODEL \
    --config model_reasoning_effort="$MODEL_REASONING_EFFORT" \
    exec "$RAW_AGENT_PROMPT\n##Recent changes\n$COMMIT_LOG" > $AGENT_LOGS 2>&1
  echo "✅ completed"
done

echo -n "🤖 reviewing ... "
RAW_REVIEW_PROMPT=$(cat $REVIEW_PROMPT)
COMMIT_LOG=$(git log -n 5 -p)
codex --sandbox workspace-write \
  --ask-for-approval never \
  --model $REVIEW_MODEL \
  --config model_reasoning_effort="$REVIEW_MODEL_REASONING_EFFORT" \
  exec "$RAW_REVIEW_PROMPT\n$COMMIT_LOG" > $AGENT_LOGS 2>&1
echo "✅ reviewed."
