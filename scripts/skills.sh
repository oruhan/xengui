#!/usr/bin/env bash
set -euo pipefail

REPO="https://github.com/abhixv/m3-expressive-design-skill.git"
BRANCH="main"
REMOTE_SUBDIR="skills/m3-expressive"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

DEST="$ROOT/external/skills/m3-expressive"
VERSION_FILE="$DEST/.upstream-commit"
MANIFEST_FILE="$DEST/.upstream-manifest"

TMP="$ROOT/.temp/tmp-m3-expressive"
STAGED_DEST="$ROOT/.temp/tmp-m3-expressive-new"

cleanup() {
    rm -rf "$TMP" "$STAGED_DEST"
}

trap cleanup EXIT

if ! command -v git >/dev/null 2>&1; then
    echo "[m3-expressive] Error: git is not available in PATH." >&2
    exit 1
fi

if ! command -v sha256sum >/dev/null 2>&1; then
    echo "[m3-expressive] Error: sha256sum is not available in PATH." >&2
    exit 1
fi

create_manifest() {
    local dir="$1"
    local output="$2"

    (
        cd "$dir"

        find . \
            -type f \
            ! -name '.upstream-commit' \
            ! -name '.upstream-manifest' \
            -print0 \
            | sort -z \
            | xargs -0 -r sha256sum
    ) > "$output"
}

verify_local_files() {
    if [[ ! -d "$DEST" ]]; then
        return 1
    fi

    if [[ ! -f "$MANIFEST_FILE" ]]; then
        return 1
    fi

    (
        cd "$DEST"
        sha256sum --quiet --check ".upstream-manifest"
    )
}

echo "[m3-expressive] Checking upstream..."

REMOTE_COMMIT="$(
    git ls-remote "$REPO" "refs/heads/$BRANCH" |
        awk 'NR == 1 { print $1 }'
)"

if [[ -z "$REMOTE_COMMIT" ]]; then
    echo "[m3-expressive] Error: could not resolve upstream commit." >&2
    exit 1
fi

LOCAL_COMMIT=""

if [[ -f "$VERSION_FILE" ]]; then
    LOCAL_COMMIT="$(<"$VERSION_FILE")"
fi

NEEDS_UPDATE=false

# Check whether the local version matches the latest upstream commit.
if [[ "$LOCAL_COMMIT" != "$REMOTE_COMMIT" ]]; then
    if [[ -n "$LOCAL_COMMIT" ]]; then
        echo "[m3-expressive] Upstream update detected:"
        echo "  local : $LOCAL_COMMIT"
        echo "  remote: $REMOTE_COMMIT"
    else
        echo "[m3-expressive] No valid local version metadata found."
    fi

    NEEDS_UPDATE=true
fi

# Even when the commit matches, verify every downloaded file.
if [[ "$NEEDS_UPDATE" == false ]]; then
    echo "[m3-expressive] Verifying local files..."

    if verify_local_files; then
        echo "[m3-expressive] Already up to date. All files are intact."
        exit 0
    fi

    echo "[m3-expressive] Missing or modified files detected."
    NEEDS_UPDATE=true
fi

echo "[m3-expressive] Fetching clean copy..."

rm -rf "$TMP" "$STAGED_DEST"

git clone \
    --quiet \
    --depth 1 \
    --filter=blob:none \
    --sparse \
    --branch "$BRANCH" \
    "$REPO" \
    "$TMP"

git -C "$TMP" sparse-checkout set "$REMOTE_SUBDIR"

SOURCE="$TMP/$REMOTE_SUBDIR"

if [[ ! -d "$SOURCE" ]]; then
    echo "[m3-expressive] Error: upstream directory was not found: $REMOTE_SUBDIR" >&2
    exit 1
fi

mkdir -p "$(dirname "$DEST")"

mv "$SOURCE" "$STAGED_DEST"

# Store the exact upstream commit used for this copy.
printf '%s\n' "$REMOTE_COMMIT" > "$STAGED_DEST/.upstream-commit"

# Create a checksum manifest for every downloaded file.
create_manifest \
    "$STAGED_DEST" \
    "$STAGED_DEST/.upstream-manifest"

# Verify the newly downloaded copy before replacing the current one.
(
    cd "$STAGED_DEST"
    sha256sum --quiet --check ".upstream-manifest"
)

# Replace the existing copy only after the new copy is fully verified.
rm -rf "$DEST"
mv "$STAGED_DEST" "$DEST"

echo "[m3-expressive] Updated successfully."
echo "[m3-expressive] Commit: $REMOTE_COMMIT"