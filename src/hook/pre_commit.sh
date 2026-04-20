#!/bin/sh
# Installed by tenet. Ensures generated AGENTS.md files are in sync
# with .context/ whenever rule files are committed.

set -e

# Only run if .context/ or AGENTS.md files are staged.
if ! git diff --cached --name-only --diff-filter=ACMR | \
     grep -qE '^(\.context/|.*AGENTS\.md$)'; then
    exit 0
fi

# Verify compiled tree matches .context/.
if ! command -v tenet >/dev/null 2>&1; then
    echo "warning: tenet command not found; skipping compile check." >&2
    exit 0
fi

if ! tenet lint --check-compiled --quiet 2>/dev/null; then
    echo "error: .context/ is out of sync with generated AGENTS.md files." >&2
    echo "  fix: tenet compile && git add -A" >&2
    exit 1
fi

exit 0
