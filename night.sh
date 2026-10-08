#!/bin/zsh
# Overnight session, unattended: headless Claude in auto mode inside the sandbox defined in
# night-settings.json, on Sonnet to stretch the Pro usage limit. Keeps the Mac awake (on AC power,
# lid open) only while it runs.
#
#   ./night.sh            run until 07:00
#   ./night.sh 05:30      run until 05:30
#
# When a run ends on an error (usually the 5-hour usage limit) it waits 30 minutes and resumes the
# same session. When a run ends normally it resumes, unless that run made no new commit (the agent
# stopped on purpose: done or blocked). Logs: target/logs/night-<date>.log
cd "${0:A:h}" || exit 1
[[ -z $NIGHT_AWAKE ]] && NIGHT_AWAKE=1 exec caffeinate -i -s "$0" "$@"

CLAUDE=/Users/arthursantos/.claude/local/claude
END=$(date -j -f %H:%M "${1:-07:00}" +%s)
(( END < $(date +%s) )) && (( END += 86400 ))
SID=$(uuidgen | tr A-Z a-z)
LOG=target/logs/night-$(date +%Y-%m-%d).log
mkdir -p target/logs

run() {
  "$CLAUDE" -p "$1" "${@:2}" --model sonnet --permission-mode auto \
    --settings ./night-settings.json --output-format json >>"$LOG" 2>&1
}
say() { echo "$1" | tee -a "$LOG" }
commits() { git rev-list --count HEAD 2>/dev/null || echo 0 }

say "== $(date) start, session $SID, until $(date -r $END +%H:%M). Running; log: $LOG"
before=$(commits)
run "$(<NIGHT_PROMPT.md)" --session-id "$SID"
rc=$?
while (( $(date +%s) < END )); do
  if (( rc != 0 )); then
    say "== $(date) exit $rc, waiting 30 min"
    sleep 1800
  elif (( $(commits) == before )); then
    say "== $(date) clean exit with no new commit: done or blocked"
    break
  fi
  before=$(commits)
  say "== $(date) resuming"
  run "Continue per AGENTS.md: next unchecked item in PROGRESS.md, or update it and exit if everything
left is blocked." --resume "$SID"
  rc=$?
done
say "== $(date) end"