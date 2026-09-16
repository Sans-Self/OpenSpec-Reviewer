#!/bin/sh
# A fake `claude -p --output-format json`: the reply sits in the
# envelope's `result` field.
printf '%s\n' "$@" > "$0.argv"
cat > "$0.stdin"
cat <<'REPLY'
{"type":"result","is_error":false,"result":"[{\"kind\":\"plain_language\",\"message\":\"says appropriate without naming the thing\"}]"}
REPLY
