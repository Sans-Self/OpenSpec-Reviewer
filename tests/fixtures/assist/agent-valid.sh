#!/bin/sh
# A fake agent CLI: records the argv and the stdin it was given next to
# itself, then answers with two hints.
printf '%s\n' "$@" > "$0.argv"
cat > "$0.stdin"
cat <<'REPLY'
[
  {
    "kind": "compound_condition",
    "message": "the WHEN joins mounting a page and rendering the row",
    "quote": "- **WHEN** a route mounts a page and the index renders",
    "scenario": "Route-mounted page appears as a row"
  },
  {
    "kind": "uncovered_must",
    "message": "no scenario exercises the inherited paths clause"
  }
]
REPLY
