**OPEN: ruled 9d by Moritz 2026-10-18 (Phase 9d inventory).** no probe; tests/refusal_sites.rs code_lines (:73) still drops only inline #[cfg(test)] mod blocks, and Task 3 reads that table.

# The refusal-site scanner reads file-level test modules as production code

Found by File split Task 2 (implementer's concern 1, confirmed by the review).

`rust/crates/rexx-exec/tests/refusal_sites.rs`'s `sources()` walks every `.rs`
file under `src/`, and `code_lines` drops only **inline** `#[cfg(test)] mod {}`
blocks. A test module in its own file (`#[cfg(test)] mod tests;`) is scanned as
if it were surface code: at `d6dd60d90` that already covered `run/tests.rs`,
`run/tests/*.rs` and `ir/drive/tests.rs`, and since `afa268640` it covers
`dispatch/tests.rs` too. No row of `rust/corpus/refusal-sites.tsv` changes
today, but a refusal written in a test would be counted as a production
refusal site, under the `body`, `ir` or `send` tag.

Not silent: any effect changes the derived table, which fails until refreshed.

Fix: resolve each file-level `#[cfg(test)] mod x;` to its file(s) and exclude
them uniformly for every tag, in a commit of its own (a change to test logic,
not a move). Witness: a planted refusal in a file-level test module must not
appear in the table, before and after.

## Second instance, same root: constructors classified by their defining file

Found by File split Task 4, 2026-09-24, by a trial move in a scratch worktree
before any commit. `tests/refusal_sites.rs:265-271` (`surface`) treats a
`Loud` constructor as a method only when it is defined in `error.rs` or
`lib.rs`; anywhere else it is a free function, and every bare occurrence of
its name counts as a construction site. Moving `Loud` to `src/refusal.rs`
would therefore change column 3 on four rows through false sites:
`accessor_variable` (a free fn in lib.rs and dispatch.rs) and
`delegate_variable` (dispatch.rs) `send -> body+send`; `expression` (a field
in `ir/drive.rs`) `body -> body+ir`; `redirection` (a word in a string literal
in `dispatch/tests.rs`) `body -> body+send`, which also blanks that row's
verdict cells.

Both instances are the scanner deciding what a site is by file name. The fix
should key on syntax instead: a constructor is a method when it is inside an
`impl` block; a file-level `#[cfg(test)] mod x;` target is test code. Then the
deferred move of `Loud` into its own module (File split Task 4, ruled out of
that task) can run as a pure move.
