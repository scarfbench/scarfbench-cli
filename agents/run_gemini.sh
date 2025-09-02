#!/usr/bin/env bash
set -euo pipefail

PROMPT_FILE=""
OUT_DIR="./.agent_out"
MODEL="gemini-2.5-flash"
FORMAT="json"  # json | text (depending on CLI support)

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
if [[ -z "${GEMINI_API_KEY:-}" ]]; then
  echo "ERROR: GEMINI_API_KEY not set in container env." >&2
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

# --- Ensure npm/npx available ---
if ! command -v npx >/dev/null 2>&1; then
  echo "[install] npm/npx not found; installing..."
  apt-get update && apt-get install -y npm
fi

mkdir -p "$OUT_DIR"
ts="$(date +'%Y%m%d-%H%M%S')"
stdout_log="${OUT_DIR}/gemini-${ts}.out.log"
stderr_log="${OUT_DIR}/gemini-${ts}.err.log"
out_path="${OUT_DIR}/gemini-${ts}.${FORMAT}"

echo "== Gemini run"
echo "Model:  $MODEL"
echo "Prompt: $PROMPT_FILE"
echo "Out:    $out_path"

set -o pipefail
bash -lc "npx https://github.com/google-gemini/gemini-cli --model \"${MODEL}\" --file \"${PROMPT_FILE}\" --format ${FORMAT}" \
  > "$out_path" \
  2> >(tee -a "$stderr_log" >&2) \
  | tee -a "$stdout_log" >/dev/null

echo "Wrote $out_path"