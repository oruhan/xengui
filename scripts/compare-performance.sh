#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
base_ref="${1:-HEAD^}"
head_ref="${2:-HEAD}"
threshold_percent="${3:-10}"
rounds="${BENCH_ROUNDS:-3}"
temp_root="$(mktemp -d)"

cleanup() {
  git -C "$repo_root" worktree remove --force "$temp_root/base" >/dev/null 2>&1 || true
  git -C "$repo_root" worktree remove --force "$temp_root/head" >/dev/null 2>&1 || true
  rm -rf "$temp_root"
}
trap cleanup EXIT

git -C "$repo_root" worktree add --detach "$temp_root/base" "$base_ref" >/dev/null
git -C "$repo_root" worktree add --detach "$temp_root/head" "$head_ref" >/dev/null
base_sha="$(git -C "$repo_root" rev-parse "$base_ref")"
head_sha="$(git -C "$repo_root" rev-parse "$head_ref")"

prepare_benchmark() {
  local label="$1"
  local worktree="$2"
  mkdir -p "$worktree/benchmarks"
  cp -R "$repo_root/benchmarks/frame_orchestration" "$worktree/benchmarks/frame_orchestration"
  CARGO_TARGET_DIR="$temp_root/target-$label" \
    cargo build --quiet --release --manifest-path "$worktree/benchmarks/frame_orchestration/Cargo.toml"
}

run_benchmark() {
  local label="$1"
  local ref="$2"
  BENCH_REF="$ref" "$temp_root/target-$label/release/xengui-frame-orchestration-benchmark"
}

extract_median() {
  sed -n 's/.*"median_ns_per_frame":\([0-9][0-9]*\).*/\1/p'
}

median_of_runs() {
  sort -n | awk '{ values[NR] = $1 } END { print values[int((NR + 1) / 2)] }'
}

prepare_benchmark base "$temp_root/base"
prepare_benchmark head "$temp_root/head"

base_runs=""
head_runs=""
all_json=""
for _ in $(seq 1 "$rounds"); do
  base_json="$(run_benchmark base "$base_sha")"
  head_json="$(run_benchmark head "$head_sha")"
  base_runs="${base_runs}$(printf '%s\n' "$base_json" | extract_median)"$'\n'
  head_runs="${head_runs}$(printf '%s\n' "$head_json" | extract_median)"$'\n'
  all_json="${all_json}${base_json}"$'\n'"${head_json}"$'\n'
done

base_ns="$(printf '%s' "$base_runs" | sed '/^$/d' | median_of_runs)"
head_ns="$(printf '%s' "$head_runs" | sed '/^$/d' | median_of_runs)"

if [[ -z "$base_ns" || -z "$head_ns" ]]; then
  printf 'Could not parse benchmark output.\n%s\n' "$all_json" >&2
  exit 2
fi

change_percent="$(awk -v base="$base_ns" -v head="$head_ns" 'BEGIN { printf "%.2f", ((head-base)/base)*100 }')"
mkdir -p "$repo_root/artifacts"
printf '%s' "$all_json" > "$repo_root/artifacts/performance-comparison.jsonl"

max_ns="$base_ns"
if (( head_ns > max_ns )); then
  max_ns="$head_ns"
fi
base_bar="$(awk -v value="$base_ns" -v max="$max_ns" 'BEGIN { printf "%.0f", (value / max) * 480 }')"
head_bar="$(awk -v value="$head_ns" -v max="$max_ns" 'BEGIN { printf "%.0f", (value / max) * 480 }')"
base_short="${base_sha:0:12}"
head_short="${head_sha:0:12}"

{
  printf '<svg xmlns="http://www.w3.org/2000/svg" width="720" height="190" viewBox="0 0 720 190" role="img" aria-labelledby="title desc">\n'
  printf '<title id="title">XenGui frame orchestration benchmark</title>\n'
  printf '<desc id="desc">Median nanoseconds per frame. Lower is better.</desc>\n'
  printf '<rect width="720" height="190" rx="20" fill="#17121f"/>\n'
  printf '<text x="28" y="34" fill="#f2eaff" font-family="system-ui,sans-serif" font-size="17" font-weight="700">Full layout + paint CPU orchestration</text>\n'
  printf '<text x="28" y="58" fill="#cfc2dc" font-family="system-ui,sans-serif" font-size="13">Median ns/frame · lower is better · %s alternating rounds</text>\n' "$rounds"
  printf '<text x="28" y="99" fill="#f2eaff" font-family="monospace" font-size="13">%s</text>\n' "$base_short"
  printf '<rect x="150" y="80" width="%s" height="25" rx="12.5" fill="#b8c4ff"/>\n' "$base_bar"
  printf '<text x="645" y="98" text-anchor="end" fill="#f2eaff" font-family="system-ui,sans-serif" font-size="13">%s</text>\n' "$base_ns"
  printf '<text x="28" y="146" fill="#f2eaff" font-family="monospace" font-size="13">%s</text>\n' "$head_short"
  printf '<rect x="150" y="127" width="%s" height="25" rx="12.5" fill="#d0bcff"/>\n' "$head_bar"
  printf '<text x="645" y="145" text-anchor="end" fill="#f2eaff" font-family="system-ui,sans-serif" font-size="13">%s</text>\n' "$head_ns"
  printf '</svg>\n'
} > "$repo_root/artifacts/performance-comparison.svg"

{
  printf '# XenGui performance comparison\n\n'
  printf 'Metric: full layout + paint CPU orchestration, release build, identical workload. Lower is better.\n\n'
  printf '![Measured performance comparison](performance-comparison.svg)\n\n'
  printf '| Revision | Median ns/frame |\n| --- | ---: |\n'
  printf '| `%s` | %s |\n' "$base_sha" "$base_ns"
  printf '| `%s` | %s |\n\n' "$head_sha" "$head_ns"
  printf 'Change: **%s%%**. Regression budget: **%s%%**. Result is the median of %s alternating process runs per revision.\n' "$change_percent" "$threshold_percent" "$rounds"
} > "$repo_root/artifacts/performance-comparison.md"

printf 'base=%s ns/frame head=%s ns/frame change=%s%%\n' "$base_ns" "$head_ns" "$change_percent"
if awk -v change="$change_percent" -v limit="$threshold_percent" 'BEGIN { exit !(change > limit) }'; then
  printf 'Performance regression exceeds %s%%.\n' "$threshold_percent" >&2
  exit 1
fi
