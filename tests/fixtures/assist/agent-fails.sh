#!/bin/sh
# A fake agent CLI that refuses: the reviewer sees its stderr.
printf '%s\n' "$@" > "$0.argv"
cat > "$0.stdin"
echo "not logged in; run the agent once by hand" >&2
exit 3
