You are running unattended overnight; the owner is asleep and will read `PROGRESS.md` and `git log`
in the morning. Follow `AGENTS.md` exactly.

Start by checking health: `cargo test -q`, `cargo clippy -q --all-targets`, `.venv/bin/python tools/compare.py`.
Anything red is fixed and committed first. Then read `PROGRESS.md` (skip its Log) and only the
`docs/formats.md` sections your item needs. Work on the first unchecked item: it is one feature.
Before every commit, review your own `git diff` for bugs and fix them; fix bugs you find in older code
too, with a test. Commit each green step with `PROGRESS.md` updated in the same commit, and keep going
without stopping to report progress. When the item is checked off, add one Log line, commit and exit:
the runner starts the next feature in a fresh session. Never exit with uncommitted work (Codex: the runner commits for you).

Never ask the owner to decide anything. Every decision is yours: take your own recommendation, note it
in the Log line, mark guessed values `ESTIMATE`, and keep going. An item that needs an executable lookup
is not blocked: do the lookup. An approach that failed 3 times closes the item as `ESTIMATE`, not a Blocker.
A gameplay clip that would help goes under Requests in `PROGRESS.md`; it never blocks.

Blockers are only for: work outside the project goal, or changing files outside this folder (including
system software). One `- ` line per question, answerable in one line. The runner may ask the owner and
write each answer below its question as `  - Owner: ...`. Act on each Owner answer, then remove it and its Blocker.
If no unchecked item is unblocked when you start, exit at once without a commit.
