# Task 22 review: the native entry-point registry

**Spec compliance: PASS.**
**Task quality: CHANGES REQUIRED.**

Four major findings, seven minor. All four majors are false statements shipped in the neighbourhood
of the change; none of them changes what any program does. The implementation itself I could not
falsify: every probe I put against it matched the oracle byte for byte on three descriptors and both
engines, including several the task did not write.

Range reviewed: `7c23bf891..HEAD` (`c59a80672`, `592967125`, `ee083bd7c`). No tree mutation, no
rebuild. All probes from a fresh empty directory with absolute paths.

---

## What I verified myself, and what it showed

### The eager bind and the missing entry point (brief item 2)

All five new `corpus/lang/directive_method_external_*.rex` run against the oracle, three descriptors,
never merged, on `ir` and `tree-walker`:

| program | oracle | ir | tw | |
|---|---|---|---|---|
| `_bind.rex` | rc 0 | rc 0 | rc 0 | match |
| `_missing.rex` | rc 166, stdout empty | 166 | 166 | match |
| `_arguments.rex` | rc 168 | 168 | 168 | match |
| `_duplicate_wins.rex` | rc 157 | 157 | 157 | match |
| `_source_order.rex` | rc 166 | 166 | 166 | match |

`_bind.rex`'s stdout is substantive rather than a bare `rc 0`: `/`, `:`, the default entry name, the
case that differs from the exported one, `self~sep` from inside a body, `hasMethod GETSEP 0` (so the
plain `::METHOD` form mints no accessor pair), `Method`, `1`.

### The 88.922 and the missing source line

Independently reproduced. `say .k~sep(1)` on a class method bound to `file_separator`:

```
       *-* Compiled method "SEP" with scope "K".
     2 *-* say .k~sep(1)
Error 88 running <path>:  Invalid argument.
Error 88.922:  Too many arguments in invocation; 0 expected.
```

rc 168, oracle and both engines byte-identical, **with no ` line 2` on the major line** — which is
what `Delivery::lineless` is for. Two arguments (`.k~psep(1,2)`) gives the same report. An omitted
argument counts: `.k~sep(,1)` is 88.922 on both sides, while `.k~sep(,)` is rc 0 answering `/` on
both sides. `file_separator` and `file_path_separator` are `RexxMethod0`
(`streamLibrary/FileNative.cpp:56`, `:64`), so `Arity::Fixed(0)` is right, and
`SysFileSystem::getSeparator`/`getPathSeparator` return `"/"` and `":"`
(`platform/unix/SysFileSystem.cpp:1358`-`:1361`, `:1369`-`:1372`) — both citations verified with
`/bin/grep -n`, both on the path the doc's own example takes.

The report's counterfactual — "without this the arguments would have been silently ignored and `/`
answered" — is the right shape. `Delivery::lineless` is the rendering half; the raise itself is the
arity arm in `dispatch.rs:1470`-`:1472`. Both are needed for the byte match and both are present.

### The refusal split, and whether any spelling falls on the wrong side

`dispatch::native::method_external` is the only place the boundary is drawn, and it has three
readers, all consistent: `directive_gap` (`lib.rs:1494`), `install_directives`' first walk
(`lib.rs:4522`) and `install_method` (`lib.rs:5408`). Nothing else decides.

I then enumerated the spellings that can reach it. `rexx-parse`'s `decode_external`
(`crates/rexx-parse/src/directive.rs:758`-`:800`) is called with `registered: false` for all three
method/attribute sites and `true` only for `::ROUTINE`, and it admits exactly `LIBRARY <lib>` and
`LIBRARY <lib> <entry>`. So for a `::METHOD` the only variable is `<lib>`, compared exactly against
`REXX`. Probed, fresh dir, oracle plus both engines:

| spelling | oracle | crate |
|---|---|---|
| `'LIBRARY REXX file_separator extra'` | 99.917 rc 157 | rc 120 loud (parse-error-rendering deferral, pre-existing) |
| `'REXX file_separator'` | 99.917 rc 157 | same |
| `'REGISTERED REXX file_separator'` | 99.917 rc 157 | same |
| `"library REXX file_separator"` | rc 0, `/` | **match** |
| `'LIBRARY REXX<TAB>file_separator'` | rc 0, `/` | **match** |
| `abstract external 'LIBRARY REXX ...'` | 25.902 rc 231 | rc 120 loud (pre-existing) |
| `"library rexx file_separator"` | 98.903 rc 158 | rc 120 loud, "naming a library other than REXX" |
| `"LIBRARY REXXUTIL file_separator"` | 98.903 rc 158 | rc 120 loud, same |
| `::method 'a=' class external 'LIBRARY REXX'` | 90.998 rc 166 on `"A="` | **match** |
| `::method sep ... external 'LIBRARY REXX'` + a body | 99.936 rc 157 | rc 120 loud (pre-existing) |
| `::class j subclass k`, `.j~sep` | rc 0, `/` | **match** |
| `trace i` over `say .k~sep` | `>M> "SEP" => "/"`, no `>I>`/`<I<` | **match** |
| `trace r` over `x = .k~sep` | `>>> "/"` | **match** |
| `::method sep class protected external ...` | rc 0, `/` | **match** |
| `::constant c (.k~probe)` on a deferred entry | 88.901 rc 168 | rc 120 loud, names the entry and Phase 7 |

I found **no spelling that binds where the oracle does not, and none that answers where the oracle
refuses.** Every disagreement is the crate refusing louder, which is the licensed direction. The
`::ROUTINE`/`::ATTRIBUTE` halves are untouched and still refuse — re-measured: `::routine r external
'LIBRARY REXX file_separator'` is oracle `90.999` rc 166 (verified, the report's number is right),
`::routine Filespec external 'LIBRARY REXX Filespec'` runs at rc 0, `::attribute a external 'LIBRARY
REXX file_separator'` is `90.998` rc 166 naming `GETfile_separator`.

Instance-side external methods cannot be sent to at all yet, because `.k~new` on a user class is
still a Phase 5 refusal (`rexx-exec: method "NEW" of class "Object" is not implemented (Phase 5)`).
That is why `_bind.rex` reaches `::method isep` only through `~method('ISEP')`. Pre-existing, loud,
not this task's.

### The registry against `NativeMethods.h`

Extracted every `INTERNAL_METHOD(...)` from `interpreter/runtime/NativeMethods.h` and diffed it
against the first string literal of every row of `LIBRARY_REXX_METHODS`: **identical, names and order,
no differences.** `SysNativeMethods.h` adds nothing on unix, as claimed.
`runtime/InternalPackage.cpp:213`, `:230`, `:241` all check out.

The family partition is exact, and I re-derived it rather than trusting the report: for each of the
65 names, `RexxMethod<N>(... <name>)` matches exactly one translation unit —
`streamLibrary/StreamNative.cpp`, `classes/RexxQueueMethods.cpp`, `streamLibrary/FileNative.cpp`, and
`platform/{unix,windows}/TimeSupport.cpp` for the five timer entries (two files, one per platform;
the `Family::Timer` doc names the unix one, which is the platform this crate is checked against).

The bootstrap declares 64 explicitly-named `LIBRARY REXX` entries plus one default-named one; the
one reached only by the default is `file_temporary_path`, and `StreamClasses.orx:510` is indeed
`::method file_temporary_path class private external "library REXX"` — a lower-case `library` with no
third word, exactly as the module doc says. `StreamClasses.orx:546`-`:549` is the
`::CONSTANT separator`/`pathSeparator` pair that `_bind.rex` mimics. `CoreClasses.orx:1555`/`:1557`
are `reply` and `self~!startTimer(...)` inside `::METHOD init` of `::CLASS 'Alarm'` (`:1470`,
`:1472`), and `!startTimer` binds `alarm_startTimer` at `:1590`. All verified with `/bin/grep -n`.

### The three `.orx` files (brief item 1)

Run through `rexx-run` at HEAD, `ir`:

* `platform/unix/PlatformObjects.orx` — rc 0
* `RexxClasses/StreamClasses.orx` — rc 158, `98.909 Class "COMPARABLE" not found` at its own line 506
* `RexxClasses/CoreClasses.orx` — rc 120, message scope override, Phase 5b

Exactly what the report claims. No unresolved external in any of them.

### Invoking an unimplemented entry (brief item 3)

All four `corpus/gate-tables/native-entries/*.rex` on both engines: stdout is exactly `main\n` (so
the bind succeeded and the *send* refused), rc 120, stderr exactly

```
rexx-exec: the LIBRARY REXX entry point "file_exists" is not implemented (Phase 7)
rexx-exec: the LIBRARY REXX entry point "rexx_query_queue" is not implemented (Phase 7)
rexx-exec: the LIBRARY REXX entry point "stream_chars" is not implemented (Phase 7)
rexx-exec: the LIBRARY REXX entry point "alarm_startTimer" is not implemented (Phase 6)
```

The oracle answers each of them differently — rc 168, 158, 208, 168 — so none is expressible as a
corpus row, which the task says plainly and which the brief anticipated.

### The corpus and the sitting

575 `.rex` files under `corpus/` (the README's new figure) and 238 non-comment lines across
`phase-*.txt` (the gate's count). Both check out arithmetically.

Pin staleness: `15a1ffa98` is an ancestor of HEAD; `bench-baselines/pinned/rexx-run-15a1ffa98` hashes
to `141c3fa96…`, which is what `PINNED.md` records; the 115 crate-source commits since the pin all
read as this plan's own tasks.

The sitting ran all eight axes (`alloc4c arith compound dispatchclass emptyloop rexxcps strings
varlookup`) plus a `22-contribution` arm. Every figure the report quotes is in the TSV to the digits
quoted. The drift argument holds for all four axes at or above 1%: `dispatchclass` 1.015507 and
`rexxcps` 1.015346 at `21-fixround-3`, `strings` 1.008309/1.011944 identical at
`21-fixround-3/4/5`, `compound` 1.006966/1.010473 identical at `21-fixround-2/3` — all taken before
`c59a80672` existed. The contribution arm covers only `dispatchclass`, `alloc4c` and `rexxcps`, but
`compound` and `strings` are byte-identical to earlier sittings, so nothing is left unexplained.

### The control, and the bootstrap test

**The control I could not re-run** — inverting the eager bind needs a release rebuild, which I was
told not to do. Reading it: disabling the `install_directives` block makes `method_external`'s `Err`
arm fall through `install_method`'s `_ => None`, so `m` installs as a written body and the program
reaches its prologue. `_missing` and `_source_order` must redden (the 90.998 no longer wins over the
later duplicate `::ROUTINE`), `method__external__subkeyword.rex` must redden, and `_duplicate_wins`
must stay green because `check_member_keys` answers first either way. That is exactly the 238→235
split the report records, and I can see no second cause that would produce it. The restore is
evidenced by a `sha256` and a re-green.

**Could the bootstrap test pass against a wrong registry?** For the failure modes that matter, no —
but not from one test. A missing name is caught by
`every_bootstrap_external_binds_to_an_entry_point_this_registry_holds` (it panics on `Err`) and *not*
by the set-equality test, because `declared` is built only from `Ok` rows and would shrink in step
with `held`. An extra name is caught only by the set-equality test. The pair is sound; either alone
is not, and the report's framing ("the reverse direction is deliberate") is right about why.

What neither catches is a **wrong family assignment** — nothing in the crate asserts it, and the
`Family` doc's "checked by grepping each name's `RexxMethodN` definition" is prose. I re-ran that
grep myself (above) and it holds today. A wrong body or a wrong arity is caught by
`_bind.rex`/`_arguments.rex`; a wrong `implemented` flag by
`the_implemented_entry_points_are_the_ones_the_plan_names`.

---

## Findings

### MAJOR 1 — an orphaned doc block: `run_program` is now undocumented and its doc describes a struct

`rust/crates/rexx-exec/src/lib.rs:6227`-`:6295`.

`pub struct NativeEntryPoint` and `pub fn native_entry_points()` were inserted **between**
`run_program`'s doc comment and `pub fn run_program`. The 28-line block that opens

```
// ---- the public entry point ----

/// Runs a Rexx program and returns what it produced.
```

now documents `pub struct NativeEntryPoint` (`:6272`), and `pub fn run_program` (`:6295`) carries no
documentation at all. Every paragraph in that block is now false of the item it is attached to:
"**This entry point owns everything from `parse_program` onward** and does it on a thread with an
explicitly sized stack (D19)", "`path` is the program's location **as the oracle prints it**",
"`invocation` is what the command line supplied", "**A third parameter rather than a sibling entry
point.**" The section heading `// ---- the public entry point ----` now heads a data struct.

`fmt`, `clippy` and the whole suite pass over this; only `cargo doc` sees it, and that is not a gate.
This is the hazard the plan already has a name for.

Fix: move the two new items below `run_program`, or re-anchor the block.

### MAJOR 2 — the "REASONED, NOT PROBED" caveat is now false, and the correction written for it is also false

`rust/crates/rexx-exec/src/lib.rs:1626`-`:1628` and
`docs/superpowers/plans/phase-4-exclusions.txt:491`-`:496`.

`staged_gap`'s doc still reads:

> The second row is **reasoned rather than probed**: it follows the same code path as the row above
> it, but **no library in this tree loads**, so nothing here has measured it and it must not be read
> as a measurement.

A library in this tree does load: `REXX`. This source copy was not corrected at all, while the
doc-file copy of the same sentence was.

The doc-file correction the task did write says:

> `LIBRARY REXX` is not a counterexample: it is compiled into the interpreter rather than loaded, and
> since Task 22 the `::METHOD` form of it is not a gap here at all, so it never reaches the staged
> check this paragraph is about.

That justifies only the `::METHOD` form. `::ROUTINE EXTERNAL` and `::ATTRIBUTE EXTERNAL` naming
`LIBRARY REXX` are still gaps, still reach the staged check, and their library does resolve.
Measured by me, fresh dir, both engines, on

```rexx
say 'main ran'

::class a
::constant kk (1/0)

::routine r external 'LIBRARY REXX Filespec'
```

oracle rc **214**, `Error 42.3: Arithmetic overflow; divisor must not be zero.` blaming line 4;
crate rc 120, `rexx-exec: ::ROUTINE EXTERNAL is not implemented (Phase 7)`, both engines. That is
precisely the row both paragraphs call unprobed. The task's own report already carried the
measurement that makes it probeable (`::routine r external 'LIBRARY REXX Filespec'` is oracle rc 0
and the routine runs), so the sentence was falsified by evidence the implementer was holding.

The row is a real loss and should stay in both tables — what has to go is the claim that it cannot be
measured, and the reason given for it.

### MAJOR 3 — `check_member_keys`' doc quotes a refusal message this task deleted

`rust/crates/rexx-exec/src/lib.rs:4739`-`:4742`:

> measured, the same pair with the `EXTERNAL` first is the oracle's own 98.903 at rc 158 and this
> crate's `::METHOD EXTERNAL is not implemented (Phase 7)` at rc 120, on both engines.

The program the sentence is about is `::method m external "LIBRARY nosuchlib nosuchfn"`. Measured at
HEAD, both engines: rc 120, `rexx-exec: ::METHOD EXTERNAL naming a library other than REXX is not
implemented (Phase 7)`. The quoted message no longer exists anywhere in `rust/`.

Note for the fix round: the string is hard-wrapped across two `///` lines, so
`grep '::METHOD EXTERNAL is not implemented'` over the tree returns nothing but two entries in
`docs/superpowers/records/`. A sweep by grep would miss it; a collapsed-comment scan finds it.

### MAJOR 4 — `invoke`'s "No `blame_native_method` below this arm" is falsified by the arm inserted below it

`rust/crates/rexx-exec/src/dispatch.rs:1440`:

```rust
// **No `blame_native_method` below this arm**, measured twice
// over. ...
Invocable::Rexx(installed) => { ... }
```

The `Invocable::External` arm this task inserted immediately below it calls
`self.blame_native_method(name, &scope)` at `:1477`. The new arm's own comment states the
contradiction three lines earlier — "The implemented arm does blame, because its refusals are the
oracle's own" — so the two comments disagree with each other inside one `match`.

The behaviour is right (I verified the `Compiled method "SEP" with scope "K".` line is present and
byte-identical to the oracle). It is the older sentence that has to move or be narrowed.

### MINOR 1 — the invocable-kind count was swept in two places out of six

`rust/crates/rexx-exec/src/dispatch.rs:63`, `:85`-`:93`, `:128`, `:136`.

The report lists this class as corrected: "`Invocable`'s doc and `invocable`'s doc both counted the
kinds ('both invocable kinds', 'neither way') ... now phrased over the set." Those two are fixed.
Four more in the same file are not, and one of them is an entire section:

* `:63` — "both invocable kinds take one by value ... so neither runs without a value produced inside
  `mod seam`"
* `:85`-`:93` — "# The invocable kinds, and the one clearance ... A resolved `MethodId` names **either**
  a `NativeMethod` **or** a `::METHOD` directive's own Rexx body ... and **both** entry points take
  the `Cleared` token by value"
* `:128` — "Every method invocation passes here, native or Rexx-bodied"
* `:136` — "the one place both invocable kinds pass"

There are four kinds now. All four sentences were already stale for `Generated`; this task added the
fourth and edited the enum they describe.

(For what it is worth, `tests/dispatch_seam.rs`'s module doc at `:78`-`:80` anticipated exactly this
task — "what is **not** closed is a *third* invocable kind added later with no `Cleared` parameter at
all ... and the same sentence will be true of the fourth" — and is still true.)

### MINOR 2 — `Arity`'s doc now over-claims

`rust/crates/rexx-exec/src/dispatch.rs:205`-`:213`. "How many arguments a primitive method's own entry
admits, which is the second half of every `AddMethod` row in `memory/Setup.cpp`", and `Fixed`'s own
doc says the count is refused "with 93.902" by `CPPCode::run`. This task gave `Arity` a second
consumer whose counts come from `RexxMethod<N>` in `streamLibrary/FileNative.cpp` and whose refusal
is 88.922 from inside `NativeActivation`. The distinction is stated on `Invocable` and on
`Raised::too_many_external_arguments`; `Arity`'s own doc still describes one of its two uses as
though it were both.

### MINOR 3 — comment set cardinalities

`rust/CLAUDE.md:87` forbids naming a set's size in a comment, true counts included. Four added:

* `crates/rexx-exec/src/dispatch/native.rs:16` — "# Exactly one of the three `EXTERNAL` forms binds here"
* `:128` — "**The other three are Phase 7's**" (a count of `Family`'s own variants: in-repo, and it
  rots if a family is ever added)
* `:433` — "Every `EXTERNAL` the three bootstrap files declare"
* `crates/rexx-exec/tests/native_entries.rs:25` — "and the four beside it" (a count of this task's own
  corpus rows)

The second and fourth are the rot-prone kind the rule is actually aimed at.

### MINOR 4 — a citation naming the wrong function

`rust/crates/rexx-exec/src/error.rs:131`. `Delivery::lineless`'s doc attributes
`concurrency/Activity.cpp:1453`-`:1459` to `Activity::displayCondition`. Those lines are inside
`Activity::display`, which starts at `:1414`; `displayCondition` is at `:536` and calls it. The line
range, the mechanism (` line <n>` only when the condition object carries a `POSITION`) and the
measurement are all correct — only the function name is wrong.

### MINOR 5 — two blemishes in `corpus/gate-tables/README.md`

`rust/corpus/gate-tables/README.md:3`-`:10`. The reflow of the opening paragraph left a bare `--`
alone on line 6. And this commit puts the first `gate-tables/` path into a `phase-*.txt` — checked,
`gate-tables/directives/method__external__subkeyword.rex` in `phase-5a.txt` is the only one — while
the flat sentence "Nothing here belongs in `phase-*.txt`" was left standing above the clause that
qualifies it. The paragraph as a whole still parses correctly; the leading sentence now has an
on-disk counterexample it did not have before this commit.

### MINOR 6 — the report's reason for concern 7 is not the true one

The report says the seam fix means "`native.rs` names no `Cleared`". It does, at
`crates/rexx-exec/src/dispatch/native.rs:162`, in a doc comment. The assertion
`the_seam_token_is_named_only_by_the_dispatch_module` passes because `code_occurrences`
(`tests/dispatch_seam.rs:162`-`:176`) skips any line whose trimmed start is `//` — which the same
file's own module doc at `:87`-`:90` says it does not do ("A mention inside a comment or a string
counts toward the needle tallies, so a stray one fails rather than passes"). The conclusion holds —
the test is untouched and still true — but the stated reason does not, and the module doc it rests on
is itself inaccurate. The pre-existing half is not this task's to fix; the report sentence is.

### MINOR 7 — the bootstrap guard cannot see a file drop out

`every_bootstrap_external_binds_to_an_entry_point_this_registry_holds` guards against a vacuous pass
with one global `reached > 0`. Checked: `platform/unix/PlatformObjects.orx` declares no `EXTERNAL` at
all (`CoreClasses.orx` 5, `StreamClasses.orx` 61, `PlatformObjects.orx` 0), so the third file
contributes nothing to `reached` and a change that stopped reaching its directives would leave the
test green. Not a defect — the file genuinely has none — but "all three `.orx` files install with no
unresolved external" reads stronger than what is asserted for one of the three.

---

## What I did not check

* **The control.** Inverting the eager bind needs a release rebuild, which the dispatch forbids. I
  read it instead; the reasoning is above.
* **`ee083bd7c`'s `.text`-identical claim.** Same reason.
* **The five gate commands.** Run by the controller; not repeated.

## Nothing found on

Every C++ line citation added or edited by this task, checked with `/bin/grep -n` and read in
context: `InternalPackage.cpp:213`/`:230`/`:241`; `DirectiveParser.cpp:867`-`868`, `:1385`, `:1406`,
`:1678`-`1679`, `:1683`, `:1690`, `:2664`; `StringClass.cpp:1405`-`:1416` (and it does put its
argument in front of the receiver, so `GET` is a prefix); `LibraryPackage.cpp:313`, `:324`;
`PackageManager.cpp:86`, `:233` (on the path: `resolveMethod` → `getLibrary` → `loadLibrary` →
`packages->get`); `SysFileSystem.cpp:1358`-`:1361`, `:1369`-`:1372`; `FileNative.cpp` `RexxMethod0`
rows; `CoreClasses.orx:1555`, `:1557`, `:1590`; `StreamClasses.orx:506`, `:510`, `:546`-`:549`. Every
one is on the path its own example takes. The only exception is MINOR 4, where the lines are right
and the function named around them is not.

No non-ASCII and no em-dashes were added to any source file or corpus program (the four non-ASCII
lines in `lib.rs` predate this range and are untouched). No historical framing in the new comments.
The `Family::owner` cost-if-wrong claim is true in the code: `"Phase 6"` is written in exactly one
place, `native.rs:135`, and `tests/native_entries.rs` derives the expected message from `row.owner`
rather than holding a copy, so re-assigning the timer or queue family is a one-line edit.
