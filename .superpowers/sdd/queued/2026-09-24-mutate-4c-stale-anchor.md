# mutate-4c.sh mutation 5's anchor matches nothing

Found by file-split Task 5's fix round (2026-09-24), not caused by it:
`mutate-4c.sh`'s mutation 5 anchor, `cursor = self.next_template(&mut strings, parse, indent);`,
matches nothing in `rexx-exec/src/parse_template.rs` at `1b851ae96` or at
`ed8cb3f03`. A mutation whose anchor is absent never applies, so whatever it
was meant to witness is unwitnessed. Find when the anchor went stale
(`git log -S`), re-anchor it to the current code, and run it to show it is
caught.
