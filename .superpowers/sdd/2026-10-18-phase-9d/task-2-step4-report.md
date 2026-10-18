# Phase 9d Task 2, Step 4: disposition lines

## Counting

Every non-`gate:` row of `queued-inventory.tsv` was processed by one script, which writes one file per row. The script printed the count of files written, and `git diff --stat` over `.superpowers/sdd/queued` agreed. A final loop over every `*.md` in the queued directory checked that line 1 starts with `**` and one of the five keywords, and that line 2 is blank.

## Skipped

None. Every row names an existing markdown file, so no `.patch` or other non-markdown file needed listing.

## Old first lines replaced

* `2026-10-10-translation-error-inserts.md`: CLOSED line rewritten, report path kept.
* `2026-10-11-output-lost-on-kill.md`: three-line CLOSED paragraph replaced, sha and Deviation 40 reference kept.
* `2026-10-13-class-side-string-method-not-sent.md`: CLOSED line rewritten, sha and date kept in the evidence and text.
* `2026-10-13-error-position-ampersand.md`: INSERTS CLOSED line rewritten into the OPEN cluster shape, with the insert facts and the open `instanceMethod` item kept.

## Notes

The seven `debug-*` rows were proposed 9d but ruled `phase:11.1`; the ruled column was used.
