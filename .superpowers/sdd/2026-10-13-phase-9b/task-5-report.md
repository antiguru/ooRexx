# Phase 9b Task 5 report: SysFileTree

## Status

DONE_WITH_CONCERNS. `base/rexxutil/SysFileTree` is 26 of 26 under the harness (17 failed at Task 2's
baseline and the 9a close). Commits:

- `813f71787` Phase 9b Task 5: SysFileTree
- the records commit that carries this report (sha in the reply)

## What changed

`SysFileTree` moved out of `builtin/rexxutil.rs` into `builtin/rexxutil/tree.rs` and now follows
`TreeFinder` (`runtime/RexxUtilCommon.cpp`, `platform/unix/SysRexxUtil.cpp`,
`SysFileIterator` in `platform/unix/SysFileSystem.cpp`):

- **Arguments.** All five marshalled first as the C++ does (88.922, 88.901, and 88.909 for an
  argument without a string value, measured for arguments 1, 3 and 4, each `CSTRING` cut at a NUL). Then the target, resolved by Task 4's
  `Found` (now taking its argument position): 40.925 for `""`, `"s.1"`, `.nil`, `12`. Then 40.21
  for a null spec, 40.21 for a null option string, 40.900 for a bad option letter, 40.900 for a
  bad mask in argument 4 or 5, each message cut at 255 bytes as the oracle's 256-byte `snprintf`
  does. Any of those after the target resolved leaves tail `0` set to `0` (an array is already
  emptied), as `StemHandler`'s destructor does. Measured on the oracle (`probes/args.rex`, its
  output `probes/args.oracle.out`).
- **Options.** `F`, `D`, `B` each replace the earlier choice ("last wins"), `L` over `T`, `H` a
  20-wide uncapped size, the default size capped at `9999999999` and 10 wide, `O` the name alone,
  `I` folds case for the final name only.
- **Paths, bytes throughout.** `adjustDirectory` (`*` to `./*`, a trailing `/`, `/.`, `/..` gain a
  wildcard, a leading `~` canonicalised in place, with the C++'s failure behaviour: an unknown
  `~user` stays as spelled, a relative `HOME` is substituted but not normalised). `getFullPath`:
  a spec with no `/` searches the unqualified current directory, `.` as `*`, and `..` as the
  current directory's name with `..` appended, so it finds nothing except from `/`, where it lists
  `/../bin` (measured). A spec with a `/` qualifies its directory part through Task 4's
  `qualify_name`, and where that fails the search is in `/` (measured:
  `~nosuchuser.rexxutil/bin` finds `/bin`). The lossy sites at `rexxutil.rs:797/798/841` are gone.
- **Entries.** `stat` through links, so a link to a directory is listed with `d` and recursed into
  (measured on the oracle). An unreadable directory is listed and contributes nothing below it.
  Recursion reads each directory twice, matches first, then subdirectories, as `recursiveFindFile`
  does.
- **Line format.** `formatFileAttributes`: the three date layouts, the hour rule, two-digit years
  as `(tm_year + 10000) % 100` (1890 renders `90`), and the mode string with the kind letter
  (`d`, `l`, `p`, `b`, `c`, `s`) and `s`/`S`, `s`/`S`, `t`/`T`. Local time through Task 4's
  `program_zone` and `local_time`, so the program's own `TZ` applies.
- **Pattern.** `fnmatch` with `FNM_PATHNAME | FNM_NOESCAPE` in the C locale (the oracle never calls
  `setlocale`): `[:class:]` names added, an unknown class fails the bracket, case folded to lower as
  `FNM_CASEFOLD` does. Every bracket form in the unit test was checked against the oracle
  (`probes/g.rex`, identical on both sides).
- **Dead code.** `datetime::Calendar` and `calendar_of` had only the old `SysFileTree` as caller and
  are deleted. That also gives `local_offset_micros` its own doc block back (the struct had been
  inserted between them).
- **New constructor.** `Raised::routine_null_argument` (40.21). `refusal-sites.tsv` re-derived:
  ignoring the location column, the only change is that row, `body`, `off-send-surface`.

### Order

`SysFileTree`'s order is `readdir` order on the oracle and on this crate, which neither promises.
The witness sorts each result before printing, so its stdout is deterministic and compares
byte for byte, and the doc comment on `file_tree` says the order is not specified. **No Deviation
number is assigned**: the controller should record this as an unspecified order in the manner of
Deviation 33 if that is the ruling. `HASH_ORDERED_STDOUT` was not needed.

### TEST_HUGE

`/dev/core` is a link to `/proc/kcore`, 140737471594496 bytes. The old code used `lstat` (the
link's 11 bytes) and a 12-wide size field. With `stat` and the oracle's widths it passes under the
harness's `memcap 2G` (`CRATE_MEMORY`, `tests/suite.rs:76`); it does no allocation proportional to
the file. The witness covers the same shape with a 20 GiB sparse file (`truncate -s 20G`):
`9999999999` without `H`, `21474836480` with it.

## Tests and results

Commands from `rust/`, every one in the foreground, statuses read unpiped. Raw outputs in
`docs/superpowers/records/2026-10-13-phase-9b/task-5/`, each with its commit on the first line.

| check | command | result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 at `813f71787`'s tree |
| clippy | `CARGO_INCREMENTAL=0 memcap 8G cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 at `813f71787`'s tree |
| lib + tables | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib --test closed_phases --test internal_routines --test owners --test phase_ten_reasons --test refusal_dispositions --test refusal_sites --test no_process_state` | exit 0 at `813f71787`: lib 1095, then 8, 2, 1, 6, 5, 3, 5 passed (`lib-tables-output.txt`) |
| refusal sites | `REXX_REFUSAL_SITES_REFRESH=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --test refusal_sites` | exit 0, one new row as above |
| sourceline | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-parse --test sourceline_oracle` | exit 0 |
| suite group | `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/rexxutil/SysFileTree REXX_SUITE_TABLE=<records>/table.tsv CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test suite l3_core_suite` | exit 0 at `813f71787`, wall 77 s (`run-output.txt`); verdict green, 26 of 26 crate `passed` in `tests.tsv`, `oracle-differences.tsv` empty |
| corpus | `REXX_CORPUS_GATE=1 CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test corpus` | exit 0 at `813f71787`, `1037 of 1037 matching` (`corpus-output.txt`) |
| collect_stress | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test collect_stress` | exit 0 at `813f71787`, 37 passed (`collect-stress-output.txt`) |
| others | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 --release -p rexx-exec --test ir_recorded --test trace_oracle --test coverage --test datetime_zone` | exit 0 at `813f71787`: coverage 21, datetime_zone 22, ir_recorded 28, trace_oracle 41 passed (`other-tests-output.txt`) |
| crate-only | `CARGO_INCREMENTAL=0 memcap 8G cargo test -j 4 -p rexx-exec --lib rexxutil::tree` | 2 passed |

Not run, per the dispatch: the full suite and callgrind. The change touches no dispatch,
allocation or per-call path outside `SysFileTree` itself.

### Before and after

RED: at Task 2's baseline the group was 9 of 26 with 17 failed (`task-2-baseline/tests.tsv`), and
the harness run here before the change named the same 17 as not passed. The argument probe
(`probes/args.rex`) raised none of the 40.21, 40.925, 40.900 and 88.909 errors the oracle raises,
and the tree probe (`probes/tree.rex` over the tree `probes/mktree.sh` builds) differed on F/D
last-wins, `H`, links, mode letters and the `'ff'x` name, which printed as `'efbfbd'x`.

GREEN: after the change both probes are byte-identical to the oracle except the dangling link
(below), and `probes/tree2.rex` (tilde failure forms, a NUL in the spec, an `'ff'x` directory,
`/.`, `/..`, `//`, the `..` and `.` forms run from `/`) is byte-identical.

### Corpus witness

`rust/corpus/lang/rexxutil_file_tree.rex`, listed in `phase-9.txt`, with
`crates/rexx-parse/tests/sourceline_oracle/rexxutil_file_tree.txt` generated by that test's driver
from an empty directory (nothing was left there). It makes `filetree.d` in its run directory and
removes it (the closed directory is `chmod 700` first), and ends with one untrapped 40.21 so the
rendering is compared. Run on both sides from fresh directories: stdout, stderr and rc 216
identical. Its oracle stdout was read line by line; every case prints what its label claims, among
them the names with a blank, the `'ff'x` name beside its `'efbfbd'x` decoy (both listed), a trailing slash, `link` listed as `d` and recursed (`link/F`), `closed`
listed with nothing below it, `..` finding nothing, `~nosuchuser.rexxutil/bin` finding `/bin`,
the target forms (a stem name, a stem object, an array emptied of three items, a stem that keeps
its default value and its tails past the count), and every argument error.

Review Focus 2 shapes and where each is pinned: a name with a blank, a non-ASCII byte, a trailing
slash, a link to a directory and an unreadable directory are in the witness. A dangling link is not
(the oracle's answer is undefined, below) and is pinned crate-only.

Mutations, each built with `--profile mutation`, the witness run and compared with the oracle's
stdout, then reversed by an exact edit and confirmed with `git diff --stat` and `cmp` against a copy:

- `..` searching `<cwd>/..` instead of `<cwd>..`: red, `[..|o]` lists one entry.
- no tail-`0` write on a validation failure: red, two 40.21 rows print `f.0 unset`.
- names decoded lossily in `entries`: red, the `'ff'x` lines print as `'efbfbd'x`.

## Concerns

1. **The shared normaliser diverges on a doubled slash before `..`.** Not fixed here: it is
   `crate::paths::normalize_bytes`, shared with `QUALIFY`, `.File`, streams and every RexxUtil
   path, which the global constraints say to report rather than widen into. Measured with
   `probes/q.rex` (`say qualify(n)`), oracle then crate:

   | input | oracle | crate |
   |---|---|---|
   | `/a/b//../c` | `/a/b/c` | `/a/c` |
   | `/a/b///../c` | `/a/b/c` | `/a/c` |
   | `/a/b/.//../c` | `/a/b/c` | `/a/c` |
   | `/a/b//..` | `/a/b` | `/a` |

   `SysFileTree` reaches it with `HOME` ending in `/` and `~/../x` (the first witness
   draft did, and differed; the witness now sets `HOME` without the slash). A fix ports the C++
   loop as it is.
2. **A dangling link is an undefined answer on the oracle** (`corpus/oracle-crashes.txt` entry 40,
   added): `stat64`'s failure is ignored and the line carries whatever the buffer held, measured
   as zeros in one program, `23  l-----S--T` in another, and the previous entry's directory data
   inside a listing, so it was counted under `D`. This crate lists it from `lstat`, as a link of
   its own size counted under `F`, pinned by
   `tree::tests::a_dangling_link_is_listed_by_its_own_status`. That choice is mine and wants a
   ruling.
3. **Order is unspecified** and has no Deviation number yet (above).
4. **Per-directory case folding is not consulted.** The oracle asks `FS_IOC_GETFLAGS` for
   `FS_CASEFOLD_FL` on the searched directory and matches without case there even without `I`.
   This crate treats every directory as case-sensitive, as `.File~isCaseSensitive` already does
   here (`dispatch/files.rs:486`). No casefolded directory exists on this machine to measure.
5. **`CHAROUT` to a name holding `'ff'x` is lossy**: the first witness draft created
   `n || 'ff'x` with `charout` and the crate wrote into `n || 'efbfbd'x` instead. The witness now
   creates it with `SysFileCopy`. This is the stream-open site already in
   `.superpowers/sdd/queued/2026-10-13-lossy-path-sites.md`.
6. **A link cycle** recurses on both sides until the path stops opening. Not run.

## Files changed

- `rust/crates/rexx-exec/src/builtin/rexxutil/tree.rs` (new), `tree/tests.rs` (new)
- `rust/crates/rexx-exec/src/builtin/rexxutil.rs` (old `SysFileTree` removed, `mod tree`)
- `rust/crates/rexx-exec/src/builtin/rexxutil/files.rs` (`Found` takes the position, the time helpers
  and the message limit visible to the module's siblings)
- `rust/crates/rexx-exec/src/builtin/platform.rs` (`expand_tilde` crate-visible)
- `rust/crates/rexx-exec/src/builtin/datetime.rs` (dead `Calendar` removed)
- `rust/crates/rexx-exec/src/error.rs` (`routine_null_argument`)
- `rust/crates/rexx-exec/src/internal_routines.rs` (row points at `tree::file_tree`)
- `rust/corpus/lang/rexxutil_file_tree.rex`, `rust/corpus/phase-9.txt`,
  `rust/crates/rexx-parse/tests/sourceline_oracle/rexxutil_file_tree.txt`
- `rust/corpus/oracle-crashes.txt` (entry 40), `rust/corpus/refusal-sites.tsv`
- `docs/superpowers/records/2026-10-13-phase-9b/task-5/` (run records and probes)

Derived rows: no concurrency whole-group row, `sim-gate.tsv` row or 9d-set line names this group
(`grep -rn SysFileTree corpus/ crates/rexx-exec/tests/`), so the suite record above is the only
derived row this task re-derives.

## Fix round 1

Commit `00ba9f620`, against `task-5-review.md` and the Task 5 rulings in `progress.md`.

**(1) Deviation 34, `normalize_bytes` unchanged.** Added to
`docs/superpowers/plans/phase-4-exclusions.txt` after 33, in its format. It cites
`normalizePathName` (`platform/unix/SysFileSystem.cpp:687-776` in the oracle's tree, line 706 for
the `previousSlash` condition; the function is identical in this checkout's `interpreter/`), the
rule the oracle applies, and the differing rows. I re-measured them with `probes/q.rex` (oracle
and crate side by side in `probes/q.oracle-then-crate.out`). Ten rows differ, the review's nine
plus `/a/.config/../x` (`/x` against `/a/x`). Nine rows agree. The agreeing rows are now in
`corpus/lang/platform_qualify_setlocal.rex`, run on both sides with stdout, stderr and rc
identical, and its sourceline file was regenerated. The differing rows are pinned by
`paths::tests::a_parent_reference_removes_one_component_where_the_oracle_does_not`, which names
Deviation 34. `normalize`'s doc says the same in one sentence.

**(2) `[.c.]` and `[=c=]`.** I measured every form on the oracle first (`probes/brackets.rex`,
`probes/brackets.oracle.out`, names `a b c d A B - ] ^ ! [ = x.y`). What glibc does in the C
locale:
- A one-byte `[.c.]` or `[=c=]` is that byte, compared without folding even under `I`.
  `[[.a.]]` and `[[=a=]]` with `I` match `a` and not `A`.
- `[.c.]` can start or end a range, and the range folds.
- `[=c=]` followed by `-` starts no range: `[[=a=]-c]` is `- a c`.
- `[=c=]` ending a range fails the whole pattern, as do an unclosed `[.`, an empty symbol and a
  symbol longer than one byte.
- A `[=` not closed by `=]` two bytes on is a literal `[`.

`bracket` now does each of these, through the helpers `collating` and `equivalence`. The probe
is identical on both sides. The witness carries every form the review measured, every one of its
agreeing controls, and the forms above, each with `o` and `oi`, in a
`filetree.g` directory the program makes and removes, and I read each row of the oracle's output.

**(3)** `entries` takes a filter, and the first pass stats only the names `matches` accepts, as
`findNextEntry` does. The recursion pass accepts every name. The doc now also covers the
both-fail drop, which was left as it is.

**(4)** Entry 40 is retitled to cover any `stat` failure and names `ELOOP`, with the review's loop
probe. I re-ran that probe here (`probes/loop.rex`): both sides give count 123, and the deepest
`up` prints `80  drwxrwxr-x` on the oracle and `2  lrwxrwxrwx` here. The unit test's doc says the
same.

Mutation: folding the lone-symbol comparison (`low == byte` to `fold(low) == fold(byte)`, built
with `--profile mutation`) turns four witness rows red (`[[.a.]] oi`, `[![.a.]] oi`, `[[.A.]] oi`,
`[[.a.][=b=]] oi`). It was restored from a copy, `git diff --stat` empty.

| check | result at `00ba9f620` (raw output in `records/.../task-5/fix1/`) |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0 |
| lib + tables (the command above) and `rexx-parse --test sourceline_oracle` | exit 0: lib 1096, then 8, 2, 1, 6, 5, 3, 5; sourceline 1 (`lib-tables-output.txt`) |
| suite group `base/rexxutil/SysFileTree` | exit 0, green, 26 of 26 (`run-output.txt`, `tests.tsv`) |
| corpus, `REXX_CORPUS_GATE=1` release | exit 0, `1037 of 1037 matching` (`corpus-output.txt`) |
| `collect_stress`, `ir_recorded`, `trace_oracle`, `coverage` release | exit 0: 37, 28, 41, 21 passed (`other-tests-output.txt`) |

## Fix round 2

Commit `064498811`, against `task-5-rereview.md`.

**(1) Malformed `[=` after earlier items.** I read the source path the oracle uses first. That is
the bracket case of glibc 2.43's `posix/fnmatch_loop.c` (the oracle's libc is Debian GLIBC
2.43-7), fetched from sourceware at tag `glibc-2.43`. The oracle runs it in the C locale, where
`nrules == 0` and `MB_CUR_MAX == 1` keep it on the narrow path, with `FNM_NOESCAPE`.

The loop does not build a set. It tries each item in turn, and the first item that matches jumps
to `matched:`. That code skips to the closing `]` and returns `FNM_NOMATCH` on any `[=` that is
not `[=c=]` and on any `[.` with no `.]` it passes over. So `[b[=a]` fails the name `b`: the item
`b` matched, then the skip met `[=a]`. Names that reach the malformed `[=` itself still treat it
as a literal `[`.

Under negation the outcome is the same, since a match fails a negated bracket either way.
Measured: `[!b[=x]`, `[!bc[=x]` and `[^bc[=x]` exclude exactly `b`/`bc`, `[` and `=`, and so does
this crate.

A malformed `[.` fails the pattern wherever it is (`[b[.a]` and `[!b[.a]` match nothing). A `[:`
that is not a class name is never fatal: in the main scan it is a literal `[`, and the skip passes
over it.

Fix round 1's rule-by-rule `bracket` is replaced by a port of that loop, its doc rewritten. The
port also corrects two sentences fix round 1 got wrong:
- "A `[=` not closed by `=]` two bytes on is a literal `[`" held only as the first item.
- "`[=c=]` ending a range fails the whole pattern" was wrong. `[a-[=c=]]` is a range `a`-`[`
  followed by the items `= c =`, and then a literal `]`, so it matches the name `c]` on both
  sides.

The port keeps glibc's quirks:
- A class-name byte must be in `a`-`y` (glibc tests `c >= 'z'`).
- After a `[.c.]`, a `-` followed by `]` makes the symbol not match on its own (`[[.a.]-]` matches
  only `-`).

The witness carries every variant the re-review listed, `[!b[=x]`, `[!bc[=x]`, `[^bc[=x]`, the
`[.` and `[:` forms after an item, negated and not, and `[b[=a=]]`/`[!b[=a=]]` as well-formed
controls, each with `o` and `oi`. Its tree gained the names `:`, `c]` and `A]`. Oracle and crate
stdout, stderr and rc (216) are identical, and I read each row.

Sweep (`probes/sweep.rex`, `probes/sweep-patterns.txt`, `probes/sweep.oracle.out`): 1700
patterns, generated with a fixed seed over `[ ] . = - a b c A ! ^ : z \`, the fragments `[=`,
`[.`, `[:`, `=]`, `.]`, `:]`, `alpha`, `upper` and the re-review's rows, each with `o` and `oi`,
against 20 names. **0 of 3400 rows differ.** 888 rows list at least one name. The re-review's
1500-pattern sweep against fix round 1 found 5 differing rows.

Remaining divergence: `^` negates as `!` does. glibc reads `POSIXLY_CORRECT` once per process
and then does not, and this crate does not read it.

**(2) Deviation 34** cites the oracle tree's copy by path (`SysFileSystem.cpp:687-776`, line 706)
and this checkout's lines (604-693, line 623).

| check | result at `064498811` (raw output in `records/.../task-5/fix2/`) |
|---|---|
| `cargo fmt --all --check`, `cargo clippy -j 4 -p rexx-exec --all-targets -- -D warnings` | exit 0, exit 0 |
| lib + tables and `rexx-parse --test sourceline_oracle` | exit 0: lib 1096, then 8, 2, 1, 6, 5, 3, 5; sourceline 1 (`lib-tables-output.txt`) |
| suite group `base/rexxutil/SysFileTree` | exit 0, green, 26 of 26 (`run-output.txt`) |
| corpus, `REXX_CORPUS_GATE=1` release | exit 0, `1037 of 1037 matching` (`corpus-output.txt`) |
| `collect_stress` release | exit 0, 37 passed, wall 259 s (`collect-stress-output.txt`) |
| `ir_recorded`, `trace_oracle`, `coverage` release | exit 0: 28, 41, 21 passed (`other-tests-output.txt`) |
