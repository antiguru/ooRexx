# Queued: two gaps in closed_phases' exclusions OWNER check

From the Phase 8 final re-review (N2, scratchpad/review-final8/task-9-rereview.md):
- NC-h: an `OWNER: Phase 8` row followed, in the same paragraph and before the next OWNER, by a
  CLOSED/FIXED word passes, because that word is read as the row's resolution.
- NC-i: a lower-case `Owner:` line is not read.
Neither is reachable by any row today. Tighten the paragraph rule and case-fold the key.

## Closed 2026-10-09 by Phase 6.1 Task 7

NC-i was already closed by `51e1d37cc`: `OWNER`, or `Owner:` in any case, is read; Task 7 adds a
fully lower-case `owner:` row to `the_row_check_reads_owners_in_any_case_and_no_negated_resolution`.
NC-h: a resolution word resolves only where it opens a sentence, a paragraph or the clause after a
colon, and a sentence ends at a full stop followed by whitespace;
`the_row_check_reads_only_a_resolution_that_opens_a_clause` holds `EXTERNAL was DELIVERED` open.
