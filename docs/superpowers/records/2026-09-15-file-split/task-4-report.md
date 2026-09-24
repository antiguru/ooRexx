# Task 4 report: `rexx-exec/src/lib.rs`

BASE `2f3065b2b`. Five code commits, then this report's commit. Every artifact
cited is under `docs/superpowers/records/2026-09-15-file-split/task-4-files/`
(`files/` below). The tooling is Task 3b's, adapted, in `files/tools/`.

## Commits

| # | commit | what moved | child `wc -l` | `lib.rs` after |
| --- | --- | --- | --- | --- |
| c1 | `b4f42bbfc` | the test module, to `tests.rs` | 982 | 6168 |
| c2 | `3fc6cbd71` | `seal_site_level`, from `run/interpret.rs` to `run.rs`, beside `record_failure_site_at` | (`run.rs` 3438) | 6168 |
| c3 | `bdef7b8b8` | the directive-analysis free functions, to `directives.rs` | 482 | 5719 |
| c4 | `377d0ac3d` | variable storage and reads, to `variables.rs` | 251 | 5498 |
| c5 | `97ad8c334` | directive installation, libraries and the executable records, to `install.rs` | 2333 | 3206 |

`lib.rs` went from 7142 lines to 3206. Each commit message says
`git blame -w -C -C -C` recovers the moved lines. The moved `impl Interp`
members are in one `impl Interp { ... }` block per child, still indented as
members. Their text is byte-identical to BASE except where this report lists
a difference.

**`Loud` did not move** (departure 1, ruled). Nothing was committed for it.

## What stays in `lib.rs`

The module declarations and the public surface (`run_program`, `render_ir`,
`native_entry_points`, `execute`, the exit-code constants, `Outcome`,
`StackSpan`), `Interp` and the types it holds, and the GC block
(`exit_code_for` through `alloc_immortal_with`, `collect_now` untouched at its
four-space indent). Beside those:

* `Loud`, its constructors, and the functions that build its messages:
  `form_name`, `owned_message`, `instruction_owner`, `expr_owner`.
* `Code` and the `#[cfg(test)] fn planned_code`. `plan.rs` and `stem.rs` tests
  name `planned_code` through `crate::`, which a private item in the crate
  root satisfies, so it keeps its place and its visibility.
* `Interp::new` through `run_loaded`: construction, the bootstrap and the
  entry paths.
* The members from `env_get` to `shadow_var`: environment variables, the
  current directory, the output buffers, `SETLOCAL`/`ENDLOCAL`, and
  `RXTRACE`. They sit inside the survey's install ranges but are not
  installation (departure 4).

## Departures from the ruled proposal

1. **`refusal.rs` was not made.** Before the move I read
   `tests/refusal_sites.rs`'s tag derivation and ran the move in a scratch
   worktree. It changes column 3 on four rows:
   `Loud/accessor_variable` and `Loud/delegate_variable` go from `send` to
   `body+send`, `Loud/expression` from `body` to `body+ir`, and
   `Loud/redirection` from `body` to `body+send`. `Loud/redirection` also
   becomes a send-surface row, so the refresh blanks its verdict, reached,
   answer and witness cells. The cause is `surface()`
   (`refusal_sites.rs:265-271`). It treats a constructor defined outside
   `error.rs` or `lib.rs` as a free function and counts every bare
   occurrence of its name as a construction site. After the move, these
   count: the free fn `accessor_variable` and `dispatch.rs`'s method of that
   name, `delegate_variable` in `dispatch.rs`, the `expression` field in
   `ir/drive.rs`, and the word "redirection" inside a string in
   `dispatch/tests.rs`. The brief said to report this to the controller
   before committing that child. I did, with the rows, the cause and two
   options: keep `Loud` in `lib.rs`, or fix the scanner first in a separate
   commit. The controller ruled for the first, after the commits below
   were made: `Loud` stays in `lib.rs`, and the move waits for a scanner
   fix, queued together with the scanner reading file-level test modules
   as surface code, since both decide surface by file name.
   `form_name`, `owned_message`, `instruction_owner` and `expr_owner` stay
   with it. The survey listed `instruction_owner` and `expr_owner` under the
   directive-analysis range, but they build `Loud`'s messages, not
   directive facts. Their callers are `Loud::instruction`/`expression` and
   `run.rs`.
2. **Boundaries re-derived at BASE**, by key, from each commit's parent
   (`files/c<N>/removed.json`). The survey's ranges are from `5d84dd8cb` and
   do not line up with item boundaries at BASE. For example, its "2674"
   falls inside `run_loaded` and its "5060" inside `record_access_scope`.
3. **`directives.rs`** holds `class_references` through `delegate_variable`,
   minus the owner functions (departure 1). That includes `method_body_gap`,
   `accessor_setter_name`, `accessor_variable` and `delegate_variable`.
   Those answer questions about a directive, and `dispatch.rs`,
   `dispatch/native.rs` and `environment.rs` reach them through `crate::`
   paths. A plain `use directives::{..}` in `lib.rs` keeps those paths
   working, so no `pub(crate) use` was needed.
4. **`install.rs`** holds `install_directives` through `blame_directive_in`,
   minus the environment members above. The survey gave two ranges,
   "2674 to 3560" and "3982 to 5060", and left the members between them
   unassigned. Those members (`resolve_requires`, `load_package`,
   `package_from_source`, the annotation, member-key and constant steps,
   `send_directive_message`, `push/pop_directive_activation`) are
   `::REQUIRES` loading and `install_directives`' own helpers, so they went
   to `install.rs`. `program_display_name`, `enter_installed_routine`,
   `make_method_private` and `special_method_row(_mut)` are inside the
   survey's second range and went with it.
5. **The test module is `src/tests.rs`**, not `src/lib/tests.rs`: a crate
   root's children sit beside it, per the brief. No existing file or module
   in `src/` had any of the new names. `run/tests.rs` declares its own
   `mod directives;`, which is `run::tests::directives`, a different module.
6. **`seal_site_level`'s callers** are `run/interpret.rs`, `run/call.rs` and
   `lib.rs`. `dispatch.rs`, which the brief names, does not call it. It
   keeps `pub(crate)`, which `lib.rs` needs.
7. **Import lists.** Moving code out left names in `lib.rs`'s `use`
   declarations that only the moved code used: rustc warns, and clippy
   `-D warnings` fails. Those names were dropped in the commit that made
   them unused. Instrument 1 (a) allows this and lists the names dropped
   (below).
8. **Field visibility.** `FileClasses`'s fields (`lib.rs` builds and reads
   the struct) and `MissingTarget`'s fields (`lib.rs` reads them) became
   `pub(super)`, and so did `MissingTarget` itself, because
   `annotation_target` returns it. Both are in c3.
9. No test moved out of its module path, so no test's fully qualified name
   changed: `tests::x` before, `tests::x` after.

Visibility: every moved item another module reaches became `pub(super)`
(`pub(crate)` for a child of the crate root in all but name). Each change is
listed in its commit's `files/c<N>/c<N>-instrument2.txt`, and all 32 are in
`files/final/cumulative.txt`. Items that were `pub(crate)` kept it.

## Pinned items

* `collect_now` did not move. `dispatch_seam.rs::heap_collect_is_called_from_collect_now_alone`
  ran in instrument 4 at every commit and passed.
* `/bin/grep -rn 'src/lib\.rs\|lib\.rs' rust/crates/*/tests rust/crates/*/src rust/corpus`:
  * `dispatch_seam.rs` (the `collect_now` pin);
  * `refusal_sites.rs` (departure 1);
  * `unsafe_sites.rs`, for other crates' `lib.rs`;
  * `loud.rs`'s comment on `instruction_owner`/`expr_owner`, which stayed;
  * prose naming `lib.rs` for things that stayed (`execute`,
    `exit_code_for`, `INTERPRETER_STACK_BYTES`, `current_clause_line`,
    `owned_message`, `instruction_owner`);
  * `run.rs:338`'s quoted compiler output.

  Two mentions named something that moved and were corrected in the commit
  that moved it: `run.rs`'s "`read` lives in `lib.rs`" (c4, now
  `variables.rs`) and `environment.rs`'s "`lib.rs`'s
  `Interp::install_directives`" (c5, now `install.rs`). The diffs are
  `files/c4/c4-other-edits.diff` and `files/c5/c5-other-edits.diff`. The
  specs under `docs/superpowers/specs/` that name `directive_gap` in
  `lib.rs` are records of their day and were left alone.
* `refusal-sites.tsv` was re-derived at every commit
  (`files/c<N>/c<N>-refusal-sites.txt`, each showing that the table's mtime
  moved). c1 and c2 changed no row. c3 to c5 moved the line numbers of the
  `lib.rs` `Loud` rows, because the new `mod` declarations sit above them.
  At every commit, every column except column 4 is identical row for row
  across all 247 rows, and the header lines are identical.
* **Found at c1:** `code_lines` skips an inline `#[cfg(test)] mod x {`
  block, so since c1 the code of `tests.rs` is scanned as `body`-surface
  call sites, which it was not while it was inline in `lib.rs`. No row
  changed. This is the same shape as Task 2's finding for
  `dispatch/tests.rs`.

## Comment and doc edits

Each one is in the commit that made the text false or broken, and each is a
declared edit the instruments show:

* c3, `Interp::install_attribute`: [`AttributeStyle`] resolved through the
  `lib.rs` import that only moved code used. It is now
  [`AttributeStyle`](rexx_parse::AttributeStyle), which renders the same
  text. `cargo doc --document-private-items` showed the break (54 warning
  lines against 53) before the fix.
* c4, `run.rs`: "`read` lives in `lib.rs`, beside `Interp`'s other
  value-model" became "`read` lives in `variables.rs`, beside `Interp`'s
  other variable" (the next line, "entry points.", is unchanged).
* c5, `environment.rs`: "`lib.rs`'s" became "`install.rs`'s".

Added: a license header and a one-sentence module doc in each new file, and
a `//` comment above each new `mod` in `lib.rs`. `files/final/comments.txt`:
of BASE `lib.rs`'s 2176 comment lines, exactly one is gone, the c3 doc line
above, and everything else added is the header, doc or comment lines just
named.

## The instruments

The per-commit outputs are in `files/c<N>/`, made by `tools/checks.sh` over
the working tree before each commit. The tree the tests ran on is the tree
that was committed: `tools/commit_lib.sh` refused to commit unless the
staged `rust/` tree hash equalled the one instrument 4 ran on, and every
commit's hashes matched (`files/instrument4/i4-c<N>.meta`).

* **Instrument 1.**
  * (a) The parent minus the removed lines, compared against the new parent.
    Every equal block is also compared with `cmp`. The only differences
    allowed are inserted lines, declared edits, and import narrowing, which
    is a block made only of `use` declaration lines whose names after are a
    strict subset of the names before, with the names dropped listed.
  * (b) Each moved unit's text, comments above it included, compared with
    `cmp` against its new text, after undoing only a visibility change: on
    the declaration line, or on a struct field.
  * (c), new for c2: an existing destination (`run.rs`) compared before and
    after, where only inserted lines are allowed.
* **Instrument 2.** Token streams per unit, with visibility stripped, and a
  field's `pub(super)` stripped only directly after `{` or `,`.
* **Instrument 3.** Decoded literals per unit from raw token trees, walking
  every group, cross-checked per unit on both sides against
  `tools/rustlex.py`'s independent count, plus the whole-file multiset.
* **Instrument 4.** `memcap 8G cargo test -j 4 --release -p rexx-exec
  --no-fail-fast`, compared per result block and per test with BASE, in a
  scratch worktree whose `rust/target` links to that run's target directory
  (concern 3 of Task 3b).
* **c1's de-indent.** Lines that start inside a string literal (other than a
  backslash continuation that begins with four spaces) keep their text
  verbatim; the others lose four spaces. That was 21 lines in c1.
  Instrument 1 (b) re-indents exactly the lines the mover stripped.

| # | 1 (a) unmoved | 1 (b) moved units | 2 tokens | 3 literals | 4 tests | load before / after instrument 4 (1, 5, 15 min) |
| --- | --- | --- | --- | --- | --- | --- |
| c1 | 6167 equal, 1 inserted | 41 of 43, 2 reflowed | 331 of 331 | 331 of 331; 1683 literals; 0 control mismatches | identical | 2.18 4.09 4.69 / 3.20 5.17 5.10 |
| c2 | 299 equal; (c) `run.rs` 3430 of 3430 equal, 8 inserted | 1 of 1 | 10 of 10 | 10 of 10; 43; 0 | identical | 1.82 4.32 4.81 / 3.01 5.45 5.33 |
| c3 | 5705 equal, 9 inserted, imports narrowed, 1 declared | 19 of 20, 1 reflowed (2 with field visibility) | 287 of 288, 1 declared | 287 of 288, 1 declared; 1326; 0 | identical | 4.39 3.94 4.64 / 2.25 5.27 5.27 |
| c4 | 5493 equal, 4 inserted, imports narrowed | 16 of 17, 1 reflowed | 270 of 270 | 270 of 270; 1254; 0 | identical | 3.45 4.64 5.04 / 3.37 5.93 5.71 |
| c5 | 3197 equal, 4 inserted, imports narrowed | 75 of 76, 1 reflowed | 250 of 250 | 250 of 250; 1211; 0 | identical | 1.97 3.99 4.98 / 6.46 6.72 5.93 |

"Reflowed" means rustfmt re-wrapped a statement or a signature: the
de-indent in c1, or the added `pub(super)` elsewhere
(`method_dictionary_keys`, `set_pool_variable`, `install_directives`). Each
is token-identical, allowing only the relaxation Task 3b documented. The
imports dropped were: c3, `AnnotationTarget` and `AttributeStyle`; c4,
`rexx_core::Body`; c5, `ClassKind`, `InheritRefusal`, `SlotFrame`,
`BTreeMap`, twelve `rexx_parse` names and twelve `directives` names
(`files/c5/c5-instrument1.txt` lists each). Instrument 4 gave 1582 test
lines in 50 result blocks (1581 passed, 0 failed, 1 ignored) at BASE and at
every commit.

**Tooling changes, each with a control.** `files/tools/controls_lib.py` has
eight controls on this task's own commit pairs:

* A line of a moved byte string kept verbatim gains a space (I3).
* A de-indented line changes a literal (I1b, I2, I3).
* A parent import gains a name (I1a).
* An unmoved line of the existing destination `run.rs` gains a space (I1c).
* A field type changes beside a widened field visibility (I1b, I2).
* A widened field is renamed (I1b, I2).
* A narrowed import also changes its path (I1a).
* A narrowed import also gains a name (I1a).

**Controls.** I ran Task 2's five controls on Task 2's c1 pair and Task
3b's four on Task 3b's c8 pair, with this task's tooling, on trees fresh
from `git archive`. Before the first move: 5 of 5 and 4 of 4
(`files/controls-task2-before.txt`, `files/controls-run-before.txt`). At
the end: 5 of 5, 4 of 4, and this task's 8 of 8
(`files/final/controls-*-final.txt`). The final tooling, re-run on every
commit pair, passes and gives the same instrument output as each commit's
own run (`files/final/rerun/`).

**Cumulative** (`files/final/cumulative.txt`): every one of BASE `lib.rs`'s
331 units is present exactly once after c5 (171 in `lib.rs`, 43 in
`tests.rs`, 20 in `directives.rs`, 17 in `variables.rs`, 76 in
`install.rs`), identical modulo visibility, or the declared
`install_attribute` edit.

`cargo doc --no-deps -p rexx-exec` gives 2 warning lines at BASE and at
every commit, 1 of them in `lib.rs` (BASE's link to the private
`Loud::chunk_refused`). With `--document-private-items` there are 53, and
11 are in the files this task touched: all BASE's own, the same set at
every commit as committed. With `--cfg test` as well there are 52, the same
11. `tools/tests_links.sh`, which strips the `#[test]` lines from `lib.rs`
and `tests.rs` and documents with `cfg(test)`, finds 10 warnings, all in
`lib.rs`, none in the test module, at BASE and at every commit.

## Performance

callgrind `summary:` minus `libc.so.6` and `ld-linux`, two interleaved
rounds. BASE was built in its own worktree and target directory, and c5
(`97ad8c334`) in the instrument-4 worktree's. The `.text` sha256 is
`da915d7c...` for BASE (the same as Task 3b's c8, which is code-identical)
and `a0c2d9b6...` for c5. `files/perf/` has the commands, the raw figures
and the hashes.

| program | BASE round 1 | BASE round 2 | c5 round 1 | c5 round 2 | change |
| --- | --- | --- | --- | --- | --- |
| rexxcps | 17897267831 | 17897272900 | 17897274116 | 17897273425 | +0.00002% |
| nop | 9340092236 | 9340095439 | 9340094859 | 9340095843 | +0.00002% |
| assign | 19540360172 | 19540370161 | 19540360391 | 19540355727 | -0.00004% |
| emptyloop | 9285883900 | 9285879475 | 9285886271 | 9285892402 | +0.00008% |
| varlookup | 14842898004 | 14842894836 | 14842891395 | 14842898263 | -0.00001% |
| arith | 11519335866 | 11519330758 | 11519335673 | 11519337066 | +0.00003% |
| compound | 9234397610 | 9234395682 | 9234392756 | 9234390175 | -0.00006% |
| dispatch | 20471071307 | 20471084099 | 20471071748 | 20471082364 | -0.00000% |
| strings | 17788057834 | 17788060596 | 17788056222 | 17788054328 | -0.00002% |

No axis moves more than 0.0001%, so nothing was bisected. The spread
between two runs of one binary is the same size. Every program's stdout is
identical between the two binaries except `rexxcps`'s wall-clock
clauses-per-second line.

## Gates

These ran over `b76bce29b`, this report's first commit, whose code is
identical to c5. `files/tools/gates.sh` ran them after the commit, and the
tree did not change from start to finish (`git status --short` was empty
before and after). The statuses, each read unpiped, are in
`files/gates/status.txt`. G3 to G6 use the default target directory, so
the arity suites ran the binary G3 built.

| gate | command | result | load average (1, 5, 15 min) |
| --- | --- | --- | --- |
| G1 | `cargo fmt --all --check` | exit 0 | 14.01, 16.18, 11.28 before the run |
| G2 | `cargo clippy --workspace --all-targets -- -D warnings`, empty target dir | exit 0 | |
| G3 | `cargo build --workspace --all-targets --release`, no cap | exit 0 | |
| G4 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --release --no-fail-fast` | exit 0; 135 result blocks; 2663 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 19.91, 19.34, 13.31 before; 2.40, 10.16, 11.21 after |
| G5 | `cargo build --workspace --all-targets` (debug) | exit 0 | |
| G6 | `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` (debug) | exit 0; 135 result blocks; 2664 passed, 0 failed, 4 ignored; `corpus_differential` 604 of 604, STRICT | 3.08, 9.92, 11.11 before; 3.46, 7.46, 10.03 after |

Every figure matches BASE's. The counts come from `tools/test_results.py`
run over each gate's own log (`files/gates/test-release.results`,
`test-debug.results`), and the 604 from that log's differential report
(`files/gates/corpus-differential-*.txt`).
`introspection_arity::every_unstable_row_is_really_unstable` passed in
both runs, so nothing was re-run. G4 started at load 19.9, which came from
other work on the machine, and still had no timeouts.

## Concerns

1. **`Loud` is still in `lib.rs`**, by ruling (departure 1). Once the
   scanner keys on "defined inside an `impl`" rather than on the file name,
   the move becomes location-only and can be a later commit.
2. **`lib.rs` is 3206 lines** and holds more than one responsibility:
   * `Loud`, `form_name` and the owner functions, about 650 lines;
   * `Code`, the `Interp` struct and its types, about 950;
   * `new` through `run_loaded`, about 450;
   * the environment block, about 100;
   * the GC block, about 400;
   * the entry points, about 350.
   The survey's plan ends here. The next split would be `Loud`
   (concern 1).
3. **`install.rs` is 2333 lines**, over the trigger. The ruled proposal
   made it one module. It holds four things: directive installation,
   library resolution with the package routine tables, the executable
   records, and the access-scope rows. The library part (`resolve_library`
   through `library_binding`, about 330 lines) is the most separable.
4. The survey's `refusal.rs` would have changed a derived table outside
   its location columns, and neither the plan nor the survey saw it. The
   plan's statement that column 3 "survives a `dispatch.rs` split only
   because `/dispatch/` carries the same tag" is true but incomplete:
   column 3 also depends on where a constructor is *defined*, through the
   free-function rule.
