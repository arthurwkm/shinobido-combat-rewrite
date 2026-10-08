You are running unattended overnight; the owner is asleep and will read `PROGRESS.md` and `git log`
in the morning. Anything that needs the owner goes under Blockers, written as a question answerable in\none line; the runner may ask the owner and write the answer below it as `- Owner: ...`. Act on an\nOwner answer, then remove it and the Blockers it answers.

Follow `AGENTS.md` exactly. Read `PROGRESS.md` (skip its Log) and only the `docs/formats.md` sections
your item needs. Work on the first unblocked unchecked item: it is one feature. Commit each green step
with `PROGRESS.md` updated in the same commit, and keep going without stopping to report progress.
When the item is checked off, or you record a Blocker for it, add one Log line, commit and exit:
the runner starts the next feature in a fresh session. Never exit with uncommitted work (Codex: the runner commits for you).
An item that needs an executable lookup is not blocked: do the lookup.
If no unchecked item is unblocked when you start, exit at once without a commit.
