**CLOSED 2026-10-30 by Task 8.8d (03d74f029).**

# mutate-4c.sh mutation 5's anchor matches nothing

Found by file-split Task 5's fix round (2026-09-24), not caused by it:
`mutate-4c.sh`'s mutation 5 anchor, `cursor = self.next_template(&mut strings, parse, indent);`,
matches nothing in `rexx-exec/src/parse_template.rs` at `1b851ae96` or at
`ed8cb3f03`. A mutation whose anchor is absent never applies, so whatever it
was meant to witness is unwitnessed. Find when the anchor went stale
(`git log -S`), re-anchor it to the current code, and run it to show it is
caught.

## Task 8.8d disposition

The existing mutation-5 anchor now matches the current comma-boundary source handoff. A committed, compiled faithful mutation repeats the previous PARSE ARG source and fails the existing multi-template output assertion, then passes after exact restoration. Bounded history identifies the old assignment replacement at 107d72ee6, and the full 900-second script and its broad instruments were explicitly not run under the controller ruling. Task 8.8d records the exact edits, commits, statuses and bounds in t8.8/instruments.

See [Task 8.8d report](../2026-10-18-phase-9d/task-8.8d-report.md) for per-subpart evidence and limits.
