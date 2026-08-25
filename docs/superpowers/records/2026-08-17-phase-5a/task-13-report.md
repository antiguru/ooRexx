# Task 13 -- the three access scopes, and the one chokepoint

**Status: DONE_WITH_CONCERNS.** Both `PRIVATE` programs match byte for byte on both engines, the
chokepoint test still holds and says what it counts, and the negative control reddens the row this
report names. The concerns are the send-path cost, the arms of the C++ rule that no program in this
phase can reach, and one place where the brief's own framing turned out not to hold.

Base: `1a2ec9664`. Commits:

| commit | what |
|---|---|
| `b84685f44` | the three access scopes, the corpus programs, the exclusions entry |
| `188b5ce3d` | the guard's two-build sitting at `b84685f44` |
| `6d12c033d` | search the access scopes instead of hashing them, and check the search |
| `6b1ef369d` | the guard's two-build sitting at `6d12c033d` |

---

## What the build is, and why the check is not where the brief put it

**The brief said the two access checks sit inside the chokepoint. `PRIVATE` and `PACKAGE` are not
there, and the reason is measured rather than argued.** `RexxObject::messageSend` does not raise on a
refused access check: it replaces the method the lookup found with nothing, sets an error code, and
enters `processUnknown` (`classes/ObjectClass.cpp:876`-`:889`, then `:904`). So a refused private send
reaches the receiver's own `UNKNOWN` if it has one, and `processUnknown` calls
`behaviour->methodLookup(GlobalNames::UNKNOWN)` directly (`:1004`) rather than going back through
`messageSend`, so the `UNKNOWN` forward is not itself access-checked.

Both halves were run, three descriptors, from a fresh empty directory:

| program | oracle | crate, both engines |
|---|---|---|
| `.K~m`, `m` is `CLASS PRIVATE`, class also declares `::METHOD unknown CLASS` | rc 0, `unknown saw M with 0` | the same |
| `.K~zork`, whose only method is `::METHOD unknown CLASS PRIVATE` | rc 0, `private unknown saw ZORK` | the same |

A check inside `seam::clear` cannot produce either: the seam is reached from `Interp::invoke`, which
runs *after* the name has resolved, and it is reached by the `UNKNOWN` forward too. So `PRIVATE` and
`PACKAGE` are decided in `Interp::resolve`, which is now exactly `messageSend`'s pre-`run` half --
`Interp::lookup` (the oracle's `methodLookup`/`superMethod`) followed by the access check -- and it
answers `Result<Resolution, Miss>`, where `Miss` is the oracle's own `error` local and selects 97.1,
97.2 or 97.3 for a receiver that answers no `UNKNOWN`.

`PROTECTED` **is** at the chokepoint, which is the scope the oracle asks a manager about:
`seam::clear` asks `Interp::method_is_protected` and, for a method that is, asks
`Interp::check_protected_method`.

**Was there still exactly one chokepoint after this?** `cargo test --release -p rexx-exec --test
dispatch_seam` is 5 passed / 0 failed, exit 0.

### What `dispatch_passes_through_exactly_one_chokepoint` counts

Three tallies over the `.rs` files under `crates/rexx-exec/src/`, needles built with `format!` so the
test file's own text is not counted:

* occurrences of the literal `seam::clear(` -- must be exactly 1;
* occurrences of `struct Cleared(())` -- must be exactly 1;
* occurrences of `Cleared(())` -- must be exactly 2, the declaration plus one construction.

`the_seam_module_holds_one_struct_and_one_function` brace-matches `mod seam {`'s body, strips
whole-line comments, and requires exactly one `struct ` and one `fn ` with no `impl`, `const`,
`static`, `mod`, `macro_rules!`, `trait`, `union` or `enum`, and no `derive`.
`the_seam_token_is_named_only_by_the_dispatch_module` requires every non-comment mention of `Cleared`
to be in `dispatch.rs`. `the_scan_can_tell_a_present_token_from_an_absent_one` is the control: a
token the crate contains is found and one it does not is not.

**What would make all of that pass while a second dispatch path exists** -- and this is the file's own
list, re-read rather than restated from memory:

* `use seam::{clear as sneak_clear};` plus a second invocation path calling `sneak_clear`. It adds no
  item to `mod seam` and matches neither needle.
* A `clear` returning more than one clearance per call -- a tuple, or a `Vec<Cleared>` -- feeding two
  invocation paths from one trip through the seam. One struct, one function, one call site.
* An item introduced by a macro expansion inside the module, or a second `mod seam` in another file.
* A **third invocable kind added later with no `Cleared` parameter at all.** Nothing lexical can
  require a signature that does not exist yet.
* A path in another crate. The scan reads `rexx-exec/src` only.

To that list this task adds one more, and it is the one my own change was most able to trip:
**a check placed beside the seam rather than inside it passes every count above.**
`the_protected_question_is_asked_only_inside_the_seam` narrows that for `PROTECTED` alone -- one code
mention of the predicate besides its definition, both in `dispatch.rs` -- and its doc says plainly
that it cannot see whether the call is inside `seam::clear` or merely inside the same file. With no
manager installed, no program can observe the branch at all, so that lexical assertion is the whole
instrument for `PROTECTED`'s placement.

### The `PRIVATE` rule, and why it is not "the defining scope"

`RexxObject::checkPrivate` (`classes/ObjectClass.cpp:608`-`:646`), re-read before it was relied on.
Its input is the caller's own receiver (`:616`), and its limbs in order: no activation, or an
activation whose frame carries no receiver, refuses (`:611`, `:622`-`:626`); the sending and
receiving object being the same object allows (`:617`-`:620`); the sender being an instance of the
method's scope allows (`:628`-`:633`); the sender being a class object compatible with that scope
allows (`:635`-`:643`); anything else refuses.

`PACKAGE` is `RexxObject::checkPackage` (`:658`-`:685`): no activation refuses, the same package
allows, another package refuses. The method's package is the one the `::METHOD` directive was
translated in, because `MethodClass::isSamePackage` asks the method's *code* object
(`classes/MethodClass.hpp:147`) and not its scope class.

`isSpecial()` is `protected || private || package` (`classes/MethodClass.hpp:118`), and that
disjunction is the membership rule for `Interp::special_methods`. `UNPROTECTED` is not a row: the
oracle's flag word has a `PROTECTED_FLAG` and no unprotected bit, so the keyword's whole effect is
making a second protection keyword 25.902, which `rexx-parse` already owns.

---

## The differential, three descriptors, both engines

Every row below was run as `( ulimit -v 1048576; LD_LIBRARY_PATH=.../build/lib timeout -s KILL 10
.../build/bin/rexx FILE )` against `REXX_ENGINE=tree-walker` and `REXX_ENGINE=ir`, from a fresh empty
directory with absolute paths, stdout and stderr captured to separate files and compared with `cmp`,
exit status compared separately. **Every pair in the table below matched, on both engines, and the
whole set was re-run against the final binary at `6d12c033d`**; the only program in the probe set that
did not match is the generated-accessor row named after the table, which is a pre-existing gap. **Every `before, crate` cell is a run and not a reading of the code**: the first five against a
release build of the clean tree at `1a2ec9664`, and the rest against a release build of `1a2ec9664`
made in a separate worktree, both `REXX_ENGINE=ir`. The first five are the states the dispatch brief
recorded, and none had moved.

| program | before, crate | oracle | after, both engines |
|---|---|---|---|
| `say .K~m`, `m` is `CLASS PRIVATE` | rc 120 | rc 159, `97.2 Object "The K class" cannot accept private message "M" from this context.` | matches |
| `say .K~outer`, `outer` does `self~m` | rc 120 | rc 0 `inner` | matches |
| the same with `PRIVATE` removed | rc 0 `inner` | rc 0 `inner` | matches |
| `say .S~poke`, sibling class does `.K~m` | rc 120 | rc 159, the same `97.2`, traceback two frames | matches |
| `say .Sub~poke`, `poke` does `self~m` | rc 120 | rc 0 `inner` | matches |
| `say .Sub~poke`, `poke` does `.Base~m` | rc 120 | rc 0 `inner` | matches |
| `say .Base~poke`, `poke` does `.Sub~m` | rc 120 | rc 0 `inner` | matches |
| `say r()`, a `::ROUTINE` does `.K~m` | rc 120 | rc 159, the same `97.2` | matches |
| `say .K~m` with `PROTECTED` | rc 0 `prot` | rc 0 `prot` | matches |
| `say .K~m` with `PACKAGE` | rc 0 `pkg` | rc 0 `pkg` | matches |
| `say r()`, a `::ROUTINE` does `.K~m`, `m` is `PACKAGE` | rc 0 `pkg` | rc 0 `pkg` | matches |
| `say .K~m` with `PRIVATE` and an `UNKNOWN` | rc 120 | rc 0, `unknown saw M with 0` | matches |
| `say .K~zork`, `UNKNOWN` itself `PRIVATE` | rc 120 | rc 0, `private unknown saw ZORK` | matches |
| `signal on nomethod` over a refused private send | rc 120 | rc 0, `NOMETHOD M` with `E` empty and `RC` untouched | matches |
| `say .K~a`, `::attribute a class private` | rc 120 (`a PRIVATE ::ATTRIBUTE`) | rc 159, `97.2` naming `"A"` | matches |
| `say .K~a`, `::attribute a class get private` with a body | rc 120 (`a PRIVATE ::ATTRIBUTE`) | rc 159, `97.2` naming `"A"` | matches |
| `say .K~poke` reading a private `GET` attribute from inside | rc 120 (`a PRIVATE ::ATTRIBUTE`) | rc 0 `got` | matches |
| `corpus/lang/method_access_private.rex` | rc 120 | rc 0, its own lines | matches |
| `corpus/lang/method_access_private_refused.rex` | rc 120 | rc 159 | matches |
| `corpus/lang/method_access_package_and_protected.rex` | rc 0, its own lines | rc 0 | matches |

**The two frames.** The sibling's traceback is compared as bytes, not as a shape:
`    22 *-* return .K~m` then `    14 *-* say .S~poke`, then the `Error 97 running <path> line 22`
line and the `Error 97.2:` line, identical on all three descriptors.

**`CONDITION('E')` separates 97.2 from 97.1**, which is what stops a build that dropped the method
and let the name-miss report stand from passing. Measured on the oracle under `SIGNAL ON SYNTAX`: `2`
for a refused private send, `1` for a name the behaviour does not answer.

### The one row that is still a refusal

`say .K~poke` where `poke` does `self~a` and `a` is `::attribute a class private` with no body: oracle
rc 0 printing `A`, crate rc 120 `a generated ::ATTRIBUTE accessor is not implemented (Phase 5)`, both
engines. That is the pre-existing generated-accessor gap, reached through a path the `PRIVATE` gap
used to hide. It is a clean loud refusal and not a wrong answer.

---

## Corpus

**156 -> 159, and 159 of 159 matching.** Three programs, in `b84685f44` with their
`corpus/phase-5a.txt` entries, their `EXPECTED_SUBSET_5A` rows and their `sourceline_oracle`
expectations:

* `corpus/lang/method_access_private.rex` -- rc 0. Every sending position: `self~m` from the
  declaring class and from a subclass, `.Base~m` from a subclass and `.Sub~m` from the superclass,
  and the three refusals (program frame, routine frame, sibling class) trapped so one program can
  carry them all, plus the `UNKNOWN` fall-through and the `NOMETHOD` trap.
* `corpus/lang/method_access_private_refused.rex` -- rc 159, the sibling's refusal untrapped, for the
  traceback bytes.
* `corpus/lang/method_access_package_and_protected.rex` -- rc 0. `PACKAGE`'s allowing side at every
  sending position one package can offer, `PROTECTED` from inside and outside, and a
  `::ATTRIBUTE ... GET PACKAGE`.

`method_access_private_refused.rex` joined `collect_stress.rs`'s `NO_ALLOCATION_PROGRAMS`; the other
two collect and did not.

---

## Gate tables

**Gate table C's `pubpri` row reads `agree`.** Its probe sends `.k~pub`, `.k~callPriv` and `.k~priv`,
so it exercises the outside-send this task changed.

I did **not** run that table before the change, so I make no claim about a measured transition. What
I did run is the row's own committed `control` field, which the table assigns to Task 13 -- see the
negative-control section below.

**Gate table D's `::METHOD PRIVATE` row reads `agree`, and it is not an instrument for this task.**
The brief named it as one. Its probe,
`corpus/gate-tables/directives/method__private__subkeyword.rex`, is four lines: `say 'main'`,
`::class k`, `::method m private`, `return 1`. It never sends `~m`, and the method is an instance
method, so reaching it would need `~new` as well. The row could not have witnessed the change before
and does not witness it now. `::ATTRIBUTE PRIVATE`, `::METHOD PACKAGE`, `::METHOD PROTECTED`,
`::ATTRIBUTE PACKAGE` and `::ATTRIBUTE PROTECTED` have the same declare-only shape. Table D reports
79 rows: 34 `agree`, 45 `diverge-both` and 45 loud, and all six of those keyword rows are among the
`agree`.

**One thing the table's own scanner says and this task does not answer.** `gate_table_d.rs` scans the
bootstrap `.orx` files for "an install-time `::CONSTANT` sending a private class method of its own
class" and finds occurrences, so the prologue contains that shape. Nothing here loads those files;
that is Task 21's.

---

## The negative control

**The control: let a private send from outside the object be allowed.** Applied as an early
`return Ok(())` at the top of `Interp::check_private`, with the rest of the body left in place under
`#[allow(unreachable_code)]`, so nothing but the verdict changes.

| instrument | green tree | under the control |
|---|---|---|
| gate table C, `pubpri` row | `agree` | **`diverge-both`** -- oracle rc 159 with two stdout lines, crate rc 0 with three |
| corpus, `REXX_CORPUS_GATE=1` | 159 of 159 | **157 of 159**: `lang/method_access_private.rex` (stdout differ), `lang/method_access_private_refused.rex` (stdout, stderr and exit code differ) |
| `cargo test --release -p rexx-exec --lib` | 712 passed | **3 failed**: `private_sends_are_refused_by_who_is_sending`, `a_refused_send_reaches_unknown_and_the_unknown_lookup_is_not_checked`, `a_private_send_from_another_instance_of_the_declaring_class_is_allowed` |

**The row the control reddens is gate table C's `pubpri`**, from `agree` to `diverge-both`, and it is
the outside-scope case: the probe's third line `.k~priv` answers where the oracle raises 97.2.

**The paired control, run because a one-directional control cannot tell a real rule from a blanket
refusal.** Refusing every private send instead -- an early `return Err(Miss::Private)` in the same
place -- reddens the corpus to **158 of 159** on `lang/method_access_private.rex` (the allowed rows
now raise, first frame `73 *-* return self~m`), `pubpri` to `diverge-both`, and two of the three
in-crate tests. So neither blanket answer passes.

**Where the brief's framing does not hold, stated because it changes what the gate is worth.** The
brief says the corpus gate cannot see a clean refusal becoming a wrong answer. That was true of the
starting state, where our refusal was rc 120 and the oracle's was rc 159, so no differential row
existed. It is no longer true of these three programs: our refusal now *is* the oracle's, so it is
expressible as a corpus row, and the control above reddens two of them. What the corpus still cannot
see is a refusal at a sending position none of its programs occupies -- which is why the in-crate
test enumerates the positions by limb rather than relying on the programs.

---

## The in-crate tests this task owes

In `crates/rexx-exec/src/dispatch.rs`'s test module:

* **`private_sends_are_refused_by_who_is_sending`** -- the refusal shape by scope. Three refusing
  programs, one per refusing limb (program frame, routine frame, sibling class), each asserted at
  rc 159 with empty stdout and the exact `Error 97.2:  Object "The K class" cannot accept private
  message "M" from this context.` text; three answering programs, one per allowing limb reachable
  from a program, each asserted at rc 0 `inner` with empty stderr. Both engines, via `both_engines`.
* **`a_refused_send_reaches_unknown_and_the_unknown_lookup_is_not_checked`** -- the pair. A build
  that raised at the send answers neither row; a build that checked the `UNKNOWN` lookup too answers
  the first and refuses the second.
* **`package_scope_refuses_a_caller_from_another_package`** -- `PACKAGE`'s refusing arm, which no
  program in this phase can reach. `Interp::check_package` is called directly with a caller that has
  no activation, a caller in another program's package, a caller in the method's own package, and a
  caller in a program's package against a method in the interpreter's. The allowing row is there so
  a check that refused everything fails rather than passing both refusals.
* **`a_private_send_from_another_instance_of_the_declaring_class_is_allowed`** -- `checkPrivate`'s
  `isInstanceOf` limb, likewise unreachable from a program. `Interp::check_private` is called with a
  string as the sender and `.String` as the declaring scope, then with `.Array` as the scope for the
  refusal that makes the first mean something.

In `crates/rexx-exec/tests/dispatch_seam.rs`:

* **`the_protected_question_is_asked_only_inside_the_seam`** -- one code mention of
  `method_is_protected(` besides its definition, both in `dispatch.rs`.

**A `PRIVATE` row left `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run`**,
because it is now neither a refusal nor an unconditional success; a `PACKAGE` row joined that test's
success list.

**"Nothing covers X" was checked by searching, not from memory.** `/bin/grep -rain 'PRIVATE
::METHOD|PRIVATE ::ATTRIBUTE'` over `--include=*.rs --include=*.txt --include=*.tsv --include=*.rex`
outside `target/` returns exactly one hit before this task, `dispatch.rs:2788`, which is the row
removed above. So no other test or exempt list named the retired refusals.

> **This paragraph is wrong in three ways and Fix round 1's F1 replaces it.** The command as written
> matches nothing, the `-E` form returns three hits rather than one, and the line is not `:2788`. The
> conclusion holds; the evidence for it did not.

---

## Performance

### The guard's sitting, and what its axes cannot see

Two sittings, both `pinned=bench-baselines/pinned/rexx-run-15a1ffa98` against
`head=target/release/rexx-run`, `--rounds 5`, six axes. Rows are in
`bench-baselines/phase-5a-arms.tsv` under `task = 13`, told apart by `commit`.

**The staleness test was run.** `git log --oneline 15a1ffa98..HEAD -- rust/crates rust/Cargo.toml
rust/Cargo.lock Cargo.toml` lists this plan's own commits and nothing else; `ee6ebbf64` and
`0ce35233e` are named in `task-12-report.md` and in the TSV rather than in `progress.md`, and every
other entry is named in `progress.md`. `sha256sum` of the pinned binary is
`141c3fa968ea774655bc00f3e3b9f61cf1c4b738fd2793831d0055e94f1d074b`, matching `PINNED.md`.

**The duplicate-key check prints `0`** after both sittings' rows landed.

`instructions:u`, per pass, first sitting (`commit = b84685f44`):

| axis | arm | pinned | head |
|---|---|---|---|
| `strings` | ir | 5369.524876 | 5421.525224 |
| `strings` | tw | 9044.518597 | 9129.518621 |
| `alloc4c` | ir | 3594.687394 | 3603.685526 |
| `arith` | ir | 24059.572624 | 24062.572988 |
| `compound` | ir | 1909.479928 | 1914.480214 |
| `emptyloop` | ir | 376.000016 | 376.000001 |
| `varlookup` | ir | 871.000028 | 871.000072 |

`strings`/`ir` is **+52.000348** against a ceiling of 53.695249 (1% of the pin's own per-pass), so
**1.694901 instructions per pass remain** and this task spends none of them. The second sitting
(`commit = 6d12c033d`) reads `strings`/`ir` at 5421.525320 against a pin of 5369.525681, which is
**+51.999639** against a 53.695257 ceiling and **1.695618 remaining**. No axis reached 1% against the
pin in `instructions:u` in either sitting, so no change-versus-layout attribution is owed at that
threshold; the widest is `strings`/`ir` at 1.009685.

**That is a null result, not a cheap send path, and the reason is the axis set.** `arith`,
`compound`, `emptyloop`, `strings` and `varlookup` contain no `~` at all, and `alloc4c`'s four
occurrences are all inside its own header comment. **No axis in the guard's set sends a message**, so
these rows bound what the change costs a program that never dispatches and say nothing about the path
it actually touches.

### The send path, against Task 10's baseline

Measured separately, `perf stat -e instructions:u`, `REXX_ENGINE=ir`, 3,000,000 passes per program,
nine rounds per cell **interleaved build by build**, medians reported with the observed spread.

Run-to-run spread on a single cell reaches 0.6% on the method-send programs, so single readings are
not usable here; the medians below are stable to a fraction of an instruction per pass across two
independent nine-round batches.

**Base against changed, same source tree, both built under the same path prefix** -- the head sources
copied into the base revision's worktree and rebuilt there, so the code-placement difference the
guard warns about is removed rather than bounded:

| program | base `1a2ec9664` | changed | per send |
|---|---|---|---|
| one native send per pass (`s~length`) | 6,211,262,304 | 6,253,227,496 | **+13.988** |
| one class-method send per pass (`.K~m`) | 17,993,761,850 | 18,071,740,567 | **+25.993** |
| two class-method sends per pass (`.K~outer` over `self~m`) | 31,388,783,828 | 31,544,786,678 | **+26.000** |

The cross-path batch agrees to within 0.02 instructions per send (+13.995, +26.008, +25.999), so the
figure is the change and not the layout.

**Against Task 10's baseline of about 3 instructions per native send and about 12 per method send,
this task adds about 14 and about 26.** As a ratio over the whole program the three rows read
1.006756, 1.004334 and 1.004970, all under the 1% the guard treats as a finding. Against the send
itself the share is larger: subtracting `emptyloop`'s 376 instructions per pass leaves about 1,694
for a native send and about 5,069 for a class-method send, so the additions are about 0.83% and about
0.51% of what a send costs. Either way it is between two and four times what Task 10 spent, and it is
unconditional. It is listed as a concern.

**What the rows cost when they exist**, one binary, `6d12c033d`, medians of nine interleaved rounds
against the same program without the keyword:

| program | delta per pass, two sends |
|---|---|
| a `PRIVATE` method elsewhere in the file, neither send resolving to it | +18.104 |
| the inner send resolving to a `PROTECTED` method | +26.970 |
| the inner send resolving to a `PRIVATE` method | +32.980 |

**That is after `6d12c033d`, and before it the same three rows were +580, +597 and +604.** The rows
were a `HashMap` keyed on `MethodId`; the send path asks whether a resolved method is special twice,
once in `resolve` and once at the seam, so a two-send loop paid four hashes, and the default hasher
cost more than everything it guarded -- the file whose special method neither send resolved to paid
the same 580 as the file where one did. `6d12c033d` keeps the rows sorted and binary-searches them.
The order is an invariant of the push, since the registry mints method identities from one counter
that only increments; `Interp::record_access_scope` asserts it, and the accessor checks its own answer
against a scan of the same rows.

**That accessor check was proved live.** With `self.special_methods.reverse()` appended to every push:
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` panics at
`the search over the access scopes disagrees with a scan of the same rows`, while the **release** build
of the same tree does not panic and instead reads 158 of 159 with `method_access_private.rex`
differing on stdout. The debug gate is what catches the order breaking; the release gates catch only
the wrong answer that follows from it, and only on a program whose file has more than one special
method -- `cargo test -p rexx-exec --lib` stayed at 712 passed under that mutation, because every
in-crate program has one.

---

## Gates

All from `rust/`, statuses read unpiped.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0, no failures |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0, corpus 159 of 159 |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | exit 0, no failures, corpus 159 of 159 |

**All five results above were read at `6d12c033d`, the final code revision.** All five were also run
against `b84685f44`'s source, before the search replaced the hash, and passed identically -- corpus
159 of 159 in both the release and the debug run. `cargo test --release -p rexx-exec --test dispatch_seam` at
`6d12c033d` is 5 passed / 0 failed. `command -v memcap` found it at
`/home/moritz/.local/bin/memcap`, so the debug gate ran under `memcap` rather than the
`ulimit -v` substitute.

---

## The known-gap entry

`docs/superpowers/plans/phase-4-exclusions.txt`, the entry the dispatch brief re-cited to `:3304`.
Retitled to `GUARD` alone; the `PRIVATE` and `PROTECTED` limbs are replaced by the measurements that
superseded the sentences saying what made each one safe, rather than deleted, and `GUARD`'s limb and
its "list of routes checked, not an enumeration of routes" warning are untouched. The owner paragraph
now names `GUARD` alone and says where `Protection` is read.

**Re-measuring `PROTECTED`'s route list moved one row, and it was re-run rather than carried over.**
The old limb listed `.K~method('M')` at rc 120 among the loud routes to a Method object. Measured
now, both engines: on an **instance** method it answers `a Method` at rc 0, matching the oracle, so
that object is reachable and the row was stale. The refusal has moved one send later --
`.K~method('M')~setSecurityManager(.nil)` is rc 120 here where the oracle answers `1`. `.context~package`,
`.methods['M']` and `.routines['R']` are still the same rc 120 refusal against oracle answers of
`a Package`, `a Method` and `a Routine`, and `.Method~new('x', 'return 1')` is still rc 120. So the
limb's conclusion holds and its reason changed.

---

## What the checks could not see

* **The corpus cannot see a sending position none of its programs occupies.** It now covers the
  program frame, a routine frame, a sibling class, the declaring class's own method, a subclass's own
  method in both the `self~m` and the `.Base~m` shapes, and the superclass sending to the subclass. A
  seventh position behaving wrongly is invisible to it, which is what
  `private_sends_are_refused_by_who_is_sending`'s limb-by-limb enumeration is for. The two do not
  cover the same set: the in-crate test omits the superclass sending to the subclass, which only the
  corpus program has, so neither is a superset of the other and a position neither names is covered by
  nothing.
* **`PACKAGE`'s refusing arm has no differential and 5c owes one.** The discriminating program needs
  a caller in a second package, which needs `::REQUIRES`. `package_scope_refuses_a_caller_from_another_package`
  is an in-crate test calling the check directly and is the whole instrument. There is no differential
  row for it, in this task or anywhere.
* **`checkPrivate`'s `isInstanceOf` limb has no differential either.** It allows a send from another
  *instance* of the declaring class, and an instance needs `~new`. A class object cannot stand in: its
  own class is the metaclass, never the user class the method is declared in. The in-crate test calls
  the check directly with a string sender.
* **`PROTECTED` cannot be observed by any program.** With no manager,
  `Interp::check_protected_method` cannot refuse. Its placement has a lexical assertion and nothing
  else; its *behaviour* has no instrument at all, and the corpus row for it only shows that the send
  still answers.
* **`check_package`'s no-activation arm is not reachable from a program either**, for the same reason
  as the C++'s: a send needs a running activation. The in-crate test is the whole instrument.
* **Every measurement in this task is on a class method.** Reaching an instance method needs `~new`,
  which is 5b's. The instance-method reading of every row above is untested rather than confirmed,
  including the refusals: a route that refuses on a class method is not thereby known to refuse on an
  instance one. This joins the debt the exclusions entry already records against 5b.
* **The `Miss::PackageScope` report text is not oracle-measured as a whole program**, because no
  program can reach it. The 97.3 substitutions are the catalogue's, taken from the generated
  `errors.rs` row that `rexx-inventory` builds from the oracle's own message file.
* **`cycles:u` is in the TSV and is not reported as a result**, per the guard's own rule.
* **The guard's six axes send no messages**, so they bound only the cost to a program that never
  dispatches. The send-path figures come from programs written for this report and committed nowhere;
  a later task re-measuring them has to write them again.
* **The `pubpri` transition is not a measured before-and-after.** I ran gate table C only after the
  change. The control is what stands in for the missing "before".

---

## Concerns

1. **The send path costs about 14 more instructions per native send and about 26 more per
   class-method send** than the base revision, unconditionally, against Task 10's 3 and 12. Both are
   under 1% of the send's own cost and the guard's budget is untouched, but the shape that would
   remove them is the oracle's own: flags on the method itself. In this crate that means the entry
   `rexx-classes`' method dictionary already returns, so both the emptiness test and the search
   would disappear. That is a cross-crate change and I did not take it unreviewed.
2. **The brief's placement instruction and the oracle's behaviour disagree**, and I followed the
   oracle: `PRIVATE` and `PACKAGE` are decided in `Interp::resolve`, not inside `seam::clear`. The
   plan text should be corrected, since a later reader comparing it against the tree will find the
   check somewhere the plan does not put it.
3. **Table D's `PRIVATE` row is not an instrument for this task** and the plan names it as one. Its
   probe declares an instance method and never sends it.
4. **`::ATTRIBUTE ... PRIVATE` with no body changed which refusal it gets.** From
   `a PRIVATE ::ATTRIBUTE` to `a generated ::ATTRIBUTE accessor`, both rc 120, and only when the
   caller is one the access check allows. Nothing in the tree named the retired string, checked by
   search, but a reader who remembers the old message will not find it.
   **This concern understated its own subject and Fix round 1's F2 closes it**: the same edit made
   the *refusing* side match the oracle, which nothing covered either, and both sides now have an
   instrument.
5. **The order invariant behind the access-scope search is enforced by two `debug_assert`s**, so the
   release gates do not check it. The release build's answer to a broken order is a wrong answer
   rather than a panic, which is why the accessor's check exists at all; it still means the **debug**
   corpus gate is load-bearing for this, and the in-crate suite cannot substitute because every
   program it runs has at most one special method.
   **Refined in Fix round 1's F5**: of the two, only the accessor's self-check is a demonstrated
   witness. The record-side assert guards another crate's increment-only counter and cannot be
   falsified from inside this one.

---

# Fix round 1

Base for the round: `6b1ef369d`, with Moritz's `fd04468f0` and `63628e8ec` landing on the plan
between. Commits:

| commit | what |
|---|---|
| `d3758d520` | `PRIVATE` on a `::ATTRIBUTE` gets its instruments, and the send path gets an axis |
| `ca7008069` | the two send-axis sittings, and which of the two order assertions catches which break |
| `df799fee0` | this task's own contribution on the guard's six axes, measured in one sitting |

**Both plan-level items the review raised are already fixed by Moritz, not by me**, and this report's
concerns 2 and 3 are closed by them: `fd04468f0` moves the plan's `PRIVATE`/`PACKAGE` checks out of
the chokepoint, corrects "the corpus gate cannot see that at all", and replaces table D's `PRIVATE`
row with gate table C's `pubpri` as the table instrument. `63628e8ec` settles which quantity the 1%
rule governs.

## F1 -- the search claim, restated with the pattern actually run

**What it said**, at `task-13-report.md`'s "nothing covers X" paragraph: that
`/bin/grep -rain 'PRIVATE ::METHOD|PRIVATE ::ATTRIBUTE'` over
`--include=*.rs --include=*.txt --include=*.tsv --include=*.rex`, outside `target/`, "returns exactly
one hit before this task, `dispatch.rs:2788`".

Three things wrong with that, all three re-run here rather than taken from the review:

* **The pattern matches nothing.** `/bin/grep` without `-E` reads `|` literally, so that command
  exits 1 with no output -- confirmed by running it from `rust/` at head. The claim rested on a
  command that cannot find anything, which reads exactly like a command that found one thing.
* **The count is three, not one.** With `-E`, at `1a2ec9664`, restricted to `rust/` and those
  extensions: `rust/crates/rexx-exec/src/dispatch.rs:2522`, the removed test row, and
  `rust/crates/rexx-exec/src/lib.rs:1592` and `:1601`, the two producers in `method_body_gap`. Run
  tree-wide with no extension filter it also reaches
  `docs/superpowers/plans/2026-08-17-phase-5a.md:1324` and
  `docs/superpowers/plans/phase-4-exclusions.txt:3327`, both prose. From `rust/` at head the same
  `-E` pattern matches nothing at all, which is the retirement.
* **The line was wrong.** `dispatch.rs:2788` at `1a2ec9664` is
  `assert!(stderr.contains("Error 97.1:"), "{stderr:?}")`, read out of
  `git show 1a2ec9664:rust/crates/rexx-exec/src/dispatch.rs`; the removed row's `"a PRIVATE
  ::METHOD"` string is at `:2522`.

**The conclusion survives and is restated with its pattern beside it.** Searching
`-E 'PRIVATE ::METHOD|PRIVATE ::ATTRIBUTE'` over `*.rs`, `*.txt`, `*.tsv` and `*.rex` under `rust/`
at `1a2ec9664`, every hit is either the test row this task removed or one of the two producers it
retired. No other test, exempt list or committed table named the retired refusals -- as wide as that
pattern and no wider: a test naming the refusal by a different spelling, or by matching a prefix of
it, is outside what was searched.

## F2 -- `PRIVATE` on a `::ATTRIBUTE` now has an instrument on each side

**What was true and unstated.** Retiring `a PRIVATE ::ATTRIBUTE` changed two observable things and
neither had any instrument -- not "an in-crate test only", which the Global Constraint would have
accepted if said, but nothing. The original report's concern 4 mentioned only the swapped string,
which is the smaller half.

**The refusing side is now a differential row.** `corpus/lang/method_access_private_attribute.rex`,
rc 159, in `d3758d520` with its `corpus/phase-5a.txt` entry, its `EXPECTED_SUBSET_5A` row and its
`sourceline_oracle` expectation. It carries both accessor names, because `install_attribute` records
the access scope once per generated name and the setter's dictionary key is the attribute's name with
`=` appended: the getter read through `self~a` from inside the declaring class, a bodyless private
attribute sent from outside and trapped, the setter reached through the message-assignment form from
outside and trapped, and the getter sent from outside untrapped for its traceback bytes.
`CONDITION('E')` is `2` on both trapped rows. Verified against the oracle on three descriptors, both
engines, from a fresh empty directory: **matches**.

**The allowed side is now a row in the loud test.** `say .K~poke` over `self~a` with
`::attribute a class private`, asserted at rc 120 with `a generated ::ATTRIBUTE accessor`, beside the
non-private row that was the only one there. That is the caller the access check permits, so it is
the arm that reaches the remaining gap rather than the refusal.

**Both were verified to fire, and nothing else does.** Restoring the retired limb --
`if attribute.access == Access::Private { Some(Loud::method_body("a PRIVATE ::ATTRIBUTE")) }` back at
the head of `method_body_gap`'s attribute arm:

| instrument | fixed tree | with the limb restored |
|---|---|---|
| corpus, `REXX_CORPUS_GATE=1` release | 160 of 160 | **159 of 160**, `lang/method_access_private_attribute.rex`, classified `[a PRIVATE ::ATTRIBUTE]`, rust rc 120 against oracle rc 159 |
| `cargo test --release -p rexx-exec --lib --no-fail-fast` | 712 passed | **711 passed, 1 failed**, `a_method_body_this_crate_cannot_run_is_loud_and_its_neighbours_still_run`, panicking at the new row's assertion |

So the mutation reddens both instruments added for it, which is what separates "this test can fail"
from "this test adds coverage": before `d3758d520` neither the corpus nor the lib suite had a program
with a private attribute in it.

**This said "and nothing else in the workspace", which the round-1 re-review found false.** The
mutation also fails `collect_stress.rs`'s
`the_l0_subset_passes_again_under_collect_on_every_allocation`, reproduced twice and confirmed passing
on the clean tree -- a third instrument, and one this task did not add. The claim worth making is the
one about coverage; "nothing else" was a claim about the whole workspace made from the two runs above,
and two runs cannot support it.

**Which instrument covers which side**, stated plainly because the constraint asks for it:

* the **refusing** side -- a caller the check refuses -- is covered by the corpus row, byte for byte
  on three descriptors, and by nothing else;
* the **allowed** side -- a caller the check permits, reaching the generated-accessor gap -- is
  covered by the in-crate loud test only. It cannot be a corpus row: the oracle answers `A` there and
  this crate refuses for want of the instance variable, which is a separate gap.

## F3 -- the guard's criterion, named

`63628e8ec` settled the rule: the 1% line is asked of **a task's own contribution**, which one
interleaved sitting yields, and the `pinned>head` ratio is accumulated drift since the pin, offered
as context.

**The row that carries the accumulated figure** is `scope = across_builds`, `build = pinned>head`,
`instrument = instructions:u`, in `bench-baselines/phase-5a-arms.tsv`. Read that way, this task's
widest is `strings`/`ir` at 1.009685, and it is not this task's: every `pinned>head`
`instructions:u` cell of my two guard sittings is identical to six decimal places to Task 12's for
the same axis and arm, except `alloc4c`/`tw`, where Task 12's first sitting read 1.004567 and both of
mine read 1.004566 along with Task 12's other two sittings. Each of those ratios is computed inside
its own sitting, so comparing them is not the cross-sitting per-pass subtraction this task's brief
rules out.

**This task's own contribution, measured directly.** `13-fixround-1-contribution` puts the pin, the
task's base revision and its changed source in one sitting over the six guard axes, the latter two
built under one path prefix so code placement is removed rather than bounded. `instructions:u` per
pass, changed over base:

| axis | arm | base `1a2ec9664` | changed | delta | ratio |
|---|---|---|---|---|---|
| `alloc4c` | ir | 3603.686508 | 3603.688494 | +0.001986 | 1.000001 |
| `alloc4c` | tw | 5324.690300 | 5324.690806 | +0.000506 | 1.000000 |
| `arith` | ir | 24062.575416 | 24062.571532 | -0.003884 | 1.000000 |
| `arith` | tw | 26490.779960 | 26490.777516 | -0.002444 | 1.000000 |
| `compound` | ir | 1914.480206 | 1914.480110 | -0.000096 | 1.000000 |
| `compound` | tw | 2739.480068 | 2739.479961 | -0.000107 | 1.000000 |
| `emptyloop` | ir | 375.999993 | 375.999941 | -0.000052 | 1.000000 |
| `emptyloop` | tw | 437.999983 | 438.000047 | +0.000064 | 1.000000 |
| `strings` | ir | 5421.525407 | 5421.524958 | -0.000449 | 1.000000 |
| `strings` | tw | 9129.518857 | 9129.518667 | -0.000190 | 1.000000 |
| `varlookup` | ir | 870.999983 | 871.000091 | +0.000108 | 1.000000 |
| `varlookup` | tw | 1773.000091 | 1773.000113 | +0.000022 | 1.000000 |

Every ratio is 1.000000 to six decimal places and the widest delta either way is under 0.004
instructions per pass, which is the null result the axis set predicts: **none of these programs sends
a message.** The send path's own contribution is F4's `dispatchclass` figure, and that is where this
task's cost is.

## F4 -- the send-path figure now has a committed referent

`bench-programs/dispatchclass.rex` in `d3758d520`: one class-method send per pass and nothing else in
the loop, `n = 4000000`, 0.87s under `build/bin/rexx` and inside that directory's documented 0.5-2s
band, output identical across repeated oracle runs. Registered in `rexx_bench::PROGRAMS` and in
`rexx-bench-suite`'s `AXES` as `Role::Loop`, which is what
`the_axis_list_covers_every_bench_program` and `the_benchmark_list_accounts_for_every_program` assert
in both directions.

**It does not replace `dispatch.rex` and does not relieve whoever unblocks it.** That program is the
same dimension through an *instance* and stays `Role::Blocked`, because its body needs `~new`; an
instance send resolves against the instance behaviour and this one against the class behaviour. The
program's own header says so.

**Why an axis rather than the report's hand-run `perf stat`:** `rexx-arms` runs each axis at two loop
bounds and derives per pass from the difference, which subtracts the fixed per-process offset that a
single-bound `perf stat` cannot separate.

`13-fixround-1-sendpath`, three builds in one sitting, seven rounds, `instructions:u` per pass:

| axis | arm | pinned | base `1a2ec9664` | changed |
|---|---|---|---|---|
| `dispatchclass` | ir | 6342.410531 | 6375.383679 | 6401.430375 |
| `dispatchclass` | tw | 6418.453913 | 6456.448576 | 6482.439211 |
| `emptyloop` | ir | 375.999994 | 376.000015 | 376.000027 |
| `emptyloop` | tw | 437.999983 | 438.000042 | 438.000014 |

**changed over base is +26.046696 per pass on the compiled engine and +25.990635 on the
tree-walker**, one class-method send per pass either way, and `emptyloop` in the same sitting moves by
less than 0.00003 -- the sendless control. That reproduces the report's hand-run +25.993/+26.000 on an
instrument that removes the process offset, and it is now a committed row against a committed
program. (`ca7008069`'s message truncates the first of those to `+26.046` where three decimals round
to `+26.047`; the figures here and in the TSV are the measured ones.)

As a ratio, changed over base on `dispatchclass` is 1.004086 (ir) and 1.004026 (tw), so **this task's
own contribution on the send axis is about 0.41%**, under the 1% line the plan now asks about.
Accumulated against the pin the same axis reads `pinned>changed` 1.009270 (ir) and 1.009938 (tw),
**Whoever adds the next instruction to the send path crosses 1% on this axis**, and that is worth
knowing before it happens rather than after.

**This said the accumulated figure was "the second-closest cell in the file to 1% after `strings`",
which is false.** Wider cells exist from earlier tasks: Task 11's `strings`/`tw` at 1.013710 and its
`alloc4c`/`tw` at 1.011324 and 1.011034, and Task 9's `strings`/`ir` at 1.010616. It is the closest
cell among **this task's own rows**, which is the true and narrower statement, and the forward-looking
point stands on the axis's own margin rather than on any ranking.

## F5 -- which of the two assertions catches which break

The field doc now says it: `Interp::record_access_scope` asserts the order of the push it is making,
and `dispatch::Interp::access_scope_of` checking its answer against a scan of the same rows is what
catches an order broken some other way. The review could not falsify the record-side assert from
inside this crate -- pushing the same row twice passes it and reversing the vector fires the
accessor's instead -- and that matches what it is: a guard on `ClassRegistry::next_method_id`'s
increment-only counter, which is another crate's property. The accessor's check is the demonstrated
live witness, and concern 5 in the section above now says so.

## F6 -- the doc comment

`Interp::special_methods`' field doc keeps the comparison, which justifies the shape **as it stands**
and is what the Task 1 ruling admits, and loses "four times over": the number of hash sites per pass
is a fact about code the tree no longer has, and striking it leaves the sentence saying the same
thing about the design a reader is about to rely on.

## F7 and F8 -- no action

F7 is the `::CONSTANT` divergence, pre-existing and already recorded at
`phase-4-exclusions.txt:3723`; I confirmed nothing in this round touches it. F8 is the staleness
paragraph naming its own commits, which is expected at report time and is now stale in the other
direction -- `b84685f44`, `6d12c033d`, `d3758d520` and `ca7008069` are this task's and are named
here.

## What I skipped, and why

Nothing in the review's list was skipped. The two plan-level fixes (F3's sentence in Global
Constraints, F4's promotion of `dispatch.rex` to 5b) are Moritz's and the plan's respectively:
`63628e8ec` landed the first, and the second is a note for 5b that `dispatchclass.rex`'s own header
and this section carry rather than something this task can land.

## The sitting question for this round

**Answered rather than assumed.** The only `src/` change in `d3758d520` is a row in
`dispatch.rs`'s `#[cfg(test)] mod tests` table, which is not compiled into `rexx-run` at all; the
corpus program, `corpus/phase-5a.txt`, `coverage.rs`, the `sourceline_oracle` expectation, the bench
program and the two bench registries are data, test code or programs the guard runs rather than code
it measures. `ca7008069` changes two doc comments. So no sitting is owed for the change itself -- and
three were run anyway, because F4 asked for a figure and F3 asked for the contribution the plan's
amended rule reads.

**The duplicate-key check** from `bench-baselines/README.md` prints `0` after every sitting in this
round.

## Gates, this round

From `rust/`, at `df799fee0`, the round's head. Statuses read unpiped.

| command | result |
|---|---|
| `cargo fmt --all --check` | exit 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --release --workspace` | exit 0 |
| `REXX_CORPUS_GATE=1 cargo test --release --workspace` | exit 0, corpus **160 of 160** |
| `REXX_CORPUS_GATE=1 memcap 8G cargo test --workspace --no-fail-fast` | exit 0, corpus 160 of 160 |
