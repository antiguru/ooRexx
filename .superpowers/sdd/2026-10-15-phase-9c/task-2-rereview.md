# Phase 9c Task 2 re-review (fix round 1, `aade11443..0f8af4e3a`)

Verdict: approved, no open findings.

## Findings from the first review

1. Important 1 (startup perf): addressed. `cg-9c-t2-fix1b/summary.tsv` was read and the report's
   figures recomputed from it: startup head 119,166,974 vs base 119,494,272 = -0.2739%; interpret
   4,215,836,639 vs 4,219,634,339 = -0.0900%; parse 1,652,794,456 vs 1,653,123,818 = -0.0199%.
   Spreads across the three rounds are tiny (head startup r2 and r3 identical). All match the report.
2. Minor 1 (`an_insert_is_not_substituted_again` could not fail): now `E99_943` with
   `[b"&2", b"&1"]`. A mutant substituting twice turns it red (only that test of 17).
3. Minor 2 (inventory test sampled five constants): `build.rs` generates `ARITIES` from
   `Msg::arity()`; the test compares every entry to an independently written
   `highest_placeholder` over `lookup` text and checks the length equals `MESSAGES`. A mutant
   capping each insert digit at 3 in `highest_insert` turns it red (6 passed, 1 failed).
4. Minor 3 (false "no existing row reached" sentence): replaced with a true one that points at
   the reviewer's mutants D, F, G, H.
5. Minor 4 (eager `previous()`): gone. `parse_paren_expression` and `arg_list_inner` capture
   `cursor.position() - 1` and index tokens only inside the cold `unmatched`.

## Size test

`a_parse_error_is_one_word` pins 8 for `ParseError` and `Result<(), ParseError>`. A mutant adding a
`u8` beside the box turns it red.

## Speculative construction

Searched `rexx-parse/src` for `.ok()`, `is_err`, `or_else`, `ok_or`, `unwrap_or(`, `map_or`,
`let _ =`, `Err(_)`, `if let Err` and every `ParseError::new`/`raise`/`error(` site. No parser path
builds a `ParseError` and discards it. Candidates read:
- `block.rs:395` `label_error` returns `Some(ParseError::new(..))` for every label in a block, but
  the sole caller (`block.rs:651`) returns it as `Err` at once, so nothing is discarded.
- `directive.rs:836,869,877` and `instruction.rs:863` build the error in `ok_or_else`/`map_err`
  closures, which run only on failure.
- `scanner.rs:62` and `lib.rs:119-180` consume the error and stop the parse.
Parse and startup callgrind totals fell, consistent with this. No malloc measurement was needed.

## Surface and comments

`Deref` and `DerefMut` expose only `code`, `sub` and `byte`, which were already `pub` mutable
fields on the old struct; `end` and `inserts` stay private. `echoing` uses `DerefMut`. No new
mutable surface. No em-dashes in the added lines; the new comments state current behaviour.

## Open

None. Note for the controller, not a finding: every `ParseError` now allocates its box (the
report's concern 3). The measurement above supports the trade; error paths are cold.

Scratch worktree and target dir removed.
