# Phase 8 L2 slice, residual fix round: report

Brief: `residual-fix-brief.md`. Start: HEAD `cf92ff4fb` (checked, clean tree). Scratch:
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/residual-fix/`.

Each section: reproduction (both sides), witness, control prediction (written before the run) and
result, commit, and anything not done. Sections are appended in the order the work ran.

## X1 -- interface table slots callable from safe code with a forged context

**Reproduction (run).** `residual-fix/x1-before/rust` is `git archive cf92ff4fb rust` with the
repository's read-only directories symlinked beside it and the re-review's
`boundary/tree/rust/crates/rexx-api/tests/zz_bare_context_probe.rs` copied in (`#![forbid(unsafe_code)]`,
two tests), `CARGO_TARGET_DIR=residual-fix/target-x1`, one test per process
(`residual-fix/x1-repro-{bare,forged}.txt`): `bare_struct_reaches_owner_of` exit 101, SIGABRT,
`panicked at crates/rexx-api/src/ffi.rs:225:14: misaligned pointer dereference: address must be a
multiple of 0x8 but is 0x5589f46157df`; `forged_wrapper_with_a_null_owner_reaches_owner_of` exit
101, SIGABRT, `ffi.rs:225:14: null pointer dereference occurred`. The "oracle side" of this finding
is the type system: the same probe must not compile.

**Fix.** `layout.rs`'s `entry_type!` types every function member of every `interface!` table as
`unsafe extern "C" fn`. The seven callbacks `ffi.rs` installs (`set_object_variable`,
`drop_object_variable`, and the five thread-table ones) are `unsafe extern "C" fn` too, each with
a `# Safety` section naming what the caller guarantees, and their `SAFETY:` notes now cite that
guarantee rather than who can reach them (the false "reached only through `METHOD_CONTEXT`, which
only a `Contexts` publishes"). `dropping_stub`'s call through the table is in an `unsafe` block.
`tests/layout.rs`'s `an_unbuilt_entry_refuses_loudly`, the one test that called a slot, moved into
`ffi.rs`'s unit tests (re-executing itself as `ffi::tests::an_unbuilt_entry_refuses_loudly`, and
`#[cfg_attr(miri, ignore)]` because Miri cannot spawn a process). Witness: a
`compile_fail,E0133` doctest on `METHOD_CONTEXT` holding the re-review's bare-struct call.

On the worktree: `cargo test -j 4 -p rexx-api --no-fail-fast` exit 0: lib 16 (the moved test
`ffi::tests::an_unbuilt_entry_refuses_loudly ... ok`), context 14, handles 5, invoke 9, layout 17,
load 10, values 38, doctests 5 including `ffi::METHOD_CONTEXT (line 105) - compile fail ... ok`
(`residual-fix/x1-api.txt`).

**Control predictions (written before the runs).**
* C1, the re-review's probe copied into a scratch copy of the fixed tree, `cargo test -p rexx-api
  --test zz_bare_context_probe --no-run`: exit 101 with `error[E0133]` "call to unsafe function
  ... requires unsafe block" at both call sites (probe lines 18 and 34) and no other error.
* C2, stable, the fixed scratch copy with what the doctest guards removed (`unsafe` dropped from
  both `entry_type!` arms and from the seven callback definitions, which must move with it or the
  assignments into the tables stop type-checking): `cargo test -p rexx-api --doc` exit 101, exactly
  `ffi::METHOD_CONTEXT (line 105)` failing with `Test compiled successfully, but it's marked
  compile_fail`, the other four doctests ok.
* C3, nightly rustdoc (which checks the listed code) on the fixed scratch copy: `cargo +nightly
  test -p rexx-api --doc` exit 0, five passed, so the failure the doctest expects is E0133.
* C3b (added after C3 ran, prediction written before C3b ran), that same nightly run on a copy
  whose doctest says `compile_fail,E0425` instead: exit 101, `ffi::METHOD_CONTEXT (line 105)`
  FAILED with `Some expected error codes were not found: ["E0425"]`, so C3 is not a pass over a
  toolchain that ignores the code.
* C4, slice A's layout comparison: `final-a/tree/rust/crates/rexx-api/tests/zz_offsets_probe.rs`
  copied into the fixed scratch copy and run with `--nocapture`: its printed lines are
  byte-identical to `final-a/offsets.cpp.txt` (111 lines), and a fresh run of the compiled
  `final-a/offsets` is byte-identical to that file too.
* C5, `cargo test -p rexx-core --test unsafe_sites` on the worktree: 2 passed (`unsafe extern` in
  `layout.rs` is a type, not one of the needles).
* C6, Miri (Stacked Borrows) over the fixed scratch copy's `-p rexx-api --lib`: 15 passed, 1
  ignored (`an_unbuilt_entry_refuses_loudly`), exit 0.

**Control results (run).** Scratch copies `residual-fix/x1-after/rust` (HEAD plus the three
worktree files, `cmp` identical) and `residual-fix/x1-mut/rust` (the same with the C2 mutation,
`diff` shown: exactly the two `entry_type!` arms and the seven `unsafe extern "C" fn` definition
lines). Statuses in `residual-fix/x1-controls-status.txt`.
* C1 exit 101 (`x1-C1.txt`): `error[E0133]: call to unsafe function is unsafe and requires unsafe
  block` at `zz_bare_context_probe.rs:18:5` and `:34:5`, `could not compile ... due to 2 previous
  errors`. Confirmed.
* C2 exit 101 (`x1-C2.txt`): `ffi::METHOD_CONTEXT (line 105) - compile fail ... FAILED`, `Test
  compiled successfully, but it's marked compile_fail`; `load::Library::method (line 283)`,
  `(line 292)`, `ffi::value_of (line 54)`, `load::NativeMethodEntry (line 90)` ok; no warning.
  Confirmed.
* C3 exit 0 (`x1-C3.txt`, `rustdoc 1.100.0-nightly (4aa1fbcf4 2026-09-08)`): five doctests ok,
  `ffi::METHOD_CONTEXT (line 105) - compile fail ... ok`. Confirmed.
* C3b exit 101 (`x1-C3b.txt`): `ffi::METHOD_CONTEXT (line 105) - compile fail ... FAILED`, `Some
  expected error codes were not found: ["E0425"]`; the copy restored afterwards (`cmp` against the
  worktree file). Confirmed, so C3 checked the code.
* C4 exit 0: `x1-C4.txt` (the probe's printed lines) 111 lines, `diff` against
  `final-a/offsets.cpp.txt` empty; a fresh run of `final-a/offsets` into `x1-C4-cpp/offsets.txt`,
  `diff` against the same file empty. Confirmed: the ABI measured by slice A is unchanged.
* C5 exit 0 (`x1-C5.txt`): `the_scan_reaches_the_whole_workspace ... ok`,
  `only_the_granted_module_may_say_unsafe ... ok`. Confirmed.
* C6 exit 0 (`miri-x1-C6-sb.txt`, `residual-fix/miri.sh`, the previous implementer's scratch
  `RUSTUP_HOME` read only, my own `XDG_CACHE_HOME` and target): `15 passed; 0 failed; 1 ignored`,
  `an_unbuilt_entry_refuses_loudly ... ignored, spawns a process`. Confirmed.

**Gates (worktree).** `cargo fmt --all --check` exit 0; `cargo clippy -j 4 --workspace
--all-targets -- -D warnings` exit 0 (`x1-clippy2.txt`); `cargo test -j 4 -p rexx-api
--no-fail-fast` exit 0 as above (`x1-api2.txt`); `REXX_CORPUS_GATE=1 cargo test -j 4 --release -p
rexx-exec --test corpus` exit 0, `540 of 540 matching` (`x1-corpus.txt`). The last was run before a
one-sentence rewording of `entry_type!`'s doc comment; fmt, clippy and the rexx-api tests were
re-run after it, and the corpus gate is re-run at the commit (below).

**X1 commit: `69a579370`.** Corpus gate re-run on the committed tree (`x1-gate-at-commit.txt`:
first line `69a5793704d44addc39838e9fdaf631325b7f465`, zero dirty paths, `corpus exit 0`, `540 of
540 matching`, `finished`).

## X2 -- a routine's shared code is one object per library entry, found caselessly (R1, R2)

**Reproduction (run).** Runner `residual-fix/cmp.sh NAME SRCDIR` (a fresh `p/NAME`, the oracle
under the standard wrapper and the build in that same directory, `</dev/null`, three descriptors
to files, stdout shown `oracle|rust` per line). Before-binary `residual-fix/bins/rexx-run-x1`
(release build of `69a579370`, sha256 prefix `94be87cad3e7865f`; X1 does not touch `rexx-exec`).
The re-review's probes copied to `residual-fix/src/`:
```
f4b-before  (pk.cls: ::routine sq public external "LIBRARY rxmath RXCALCSQRT")   oracle | x1
  early before                          nil     | nil
  early after upper binder              pk.cls  | nil
  binder                                pk.cls  | pk.cls
  loaded exact after (RxCalcSqrt)       pk.cls  | nil
  loaded upper after (RXCALCSQRT)       pk.cls  | pk.cls
  loaded lower after (rxcalcsqrt)       pk.cls  | nil
f4a3-before (pk.cls and pk2.cls each bind RxCalcSqrt)
  pk routine                            pk.cls  | pk.cls
  pk2 routine                           pk.cls  | pk2.cls
  loaded / pk3 method / pk4 later method  SAME
f4e-before  (R3, for X3)
  trapped 90.998                        SAME
  early after failed binder             pk.cls  | nil
  loaded after failed binder            pk.cls  | nil
```
rc 0 and stderr empty on both sides of all three.

**Witness draft** `residual-fix/x2-draft/` (main.rex, `pk.cls` binding `RXCALCSQRT`, `pk2.cls`
binding `RxCalcSqrt`; a `loadExternalRoutine` of `RxCalcSqrt` before either loads, the same
object after `pk.cls`, `pk.cls`'s and `pk2.cls`'s own routines, `loadExternalRoutine` as `pk.cls`
spells it and in lower case, and `RxCalcPower`, an entry nothing binds, as the control that the
key is per entry and not per library). `p/x2-witness-before`: oracle `nil` / `pk.cls` / `pk.cls` /
`pk.cls` / `pk.cls` / `pk.cls` / `nil`; x1 differs on lines 2 (`nil`), 4 (`pk2.cls`) and 6
(`nil`); stderr and rc SAME.

**Fix.** `LibraryCodeKey`'s procedure for a routine is the name its routine table entry declares
(`Library::routine`'s row, which is found without regard to case), at both key sites
(`native_load_external` and `resolve_directive_library`); a method's stays the spelling asked. A
new `Interp::library_routine_codes` records the row each library-backed `::ROUTINE` directive
bound, and `source_package` answers that row's package for the directive's routine, so a later
binder's routine reports the first binder. Docs on `LibraryCodeKey` and `library_codes` corrected
(they described routines as keyed like methods), and `library_method_package.rex`'s header scoped
to methods.

**Control predictions (written before the runs).**
* The draft witness on the fixed binary (`bins/rexx-run-x2`): SAME x3. The re-review's `f4b` and
  `f4a3` SAME x3; `f4e` still DIFF on its two lines (X3's).
* M-X2a (both key sites use the spelling asked for routines again): stdout DIFF on exactly line 2
  (`nil`), line 4 (`pk2.cls`: `pk2.cls`'s `RxCalcSqrt` key is then the unbound row the early
  object made, which `pk2.cls` binds to itself) and line 6 (`nil`); lines 1, 3, 5, 7 SAME; stderr
  and rc SAME.
* M-X2b (`source_package` ignores `library_routine_codes`): stdout DIFF on exactly line 4
  (`pk2.cls`), everything else SAME.
* The gated corpus with the witness committed: every program matching, including
  `library_method_package` and the new one; the per-part sidecar control ok (the new program's
  environment and fixtures both live).

**Control results (run).**
* Fixed binary `bins/rexx-run-x2` (sha256 prefix `77e3191d22ab2947`): `p/x2-witness-after` SAME
  x3 (`nil` / `pk.cls` / `pk.cls` / `pk.cls` / `pk.cls` / `pk.cls` / `nil`); `p/f4b-x2` and
  `p/f4a3-x2` SAME x3; `p/f4e-x2` DIFF on its two lines as before. Confirmed. The F4 neighbourhood
  re-run on the same binary, SAME x3 each: the re-review's `f4d`, `f4f`, `f4g`, and the previous
  implementer's `f4-pkg-2` (rc 159 both), `f4-pkg-3`, `f4-pkg-4`, `f4-key-1` (with
  `final-fix/libs` on the search path, read only).
* M-X2a (`--profile mutation`, worktree edited and restored from `residual-fix/x2-backup/`, `cmp`
  clean, `diff` shown: exactly the two `map(|row| row.name.clone())` lines; binary
  `149899c92e38b32a`), `p/x2-MX2a`: stdout DIFF on lines 2 (`nil`), 4 (`pk2.cls`) and 6 (`nil`),
  every other line, stderr and rc SAME. Confirmed exactly.
* M-X2b (the `library_routine_codes` lookup made to miss with `.filter(|_| false)`, same restore;
  `5a404af89502a354`), `p/x2-MX2b`: stdout DIFF on line 4 alone (`pk2.cls`). Confirmed exactly.
* The existing suite cannot see either mutant: the re-review's gated corpus at `cf92ff4fb`, which
  has both defects, was `540 of 540 matching`; `library_method_package.rex` binds and loads its
  routine under one spelling and binds it from one package.

**Witness.** `corpus/lang/library_routine_package.rex` (the draft, byte for byte) + `.env`
(`LD_LIBRARY_PATH={oraclelib}`) + `.d/pk.cls` + `.d/pk2.cls`, a row in `phase-8.txt`. Sourceline
companions for it and for `library_method_package` regenerated from scratch copies
(`residual-fix/srcgen/gen.sh`, the previous implementer's script with its scratch path changed:
counts 21 and 28, and each companion's lines after the count `cmp` identical to its program).

**Gates (uncommitted tree, committed unchanged).** `cargo fmt --all --check` exit 0; `cargo clippy
-j 4 --workspace --all-targets -- -D warnings` exit 0 (`x2-clippy.txt`); `residual-fix/gates.sh
x2` (`x2-status.txt`: `exec exit 101`, `corpus exit 0`, `srcline exit 0`, dirty count the same
before and after): `cargo test -j 4 --release -p rexx-exec --no-fail-fast` 1524 passed / 5 failed
summed over its binaries, the five being G3's (the three `ir::drive` counter tests,
`a_loops_per_pass_roots_outlive_the_pass_and_not_the_loop`,
`the_l0_subset_passes_again_under_collect_on_every_allocation`); `REXX_CORPUS_GATE=1` corpus
`541 of 541 matching`, `a_sidecar_changes_what_one_of_the_interpreters_answers ... ok`,
`every_sidecar_names_a_program_the_subset_runs ... ok`; sourceline 1 passed.

**X2 commit: `0e57202cd`.**

## X3 -- bind as each directive is translated (R3)

**Reproduction (run).** `p/f4e-before` above (oracle `pk.cls` / `pk.cls`, x1 `nil` / `nil` after a
trapped 90.998), and again on the X2 binary (`p/f4e-x2`, the same two lines DIFF), so X2 did not
touch it.

**Witness draft** `residual-fix/x3-draft/`: `pk.cls` is `::routine sq public external "LIBRARY
rxmath RxCalcSqrt"`, `::class Re public`, `::method doparse external "LIBRARY rxregexp
RegExp_Parse"`, `::method missing external "LIBRARY rxregexp NoSuchEntry"`, `::method pos
external "LIBRARY rxregexp RegExp_Pos"`. `main.rex` asks `loadExternalMethod(RegExp_Parse)` and
`loadExternalRoutine(RxCalcSqrt)` first, then a trapped `loadPackage('pk.cls')`, then prints the
code, both early objects' packages, a fresh `RegExp_Parse` answer's, and `RegExp_Pos`'s (the
directive after the refusal, which must stay unbound). On the X2 binary (`p/x3-witness-before`):
oracle `loadPackage raised 90.998` / `pk.cls` / `pk.cls` / `pk.cls` / `nil`; X2 differs on lines 2,
3 and 4 (`nil`); line 5, stderr and rc SAME.

**Fix.** `install_directives` binds each key `resolve_directive_library` answers as soon as that
directive resolved, inside the first walk, instead of collecting them and binding once the walk
finished; the comment that stated the old rule is replaced by the oracle's
(`createNativeMethod`, `parser/DirectiveParser.cpp:1381-1388`).

**Control predictions (written before the runs).**
* The draft witness on the fixed binary (`bins/rexx-run-x3`): SAME x3.
* `f4e` on the fixed binary: SAME x3. The X2 witness (`x2-draft`) and `f4b`/`f4a3`: still SAME x3.
* The negative control is the X2 binary, whose bind is the unfixed one: already DIFF on lines 2-4
  above (run, so not a prediction). Line 5 is the pairing control, SAME on both binaries: the
  binding stops at the refusal.
* The gated corpus with the witness committed: every program matching, the per-part sidecar
  control ok.

**Control results (run).** `bins/rexx-run-x3` (sha256 prefix `949669ee91ea8f04`):
`p/x3-witness-after` SAME x3 (`loadPackage raised 90.998` / `pk.cls` / `pk.cls` / `pk.cls` /
`nil`); `p/f4e-x3`, `p/f4b-x3`, `p/f4a3-x3`, `p/x2-witness-x3` SAME x3 each. Confirmed. No second
mutant was built: the X2 binary is the unfixed bind, and line 5 is the pairing control that the
binding stops at the refusal, SAME on both binaries.

**Witness.** `corpus/lang/library_bound_before_refusal.rex` (the draft, byte for byte) + `.env`
(`LD_LIBRARY_PATH={oraclelib}`) + `.d/pk.cls`, a row in `phase-8.txt`, companion regenerated
(count 23, lines `cmp` identical).

**Gates (committed tree).** fmt exit 0 and clippy exit 0 (`x3-clippy.txt`) before the commit;
then `residual-fix/gates.sh x3` at `f83b028a727cc2db150397ecf42738e557b5ef17`, dirty 0 before and
after (`x3-status.txt`): rexx-exec release exit 101, 1524 passed / 5 failed, the same G3 five;
gated corpus exit 0, `542 of 542 matching`, both sidecar tests ok; sourceline exit 0.

**X3 commit: `f83b028a7`.**

## X4 -- a retried `loadPackage` after a library failure (R4)

**Investigation (read, then run).**
* Where the oracle keeps a package: `PackageManager::getRequiresFile`
  (`package/PackageManager.cpp:824-838`) translates first (`LanguageParser::createPackage(name)`,
  `:828`) and caches after (`addToRequiresCache`, `:836`). A library a directive names is resolved
  inside that translation (`createNativeMethod`, `parser/DirectiveParser.cpp:1381-1388`), so a
  translation that raises never reaches the cache and a later ask translates the file again.
  Installation (`::REQUIRES`, class resolution) and the prologue run later, against the cached
  package, which is why `pk-retry`'s missing base class stays cached.
* Where the crate keeps it: `Interp::load_requires` (`rust/crates/rexx-exec/src/lib.rs`) inserts
  both names into `required_packages` **before** `run_loaded` (so that the prologue finds the
  entry), and `run_loaded` starts with `install_directives`, whose first walk is this crate's
  translation stage: duplicate names, `::OPTIONS`, `::ANNOTATE` targets, `unresolved_external`
  and `resolve_directive_library` all refuse there. A refusal from that walk returns through
  `load_requires` with the entry still in `required_packages`, and the next ask answers the cached
  id at the top of `load_requires` without installing anything.
* So the defect is wider than libraries. Measured (`residual-fix/cmp.sh`, oracle against
  `bins/rexx-run-x1`), each program a trapped `loadPackage('pk.cls')`, then the same again, then
  `say 'k class' .K`:
  ```
  x4-dup   (pk.cls: ::class K public / ::routine a / ::routine a)        oracle              | x1
    first raised 99.903 / second raised 99.903                         | first raised 99.903 / second loaded / k class .K
  x4-entry (::class K public / ::method a external "LIBRARY rxregexp NoSuchEntry")
    first raised 90.998 / second raised 90.998                         | first raised 90.998 / second loaded / k class .K
  x4-req   (::requires "zz_no_such_file_x4.cls" / ::class K public)   SAME x3: first raised 43.901 / second loaded / k class .K
  ```
  and the re-review's `pk-retry` (install-time, a missing base class) is SAME x3 with `k class .K`.
  The translation-stage refusals are not kept by the oracle; the install-time ones are, on both
  sides.
* The brief's parenthetical reads `pk-retry` as "kept and re-installed on the retry"; its
  transcript (`rereview-int/p/pk-retry/o.out`: `first raised 98.909` / `second loaded` / `k class
  .K`) shows kept and **not** re-installed, which is also what the crate does. The shape that
  matches both measurements is "a package whose translation failed is not kept".

**Decision: local, fixed.** The whole behavioural change is whether `required_packages` keeps an
entry: `load_requires` drops the two names it inserted when `run_loaded` fails before
`install_directives`' first walk completed. What it needs from outside the cache is one fact, "did
this program's first walk finish", which `install_directives` records in one line at the end of
that walk. Size as planned: one `Interp` field, one insert, the removal in `load_requires`.

**Witness draft** `residual-fix/x4-draft2/` (the corpus sidecar shape of `library_load_retried`:
`LD_LIBRARY_PATH=<oracle lib>:<run>`, `RETRY_FROM`, `RETRY_TO`; runner `residual-fix/cmpsep.sh`,
each side in its own directory). Three packages, each loaded twice through a trapped helper:
`pk2.cls` (the `x4-dup` shape: a duplicate `::ROUTINE`), `pk3.cls` (the `pk-retry` shape, the base
class put into `.environment` between the two asks, then `.K3`), and `pk.cls` (the `f5-t3` shape:
`LIBRARY yyregexp`, the library copied in between, then `.K` and `.K~new('a*b')~doparse('x')`).
On the X2 binary (`p/x4-witness2-before-{o,r}`):
```
                        oracle            x2
pk2.cls first           raised 99.903     raised 99.903
pk2.cls again           raised 99.903     loaded
pk3.cls first           raised 98.909     raised 98.909
pk3.cls again           loaded            loaded
its class               .K3               .K3
pk.cls first            raised 98.903     raised 98.903
pk.cls again            loaded            loaded
its class               The K class       .K
its method              0                 (rc 159: 97.1 .K does not understand NEW)
```
(An earlier draft, `x4-draft/`, put the library package first; the unfixed build died at its
`.K~new` and hid the rest, so the order was changed.)

**Control predictions (written before the fix is built).**
* The draft on the fixed binary (`bins/rexx-run-x4`): SAME x3, rc 0.
* `x4-dup`, `x4-entry`, `x4-req` (`cmp.sh`) and the re-review's `pk-retry` (`cmp.sh`) and `f5-t3`
  (`cmpsep.sh`): SAME x3 each.
* `f5-t2` through the forged `final-fix/ext/libforgever.so` (scratch only, read only): SAME x3
  against the oracle, including `pk loaded again`, `k 7`, `method pk.cls`, and `k4 method pk4.cls
  m pk.cls`.
* The unfixed control is the X3 binary (`bins/rexx-run-x3`): the draft DIFF on lines 2, 8, 9, and
  stderr and rc, as on the X2 binary above.
* M-X4b (the uncache taken for every failure, ignoring the first-walk marker): the draft DIFF on
  line 5 alone (`its class The K3 class`: the kept-and-not-reinstalled package is translated and
  installed again), `pk-retry` DIFF on its last line; everything else SAME.
* The X2 and X3 witnesses stay SAME; gated corpus every program matching, sidecar control ok.

**Fix, as built.** `Interp::translated` (the programs whose first walk finished), inserted at the
end of `install_directives`' first walk; `load_requires` removes the two `required_packages`
entries it made when `run_loaded` fails for a program not in that set. `run_loaded` starts with
`install_directives`, and a parse failure returns before the entries are made, so every other
failure path keeps what it kept.

**Control results (run).** `bins/rexx-run-x4` (sha256 prefix `5a94fe2b1507b2e1`):
* `p/x4-witness2-after-{o,r}` SAME x3, rc 0. Confirmed.
* `p/x4-dup-x4`, `p/x4-entry-x4`, `p/x4-req-x4`, `p/pk-retry-x4` (`first raised 98.909` / `second
  loaded` / `k class .K`), `p/f5-t3-x4-{o,r}` (`first raised 98.903` / `second loaded` / `k class
  The K class` / `k 0`): SAME x3 each. Confirmed.
* `p/f5-t2-x4` through `final-fix/ext/libforgever.so`: SAME x3 (`step 1 syntax 98.982` / `pk loaded
  again` / `k 7` / `method pk.cls` / `pk4 loaded` / `step 6 syntax 43.1` / `k4 method pk4.cls m
  pk.cls` / `loadLibrary 1`). Confirmed. On the X3 binary (`p/f5-t2-x3`) it differs on three lines:
  `step 3 syntax 97.1`, `method nil`, `k4 method pk4.cls m pk4.cls`.
* The X3 binary on the draft (`p/x4-witness2-x3-{o,r}`): DIFF on lines 2, 8, 9, stderr and rc 159.
  Confirmed.
* M-X4b (`--profile mutation`; the condition made `!self.translated.contains(&required) ||
  required.0 != usize::MAX`, true for every program; worktree restored from
  `residual-fix/x4-backup/`, `cmp` clean; binary `70aadaf31e55801f`): the draft DIFF on line 5 alone
  (`its class The K3 class`), stderr and rc SAME; `pk-retry` DIFF on its last line alone (`k class
  The K class`). Confirmed exactly: the marker is what keeps an install failure cached.

**Witness.** `corpus/lang/library_package_retried.rex` (the draft, byte for byte) + `.env`
(`LD_LIBRARY_PATH={oraclelib}:{run}`, `RETRY_FROM`, `RETRY_TO`, as `library_load_retried.env`) +
`.d/pk.cls`, `.d/pk2.cls`, `.d/pk3.cls`; a row in `phase-8.txt`; companion regenerated (count 24,
lines `cmp` identical). `f5-t2` stays a scratch transcript.

**Gates (committed tree).** fmt exit 0 and clippy exit 0 (`x4-clippy.txt`) before the commit;
`residual-fix/gates.sh x4` at `9b7f77e4c7dd36d50547d9c76cd6bb286caf083c`, dirty 0 before and
after: rexx-exec release exit 101, 1524 passed / 5 failed, the same G3 five; gated corpus exit 0,
`543 of 543 matching`, both sidecar tests ok; sourceline exit 0.

**X4 commit: `9b7f77e4c`.**

## X5 -- small instruments and false statements in code

### Boundary minor 3: a Miri-runnable test through the thread table

Drafted in `residual-fix/x5-api/rust` (`git archive 69a579370 rust`; `rexx-api` has not changed
since). A `#[cfg(test)]` stub `ffi::thread_table_stub` (signature `int` result, one
`RexxStringObject` parameter) reads its method context's `threadContext`, calls `StringLength`,
`StringData`, `WholeNumberToObject` (twice), `NewPointer` and `RaiseException0` through that
thread context's table, and stores the length, the first byte and the pointer object with the
method table's `SetObjectVariable`. `invoke::tests::a_stub_reaches_its_activation_through_the_thread_context`
runs it through `Contexts::method` with the argument `hello` and asserts the outcome, the pending
condition (`STUB_CONDITION`), `LENGTH` 5, `FIRST` 104, and that `POINTER`'s object carries
`STUB_POINTER`. Stable, scratch: `cargo test -p rexx-api --lib` exit 0, 17 passed, the new test ok
(`x5-api-scratch-lib.txt`).

**Predictions (written before the Miri runs).**
* M5a, the scratch copy as drafted, Stacked Borrows, `-p rexx-api --lib`: 16 passed, 1 ignored
  (`an_unbuilt_entry_refuses_loudly`), exit 0.
* M5b, a copy with one line changed, `Contexts::method` writing `threadContext =
  (&raw mut self.thread.context).cast()` (the field rather than the whole wrapper), the same run
  narrowed to the new test: exit 1 with `Undefined Behavior: attempting a read access ... but that
  tag does not exist in the borrow stack` at `owner_of`'s read, the tag created by a
  SharedReadWrite retag of the thread context's 16 bytes at the mutated line, backtrace
  `owner_of` <- `activation_of` <- `string_length` (the first thread callback the stub calls) <-
  `thread_table_stub`. On stable, the same mutant passes the test (the read lands on the right
  bytes).

**Results (run, `residual-fix/miri.sh`, statuses in `residual-fix/miri-status.txt`).**
* M5a exit 0 (`miri-x5-M5a-sb.txt`): `16 passed; 0 failed; 1 ignored`, the new test ok. Confirmed.
* M5b exit 1 (`miri-x5-M5b-sb.txt`; copy `residual-fix/x5-api-mut/rust`, `diff` shown: the one line
  at `ffi.rs:207`): `Undefined Behavior: attempting a read access using <181353> at
  alloc56552[0x30], but that tag does not exist in the borrow stack for this location` at
  `ffi.rs:46:14` (`(*owned).owner`), `<181353> was created by a SharedReadWrite retag at offsets
  [0x20..0x30]` at `ffi.rs:207:45` (the mutated `&raw mut self.thread.context`), backtrace
  `ffi::owner_of::<RexxThreadContext_, Activation>` <- `ffi::activation_of` <- `ffi::string_length`
  <- `ffi::thread_table_stub` <- `load.rs:176` <- `invoke.rs:91` <- the test. Confirmed.
* The same mutant on stable (`x5-M5b-stable.txt`): exit 0, the test ok. Confirmed: Miri is the
  instrument for this shape, and the tree now has a test it can run there.

Ported to the worktree by copying the two files (the worktree's were `cmp` identical to
`69a579370`'s first).

### Boundary minor 4: `invoke::method`'s `# Errors`

Now says `Failure::Signature` is answered for a signature the array cannot hold, or for an unknown
parameter code given an argument or carrying the optional bit, and `Failure::MissingArgument` for a
parameter that takes an argument, is not optional and was given none, whether or not the table
knows its code (read against `values::to_native`).

### R7: `Interp::executable_package`'s doc

Now says it also answers `None` for a `loadExternal*` answer no directive has bound.

**What the oracle answers, measured** (`residual-fix/cmp.sh`, oracle against `bins/rexx-run-x2`):
`p/r7-unbound`, `m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')` (nothing
binds it; `m~package` prints `The NIL object` on both), then `.Package~new('x', 'say "prolog
ran"', m)`: the oracle answers a package, rc 0, stdout `prolog ran` / `answered The Package class
x` / `routines 0`, stderr empty (`BaseExecutable::getPackage` hands `.nil` on as the parent,
`classes/PackageClass.cpp:204`). The crate raises instead: rc 163, `Error 93.953:  Method argument
3 could not be converted to type Method, Routine, or Package object.`, nothing on stdout.
`p/r7-routine`, the same with `.Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcPower')`:
the same two answers. F4 introduced it: slice B's build of the round's start
(`final-b/target-head/release/rexx-run`, `e64202ae7`) on `r7-unbound` (`p/r7-unbound-base`) reports
`m~package` as `The REXX Package`, answers the package (`prolog ran` / `answered The Package class
x`) and then stops loudly at rc 120 on `~routines~items`. So a `.Package~new` whose context is an
unbound `loadExternal*` object now raises 93.953 where the oracle answers; not fixed here, since the
brief asks for the measurement. Not probed further: what resolution through a `.nil` parent does
on the oracle.

### R8: `library_opens`

Renamed `library_open_attempts`, documented as the asks that reach `load::open` for a name nothing
held, whether or not a library loaded; the test's two comments that said "open" say "attempt".

### R5: the inert `LD_LIBRARY_PATH` line, and the per-part control

`library_search_path_fixed.env` loses its `LD_LIBRARY_PATH={oraclelib}` line, and its comment no
longer says the library is on the search path the interpreter starts with. The split is cheap:
`tests/support/sidecar.rs`'s `Half::Environment` becomes `Half::Variable(name)`, one part per
override, and `Sidecar::without` removes that one variable; `corpus.rs` passes the part by
reference and its doc says "environment variables".

### R10: `ir_recorded` sees a program that stopped at a library it should have reached

`compare` now also fails a corpus case whose stderr carries `Unable to load library "<name>"` for a
name whose `lib<name>.so` is in the oracle's build directory (`support::oracle::oracle_root()`'s
`lib`). The witnesses of a missing library name `zorkolib`, which that directory does not hold, so
the rule is derived from the directory rather than from a list of programs.
`collect_stress`'s L0 test is left as it is: it still stops at its pre-existing panic
(`dispatch.rs:1506`, on a `phase-7.txt` program) before any `phase-8.txt` program runs.

**Control predictions (written before the runs).**
* C-R5a, the gated corpus over the X5 tree: `543 of 543 matching`,
  `a_sidecar_changes_what_one_of_the_interpreters_answers ... ok` (no other override in any `.env`
  is inert on its own).
* C-R5b, the same with the removed line put back (worktree file restored from a copy afterwards):
  the control FAILED with `lang/library_search_path_fixed.rex: neither interpreter answers
  differently without its Variable("LD_LIBRARY_PATH")`; the differential still `543 of 543`.
* C-R10a, `cargo test --release -p rexx-exec --test ir_recorded`: exit 0,
  `every_population_runs_without_a_refusal ... ok`.
* C-R10b, `compare`'s sidecar replaced by `Sidecar::default()` (restored from a copy): that test
  FAILED, its message listing `[corpus]` library programs with `stopped at 98.903 for rxregexp`,
  among them `lang/library_method_external.rex`, and no `zorkolib` program.
* `cargo test -p rexx-exec --lib dispatch::library`: 11 passed after the rename; `cargo test -p
  rexx-api`: lib 17, the rest as after X1.

**Control results (run; statuses in `residual-fix/x5-controls-status.txt`).**
* C-R5a exit 0 (`x5-C-R5a.txt`): `543 of 543 matching`, `a_sidecar_changes_what_one_of_the_interpreters_answers
  ... ok`, `every_sidecar_names_a_program_the_subset_runs ... ok`. Confirmed.
* C-R5b exit 101 (`x5-C-R5b.txt`; the line put back, `diff` shown, file restored from
  `x5-env-new.bak`, `cmp` clean): `a_sidecar_changes_what_one_of_the_interpreters_answers ...
  FAILED` at `corpus.rs:733:13`, `lang/library_search_path_fixed.rex: neither interpreter answers
  differently without its Variable("LD_LIBRARY_PATH"), so that part of the sidecar is not
  load-bearing ...`; `543 of 543 matching`. Confirmed exactly. (The same line against the
  whole-environment control is the re-review's C3, ok.)
* C-R10a exit 0 (`x5-C-R10a.txt`): 28 passed, `every_population_runs_without_a_refusal ... ok`.
  Confirmed.
* C-R10b exit 101 (`x5-C-R10b.txt`; `compare`'s sidecar replaced, `diff` shown, file restored from
  `x5-ir_recorded.bak`, `cmp` clean): `every_population_runs_without_a_refusal ... FAILED`, `11 of
  10883 programs did not run cleanly`, every line `[corpus] ... stopped at 98.903 for rxregexp,
  which the oracle's build directory holds`, for `library_method_entry_missing`,
  `library_routine_entry_missing`, `library_attribute_entry_missing`, `library_method_external`,
  `library_loads_once`, `library_method_missing_argument`, `library_method_extra_arguments`,
  `library_method_raises` and the three `library_uninit_*`; no `zorkolib` program. Confirmed. What
  it does not see: a fixture witness that stops at a missing `.d/` file (43.901) or a library whose
  load failure is trapped or answers `.nil` silently; the corpus harness's per-part control is
  still the instrument for those.
* `dispatch::library` 11 passed (`x5-lib.txt`); rexx-api exit 0, lib 17 with the new test ok,
  context 14, handles 5, invoke 9, layout 17, load 10, values 38, doctests 5 (`x5-api.txt`).
  Confirmed.

**X5 commit: `e8a6b7667`.** Miri (Stacked Borrows) over the committed worktree's `-p rexx-api --lib`
(`--locked`, scratch target and `XDG_CACHE_HOME`): exit 0, `16 passed; 0 failed; 1 ignored`, the new
test ok (`miri-x5-worktree-sb.txt`).

**Gates (committed tree).** fmt exit 0 and clippy exit 0 (`x5-clippy.txt`) and `unsafe_sites` 2
passed before the commit; then at `e8a6b76671842a2ed8100013f09cf589745bc38a`, dirty 0 before and
after (`x5-status.txt`): rexx-exec release exit 101, 1524 passed / 5 failed, the same G3 five;
gated corpus exit 0, `543 of 543 matching`, both sidecar tests ok; sourceline exit 0; rexx-api exit
0 (lib 17, context 14, handles 5, invoke 9, layout 17, load 10, values 38, doctests 5); clippy exit 0
again (`x5-clippy-commit.txt`, a warm target, so provisional in `rust/CLAUDE.md`'s sense).

## Commits

```
69a579370 X1  Make every interface table slot unsafe to call
0e57202cd X2  Share one routine object per library routine table entry
f83b028a7 X3  Bind a library procedure as its directive resolves
9b7f77e4c X4  Do not keep a package whose translation raised
e8a6b7667 X5  Close the residual instruments and doc statements from the re-review
```

## Not done

* **Parked by the controller, not touched:** R9 (a user `REQUEST` method), R6 (the two counts in
  F1's and F4's commit messages), boundary minor 2 (the previous report's Tree Borrows claim).
* **`docs/` untouched.** Facts this round changed that the docs pass may meet: every interface
  table slot is `unsafe extern "C" fn` and `METHOD_CONTEXT` carries a `compile_fail` doctest; a
  library routine's shared object is keyed by its table entry and a later `::ROUTINE` binder
  reports the first binder; binding happens per directive during the first walk; a package whose
  first walk raised is not kept in `required_packages`; `library_opens` is `library_open_attempts`;
  the per-part sidecar control asks per environment variable; `ir_recorded` fails a corpus program
  that stopped at a library the oracle's build directory holds; the thread-table Miri test.
* **Measured and left, a divergence F4 introduced (R7):** `.Package~new(name, source, <unbound
  loadExternal* object>)` answers a package on the oracle and raises 93.953 here; before the round
  the crate answered a package. Not in this brief's fixes.
* **Read, not measured or not constructible:**
  * An `::ATTRIBUTE ... EXTERNAL "LIBRARY"` whose `GET` procedure resolves and whose `SET` does not:
    the oracle binds the getter before refusing the setter (`parser/DirectiveParser.cpp:1682`,
    `:1689`), and `resolve_directive_library` binds neither. No oracle-built extension exports a `GET<name>` without its `SET<name>`, so it is not
    measurable within the corpus rules.
  * `Library::routine` answers the first caseless match; `LibraryPackage::resolveRoutine` tries the
    exact spelling first (`:420`). They differ only for a routine table with two entries that differ
    only in case; no shipped extension has one.
  * A required file that fails to parse after a directive that would have bound a procedure: the
    oracle translates in order, so the binding would stand; this crate parses the whole file first
    and refuses loudly (`Loud::required_source`), so it is a loud refusal, not a silent answer.
  * After a translation failure the uncached program's per-program records (its `routines`,
    `package_options`, `required_paths`) stay keyed by an id nothing reaches again, and the
    set-once `reqstr_armed`/`lostdigits_armed` stay armed if its first walk set them. `translated`
    keeps one entry per program that got through its first walk.
* **R10's reach:** the new assertion sees an untrapped 98.903 on a library the oracle ships; it
  does not see a missing fixture (43.901) or a trapped or silent load failure. `collect_stress`'s
  L0 test is unchanged and still stops at its pre-existing panic before `phase-8.txt`.
* **Instruments outside the gate:** Miri ran from the previous implementer's scratch `RUSTUP_HOME`;
  the stable toolchain does not check the `compile_fail` code (checked with nightly rustdoc, C3 and
  C3b). Tree Borrows was not run this round.
* **Gates not run, per the brief:** the debug `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace`
  (G4) and the full workspace gates.


## Follow-up requested by the controller after the round

Two concerns from the list above were ruled not parkable: concern 1 (R7, a regression F4
introduced) and the second half of concern 4 (`Library::routine`'s match order). One commit each,
same discipline. Sections appended as the work runs.

### Y1 -- an unbound `loadExternal*` object as a context argument (R7)

**Reading (before any run).** `BaseExecutable::getPackage` answers `resultOrNil(package)`
(`execution/BaseExecutable.cpp:121-125`), and all three consumers take that as a `PackageClass *`:
`PackageClass::newRexx` (`classes/PackageClass.cpp:202-205`), `processNewExecutableArgs`
(`execution/BaseExecutable.cpp:274-277`) and `processNewFileExecutableArgs` (`:343-346`). The
parser then makes it the new package's parent (`PackageClass::inheritPackageContext`,
`classes/PackageClass.cpp:680-684`, from `LanguageParser::generateProgram` and its siblings,
`parser/LanguageParser.cpp:603`, `:637`, `:669`). Every lookup that falls through to the parent
tests `parentPackage != OREF_NULL` and then calls a `PackageClass` member on `TheNilObject`
(`findLocalRoutine` `:836-839`, `findPublicRoutine` `:878-881`, `findInstalledClass` `:997-1000`,
`findPublicClass` `:1042-1049`, `resolveProgramName` `:931-934`), which reads `PackageClass`
fields out of an object that is not one. So on the oracle any name the new code resolves outside
itself is undefined behaviour in the C++.

**Predictions for the oracle probes (written before the runs).** `residual-fix/src/y1-*`, the
context always `m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')`, unbound:
* `y1-pkg-find` without any fall-through for the `.Package~new` itself: `prolog ran` as measured.
  For `findRoutine`, `findClass('Array')`, `findClass('NoSuchZZ')`, and in the other probes for a
  prolog `say .array`, a `call` of a routine the caller defines, `.Routine~new`/`.Method~new` and
  `newFile` over such a context: I cannot predict the answer from the C++, because it is undefined
  behaviour. The candidates are a crash (rc 139), the answers an absent context gives (a caller's
  routine 43.1, `.array` the class), or something else. Each probe is its own program so one crash
  does not hide the rest. The two controls, `y1-pkg-realcontext` (`.context~package` as context)
  and `y1-pkg-nocontext` (no context), I predict as `prolog ran` / `callerroutine ran` / `answered
  x` and `prolog ran` / `raised 43.1` respectively, both rc 0.

**Results (run, `residual-fix/cmp.sh`, oracle against `bins/rexx-run-x5`, the release build of
`e8a6b7667`, sha256 prefix `220df1dcdc8373a5`).**
```
probe                       oracle                                              crate (e8a6b7667)
y1-pkg-realcontext          prolog ran / callerroutine ran / answered x, rc 0   SAME x3
y1-pkg-nocontext            prolog ran / raised 43.1, rc 0                      SAME x3
y1-pkg-find                 prolog ran / findRoutine CALLERROUTINE, rc 139      rc 163, 93.953
y1-pkg-dotclass             prolog ran, rc 139 (at `say .array`)                rc 163, 93.953
y1-pkg-callerroutine        prolog ran, rc 139 (at `call callerroutine`)        raised 93.953, rc 0
y1-routine-new              answered The Routine class, rc 139 (at `say .array` inside `r~call`)   rc 120 "NEW of class Routine is not implemented (Phase 5)"
y1-routine-new-plain        answered / call 42 / package The Package class, rc 0                   rc 120, the same refusal
y1-method-new               answered The Method class / package The Package class, rc 0           rc 120 "NEW of class Method ..."
y1-routine-newfile          answered The Routine class, rc 139 (at `say .array` in the file)     rc 216, 40.904 "found a Method"
y1-routine-newfile-plain    answered / call 44 / package The Package class, rc 0                   rc 216, 40.904
y1-method-newfile           answered The Method class / package The Package class, rc 0           rc 216, 40.904
```
Every oracle rc 139 is a SIGSEGV (`Segmentation fault` from the runner's shell), and it reproduces:
`y1-pkg-find`, `y1-pkg-callerroutine` and `y1-routine-newfile` run again (`p/*-again`) crash the
same way. The two controls are as predicted. So the oracle answers every shape in which nothing is
resolved through the context, and segfaults on the first name that is: a routine call, a `.name`,
`findRoutine`, `findClass`. There is no defined oracle answer to match there.

On slice B's build of the round's start (`final-b/target-head/release/rexx-run`, `e64202ae7`), where
such an object reported the `REXX` package: `y1-method-newfile` and `y1-routine-newfile-plain` SAME
x3; `y1-pkg-callerroutine` answers `raised 43.1` where the oracle crashes (`p/*-base`). And
`.Method~new('mm', 'return 43', .context~package)` (`y1-method-new-realcontext`) is the same rc 120
refusal on both the base and HEAD: `Method~new` and `Routine~new` refuse any third argument, whatever
it is (`native_executable_new`'s `args.len() > 2`). That refusal predates the round, is loud, and is
not this shape's; it is left.

**Decision.** An unbound `loadExternal*` object passed as a context hands on the `REXX` package as
the parent: what the base did, and a parent this crate's routine walk stops at (`run.rs`'s walk over
`package_parents` ends at anything but `Package::Program`), so every shape the oracle answers is
answered the same. Where the oracle segfaults, the crate answers as the base did (43.1 for a
caller's routine; `.array` the class). `Interp::executable_package` becomes
`Interp::executable_context_package`, its doc saying so, and the crash goes into
`corpus/oracle-crashes.txt` so no sweep runs it.

**Witness draft** `residual-fix/y1-draft/` (`main.rex`, `mf.cls`, `rf.cls`, `rf2.cls`, `pk.cls`):
an unbound method and an unbound routine as contexts of `.Package~new` (string and array source)
and of `Method~newFile`/`Routine~newFile`, printing only what runs without a lookup through the
context; then `pk.cls` binds the method, and the same object as context makes `call pkroutine` and
`pkfunction()` resolve through `pk.cls`, the paired success. Oracle rc 0 on every line
(`p/y1-witness-before`); the crate stops at line 8 with 93.953, rc 163.

**Control predictions (written before the fix is built).**
* The draft on the fixed binary: SAME x3, rc 0.
* M-Y1 (the unbound object answers `REXX` for every executable, bound or not): the draft DIFF from
  `bound method package`'s next line on, since `call pkroutine` then raises 43.1 untrapped, rc 213.
* The crash shapes on the fixed binary: `y1-pkg-callerroutine` prints `prolog ran` / `raised 43.1`;
  `y1-pkg-dotclass` prints `prolog ran` / `The Array class` / `.NOSUCHZZ` / `answered x`;
  `y1-routine-newfile` prints `answered The Routine class` / `The Array class` / `call 44`. Oracle
  unchanged (rc 139).
* The base's newFile probes and `y1-pkg-realcontext`/`y1-pkg-nocontext` SAME x3; gated corpus
  every program matching.

**Control results (run).** Fixed binary `bins/rexx-run-y1` (sha256 prefix `d036d80a529fd375`):
* `p/y1-witness-after` SAME x3, rc 0, every line as the oracle's. Confirmed.
* M-Y1 (`--profile mutation`; every `Loaded` executable, bound or not, hands on `REXX`; restored
  from `residual-fix/y1-backup/`, `cmp` clean; `90479f482901eb26`), `p/y1-MY1`: lines 1-8 SAME, then
  the crate stops at `call pkroutine` with `Error 43 running fromsource3 line 1` / 43.1, rc 213,
  losing the last three lines. Confirmed.
* The crash shapes: `p/y1-pkg-callerroutine-y1` `prolog ran` / `raised 43.1`; `p/y1-pkg-dotclass-y1`
  `prolog ran` / `The Array class` / `.NOSUCHZZ` / `answered x`; `p/y1-routine-newfile-y1`
  `answered The Routine class` / `The Array class` / `call 44`; all rc 0 against the oracle's 139.
  Confirmed.
* `p/y1-method-newfile-y1`, `p/y1-routine-newfile-plain-y1`, `p/y1-pkg-realcontext-y1`,
  `p/y1-pkg-nocontext-y1` SAME x3. Confirmed.

**Witness.** `corpus/lang/library_context_package.rex` (the draft, byte for byte) + `.env`
(`LD_LIBRARY_PATH={oraclelib}`) + `.d/mf.cls`, `.d/rf.cls`, `.d/rf2.cls`, `.d/pk.cls`; a row in
`phase-8.txt`; companion regenerated (count 28, lines `cmp` identical). `corpus/oracle-crashes.txt`
gains entry 15 with the crashing program and its controls, cited from the new doc comment.

**Gates (uncommitted tree, committed unchanged).** `cargo fmt --all --check` exit 0, `cargo clippy
-j 4 --workspace --all-targets -- -D warnings` exit 0 (`y1-clippy.txt`); `residual-fix/gates.sh
y1-pre` (`y1-pre-status.txt`, dirty count 9 before and after): rexx-exec release exit 101, 1524
passed / 5 failed, the same G3 five; gated corpus exit 0, `544 of 544 matching`, both sidecar tests
ok; sourceline exit 0.

### Y2 -- `Library::routine` exact spelling first, then caseless

**Reading (printed).** `LibraryPackage::resolveRoutine` (`interpreter/package/LibraryPackage.cpp:410-435`)
looks the name up in `routines` byte for byte (`:420`), and only on a miss asks
`locateRoutineEntry` (`:344-363`), which answers the first row matching without regard to case
(`:355`), and then looks up that row's own spelling (`:428`). `loadRoutines` fills `routines` keyed
by each row's own spelling (`:291`), and `HashContents::put` replaces the value of a key already
present (`classes/support/HashContents.cpp:245-256`), so of rows spelled alike the table holds the
last. `Library::routine` (`rust/crates/rexx-api/src/load.rs`) answered the first row matching
without regard to case.

**Can a shipped extension tell them apart? No.** Every `REXX_TYPED_ROUTINE`/`REXX_CLASSIC_ROUTINE`
table under `extensions/`, `interpreter/` and `testbinaries/` scanned for names equal without regard
to case: no pair differing only in case. The one duplicate is `rexxutil_routines`' `SysUtilVersion`,
listed twice with the same spelling and the same entry point (`interpreter/runtime/RexxUtilCommon.cpp:2161`,
`:2176`), which no row choice can observe, in a package this crate does not open through
`Library`. So the witness is a unit test at the `Library` level, over a routine table the test
builds (`Library` with `libloading::os::unix::Library::this()` as its handle, which opens no new
image).

**Change.** `Library::routine` answers the last row spelled exactly as asked; failing that, the
last row spelled as the first row matching without regard to case.

**Control predictions (written before the runs).** Test
`load::tests::a_routine_is_found_by_its_exact_spelling_before_its_case` over rows `Foo` (entry 1),
`FOO` (2), `Foo` (3), `bar` (4):
* the fixed code: ok; `FOO` answers 2, `Foo` answers 3, `foo` answers 3, `BAR` answers 4, `baz` none.
* M-Y2a (the old body, first caseless match): the test fails on `FOO`'s assertion (1 answered).
* M-Y2b (exact first, but the first row of a spelling rather than the last): fails on `Foo`'s
  assertion (1 answered).
* Under Miri the test is ignored (`this()` is a `dlopen`), so under Miri the lib is 16 passed, 2
  ignored. The gated corpus and `dispatch::library` stay as they are (no shipped library can see
  it).

**Control results (run).** Built in `residual-fix/y2-api/rust` (`git archive e8a6b7667 rust`;
`load.rs` is unchanged by Y1), `CARGO_TARGET_DIR=residual-fix/target-y2`:
* The fixed code: `load::tests::a_routine_is_found_by_its_exact_spelling_before_its_case ... ok`
  (`y2-scratch.txt`). Confirmed.
* M-Y2a (the old body; `diff` shown against `y2-load.rs.fixed`), `y2-MY2a.txt`: exit 101, FAILED at
  the `FOO` assertion (`load.rs:596` in the mutant, eight lines shorter), `left: Some(1)`, `right:
  Some(2)`. Confirmed.
* M-Y2b (`.rev()` dropped from the last line), `y2-MY2b.txt`: exit 101, FAILED at the `Foo`
  assertion (`:605`), `left: Some(1)`, `right: Some(3)`. Confirmed. Restored from the copy, `cmp`
  clean.
* Miri, Stacked Borrows, `-p rexx-api --lib` over the scratch copy (`miri-y2-sb.txt`): exit 0, `16
  passed; 0 failed; 2 ignored`, the new test `ignored, opens the running image`. Confirmed.

Ported by copying the file (the worktree's `load.rs` was `cmp` identical to `e8a6b7667`'s), then
two doc sentences reworded. Worktree: fmt exit 0; clippy exit 0 (`y2-clippy2.txt`); `cargo test -j
4 -p rexx-api --no-fail-fast` exit 0, lib 18 with the new test ok, context 14, handles 5, invoke 9,
layout 17, load 10, values 38, doctests 5 (`y2-api.txt`); `unsafe_sites` exit 0.

**Gates (committed tree).** `residual-fix/gates.sh y2` at `233d2766d780a59b0fc0e41a9bf31196e5fc823f`,
dirty 0 before and after (`y2-status.txt`): rexx-exec release exit 101, 1524 passed / 5 failed,
the same G3 five; gated corpus exit 0, `544 of 544 matching`, both sidecar tests ok; sourceline
exit 0; rexx-api exit 0 (lib 18).

**Y1 commit: `9c0d44d8e`. Y2 commit: `233d2766d`.**

## Correction to "Not done" (residual re-review, minor 2)

The sentence under "Not done" reading "After a translation failure the uncached program's
per-program records (its `routines`, `package_options`, `required_paths`) stay keyed by an id
nothing reaches again" is false. A `loadExternal*` object that a directive of that program bound
before the failure (X3) still reports the program's id as its package, and that id reaches
`routines` as the parent of a `.Package~new` or `newFile` built with the object as context: the
re-review's `x34-method-lookup` answers `pkr ran` at rc 0 where the oracle raises 43.1 at rc 213.
The sentence was written as a reason not to clear those records, and it hid the defect. Z1 below.

## Z1 -- a package whose translation raised keeps what its first walk recorded

**Finding** (`residual-rereview.md`, section 2.4 `x34-method-lookup` and section 6 finding 1, read
in full): X3 binds a library procedure to `pk.cls` while it translates; a later directive refuses;
X4 drops `pk.cls` from `required_packages` but the `routines` record the first walk made under
that `ProgramId` stays, and `installed_routine`'s parent walk finds it through the bound object's
package.

**Reading (printed).** The oracle's parser fills its own `routines`, `publicRoutines`,
`unattachedMethods` and `resources` tables while translating, and hands them to the package only in
`resolveDependencies`, after every directive (`parser/LanguageParser.cpp:1893-1908`, reached from
`translate`, `:1128`); a translation that raises never gets there, so the discarded package's
tables are null, and each reader answers an empty table for a null one
(`classes/PackageClass.cpp:1606-1620` for `~routines`; `~publicRoutines` `:1633-1642`,
`~resources` `:1693-1706`, `~classes` `:1546-1555` alike) and `findRoutineRexx` answers
`resultOrNil` (`:2007-2011`). `::OPTIONS` writes straight onto the package while translating
(`parser/DirectiveParser.cpp:993`, `:1051`, `:1075`, and the rest of `optionsDirective`), so the
discarded package keeps its settings.

The crate's first walk writes, under the program's id: `routines` and `package_public_routines`
(each `::ROUTINE`), `package_options` (`::OPTIONS`), `library_routine_codes` (X2), plus the
library bindings X3 intends and the set-once `reqstr_armed`/`lostdigits_armed` gates.
`~definedMethods`, `~resources`, `~resource` and the `.METHODS`/`.RESOURCES` tables are built from
the parsed program itself (`environment.rs`'s `package_table_entries`), not from anything the walk
recorded.

**Predictions (written before the runs).** `residual-fix/src/z1-*`, each one reader after a
trapped `loadPackage('pk.cls')` whose `pk.cls` is `::options digits 12`, public routines `pkr` and
`pkf`, an unattached `::method um`, `::resource res`, `::class K`, a method bound to `RegExp_Parse`,
and a method on `NoSuchEntry`; `pkg = early~package`, where `early` is a `loadExternalMethod` of
`RegExp_Parse` made before the load. Oracle, from the reading: `loadPackage raised 90.998` /
`package pk.cls`, then `routines PKR 0`, `publicRoutines PKR 0`, `findRoutine PKR The NIL object`,
`classes K 0`, `publicClasses K 0`, `findClass K The NIL object`, `definedMethods UM 0`,
`resources RES 0`, `resource RES The NIL object`, `digits 12`, `sourceSize 13`, `importedPackages
0`, `parent call raised 43.1`, `parent function raised 43.1`. Crate at `233d2766d`
(`bins/rexx-run-y2`): `routines PKR 1`, `publicRoutines PKR 1`, `findRoutine PKR` a routine,
`definedMethods UM 1`, `resources RES 1`, `resource RES` an array, `pkr ran` and `pkf answered`
for the two parent probes; `classes`, `publicClasses`, `findClass`, `digits`, `sourceSize`,
`importedPackages` as the oracle; any `StringTable` reader this crate has not built is a loud rc
120 rather than an answer.

**Results (run, `residual-fix/cmp.sh`, oracle against `bins/rexx-run-y2`, the release build of
`233d2766d`, sha256 prefix `3f6ea45fda8147c3`).** Every probe prints `loadPackage raised 90.998` /
`package pk.cls` on both sides first; the reader line:
```
probe (p/z1-<name>-y2)   oracle                              crate 233d2766d
routines                 routines PKR 0                      rc 120, StringTable HASINDEX not implemented
publicroutines           publicRoutines PKR 0                rc 120, the same
classes, publicclasses   K 0                                 rc 120, the same
definedmethods           definedMethods UM 0                 rc 120, the same
resources                resources RES 0                     rc 120, the same
routines-isempty         routines isEmpty 1                  rc 120, ISEMPTY
routines-supplier        routines supplier 0                 rc 120, SUPPLIER
routines-allindexes      routines allIndexes 0               rc 120, ALLINDEXES
findroutine              findRoutine PKR The NIL object      findRoutine PKR a Routine          silent
findpublicroutine        The NIL object                      a Routine                          silent
routines-at              routines[PKR] The NIL object        a Routine                          silent
routines-over            routines over 0                     routines over 2                    silent
publicroutines-at        The NIL object                      a Routine                          silent
definedmethods-at        definedMethods[UM] The NIL object   a Method                           silent
resources-at             resources[RES] The NIL object       resource line                      silent
resource                 resource RES The NIL object         resource RES resource line         silent
prolog                   prolog The NIL object               prolog a Routine                   silent
parentcall               parent call raised 43.1             pkr ran / package answered ...     silent
parentfunction           parent function raised 43.1         pkf answered / package answered    silent
newfile-parent           newFile parent raised 43.1          pkr ran / newFile answered 1       silent
findclass                findClass K The NIL object          SAME
digits                   digits 12                           SAME
settings                 form SCIENTIFIC fuzz 0 trace N      SAME
sourcesize, sourceline   sourceSize 13, sourceLine 2 ::routine pkr public   SAME
imported, annotations, namespaces, importedroutines   0 / The NIL object     SAME
```
rc 0 on the oracle for every probe. Every prediction held; not predicted: `do over` and `[]` are
answered silently by the crate where `hasIndex` and `items` are loud, `~prolog` was not in the
prediction and is silent too (`PackageClass::getMainRexx` answers `.nil` without `initCode`,
`classes/PackageClass.cpp:2115-2119`, which `generateProgram` sets only after a translation that
finished, `parser/LanguageParser.cpp:656-665`), and `newFile` with the object as context reaches
the routine like `.Package~new` does.

**Fix, as planned.** Two parts.
* The first walk keeps each `::ROUTINE`'s records (`routines`, `package_public_routines`, and the
  `library_routine_codes` row X2 reads) in locals and hands them to the interpreter where the walk
  finishes, as the oracle hands its tables over in `resolveDependencies`. A translation that raises
  leaves no routine record under its id, so `findRoutine`, `~routines`, `~publicRoutines` and a parent
  walk through it find nothing.
* X4's `translated` becomes `untranslated`: the id goes in when `install_directives` starts and comes
  out where the first walk finishes, so it stays only for a program whose translation raised, and never
  for a program that did not come through `install_directives`. `load_requires` drops the cache
  entries for such a program (X4, unchanged behaviour); the package tables built from the parsed
  program (`~definedMethods`, `~resources`, `~resource`) are empty for it; `~prolog` is `.nil`.
  `package_options` stays (the oracle keeps its settings, `digits 12` SAME).

**Control predictions for the fix (written before building it).**
* `bins/rexx-run-z1`: every silent row above SAME x3; every row SAME before stays SAME; the loud rows
  stay loud (rc 120, the same messages).
* The re-review's `x34-method-lookup`: SAME x3 (43.1, rc 213). `x34-method`, `x34-method-class`,
  `x34-routine` (first five lines): as on HEAD. X3's and X4's witnesses and `pk-retry`, `f5-t2` (forged,
  scratch): SAME x3.
* M-Z1a (the routine records written straight into the interpreter again, as before): the new witness
  DIFF on its parent-call and `findRoutine` lines (`pkr ran`, `a Routine`), everything else SAME.
* M-Z1b (`untranslated` never consulted by the table and prolog readers): DIFF on the
  `definedMethods`/`resource`/`prolog` lines alone.
* Gated corpus: every program matching; X4's `library_package_retried` included.

**Results of the fix (run, `bins/rexx-run-z1`, sha256 prefix `578de62d8dda0ee9`, built before a
one-sentence rewording of `prolog`'s doc).** Every silent row of the table above is SAME x3
(`p/z1-*-z1`); the rows SAME before stay SAME; the loud rows stay loud with the same messages.
Confirmed. The re-review's probes on the same binary: `rr-x34-method-lookup-z1` stdout SAME (`first
raised 90.998` / `method after dropped binder pk.cls`), rc SAME 213, stderr DIFF only by the
oracle's `Compiled method "NEW" with scope "Package".` and `4 *-* p = .Package~new(...)` lines,
which the base lacks too (the re-review's "loud, pre-existing, not counted" traceback gap;
`rereview/p/x34-method-lookup-base/r.err` is the same three lines); `rr-x34-method-z1`,
`rr-x34-method-class-z1`, `rr-y1-more-z1` SAME x3; `rr-x34-routine-z1` its five lines SAME and then
the pre-existing rc 120 at `call`, as on HEAD; `rr-x2-lower-z1` the same rc 120 as on HEAD. Earlier
witnesses: the X2, X3, X4 (`cmpsep`) and Y1 drafts, `pk-retry`, and `f5-t2` through the forged
library SAME x3.

**Witness draft** `residual-fix/z1-draft/`: `pk.cls` is the finding's shape widened by one line per
reader (`::options digits 12`, `::routine pkr public`, an unattached `::method um`, `::resource
res`, `::class Re`, a method bound to `RegExp_Parse`, one on `NoSuchEntry`); `pk2.cls` translates
(`::routine pkr2 public`, `::class Re2`, a method bound to `RegExp_Pos`) and is the paired success;
`rf.cls` is `call pkr` / `return 1`. Each raise is trapped inside a helper, so the traceback gap
above does not enter the witness. Oracle rc 0, 19 lines. On `bins/rexx-run-y2`
(`p/z1-witness-before`): lines 3-9 and 11-12 differ (`a Routine` x3, `routines over 1`, `a
Method`, `resource line`, `a Routine`, `pkr ran` twice), which shifts the rest; stderr and rc SAME.
On `bins/rexx-run-z1` (`p/z1-witness-after`) SAME x3.

**Mutant predictions (written before building them), refined to the witness's lines.** Lines are
numbered from `pk.cls raised 90.998` as 1.
* M-Z1a (the routine arm also writes `routines` and `package_public_routines` straight into the
  interpreter, as before the fix): stdout DIFF from line 3: `findRoutine a Routine`,
  `routines[PKR] a Routine`, `publicRoutines[PKR] a Routine`, `routines over 1`; lines 7-10 as the
  oracle's; line 11 becomes `pkr ran` / `Package~new context answered` and line 12 `pkr ran` /
  `newFile context answered 1`, shifting the `pk2.cls` block by two lines, whose own content is
  unchanged. stderr and rc SAME.
* M-Z1b (`build_package_string_table` and `prolog` no longer consult `untranslated`): stdout DIFF
  on exactly lines 7, 8, 9 (`a Method`, `resource line`, `a Routine`). stderr and rc SAME.

**Mutant results (run; each built with `--profile mutation`, the worktree restored from
`residual-fix/z1-backup/` afterwards, `cmp` clean).**
* M-Z1a (`diff` shown: the routine arm writing both maps into the interpreter again, beside the
  locals; `5affee505bb5c455`), `p/z1-MZ1a`: **prediction partly falsified.** Lines 1-10 SAME,
  including `findRoutine`, `routines[PKR]`, `publicRoutines[PKR]` and `routines over 0`; line 11
  becomes `pkr ran` / `Package~new context answered` and line 12 `pkr ran` / `newFile context
  answered 1`, shifting the `pk2.cls` block by two lines with its content unchanged; stderr and rc
  SAME. Why the `~package` readers stayed right (read, then matched against the run): `routine_entries`
  and `package_find_routine` turn each record into an object through `Interp::routine_object`,
  which materialises the program's `.ROUTINES` table (`dispatch/package.rs`'s comment in
  `routine_entries` says so), and that table is now empty for an untranslated program. So those
  readers are guarded by the second part of the fix, and the parent walk (`installed_routine`,
  which reads the records alone) only by the first.
* M-Z1b (`diff` shown: both `untranslated` checks made unsatisfiable with `&& program.0 ==
  usize::MAX`; `e29bdb347ba98d7e`), `p/z1-MZ1b`: stdout DIFF on exactly lines 7, 8, 9 (`a Method`,
  `resource line`, `a Routine`); stderr and rc SAME. Confirmed exactly.

So each part reddens the witness on lines the other does not: the parent walk for the first, the
parsed tables and the prolog for the second.

**Minor 3 (same commit).** `LibraryCodeKey`'s doc said `resolveRoutine` finds a routine without
regard to case; it now says by its exact spelling or else by the first entry matching without
regard to case, as `Library::routine` finds it (Y2).

**Witness.** `corpus/lang/library_package_discarded.rex` (the draft, byte for byte) + `.env`
(`LD_LIBRARY_PATH={oraclelib}`) + `.d/pk.cls`, `.d/pk2.cls`, `.d/rf.cls`; a row in `phase-8.txt`;
companion regenerated (count 62, lines `cmp` identical). On a release build of the tree as it is
committed (`bins/rexx-run-z1b`, `b8106b3fcf3f238b`, after the doc edits): the draft SAME x3.

**Fast checks (uncommitted tree, committed unchanged).** `cargo fmt --all --check` exit 0; `cargo
clippy -j 4 --workspace --all-targets -- -D warnings` exit 0 (`z1-clippy.txt`).

**Gates (committed tree).** `residual-fix/gates.sh z1` at `362f5045394214ee43bfb71f1bd6dac5642cef5f`,
dirty 0 before and after (`z1-status.txt`): rexx-exec release exit 101, 1524 passed / 5 failed,
the same G3 five; gated corpus exit 0, `545 of 545 matching`, both sidecar tests ok; sourceline
exit 0. Then `cargo test -j 4 --release -p rexx-exec --lib dispatch::library` exit 0, 11 passed
(`z1-lib.txt`).

**Z1 commit: `362f50453`.**

**Not done for Z1.** Not probed: `~addRoutine` or `~addPackage` on the discarded package object
(both write into the tables this commit empties); identity (`==`) between a discarded package and
its re-translation; a discarded package reached through `newFile` or `.Package~new` of a source that
raised, rather than through a required file (the first walk is the same code for all three, read,
not run). Class resolution through a package parent (the re-review's finding 4) is left as ruled.
