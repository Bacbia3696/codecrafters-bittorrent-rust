#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTER_DIR="${TESTER_DIR:-/Users/datnguyen/playground/bittorrent-tester}"
REPO_DIR="${CODECRAFTERS_REPOSITORY_DIR:-$SCRIPT_DIR}"
SKIP_ANTI_CHEAT="${CODECRAFTERS_SKIP_ANTI_CHEAT:-true}"
COURSE_DIR="${COURSE_DIR:-/Users/datnguyen/playground/build-your-own-bittorrent}"
STAGE_DESC_DIR="${STAGE_DESC_DIR:-$COURSE_DIR/stage_descriptions}"

ALL_STAGES=(
  ns2 eb4 ah1 mn6 ow9 rb2 bf7 fi9 ca4 nd2 jv8
  hw0 pk2 xi4 jk6 ns5 zh1 qv6 dv7
)

usage() {
  cat <<'EOF'
Usage:
  ./test_local.sh                     # run all stages in order
  ./test_local.sh all                 # run all stages in order
  ./test_local.sh until <slug>        # run from ns2 up to <slug>
  ./test_local.sh <slug1> <slug2> ... # run explicit stages in provided order

Env overrides:
  TESTER_DIR=/path/to/bittorrent-tester
  CODECRAFTERS_REPOSITORY_DIR=/path/to/your-solution
  CODECRAFTERS_SKIP_ANTI_CHEAT=true|false
  COURSE_DIR=/path/to/build-your-own-bittorrent
EOF
}

stage_title() {
  case "$1" in
    ns2) echo "Decode bencoded strings" ;;
    eb4) echo "Decode bencoded integers" ;;
    ah1) echo "Decode bencoded lists" ;;
    mn6) echo "Decode bencoded dictionaries" ;;
    ow9) echo "Parse torrent file" ;;
    rb2) echo "Calculate info hash" ;;
    bf7) echo "Piece hashes" ;;
    fi9) echo "Discover peers" ;;
    ca4) echo "Peer handshake" ;;
    nd2) echo "Download a piece" ;;
    jv8) echo "Download the whole file" ;;
    hw0) echo "Parse magnet link" ;;
    pk2) echo "Announce extension support" ;;
    xi4) echo "Send extension handshake" ;;
    jk6) echo "Receive extension handshake" ;;
    ns5) echo "Request metadata" ;;
    zh1) echo "Receive metadata" ;;
    qv6) echo "Download a piece (magnet)" ;;
    dv7) echo "Download the whole file (magnet)" ;;
    *) echo "$1" ;;
  esac
}

stage_summary() {
  local slug="$1"
  local f line
  for f in "$STAGE_DESC_DIR"/*-"$slug".md; do
    if [[ -f "$f" ]]; then
      line="$(awk 'NF { print; exit }' "$f" | sed -E 's/^#+[[:space:]]*//')"
      if [[ -n "$line" ]]; then
        printf '%s' "$line"
        return 0
      fi
    fi
  done
  return 1
}

contains_stage() {
  local needle="$1"
  local item
  for item in "${ALL_STAGES[@]}"; do
    if [[ "$item" == "$needle" ]]; then
      return 0
    fi
  done
  return 1
}

build_json() {
  local slugs=("$@")
  local json="["
  local i slug upper
  for i in "${!slugs[@]}"; do
    slug="${slugs[$i]}"
    upper="$(printf '%s' "$slug" | tr '[:lower:]' '[:upper:]')"
    json+="{\"slug\":\"${slug}\",\"tester_log_prefix\":\"tester::#${upper}\",\"title\":\"Stage #${upper} (${slug})\"}"
    if (( i < ${#slugs[@]} - 1 )); then
      json+=","
    fi
  done
  json+="]"
  printf '%s' "$json"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

SELECTED_STAGES=()

if [[ $# -eq 0 || "${1:-}" == "all" ]]; then
  SELECTED_STAGES=("${ALL_STAGES[@]}")
elif [[ "${1:-}" == "until" ]]; then
  if [[ $# -ne 2 ]]; then
    echo "Error: expected 'until <slug>'." >&2
    usage
    exit 1
  fi

  target="$2"
  if ! contains_stage "$target"; then
    echo "Error: unknown stage slug '$target'." >&2
    exit 1
  fi

  for slug in "${ALL_STAGES[@]}"; do
    SELECTED_STAGES+=("$slug")
    if [[ "$slug" == "$target" ]]; then
      break
    fi
  done
else
  SELECTED_STAGES=("$@")
  for slug in "${SELECTED_STAGES[@]}"; do
    if ! contains_stage "$slug"; then
      echo "Error: unknown stage slug '$slug'." >&2
      exit 1
    fi
  done
fi

if [[ ! -d "$TESTER_DIR" ]]; then
  echo "Error: TESTER_DIR not found: $TESTER_DIR" >&2
  exit 1
fi

TEST_CASES_JSON="$(build_json "${SELECTED_STAGES[@]}")"

echo "Running stages in order:"
for slug in "${SELECTED_STAGES[@]}"; do
  printf '  - %s: %s\n' "$slug" "$(stage_title "$slug")"
  if summary="$(stage_summary "$slug")"; then
    printf '    %s\n' "$summary"
  fi
done
echo "Tester dir: $TESTER_DIR"
echo "Repo dir: $REPO_DIR"

cd "$TESTER_DIR"
CODECRAFTERS_REPOSITORY_DIR="$REPO_DIR" \
CODECRAFTERS_SKIP_ANTI_CHEAT="$SKIP_ANTI_CHEAT" \
CODECRAFTERS_TEST_CASES_JSON="$TEST_CASES_JSON" \
go run ./cmd/tester
