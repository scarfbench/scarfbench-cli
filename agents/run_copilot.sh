#!/usr/bin/env bash
set -euo pipefail
# Usage: ./run_copilot.sh --prompt <prompt.txt> [--out <OUTDIR>]
PROMPT_FILE=""
OUT_DIR="./.agent_out"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --prompt) PROMPT_FILE="$2"; shift 2 ;;
    --out)    OUT_DIR="$2";     shift 2 ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
done
[[ -z "${PROMPT_FILE}" ]] && { echo "Missing --prompt"; exit 1; }
[[ -f "${PROMPT_FILE}" ]] || { echo "Prompt not found: ${PROMPT_FILE}"; exit 1; }

mkdir -p "${OUT_DIR}"
ts="$(date +'%Y%m%d-%H%M%S')"
stdout_log="${OUT_DIR}/copilot-${ts}.out.log"
stderr_log="${OUT_DIR}/copilot-${ts}.err.log"

# TODO: replace with your real non-interactive Copilot/Claude CLI:
# example placeholder:
bash -lc "gh copilot chat --model 'claude-3.5' --prompt-file \"${PROMPT_FILE}\" --format json" \
  > "${OUT_DIR}/copilot-${ts}.json" \
  2> >(tee -a "${stderr_log}" >&2) \
  | tee -a "${stdout_log}" >/dev/null

echo "Wrote ${OUT_DIR}/copilot-${ts}.json"