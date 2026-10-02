#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
output_dir="$repo_root/docs/assets"
mkdir -p "$output_dir"

for command in xvfb-run xwininfo import; do
  if ! command -v "$command" >/dev/null 2>&1; then
    printf 'Missing required command: %s\n' "$command" >&2
    exit 2
  fi
done

cargo build --manifest-path "$repo_root/Cargo.toml" --release \
  --package xengui-showcase --package xengui-quickstart

capture_window() {
  local binary="$1"
  local title="$2"
  local output="$3"
  local width="$4"
  local height="$5"

  xvfb-run -a -s "-screen 0 ${width}x${height}x24" bash -c '
    set -euo pipefail
    binary="$1"
    title="$2"
    output="$3"
    "$binary" &
    app_pid=$!
    trap "kill $app_pid >/dev/null 2>&1 || true" EXIT
    window_id=""
    for _ in $(seq 1 120); do
      window_id=$(xwininfo -name "$title" 2>/dev/null \
        | sed -n "s/^xwininfo: Window id: \\([^ ]*\\).*/\\1/p" \
        || true)
      if [[ -n "$window_id" ]]; then
        break
      fi
      sleep 0.25
    done
    if [[ -z "$window_id" ]]; then
      printf "Window not found: %s\n" "$title" >&2
      exit 3
    fi
    sleep 1
    import -window "$window_id" "$output"
  ' _ "$binary" "$title" "$output" "$width" "$height"
}

capture_window "$repo_root/target/release/xengui-showcase" "XenGui Showcase" \
  "$output_dir/xengui-showcase.png" 1600 1000
capture_window "$repo_root/target/release/xengui-quickstart" "Focus Board · XenGui" \
  "$output_dir/xengui-quickstart.png" 960 640

printf 'Updated %s and %s\n' \
  "$output_dir/xengui-showcase.png" "$output_dir/xengui-quickstart.png"
