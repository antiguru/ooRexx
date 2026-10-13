# Phase 9b Task 3 report: the RexxUtil stem routines

Status: DONE_WITH_CONCERNS (concerns at the end, none blocking).

Commits:

- `a6c147e62` Phase 9b Task 3: the RexxUtil stem routines and SysDumpVariables
- `28aa77b29` Phase 9b Task 3: SysDumpVariables holds its tail snapshot
- the records commit that carries this report (sha in the reply)

## What was implemented

`SysStemSort`, `SysStemCopy`, `SysStemDelete`, `SysStemInsert` and `SysDumpVariables`, in a new
`rust/crates/rexx-exec/src/builtin/rexxutil/stems.rs`, each row switched from
`util(.., Owner::Phase9)` to `util_run(..)` in `internal_routines.rs`.

- Each body follows its `RexxUtilCommon.cpp` function step for step, in the oracle's order of
  checks and writes, so the partial work a missing element leaves behind matches (witnessed).
- Stems are reached through the stem object, never through variable names:
  `GetStemArrayElement` is `Interp::stem_object_tail` (no default, a tombstone answers `None`),
  `SetStemArrayElement` is `stem_object_write`, `DropStemArrayElement` is a new
  `Interp::stem_drop_element` (drops only a tail the stem holds), `GetAllStemElements` is a new
  `stem_live_tails`, `GetStemValue` a new `stem_own_value`. A stem name argument resolves through a
  new `Interp::context_stem_named` (`getContextStem`), which `dispatch/library.rs`'s `context_stem`
  now calls too, so the native API and RexxUtil share one resolver.
- `SysStemSort` ports `StemClass::sort`: the count read at nine digits, the `999999999999999999`
  "to the end" sentinel for `last` and `lastcol` (given or defaulted, as in the C++), the
  `s.foo` form split at its first period with the tail prefix upcased, the merge sort itself
  (insertion below eight, exponential-search merge), `sortCompare` and `sortCaselessCompare` with
  `caselessCompare`'s signed-`char` bytes (a byte from 0x80 up orders below ASCII), and the column
  forms' read of the terminating NUL one byte past the shorter value. Each quirk is witnessed.
- `SysStemCopy`'s 40.924 for a missing source element names the loop offset, not the index, as
  the C++ does (witnessed: `position "0"`).
- `SysDumpVariables` reads the caller's variables through a new
  `dispatch::context::local_variable_entries` (split out of `local_variables`, which builds its
  `Directory` from it), writes a stem's own value and then its live tails, simple lines ending LF
  and tail lines CR LF, appends to the named file (created `0666` before the umask) or writes to
  standard output, and raises 40.1 where the file does not open.
- New `Raised` constructors: `routine_failed` (40.1), `stem_argument_unresolved` (a body's 40.919),
  `invalid_routine_option` (40.920), `arguments_out_of_order` (40.921), `stem_count_invalid`
  (40.922), `stem_range` (40.923), `stem_element_unset` (40.924). Argument-marshalling raises
  carry `lineless` and `internal_package` ("Error 88 running REXX:"), body raises do not ("running
  <program> line N"), both measured.
- `redirect.rs`'s `stem_object_tail` and `is_stem_object` widened to `pub(crate)`.

No `unsafe`, no new dependency, no side map.

## Oracle measurements behind it

All runs under `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib
timeout -k 5 20 /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` from a fresh `mktemp -d`
directory, stdout, stderr and status read separately.

- An `OPTIONAL_CSTRING` argument with no string value is 88.909 (`.nil` measured). The crate first
  used the `STRING`-fallback conversion and answered 40.920 with `"The NIL object"`; corrected to
  `Interp::string_conversion` (`requiredString`).
- An error raised before the routine's work stops it: `SysStemSort 's.', 'Q'` leaves the stem
  unsorted though the C++ code continues past `ThrowException` (measured, stem unchanged).
- `SysStemSort` with element 0 of `999999999` answers 5.0 on the oracle (`new_array(bounds * 3)`
  under the 1 GiB cap). This crate answers 40.924 for that program. Licensed best-effort OOM
  divergence, not in any witness.
- Review Focus 1 shapes: a stem default never counts as an element (40.922 with no element 0,
  40.924 for a missing element below the count, on all four routines); `'2.0'` and `' 2 '` are
  counts, `'2.5'`, `''`, `'x'` are 40.922; `-1` is 40.922 for Delete and Sort but a usable count
  for Copy and Insert (40.923 or a no-op, each witnessed); `'1E1'` is a count of 10;
  `1234567890` is a count for Copy, Delete and Insert (18 digits) and 40.922 for Sort (9 digits);
  `a.01` is not `a.1`; tails that are not whole numbers are ignored; a dropped tail reads back as
  its name, not the stem default.

## Review Focus 3: the SF #2068 shape

`rust/corpus/oracle-crashes.txt` had no `SysStemSort` entry (checked: `grep -n -i "stemsort\|2068"
corpus/oracle-crashes.txt` printed nothing). Entry 37 now records it. The shape is wider than the
one line: every error `StemClass::sort` raises itself crashes the oracle when untrapped. Measured
2026-10-13, 3 runs of 3 each, rc 139, stdout `go`: an element-2-missing program (40.924) and a
`first` past the count (40.923). The SF #2068 one-liner was **not** run (project memory's
measurement stands). Trapped, all three errors answer on the oracle, which is how ooTest's group
and the corpus witness reach them.

The crate's untrapped answer is pinned crate-only by
`builtin::rexxutil::stems::tests::an_untrapped_error_inside_the_sort_is_reported`, a datadriven
file `crates/rexx-exec/tests/stem_sort_untrapped_cases/crate` with five raising shapes (the SF
#2068 line, a stem default only, elements from 4 up, 40.923, 40.924) and two adjacent successes.
Its header gives the reason: no differential can run these, so the expected blocks are this
crate's own output, the error the trapped oracle raises reported as any routine's error is.

## Tests and results

Commands from `rust/`, every one in the foreground, exit statuses read unpiped.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0, at `28aa77b29`'s tree |
| lib + tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib --test closed_phases --test internal_routines --test owners --test phase_ten_reasons --test refusal_dispositions --test refusal_sites` | exit 0: lib 1090 passed, 8, 2, 6, 5, 3, 5 passed |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 ... cargo test -p rexx-exec --test refusal_sites` | re-derived, exit 0: the seven new constructors as `body`, three marshalling constructors now `body+send`, the rest line shifts |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| suite groups | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/rexxutil/SysStemCopy,base/rexxutil/SysStemDelete,base/rexxutil/SysStemInsert,base/rexxutil/SysStemSort,base/rexxutil/SysDumpVariables REXX_SUITE_TABLE=<records>/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 0 at `28aa77b29`: all five groups green, 44 of 44 tests passed on both sides (Task 2's start: 0 passed, 44 blocked) |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0 at `28aa77b29`, `1029 of 1029 matching` |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 0 at `28aa77b29`, 37 passed, the L0 subset (with the four witnesses) included |
| ir_recorded | `... cargo test -j 4 --release -p rexx-exec --test ir_recorded` | exit 0 at `28aa77b29`, 28 passed |
| trace_oracle, coverage | `... cargo test -j 4 --release -p rexx-exec --test trace_oracle --test coverage` | exit 0 at `28aa77b29`, 41 and 21 passed |

Raw outputs: `docs/superpowers/records/2026-10-13-phase-9b/task-3/` (`run-output.txt`,
`table.tsv`, `verdicts.tsv`, `tests.tsv`, `oracle-differences.tsv`, `corpus-output.txt`,
`collect-stress-output.txt`, `ir-recorded-output.txt`, `trace-coverage-output.txt`), each with
its commit on the first line.

### Corpus witnesses

`rust/corpus/lang/rexxutil_stem_sort.rex`, `rexxutil_stem_copy.rex`,
`rexxutil_stem_delete_insert.rex`, `rexxutil_dump_variables.rex`, listed in
`rust/corpus/phase-9.txt`, each with its `sourceline_oracle/<name>.txt` generated by that test's
driver (`grep -a`, because the sort witness prints a NUL byte and the wrapper's `grep` skips
binary input). Each was run on the oracle first and its stdout read before the crate ran it; each
ends with one untrapped error so the rendering is compared too. Oracle against crate, all three
descriptors identical for all four at `28aa77b29`'s binary.

`SysDumpVariables` lists variables in the oracle's hash order (`GetAllContextVariables` and
`GetAllStemElements` are both `Directory` objects), so the witness writes the dump to a file and
sorts its lines before printing (a multiset comparison). The standard-output form is used only
where it prints one line or none. This crate writes the activation's variables in plan order and
a stem's tails in the stem's own order. Bucket order is not copied.

A further probe, not committed: 72 sort configurations (8 option sets, sizes 1 to 297) of
pseudo-random strings with `_`, a high byte and NUL, checksummed: identical on both sides.

### TDD evidence

RED, before any body existed (`c.sh` runs `target/release/rexx-run` under `memcap 2G`): each of
the four witnesses exited 120 with `rexx-exec: routine "SYSSTEMSORT" is not implemented (Phase 9)`
(and the `SYSSTEMCOPY`, `SYSSTEMDELETE`, `SYSDUMPVARIABLES` equivalents), stdout empty. Expected:
the rows were `util(.., Owner::Phase9)`.

GREEN: the differential script after implementation printed `stdout same` and `stderr same` with
equal status for all four (rc 216, 168, 216, 216).

### Mutations, each reversed by an exact edit

- Column compare without the terminating NUL (`- start + 1` to `- start`): the sort witness's
  stdout differs from the oracle's. Reversed, checked by `grep -n "let reachable"`.
- Dropping `SysDumpVariables`'s hold on the tail snapshot:
  `a_dumped_stem_keeps_its_tails_when_a_string_method_empties_it` panics at `not_in_arena`'s
  `"a live value"` assertion (`dispatch.rs:1710`) under collect-on-every-allocation. Restored from a copy, `grep -c` 1.
  The first version of that test used a literal value and stayed green with the hold deleted,
  because a literal is rooted by the program. It now uses a computed value.
- Dropping the sort's own `push_temp` of its converted values left a collect-every-allocation
  test green: `required_string_dispatch` already roots what it answers. The `push_temp` and that
  test were deleted (`28aa77b29`), not kept as a test that cannot fail.

## Derived rows

None record these groups. `grep -rln -a "SysStem\|SysDumpVariables" corpus/
crates/*/tests/*.tsv crates/*/tests/*.txt` names only `suite-9b-groups.txt`, `oracle-crashes.txt`,
`phase-9.txt` and the witnesses. `grep -n -a rexxutil corpus/sim-gate.tsv corpus/sim-exempt.tsv`
prints nothing, and `concurrency_tests.rs`'s whole-group rows name only `SysSleep` among the
RexxUtil groups. The five groups were blocked by the refusal, not by a row, so no suite row moves.
`refusal-sites.tsv` was re-derived in `a6c147e62`.

## Files changed

- `rust/crates/rexx-exec/src/builtin/rexxutil/stems.rs` (new), `stems/tests.rs` (new)
- `rust/crates/rexx-exec/src/builtin/rexxutil.rs` (`mod stems`, module doc)
- `rust/crates/rexx-exec/src/internal_routines.rs` (five rows)
- `rust/crates/rexx-exec/src/error.rs` (seven constructors)
- `rust/crates/rexx-exec/src/dispatch/hash/stem.rs` (`stem_drop_element`, `stem_live_tails`,
  `stem_own_value`)
- `rust/crates/rexx-exec/src/stem.rs` (`context_stem_named`)
- `rust/crates/rexx-exec/src/dispatch/library.rs` (`context_stem` calls it)
- `rust/crates/rexx-exec/src/dispatch/context.rs` (`local_variable_entries`)
- `rust/crates/rexx-exec/src/redirect.rs` (two helpers widened)
- `rust/crates/rexx-exec/src/run/tests/directives.rs` (the refusal example is now `SysGetKey`,
  and `SysStemSort` joins the delivered side)
- `rust/crates/rexx-exec/tests/stem_sort_untrapped_cases/crate` (new)
- `rust/corpus/lang/rexxutil_*.rex` (four new), `rust/corpus/phase-9.txt`,
  `rust/corpus/oracle-crashes.txt` (entry 37), `rust/corpus/refusal-sites.tsv` (re-derived)
- `rust/crates/rexx-parse/tests/sourceline_oracle/rexxutil_*.txt` (four new)

## Concerns

1. **File layout.** `rexxutil.rs` would have stayed under 1500 lines (804 plus about 800). The
   stems went to `builtin/rexxutil/stems.rs` anyway, as one family per file, since Tasks 4 to 8 add
   to `rexxutil.rs`. The controller's note made the split conditional, so this is a judgement call.
2. **`SysDumpVariables` file names go through `String::from_utf8_lossy`**, as `path_argument` does
   for the existing file routines. A non-UTF-8 name would be altered. That is Review Focus 2's
   ground (Task 4), and the fix belongs with `path_argument`.
3. **The refusal example in `directives.rs`** now names `SysGetKey`, still a Phase 9 row. Whichever
   task implements it has to pick another.
4. **No callgrind run**, per the dispatch. Nothing on a per-clause or dispatch path changed: the
   bodies are new, `context_stem` is the native API's `ResolveStemVariable` path, and
   `local_variables` only split its loop out.
5. `SysStemSort` with a 999999999 count: oracle 5.0, crate 40.924 (licensed OOM divergence).
