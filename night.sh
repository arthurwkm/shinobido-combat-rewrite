#!/bin/zsh
# Overnight session, unattended. Keeps the Mac and display awake (AC power, lid open; display for viewer shots) while tools/night.py runs
# Claude (Sonnet) until its 5h usage is near full, then Codex, then waits for a reset.
# Thresholds and rules: tools/night.py. Logs: target/logs/night-<date>.log
#
#   ./night.sh            run until 07:00
#   ./night.sh 05:30      run until 05:30
cd "${0:A:h}" || exit 1
[[ -z $NIGHT_AWAKE ]] && NIGHT_AWAKE=1 exec caffeinate -i -s -d "$0" "$@"
exec .venv/bin/python tools/night.py "$@"
