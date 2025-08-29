#!/usr/bin/env bash
set -euo pipefail

PROMPT_FILE=""
OUT_DIR="./.agent_out"
MODEL="gpt-5"
FORMAT="json"   # json | text

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prompt) PROMPT_FILE="$2"; shift 2 ;;
    --out)    OUT_DIR="$2";     shift 2 ;;
    --model)  MODEL="$2";       shift 2 ;;
    --format) FORMAT="$2";      shift 2 ;;
    *) echo "Unknown arg: $1" >&2; exit 2 ;;
  esac
done

# --- Guards ---
if [[ -z "${CURSOR_API_KEY:-}" ]]; then
  echo "ERROR: CURSOR_API_KEY not set in container env." >&2
  echo "Hint: set it via devcontainer.json containerEnv or an env-file." >&2
  exit 1
fi
if [[ -z "$PROMPT_FILE" ]]; then
  echo "ERROR: Missing --prompt <prompt.txt>" >&2
  exit 1
fi
if [[ ! -f "$PROMPT_FILE" ]]; then
  echo "ERROR: Prompt not found: $PROMPT_FILE" >&2
  exit 1
fi

# --- Install Cursor CLI if missing ---
if ! command -v cursor-agent >/dev/null 2>&1; then
  echo "[install] cursor-agent not found; installing via cursor.com/install..."
  curl -fsSL https://cursor.com/install | bash
  # Add to PATH if installer drops binary in ~/.cursor/bin
  export PATH="$HOME/.cursor/bin:$PATH"
fi

mkdir -p "$OUT_DIR"
ts="$(date +'%Y%m%d-%H%M%S')"
stdout_log="${OUT_DIR}/cursor-${ts}.out.log"
stderr_log="${OUT_DIR}/cursor-${ts}.err.log"
out_path="${OUT_DIR}/cursor-${ts}.${FORMAT}"

prompt_text="$(cat "$PROMPT_FILE")"

echo "== Cursor run"
echo "Model:  $MODEL"
echo "Format: $FORMAT"
echo "Prompt: $PROMPT_FILE"
echo "Out:    $out_path"

set -o pipefail
bash -lc "cursor-agent -p \"${prompt_text}\" --model \"${MODEL}\" --output-format ${FORMAT} --force" \
  > "$out_path" \
  2> >(tee -a "$stderr_log" >&2) \
  | tee -a "$stdout_log" >/dev/null

echo "Wrote $out_path"