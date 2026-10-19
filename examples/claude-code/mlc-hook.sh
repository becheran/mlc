#!/usr/bin/env bash
# Claude Code PostToolUse hook: check the links of every markup file Claude just wrote or edited.
#
# Install: copy this file to .claude/hooks/mlc-hook.sh (chmod +x) and add the entry from
# settings.json to .claude/settings.json. Needs `mlc` and `jq` on the PATH.
#
# Exit code 2 makes Claude Code show stderr to Claude, so it can fix the links right away.
# Web links are not checked (--offline) to keep the hook fast. Remove the flag to check them too.

command -v mlc > /dev/null || exit 0
command -v jq > /dev/null || exit 0

file=$(jq -r '.tool_input.file_path // empty')

case "$file" in
  *.md | *.markdown | *.html | *.htm) ;;
  *) exit 0 ;;
esac
[ -f "$file" ] || exit 0

if ! output=$(mlc --offline --quiet "$file" 2>&1); then
  echo "mlc found broken links in $file:" >&2
  echo "$output" >&2
  exit 2
fi
