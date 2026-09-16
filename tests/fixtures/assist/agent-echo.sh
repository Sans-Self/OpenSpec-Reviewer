#!/bin/sh
# A fake agent CLI for the interactive handoff: it records its argv and
# the prompt file it was pointed at, then exits.
printf '%s\n' "$@" > "$0.argv"
exit 0
