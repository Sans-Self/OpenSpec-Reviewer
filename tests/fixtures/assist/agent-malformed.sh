#!/bin/sh
# A fake agent CLI that answers in prose instead of JSON.
printf '%s\n' "$@" > "$0.argv"
cat > "$0.stdin"
cat <<'REPLY'
I read the requirement and it looks fine to me.
Nothing further to report.
REPLY
