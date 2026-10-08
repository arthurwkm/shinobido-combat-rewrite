You are running unattended overnight; the owner is asleep and will read `PROGRESS.md` and `git log`
in the morning. Follow `AGENTS.md` exactly, including "The spirit of the goal".

Start by checking health: `cargo test -q`, `cargo clippy -q --all-targets`, `.venv/bin/python tools/compare.py`.
Anything red is fixed and committed first. Then read `PROGRESS.md` (skip its Log) and only the
`docs/formats.md` sections your item needs. If an item is tagged `(needs viewer check, for Claude)` and you
are Claude, do that check first. Otherwise work on the first unchecked item: it is one feature.
Before every commit, review your own `git diff` for bugs and fix them; fix bugs you find in older code
too, with a test. Commit each green step with `PROGRESS.md` updated in the same commit, and keep going
without stopping to report progress. When the item is checked off, add one Log line, commit and exit:
the runner starts the next feature in a fresh session. Never exit with uncommitted work (Codex: the runner commits for you).

See your work. You can run the viewer unattended: `target/debug/viewer --shot ...` or `--record ...` as a
plain command (no pipes, redirects, `&&`, `cd`). Any item that changes what the viewer shows or how it
plays is checked by looking at the result next to frames of the same action in `videos/`. Write what you
compared and what still differs in the evidence. "Look unverified" is not done: never check an item on it.
A difference you see is a bug: fix it in this item or add an item for it.

Never ask the owner to decide anything. Every decision is yours: take your own recommendation, note it
in the Log line, mark guessed values `ESTIMATE`, and keep going. An item that needs an executable lookup
is not blocked: do the lookup. An approach that failed 3 times closes the item as `ESTIMATE`, not a Blocker.
A gameplay clip that would help goes under Requests in `PROGRESS.md`; it never blocks.

The checklist is not the finish line; the goal is. When only the Acceptance review item is left, do it
(AGENTS.md Loop step 9): compare the whole result with the goal and the clips, and turn every gap into a
new item. Being "done" is earned by a review pass that finds nothing missing, not by an empty list.

Blockers are only for: work outside the project goal, or changing files outside this folder (including
system software). One `- ` line per question, answerable in one line. The runner may ask the owner and
write each answer below its question as `  - Owner: ...`. Act on each Owner answer, then remove it and its Blocker.
Exit without a commit only when the Acceptance review item is checked or every remaining item is blocked.
