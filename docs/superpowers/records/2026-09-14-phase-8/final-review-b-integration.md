# Final review, slice B: how rexx-exec consumes the boundary, and the seams between tasks

Range: `659312de0..e64202ae7`. Reviewer is read-only; probes under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/final-b/`.

**Counts: Critical 0, Important 5 (B1, B2, B5, B6, B7), Minor 7 (B3, B4, B8, B9, B10, B11, B12).**
Findings are numbered in the order they were established, not by section; every one was run unless
its text says inferred.

Probe harness: `final-b/cmp.sh DIR FILE` runs the oracle under the standard wrapper and
`target-head/release/rexx-run` (built from the worktree at `e64202ae7`, `CARGO_TARGET_DIR` in
scratch) in the same fresh directory, stdout/stderr/rc captured to separate files. Every probe
below has its own directory under `final-b/p/`.

## 1. Cross-task interfaces

### B1. Important -- a CSTRING / RexxStringObject argument with no string value is silently converted (rc 0 where the oracle is 88.909)

`rust/crates/rexx-exec/src/dispatch/library.rs:142-148` (`impl Host for Interp`, `string_value`)
answers `self.required_string_value(object).ok()`. `required_string_value`
(`dispatch.rs:3027`) is the *NOSTRING-resuming* protocol, `RexxObject::requestString`: with
nothing trapped it falls back to sending `STRING` and hands back the default name. The oracle's
native argument conversion is not that protocol: `NativeActivation::cstring`
(`interpreter/execution/NativeActivation.cpp:2028`) and the `RexxStringObject` arm (`:473`) call
`stringArgument` (`runtime/MethodArguments.hpp:136`) -> `RexxInternalObject::requiredString(position)`
(`classes/ObjectClass.cpp:1373`), which sends only `REQUEST('STRING')` and raises
`Error_Invalid_argument_string` (88.909) when that answers `.nil`. No `STRING` send, no NOSTRING.

So Task 5's conversion table (which does map `None` to 88.909) is correct and Task 8's host feeds
it an answer from the wrong protocol. Ran, `final-b/p/p2c`:

```
r = .Re~new('a*b')
say 'parsed' r~doparse(.object~new)
say 'matches an Object:' r~does('an Object') 'aab:' r~does('aab')
```
```
oracle rc 168   stdout empty
       *-* Compiled method "DOPARSE" with scope "RE".
     2 *-* say 'parsed' r~doparse(.object~new)
Error 88 running .../p2c/main.rex:  Invalid argument.
Error 88.909:  Argument 1 must have a string value.
rust   rc 0     stderr empty
parsed 0
matches an Object: 1 aab: 0
```

The extension compiled the regular expression `an Object`. Same silent rc 0 for an object whose
class defines `STRING` and no `MAKESTRING` (`p5`: oracle 88.909 rc 168, rust `parsed 0` /
`after` rc 0), and for `RexxStringObject` through `RegExp_Match` in a required package (`p2f`).
Where a `NOSTRING` trap is set the Rust side happens to reach 88.909 (`p2d`, both
`SYNTAX trapped 88 88.909`), so whether the answer is right depends on an unrelated trap.

The same line also swallows a raise: an object whose `MAKESTRING` does `raise syntax 40.1`
(`p3`) is oracle `Error 40 ... line 11` / `40.1` rc 216 and rust `Error 88 ... line 11` / `88.909`
rc 168, with an identical traceback above it. Task 8's report (section 2, open item 6) recorded
this narrowing and dismissed it as "Nothing in reach produces one -- `rxregexp` converts strings
the program already holds". That premise is false: the program chooses the argument, and `p3`
adds a three-line class to the ordinary witness. The silent-conversion half is not recorded anywhere (not in the Task 8 report, not in
the Phase 8 KNOWN GAPS block of `phase-4-exclusions.txt`).

No witness sends a non-string argument: every `library_*.rex` passes string literals.

### B2. Important -- 88.909 raised at the native boundary is reported against the running program with a line; the oracle reports the declaring package with no line

`rust/crates/rexx-exec/src/error.rs:1293` `argument_needs_a_string_value` does not set
`delivery.lineless`, while `missing_native_argument` (`:324-330`) and
`too_many_external_arguments` (`:341-345`) do. `c9616b3d0`'s package-blame rule
(`error.rs:1926-1937`) keys on `lineless`, so it covers two of the three boundary refusals that
`dispatch/library.rs:120-135` maps. Ran, `p2e` (NOSTRING trapped so the Rust side reaches the
raise at all, method declared in `re.cls`, sent from `main.rex`):

```
oracle rc 168: Error 88 running .../p2e/re.cls:  Invalid argument.
rust   rc 168: Error 88 running .../p2e/main.rex line 3:  Invalid argument.
```

Traceback lines and the 88.909 line are identical. The oracle's same-file form (`p2`) is also
lineless (`Error 88 running .../p2/main.rex:  Invalid argument.`), so this is visible without a
second file. `Refused::Signature` -> `incorrect_method_signature` (93.968, `error.rs:335`)
is not lineless either and is raised from the same `processArguments` frame in the oracle
(`NativeActivation::reportSignatureError`, `NativeActivation.cpp:190`); inferred, not run -- no library in reach declares a signature
the boundary refuses. 88.922 in a required package was run (`p1`) and matches byte for byte.

### B5. Important -- `Method~package` of a library-backed method answers `REXX`; the range turned two loud refusals into this silent answer, and the fact that fixes it is already recorded by `c9616b3d0`

`Interp::installed_executable_source` (`rust/crates/rexx-exec/src/lib.rs:4444`) answers
`ExecutableSource::Native` for any method not in `method_bodies`/`generated_methods`, which its doc
says covers "an `EXTERNAL` binding", and `ExecutableSource::Native`'s package is `REXX`.
`native_load_external` (`dispatch.rs:8410`) calls `record_native_executable` for the object it
answers, same result. At `659312de0` neither was reachable for a non-`REXX` library:
`directive_gap` refused `"::METHOD EXTERNAL naming a library other than REXX"` (old `lib.rs:804`)
and `native_load_external` refused `"loadExternalMethod naming a library other than REXX"` (old
`dispatch.rs:8344`). Task 8 removed both refusals and did not teach the reader. Task 10's
`c9616b3d0` then added `Interp::external_packages`, keyed by the same `MethodId`, holding exactly
"which package declared each `EXTERNAL` binding" -- consumed only by blame
(`external_package_path`, `lib.rs:3016`). Ran, `final-b/p/t5b`:

```
m = .Re~method('DOES')                      -- ::method does external "LIBRARY rxregexp RegExp_Match"
say 'pkg' m~package~name
l = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Match')
say 'loaded name' l~package~name
```
```
oracle rc 0: pkg <p/t5b/main.rex> ... loaded name <p/t5b/main.rex>
rust   rc 0: pkg REXX           ... loaded name REXX
```

`~source~items` (0) and `~isGuarded` (1) agree. The `LIBRARY REXX` form (`::method sep external
"LIBRARY REXX file_separator"`, same probe) answers `REXX` too where the oracle names `main.rex`;
that half predates the range (run on the base binary, `p/t5c`: `sep pkg REXX` there too), so the new
part is the reach, not the mechanism. `make_method_private` (`lib.rs:4611`) derives a package from the same answer; `setPrivate`
and `PACKAGE`/`PRIVATE` directive access on library methods were run (`p/t7`, required package,
inside/outside/same-package routine) and agree, so no access defect was found from it.

### B7. Important -- a library's routines are made callable only by `::REQUIRES ... LIBRARY`; every other load site leaves them at 43.1, the silent wrong answer `3245708c0` closed for one site

On the oracle, every load site measured below makes the library's routines callable by name from
any package, including one that loaded before the library did. `3245708c0` recorded that a library routine answering 43.1 is "a silent wrong answer
rather than a missing one" and fixed it by filling `Interp::library_routines` inside the
`requires.library` arm of `load_required_packages` (`lib.rs`, the `loaded.routines()` loop after
`self.require_library(&requires.name)`). `resolve_library` is the one resolution path every loader
goes through, but the registration is in one caller, so `Package~loadLibrary`
(`dispatch/package.rs` `load_library`), `::ROUTINE ... EXTERNAL "LIBRARY <lib> ..."`
(`resolve_directive_library`), `.Routine~loadExternalRoutine` and `.Method~loadExternalMethod`
(`dispatch.rs` `native_load_external`) load the library and register nothing; so do `::METHOD` and
`::ATTRIBUTE ... EXTERNAL` (`resolve_directive_library` again). Ran, `rxmath` (which exports
routines; loadable on both sides) for the sites below; the `loadExternalMethod` and
`::METHOD`/`::ATTRIBUTE` sites are inferred from the code only, because no library in reach exports
both a method the site can bind and a routine to call afterwards:

| probe | oracle | head | base `659312de0` |
|---|---|---|---|
| `r1`: `say .context~package~loadLibrary('rxmath')` / `say RxCalcSqrt(16)` | `load 1` / `sqrt 4`, rc 0 | `load 1` then `Error 43.1: Could not find routine "RXCALCSQRT"`, rc 213 | rc 120, "a native library load is not implemented (Phase 8)" |
| `r5`: the same `loadLibrary` inside a routine of a required package, call from main | `main 4`, rc 0 | 43.1, rc 213 | not run |
| `r6`: `::routine sq external "LIBRARY rxmath RxCalcSqrt"`, main calls `RxCalcSqrt(16)` | `main 4`, rc 0 | 43.1, rc 213 | rc 120, "::ROUTINE EXTERNAL is not implemented (Phase 8)" |
| `r7`: `.Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')`, then `RxCalcSqrt(16)` | `load Routine` / `main 4`, rc 0 | `load Routine` then 43.1, rc 213 | rc 120, "loadExternalRoutine is not implemented (Phase 8)" |
| `r2`: `::requires 'rxmath' LIBRARY` (the fixed site) | `sqrt 4` | loud rc 120 naming Phase 8 | not run |
| `r4` (control): `loadExternalMethod` on `rxregexp`, which exports no routines, then `RxCalcSqrt(16)` | 43.1 rc 213 | identical | not run |

So three loud refusals at the base became 43.1 in this range. The surface plan's Task 1
(`docs/superpowers/plans/2026-09-14-phase-8-surface.md:71`) scopes the routine half as "`::ROUTINE
EXTERNAL` and a `::REQUIRES ... LIBRARY`'s routines install and then refuse loudly at the call",
which inherits the same one-site premise: building the protocol there leaves `r1`/`r5`/`r6`/`r7`
at 43.1.

Related, loud: the `Method`/`Routine` object `loadExternalMethod`/`loadExternalRoutine` answers for
a library is `native_instance` plus `record_native_executable` (`dispatch.rs:8409-8410`), a shell.
`r8` (`r~call(16)`) is rc 120 "a routine whose body this crate does not hold is not implemented
(Phase 5)" against the oracle's `call 4`; `r10` (`.K~define` of both loaded methods, then `~new`)
is rc 120 "method "INIT" of class "K" is not implemented (Phase 5)" against `defined 1`. At the base
both were refusals naming Phase 8 at the load (`r7` run on the base binary for the routine; the
method form read from the old `dispatch.rs:8344`); now the load answers and the use refuses naming
**Phase 5, which is closed**. `define` of an `EXTERNAL`-installed method (`t6`, `LIBRARY REXX`) is
the same refusal on head (not run on the base; the `define` path is not in this range's diff), so
the Phase 5 wording looks inherited, but the library reach
into it is new and no open phase's plan names it.

## 2. Two sends, not one

Run and byte-identical on three descriptors (so no finding beyond B3): repeated sends to one
instance, a second instance, a send to a survivor after another instance was collected by
`gc 'force'`, forty create/match/drop cycles with a collection every ten, a fresh instance after
all of that, and re-parsing an existing automaton (`p/t1`); the real `rxregexp.cls` (a byte copy in
the probe directory) required twice by the program and once more through a second package that
declares its own class on the same library, instances of both classes, and a raise through its
`NEW CLASS` wrapper followed by a second, untrapped raise (`p/t2`); the same library named by a
package with `::requires "rxregexp" library` and by the program with `::requires 'rxregexp'
LIBRARY`, plus eight other spellings through `loadLibrary`, instances of both classes crossed
(`p/sp`); nested library calls inside an outer call's argument conversion (`p/n1`). A load that
failed once and is asked again is B3.

### B3. Minor -- a failed library load is held; the oracle forgets it and retries, so a second ask can differ

`rust/crates/rexx-exec/src/lib.rs:1756-1759` documents `Interp::libraries` as
"`PackageManager::packages` (`interpreter/package/PackageManager.cpp:233`). A miss is held as well
as a hit, so a name that failed once fails the same way every time", and
`dispatch/library.rs:309` `a_name_that_resolves_to_nothing_is_held_as_a_miss` pins that. The
oracle does the opposite: `PackageManager::loadLibrary` puts the new package in the table and, when
`load()` fails, `packages->remove(name); return OREF_NULL;` (`PackageManager.cpp:243`). Line 233 is
the `get`, not a hold. Ran, `final-b/p/miss`, both sides started with
`LD_LIBRARY_PATH=<build/lib>:<p/miss/libs>` (libs empty at start):

```
say 'first' .context~package~loadLibrary('yyregexp')
address system 'cp <final-b/libs/libzzregexp.so> <p/miss/libs/libyyregexp.so>'
say 'second' .context~package~loadLibrary('yyregexp')
```
```
oracle rc 0: first 0 / second 1
rust   rc 0: first 0 / second 0
```

(`libzzregexp.so` is a byte copy of `build/lib/librxregexp.so.4` in scratch; `build/` untouched.)
Silent, but it needs the file system to change mid-run. The part worth fixing regardless is that the
comment attributes the rule to the oracle; ruling 24's type-level guarantee is about a *loaded*
library outliving its objects, and nothing needs a `Missing` to be permanent. The same held miss
also serves `::REQUIRES ... LIBRARY` and `EXTERNAL` in a package loaded later at run time
(inferred from `require_library` sharing `resolve_library`; not run).

## 3. Error attribution

A second binary, `target-base/release/rexx-run`, is built from `git archive 659312de0 rust` (with
`interpreter/`, `build/` etc. symlinked beside it read-only), so "before the range" below is run,
not read.

### B6. Important -- every library load failure in a required package names the running program; the helper that names the package exists and Task 8 used it for only one of its two call sites

`Interp::resolve_directive_library` (`rust/crates/rexx-exec/src/lib.rs:4520`; its
`self.blame_directive(program, directive)` calls are at `:4539` and `:4554`) blames with
`blame_directive` (`lib.rs:4739`), which records a `FailureSite::Clause` and so reports the running
program's path. The same commit's `::REQUIRES ... LIBRARY` arm in `load_required_packages` uses
`blame_directive_in(id, program, directive)` (`lib.rs:4750`), which looks up `required_paths` and
records `FailureSite::Named`. `resolve_directive_library` is not passed the `ProgramId`; its only
caller, `install_directives` (`lib.rs:2886`), has it as `id` (`lib.rs:2657`). Ran, `final-b/p/a1..a5`: a one-line `main.rex` (`say 'main ran'` / `::requires 'pk.cls'`)
and a `pk.cls` whose third line is the directive:

| `pk.cls` line 3 | oracle `Error N running` | head | base `659312de0` |
|---|---|---|---|
| `::method x external "LIBRARY zorkolib z"` | `pk.cls line 3`, 98.903, rc 158 | `main.rex line 3`, rc 158 | loud rc 120 (Phase 8) |
| `::method x external "LIBRARY rxregexp NoSuchEntry"` | `pk.cls line 3`, 90.998, rc 166 | `main.rex line 3`, rc 166 | loud rc 120 (Phase 8) |
| `::routine x external "LIBRARY rxregexp NoSuchRoutine"` | `pk.cls line 3`, 90.999, rc 166 | `main.rex line 3`, rc 166 | not run |
| `::attribute at external "LIBRARY rxregexp NoSuchGet"` | `pk.cls line 3`, 90.998 `GETNoSuchGet`, rc 166 | `main.rex line 3`, rc 166 | not run |
| `::requires 'zorkolib' LIBRARY` | `pk.cls line 3`, 98.903, rc 158 | **identical** | not run |

Traceback lines (`3 *-* <directive>` / `2 *-* ::requires 'pk.cls'`) and the `Error N.M` line agree
in every row; only the `running` line differs. The Phase 8 KNOWN GAPS block
(`docs/superpowers/plans/phase-4-exclusions.txt`, "THREE TRACE AND BLAME DIVERGENCES, ALL PREDATING
PHASE 8") records "a directive-time error in a required package names the program" and says it
"belongs to no phase in particular" because `::class A subclass NoSuchClassHere` reproduces it
(`a6` reproduces that too). That is true of the mechanism and false of these rows: at `659312de0`
the first two were loud refusals naming Phase 8, so this range converted a refusal into the wrong
file name, and the last row shows the fix is the helper already beside it. The realistic case is
exactly this layout -- `rxregexp.cls` is a required package declaring library methods -- so a
missing `librxregexp.so` names the user's program.

The Task 8 witnesses for these numbers (`library_method_entry_missing.rex`,
`library_routine_entry_missing.rex`, `library_attribute_entry_missing.rex`,
`library_requires_missing.rex`) all declare the directive in the program being run, the same blind
spot Task 10 found for the boundary raise. `c9616b3d0` closed that one for the send and added
`library_method_package_blame.rex`; the install-time half has no two-file witness.

### B12. Minor -- `c9616b3d0`'s package attribution reaches the rendered report and not the trapped condition object

Ran, `final-b/p/co` (the `library_method_package_blame.d/re.cls` fixture, `signal on syntax` around
`r~doparse()`, then around `.Re~new('[')`):

```
                      oracle                        rust
88.901  ~position     The NIL object                3
        ~program      <p/co/re.cls>                 <p/co/main.rex>
        ~traceback    [ '     3 *-* say r~doparse()', .nil ]      [ .nil ]
38.0    ~position     13                            13
        ~program      <p/co/main.rex>               <p/co/main.rex>
        ~traceback    [ 'Compiled method "NEW" ...', '    13 *-* x = ...', .nil ]   [ .nil ]
```

Untrapped, both programs' stderr is byte-identical (the committed witnesses). So the lineless /
declaring-package rule lives in `Raised`'s rendering (`error.rs:1926-1937`, `:1957`) and not in the
values a trap reads. **Predates the range as a mechanism:** the `LIBRARY REXX` form (`p/co2`,
`.K~sep(1)` from a required `k.cls`) gives the same three differences on the base binary, and a
plain Rexx method raising 93.901 in a required package (`p/co4`) gives wrong `~position` (2 against
3), wrong `~program` and a one-item traceback too. Recorded here because it is where the
attribution work of this range stops, and no KNOWN GAPS entry names the condition object
(`grep -i traceback`/`position` over `phase-4-exclusions.txt` finds only unrelated entries).

### Error attribution checked and agreeing

Run, all byte-identical on three descriptors: 88.922 from a required package (`p1`); 38.0 raised
by `RegExp_Init` through the real `rxregexp.cls`'s `NEW CLASS` / `Signal On Syntax` /
`Forward Class(Super)` / `Raise Propagate` with the file required twice directly and once through
another package (`t2`, then a second raise, 93 from `'BOGUS'`, untrapped: `running
<t2/rxregexp.cls> line 42`, rc 163); `PACKAGE` and `PRIVATE` access on library methods from a
required package, and after `~setPrivate` (`t7`: 97.3, 97.2).

## 4. Process-global state and descriptor writes

`git diff 659312de0..e64202ae7 -- rust/crates | grep '^+'` for `set_var`, `remove_var`,
`set_current_dir`, `std::io::stdout/stderr`, `stdin()`, `from_raw_fd`, `libc::`, `eprint`,
`print`, `std::env::` finds no process-global write in interpreter code. The hits are
`std::env::consts::DLL_SUFFIX` (`rexx-api/src/load.rs:317`), a test re-executing its own binary
under `CALL_A_STUB` (rexx-api, slice A), and `REXX_REFUSAL_SITES_REFRESH` read in a test. The
`dlopen` flag is `libloading::Library::new`, `RTLD_LAZY | RTLD_LOCAL`, the oracle's own
(`SysLibrary.cpp:90`). No finding against the rule itself.

Another interpreter instance cannot see a mutation: each `Interp` owns its shadow environment, and
`a_library_named_twice_is_opened_once`'s control opens a second interpreter and gets a different
`Rc`. `librxregexp.so`'s SONAME is `librxregexp.so.4` (readelf), so an earlier in-process
`dlopen` of `<dir>/librxregexp.so` does not satisfy a later bare `dlopen("librxregexp.so")` by
name match, and `the_search_path_is_what_makes_the_name_resolve` is not order-dependent on that
account (inferred from glibc's name matching, not run under parallel tests).

### B4. Minor -- the library search reads `LD_LIBRARY_PATH` live, so the *same* interpreter's program can widen it at run time; the oracle's loader cannot

`rust/crates/rexx-exec/src/lib.rs` `library_search_path` reads `self.env_get(b"LD_LIBRARY_PATH")`
on every resolve. Its doc gives the reason as "The process loader read that variable once at
start-up, so a value this interpreter was handed afterwards reaches the search no other way." The
shadow environment is also what `VALUE(..., 'ENVIRONMENT')` writes, so a program that sets the
variable gets a search the oracle's start-up-cached loader never performs. Ran, `final-b/p/env2`,
both sides started with `LD_LIBRARY_PATH=<build/lib>`:

```
call value 'LD_LIBRARY_PATH', '<final-b/libs>', 'ENVIRONMENT'
say 'after' .context~package~loadLibrary('zzregexp')
m = .Method~loadExternalMethod('m', 'LIBRARY zzregexp RegExp_Parse')
say 'method' (m \= .nil)
```
```
oracle rc 0: after 0 / method 0
rust   rc 0: after 1 / method 1
```

What the doc describes (a value the embedder hands in through `Invocation::with_environment`) is a
start-up value; taking it once at construction would model the loader, where reading it per resolve
models a loader that does not exist. Silent, narrow reach.

## 5. Instruments that cannot fail

Runner: `final-b/witness.sh BIN LABEL` runs every program `corpus/phase-8.txt` lists, with its
`.d/` fixtures copied into a fresh directory, on the oracle (cached) and on `BIN` as a spawned
`rexx-run` with `LD_LIBRARY_PATH=<oracle build/lib>`, paths in the oracle's stderr rewritten to the
Rust run directory. Not the in-process harness, so it says nothing about the `.env` sidecar path.
Controls for the runner itself: the head binary is SAME on all fifteen programs; the base binary
(`659312de0`) is DIFF on all fifteen. Mutants are built from `git archive e64202ae7 rust` in
`final-b/mut/` with their own `CARGO_TARGET_DIR`; the worktree is not edited.

### Predictions, written before any mutant was built

* **M-A** (`error.rs:1933`, `if self.delivery.lineless {` -> `if true {`, so every raise under a
  library level names the declaring package): `library_method_program_blame` reddens (oracle names
  the program, line 1); every other phase-8 witness stays SAME, including
  `library_method_raises`, whose declaring package *is* the program.
* **M-B** (`lib.rs`, drop `self.external_packages.insert(method, program);` from the
  `InstallBody::Library` arm only): `library_method_package_blame` reddens (names `main.rex`);
  `external_method_package_blame` stays SAME (its method is `LIBRARY REXX`, the other arm);
  `library_method_missing_argument` stays SAME (same file).
* **M-C** (`dispatch/library.rs`, delete `self.native_handles.pop();`): the frame and its `owner`
  stay rooted, so a receiver never becomes collectable. `library_uninit_collected` and
  `library_uninit_twice` redden (the `sub before`/`sub after` lines move after `done`);
  `library_uninit_termination` and the rest stay SAME.
* **M-D** (`dispatch/library.rs`, `string_value` answers `Some(object)` with no conversion at all):
  every phase-8 witness stays SAME, because every argument any of them passes is a string literal.
  This is the B1 instrument gap stated as a prediction.
* **M-F** (`lib.rs` `resolve_library`, skip the `self.libraries.get(name)` early return so every
  ask re-opens): every phase-8 witness stays SAME, `library_loads_once` included, whose comment says
  a program naming the library twice "sees one load"; only the unit test
  `a_library_named_twice_is_opened_once` can see it (not run here -- needs the test build).
  **Revised before running, on reading `Libraries::hold`:** `hold` is `entry().or_insert(load)`
  and answers the *held* `Rc`, so under M-F the second `open` happens and its `Library` is dropped,
  and `Rc::ptr_eq(&first, &second)` still holds. New prediction: that unit test stays green too,
  so nothing in the tree sees a second open.

### Results (each mutant rebuilt, binary sha256 prefix recorded, pristine files restored by copy)

| mutant | sha256 prefix | phase-8 witnesses not SAME | prediction |
|---|---|---|---|
| M-A | `fb5d86fd80c1a832` | `library_method_program_blame` (stderr) | confirmed exactly |
| M-B | `a8a159d4249e6bcb` | `library_method_package_blame` (stderr) | confirmed exactly |
| M-C | `7c2e49e9c2dd9025` | `library_uninit_collected`, `library_uninit_twice` (stdout: `sub before`/`sub after` after `done`) | confirmed exactly |
| M-D | `d2a3c46a5fe2bf01` | none | confirmed: no witness sees the host's string conversion |
| M-F | test binary, not a `rexx-run` | corpus not run; `dispatch::library::tests` 8 passed, 0 failed | revised prediction confirmed, see B10 |

So `c9616b3d0`'s pair (program blame and package blame) is a real discriminating pair, and Task 9's
collected/twice witnesses see a leaked native frame. M-D is live, shown off the corpus: under it
`p6` (an argument whose `MAKESTRING` answers the pattern) goes from agreeing to rc 168, and `p2c`
goes from HEAD's rc 0 to the oracle's rc 168 with only B2's `running` line differing. **The
degenerate host that converts nothing is closer to the oracle than the committed one** on B1's
probe, and no witness can tell them apart.

### B10. Minor -- `a_library_named_twice_is_opened_once` cannot fail for the reason its doc gives

`dispatch/library.rs:257-285`. Its doc: "Two `Loaded` answers would look alike whether or not the
second one re-opened the library; the same `Rc` says the `dlopen` and the package read happened
once." M-F (above: `resolve_library`'s held-answer early return disabled with
`.filter(|_| false)`) was built as a test binary and run under gdb with
`dprintf dlopen,"DLOPEN %s\n",(char*)$rdi`:

```
M-F:      DLOPEN <worktree>/build/lib/librxregexp.so  x3   test result: ok. 1 passed
pristine: DLOPEN <worktree>/build/lib/librxregexp.so  x2   test result: ok. 1 passed
```

The third `dlopen` is the re-open the test is named for; `Libraries::hold`'s `or_insert` hands back
the first `Rc` and drops the second `Library`, so `ptr_eq` holds either way. Under M-F all eight
`dispatch::library::tests` pass (`8 passed; 0 failed`), `a_name_that_resolves_to_nothing_is_held_as_a_miss`
included, and M-F's phase-8 corpus prediction (all SAME) follows from the same reasoning but was
not separately run as a corpus binary. Behaviourally benign (the loader refcounts), so the defect
is the claim; an assertion that can see it would count opens rather than compare `Rc`s. Prediction
revised before running (above) and confirmed.

### B8. Minor -- `external_method_package_blame.env` is inert, and the sidecar control cannot see an inert half

`rust/corpus/lang/external_method_package_blame.env` carries the same text as every Phase 8 `.env`
("The oracle's own build of librxregexp.so, which D5's amendment makes the instrument") but the
program is `.K~sep(1)` over `::method sep class external "LIBRARY REXX file_separator"`, which loads
no shared object. Ran `rexx-run` on it with and without `LD_LIBRARY_PATH`: rc 168 both, stderr
identical modulo the run directory. The same comparison on `library_method_package_blame` is rc 168
against rc 158, so the variable is load-bearing there. `a_sidecar_changes_what_one_of_the_interpreters_answers`
(`tests/corpus.rs:833`) compares the whole sidecar against none; this program's `.d/k.cls` moves
both interpreters, so the control passes without asking whether the `.env` half does anything. Its
own doc says deleting a program's only sidecar escapes it; this is the other escape, an inert half
beside a live one.

### B9. Minor -- the in-crate library tests load a different `librxregexp.so` from the oracle's, under a doc that says they load the oracle's

`dispatch/library.rs:236-243` `oracle_library_directory` is documented as "The oracle's own build
directory, whose `librxregexp.so` D5's amendment makes the instrument", and resolves
`CARGO_MANIFEST_DIR/../../../build/lib`, which is this worktree's `build/lib`. The oracle is
`/home/moritz/dev/repos/ooRexx/build` (`tests/support/oracle.rs:57`, and what `{oraclelib}`
expands to at `tests/corpus.rs:237`). They are different binaries: `ls -la` gives
`librxregexp.so.4` 32,392 bytes dated Jul 27 in the worktree (`CMAKE_BUILD_TYPE:STRING=Release`)
against 126,408 bytes dated Aug 5 in the oracle's tree. `extensions/rxregexp/` is identical in the
two source trees (`diff -r`, empty), so no behavioural difference is expected and none was seen,
but "loaded, never rebuilt" is about a file the corpus never loads. The `rexx-api` tests
(`tests/load.rs:30`, `invoke.rs:53`, `context.rs:43`) read the same worktree path; that is slice A's.

### B11. Minor -- three harnesses that now read `phase-8.txt` run its library witnesses down the load-failure path

`collect_stress.rs`, `coverage.rs` and `ir_recorded.rs` each gained `"phase-8.txt"` in
`SUBSET_FILES`. None reads a `.env` sidecar (`grep '\.env\|sidecar\|with_environment'`
finds nothing in the three), and `collect_stress.rs:216-226` and `ir_recorded.rs:1027-1031` run
each program with `Invocation::none()`, whose environment is a copy of the test process's
(`invocation.rs:152-153`). The gate commands set no `LD_LIBRARY_PATH`, and `rexx-run` has no
`RUNPATH` for the oracle's `build/lib`, so the undecorated `dlopen` finds nothing. Ran the
equivalent: `env -u LD_LIBRARY_PATH rexx-run library_method_external.rex` is
`Error 98.903: Unable to load library "rxregexp"` at rc 158 from line 22, before the first clause.
So `ir_recorded`'s "every population runs without a refusal" and `collect_stress`'s plain-vs-stress
comparison (which `dispatch/library.rs:321-325` already says does not reach `phase-8.txt`) would, on
reaching them, compare two identical 98.903 runs. The witnesses whose required package lives in a
`.d/` directory would not find it either: neither `collect_stress.rs` nor `ir_recorded.rs` copies
fixtures (read from the code, not run). The in-crate tests `a_library_call_answers_the_same_under_a_collection_at_every_allocation`
and `the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation` supply the
environment explicitly for two programs, which is the honest part. `coverage.rs`'s use was not
examined.

### Checked and not a finding

* Trace of library-method sends under `trace i` and `trace r` (`p/tr`): byte-identical.
* Library-name spellings (`p/sp`): `rxregexp`, `RXREGEXP`, `RxRegExp`, trailing and leading
  space, `librxregexp`, `rxregexp.so`, the empty string and `rxmath` through `loadLibrary`, then
  the same library bound by a required package (`::requires "rxregexp" library`) and the program
  (`::requires 'rxregexp' LIBRARY`), instances of both classes matching against each other's
  patterns. Byte-identical.

* **GC timing inside a native call.** An argument's `MAKESTRING` that drops the last reference to a
  finalizable object and runs `call gc 'force'` finalises it inside the call on the oracle and at
  termination here (`p/n2`, library method). The same program with a Rexx method in place of the
  library method (`p/n3`) diverges the same way, so it is the licensed collection-timing difference
  and not the native frame: the argument's temps outlive the inner send here.
* Nested native activations (an argument's `MAKESTRING` sending to another library-backed instance
  inside the outer call's argument conversion, `p/n1`); `UNINIT` at termination after `EXIT 3`, an
  untrapped 42.3, an untrapped 38.0 from `RegExp_Init` that left a half-built object, and `EXIT 5`
  from an internal routine (`p/u1`..`u4`); forty create/match/drop cycles with a `gc 'force'` every
  ten, a send to a survivor after another instance was collected, `CSELF` pointer equality across
  instances, and re-parsing an existing automaton (`p/t1`); `~send` and `~sendWith` (`p/t3`;
  `.Message~new` refuses loudly naming Phase 5, not run on the base). All byte-identical.
* `EXIT` inside a `MAKESTRING` reached from the conversion (`p/p4`): both sides treat it as the
  method's return and agree.

## Not reached

Silence below is not coverage; each item was not examined, or examined only as far as stated.

* **`rust/crates/rexx-exec/src/run/tests.rs`** (151 lines added in the range): not read, no mutant.
* **`tests/refusal_sites.rs` and `corpus/refusal-sites.tsv`** (Task 11's refresh mode and the
  regenerated table): not examined; no check that the refresh carries verdict columns forward.
* **`tests/coverage.rs`'s use of `phase-8.txt`**: not examined (B11 covers the other two).
* **`tests/sourceline_oracle/library_*.txt`** companions: not compared against their programs.
* **`corpus/introspection-arity.tsv`** rows and `support/arity.rs`'s `LD_LIBRARY_PATH` change:
  not re-defeated (Task 8's review did); not run.
* **The in-process corpus harness path**: every probe and mutant here ran as a spawned `rexx-run`,
  so the `.env` sidecar route into `Invocation::with_environment` was checked only by B8's
  with/without run, not by running `tests/corpus.rs`. No gate command (G1-G5) was run.
* **Debug build**: all probes ran on release binaries, so no `debug_assert` (including
  `Interp::enter_clause`'s tripwire) was exercised on any probe.
* **98.982** (a package entry asking for a newer interpreter) and **93.968** (a signature the
  boundary refuses): no library in reach produces either; B2's 93.968 remark is inferred.
* **A library-backed `::ATTRIBUTE` accessor that actually runs**: `rxregexp` exports no `GET`/`SET`
  procedures, so only the entry-missing path was probed (`a5`).
* **Concurrency**: `~start`, `REPLY`, `GUARD` on a library method, and a native call on a second
  activity were not probed.
* **An `UNINIT` that raises during the termination sweep**, and a native `UNINIT` on an object whose
  class was defined in a package loaded at run time rather than by `::REQUIRES`: not probed.
* **`Package~loadLibrary` on a package that is not writable** (`writable_package_of` is checked
  after the name conversion and before the load): ordering against the oracle not probed.
* **`library_routine_owner`'s interpreter-wide scope** was probed only where it produces a loud
  refusal (`r2`, `r3`); on the oracle a library loaded inside a required package's routine makes
  its routines callable from the program (`r5`), so the interpreter-wide map is the right shape, but
  no probe checked a name that both a package's own `::ROUTINE` and a loaded library define.
* **The `.Pointer` methods** beyond `~class~id`, `==`/`\==` and `probe` reads in `t1`: `~isNull`,
  `~string` rendering against the oracle's `%p`, and `~objectName=` were not probed (no address is
  stable between runs).
* **`rexx-api` internals** (slice A) and **prose documents** (slice C): out of scope by instruction;
  B7's surface-plan sentence and B9's `rexx-api` test paths are mentioned only where a behaviour
  finding lands on them.
* **Scratch state**: `final-b/mut/` holds the pristine `e64202ae7` copy (restored and verified with
  `cmp` against the worktree after M-F), `final-b/base/` the `659312de0` copy, `final-b/libs/` a byte
  copy of the oracle's `librxregexp.so.4` renamed `libzzregexp.so`. Nothing under the worktree was
  written except this file.
