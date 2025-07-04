#!/usr/bin/env bash
set -euo pipefail
header="<!-- SPDX-License-Identifier: MIT -->"
missing=0
for f in docs/src/*.md; do
    if ! grep -Fxq "$header" "$f"; then
        echo "Adding license header to $f"
        tmp=$(mktemp)
        printf '%s\n\n' "$header" > "$tmp"
        cat "$f" >> "$tmp"
        mv "$tmp" "$f"
        missing=$((missing+1))
    fi
done
if [ $missing -gt 0 ]; then
    echo "Inserted license header in $missing files."
else
    echo "All documentation files already have license headers."
fi
