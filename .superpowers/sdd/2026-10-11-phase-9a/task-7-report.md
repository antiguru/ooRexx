# Phase 9a Task 7 report: package setting writes

Base `e0f3ee495`. Status: DONE_WITH_CONCERNS. One commit (code, witnesses, tables, this report).

Probes ran from fresh empty directories under `/tmp/claude-1000/p9/t7-probe*`, `t7-w1`, `t7-w2`,
`t7-crash`, the oracle under the standard wrapper with `timeout -k 5 20`, the crate's `rexx-run`
(release, `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t7-target`) under `memcap 2G` and `timeout 20`.
`oracle-crashes.txt` was read first; entry 9 (`::OPTIONS TRACE ?<letter>`) is the nearby hazard and
no probe writes a `?` trace setting. stdout, stderr and status were compared separately, and every
witness's stdout was read on both sides.

## Oracle probes

* `t7-probe1`: `.context~package~options(name, value)` for `D`, `FO`, `FU`, `T`, `NU`, `E`, `NOV`,
  `I`, `X`, `R`, `S`, `A`, with `::routine r1` called between writes. rc 0. A write answers the
  previous value (`9 12 9` for `D 12` read back, the running activation still at 9); a routine
  called afterwards starts from the written settings (`r1 12 ENGINEERING 3`); `NU I` then `numeric
  digits 20` makes `r1` answer `20 SCIENTIFIC 0`; `I` answers the settings before the first write;
  `R` answers the current settings and restores the initial ones; `S` replaces the settings from an
  `::OPTIONS` directive and `X` stays empty. Crate after: identical on all three descriptors.
* `t7-probe2`: `.Package~defaultOptions`. `D` answers the previous override; with the count not
  zero, the `D` write's own compile consumes one (`C after D 1` from 2) and the new override is the
  old one plus what the string names (`D2` keeps `DIGITS 5 FORM ENGINEERING`, adds `TRACE RESULTS`).
  A called file, `Routine~new`, `Package~new` and `Method~new` each consume one; `INTERPRET` does
  not. Crate after: stdout identical; stderr differs only in two pre-existing trace divergences
  (concerns 1 and 2).
* `t7-probe3`: the write errors. `D -1`/`D 0` 93.907, `D abc` 93.905, `FU -1`/`FU abc` 93.906,
  `FU 19` 26.900 `NUMERIC FUZZ value ("19") must be smaller than NUMERIC DIGITS ("9")`, `D 4` over
  fuzz 5 33.1, `FO`/`NU`/`E`/`P`/`A` with a bad letter 93.914 naming each arm's list, `T wrong`
  24.1 found `w`, `S wrong` 93.900 `argument is not an ::OPTIONS directive`, `;` 93.900 `... found:
  ";" (end of clause)`, `S '::options digits abc'` 26.5, `D '::routine foo'` accepted, leading
  blanks accepted, `D ':'` 35.1, `D .nil` 88.909, `D 1.0` and `D ' 7 '` accepted, `C 1.5` 93.905.
  Crate after: identical except the `c~message` of the two parse errors (concern 3).
* `t7-probe4`: the REXX package. `.object~package~options('D', 12)` answers `9` and reads back
  `12`; repeated calls of one file consume a count each (`-1` to `-3` over two calls). Crate after:
  identical.
* `t7-crash`: `.object~package~options('T', 'r')` is SIGSEGV rc 139, 3 runs of 3; recorded as entry
  36 of `rust/corpus/oracle-crashes.txt`. The crate stores the setting.
* `t7-probe6`: trace letters `E`, `F`, `C`, `O`, `N` print no `>I>`/`<I<` lines for a routine; `L`
  does. The witnesses use these to witness a trace override without the pre-existing divergences.

C++ read: `PackageClass::options` and `clzOptions`, `overridePackageSettings` (only the `PROLOG`
flag guards the prologue setting, so a package naming `NOPROLOG` takes an override), and
`setPackageSettings`, which compiles the directive with `createRoutine` and so runs
`installPackage`'s override. `LanguageParser::generateProgram` and `installPackage` are the two
override sites: every compile of a file, a loaded package, `Routine~new`, `Method~new`,
`Package~new` and a `DEFINE` from source.

## Implemented

* `options.rs`: `OptionName` (the letter switch both reads and writes use, replacing
  `OptionQuery`), `PackageOptions::value_of`, `set_by_letter`, `take_defaults` (the override),
  `take_settings` (`S`, keeping the explicit flags), and `OptionWrites` on `Interp` holding the REXX
  package's settings, each package's initial settings (entered at its first change), the override
  settings and the count. `Interp::package_settings_mut`, `arm_option_latches` (a write escalating
  `NOSTRING` or `LOSTDIGITS` arms the latch `install_directives` arms),
  `override_package_settings`, and `settings_from_directive` (the 93.900 checks, then the directive
  compiled through `install_executable` as `PackageSettingsFromOptionsDirectiveString`).
* `::OPTIONS PROLOG` and `NOPROLOG` now clear each other's explicit flag, as
  `DirectiveParser.cpp:1315` does; before, `::options noprolog prolog` rendered `X` as `NOPROLOG
  PROLOG` (witnessed, and its mutation reverts the witness line).
* `install.rs`: `override_package_settings` runs where the first walk ends (after the
  translation-time refusals, before any `::REQUIRES` loads) and in `record_compiled_body` and
  `record_compiled_routine`, the directive-less compiles.
* `dispatch/package.rs`: `options` and `default_options` writes; `R` now restores. The previous value
  `D` answers is built after the compile.
* `rexx-num`: `Settings::from_parts`, an unchecked constructor (`setDigits`/`setFuzz` store without
  comparing the pair).
* `error.rs`: `Raised::whole_number_user_defined` (26.900).
* Deleted: `Loud::package_option_write`; `dispatch/array.rs`'s `is_whole_method_argument`, unused
  after it. `tests/phase_ten_reasons.rs` sampled the deleted literal as a lib.rs production literal
  and now samples `"a write to a package's .ROUTINES table"`.

Hot path: nothing per clause or per call changed. `start_from_package` reads the same map; the
override check is one branch per package compile.

## Witnesses (`rust/corpus/phase-9.txt`)

* `lang/package_option_writes.rex`: every option name with a value, the previous value and the
  read-back, the effect on `::routine calc` started afterwards (digits, form, fuzz, trace,
  numeric inherit), `NOVALUE`/`LOSTDIGITS` escalation by a write and its removal by `A C`, `I`, `X`,
  `R`, `S` (including a directive that is not `::OPTIONS`), the REXX package's `D` write, the
  PROLOG/NOPROLOG naming order, and every write error above with its message. Oracle rc 0, crate rc
  0, stdout identical, stderr identical.
* `lang/package_default_options.rex` with `package_default_options_called.cls`,
  `_req.cls`, `_noprolog.cls`: `D` and `C` read and written; the `'C'` count of 1 that `rexx -o`
  sets (the next called file takes digits 6 / ENGINEERING / its own fuzz 2 and trace N; the
  following call does not); count -1 consumed by `Routine~new` with and without directives,
  `Method~new`, `DEFINE` from source, `Package~new`, `loadPackage`, the `D` write, and not by
  `INTERPRET`; `I` of an overridden package is the pre-override settings; explicit `DIGITS`/`TRACE`
  kept; the override's `NOPROLOG` suppressing a loaded package's prologue; an explicit `NOPROLOG`
  not keeping out the override's `PROLOG`; `NOVALUE`/`LOSTDIGITS` escalation by the override; the
  errors. Oracle rc 0, crate rc 0, stdout identical, stderr identical (empty).
* `sourceline_oracle` expectations for both `.rex` files, from the module comment's `.Package~new`
  driver against the oracle.

Mutations (each reversed from a copy, confirmed with `git diff --stat`): latches not armed by a
write turns `lost() -> 98.972` into `1.2346E+6` in witness 1; dropping the PROLOG/NOPROLOG clearing
changes both naming lines. The latch arm inside the override was dropped instead of kept: witness 2
stayed identical without it, because the override's settings come from an `::OPTIONS` install that
arms the latches itself (comment at the site).

## Refusal tables

`REXX_REFUSAL_SITES_REFRESH=1 cargo test -p rexx-exec --test refusal_sites` re-derived
`refusal-sites.tsv`: the `package_option_write` row is gone, `whole_number_user_defined` is new
(`agrees yes 26.900`, witness `.context~package~options('FUZZ', 9)`), `method_user_defined`'s
surface became `body+send` (it is now also raised from `options.rs`), and the rest are line-number
shifts in `lib.rs` and `error.rs`. `refusal-dispositions.tsv` had no row for it. `closed_phases`,
`owners`, `phase_ten_reasons` green.

## Package_Options

Before (Task 3's baseline, `docs/superpowers/records/2026-10-11-phase-9a/suite-baseline/table.tsv`):
crate 54 started, 0 passed, 54 blocked by `a package settings write`, 55 runs. After
(`/tmp/claude-1000/p9/t7-target/suite/table.tsv`): oracle 54/54 passed, crate 54/54 passed in one
run, verdict `green`, `oracle-differences.tsv` empty. No test the oracle passes fails, so nothing
needs `suite-9d-set.txt`.

## Checks (each unpiped, on the final tree)

| command | exit |
|---|---|
| `cargo fmt --all --check` | 0 |
| `cargo clippy -j 4 -p rexx-exec -p rexx-num --all-targets -- -D warnings` | 0 |
| `REXX_CORPUS_GATE=1 cargo test --release -p rexx-exec --test corpus --test refusal_sites --test closed_phases --test owners --test phase_ten_reasons` | 0 (`1001 of 1001 matching`) |
| `REXX_SUITE_GATE=1 REXX_SUITE_GROUPS=base/class/Package_Options cargo test --release -p rexx-exec --test suite` | 0 |
| `cargo test --release -p rexx-exec --test collect_stress --test trace_oracle --test ir_recorded --test coverage` | 0 |
| `cargo test --release -p rexx-parse --test sourceline_oracle` | 0 |
| `cargo test --release -p rexx-exec --lib options` | 0 |

All under `memcap 8G`, `-j 4`, `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=/tmp/claude-1000/p9/t7-target`.
The harness, sourceline and `--lib` runs were taken before the last edit (the `D` answer built after
the compile, the dropped override latch arm, a citation); the corpus, tables, suite, fmt and clippy
rows were re-run after it. Together these took longer than the 5-minute per-task budget.

## Concerns

1. Pre-existing: a file reached by an external `CALL` under a package `TRACE R` prints no
   `>I>`/`<I<` routine lines here where the oracle does (`t7-probe5`, no write involved).
2. Pre-existing: a `Routine~new` routine's `>I>` line names the caller's package here and the
   routine's own name (`"x"`) in the oracle (`t7-probe5`).
3. Pre-existing: a parse error raised by `Routine~new` source (and so by `S`/`D` directive text)
   carries `&1` unsubstituted in `CONDITION('O')~message` (`26.5 ... found "&1"` against `found
   "ABC"`); code and error text agree. The witnesses print no such message.
4. The override state is per `Interp`; the C++ keeps it in `PackageClass` statics, process-wide.
   The same for one interpreter per process, which is what `rexx -o` uses.
5. `rexx-run` has no `-o`/`-od` switch; the witness drives the same two `defaultOptions` sends the
   C++ launcher makes.
