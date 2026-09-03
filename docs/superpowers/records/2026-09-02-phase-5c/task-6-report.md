# Task 6 report -- table D: `::OPTIONS` and `::REQUIRES`

**BASE** `7cbabaf1b`, tree clean. **Landed** `82a8ae86d`, then `231a2bb71` for the LOSTDIGITS ruling.

## The count, before and after

Both figures from `REXX_PHASE_GATE=5c cargo test --release -p rexx-exec --test gate_table_d`:

| | 5c rows | not yet `agree` |
|---|---|---|
| BASE `7cbabaf1b` | 38 | **35** |
| this change | 38 | **2** |

The 33 that moved are every `::OPTIONS` row. The 2 that did not are `::REQUIRES LIBRARY` and
`::REQUIRES NAMESPACE`, and the measurement that blocks each is below.

No other phase's column moved: 5a 36/0, 5b 2/0, 7 1/1, `deferred-parse-error-rendering` 2/2, before
and after.

**Re-confirmed after the LOSTDIGITS follow-up**, on the same command: 5c is still 38 rows and 2 not
yet `agree`, and the corpus differential still reads **331 of 331 matching**.

## What was built

`::OPTIONS` was already parsed in full -- `rexx-parse`'s `optionsDirective` port produces a
`Vec<PackageOption>` with every keyword and every value -- and `directive_gap` refused the directive
at install. So this task is the exec half: a package-settings object, the three activation sites that
start from it, and the two condition raise sites that read it.

* **`crates/rexx-exec/src/options.rs`** (new) -- `PackageOptions` (numeric settings, trace mode,
  `NUMERIC INHERIT`, and a `ConditionSyntax`) and `ConditionSyntax` itself.
* **`Interp::package_options`**, a `HashMap<ProgramId, PackageOptions>` filled by
  `install_directives`' first walk. A program with no `::OPTIONS` gets no entry, so the lookup is an
  `is_empty` check on the ordinary path.
* **`Interp::start_from_package`**, called at the three sites that build an activation of a
  package's own code: the main body (`run_loaded`), a `::ROUTINE` (`run.rs`'s `Entered::Routine`)
  and a `::METHOD`/`::ATTRIBUTE` (`dispatch.rs`'s `enter_method_body`). An internal `CALL` inherits
  instead, through `Inherited`.
* **`Settings::set_digits`/`set_fuzz`/`set_form`** in `rexx-num`, and `reset_digits`/`reset_fuzz`
  regrown to take the package default -- a bare `NUMERIC DIGITS` resets to the package's, not to 9.
* **The two escalations.** `Interp::novalue_raised` and `dispatch.rs`'s
  `required_string_dispatch` ask `Interp::condition_raises_syntax` where nothing traps, and raise
  98.986 / 98.973.
* **`Interp::trace_package_invocation_entry`**, the second route into the `>I>`/`<I<` pair: the
  existing one needs a `TRACE` instruction as the body's first clause and `::OPTIONS TRACE` runs no
  instruction at all.

### The measured semantics, one row per thing that had to be got right

Every row is a program run from a fresh empty directory, three descriptors read separately, oracle
and both crate engines.

| what | measured |
|---|---|
| `DIGITS`/`FUZZ`/`FORM` | the package default for the main body, every `::ROUTINE` and every `::METHOD` -- `::options digits 12 fuzz 3 form engineering` gives `12 3 ENGINEERING` in all three |
| a bare `NUMERIC DIGITS`/`FUZZ`/`FORM` | resets to the **package** default: `numeric digits 30` then `numeric digits` answers 12, not 9 |
| the 33.1 check on a reset | `::options digits 12 fuzz 5`, `numeric fuzz 0`, `numeric digits 3`, then a bare `numeric fuzz` is 33.1 `("3") ... ("5")` |
| `NUMERIC INHERIT` | a `::ROUTINE` and a `::METHOD` start from the *call site's* live settings; the main body still starts from the package's |
| `NUMERIC NOINHERIT` | the default, and the same as writing nothing |
| `TRACE` | in force before the main body's first clause, and around a routine's and a method's own `>I>`/`<I<` pair; a `REPLY` announces the pair twice |
| `PROLOG`/`NOPROLOG` | accepted; **nothing here reads it** (see "what is accepted and inert") |
| the six condition options | an untrapped raise becomes a SYNTAX error; a `SIGNAL ON`/`OFF` of the same condition (or of `ANY`) turns the escalation off for the rest of that activation, so the trap wins |
| the `CALL`/`SIGNAL` split | `CALL ON ANY` leaves `NOVALUE`, `LOSTDIGITS` and `NOSTRING` escalating, and turns off only `ERROR`, `FAILURE` and `NOTREADY` -- `RexxActivation::trapOn`'s `signal &&` guards, measured both ways |
| the DIGITS/FUZZ cross-check | accumulates across `::OPTIONS` directives: `::options fuzz 5` then `::options digits 3` is the same 33.1 the two written together give |
| where that refusal sits | in the install walk in source order: it wins over a duplicate `::ROUTINE` pair after it, loses to one before it, and wins over a failing `::CLASS` on either side |

The trap-wins rule was the one this task got wrong first and then measured. The C++'s
`Activity::raiseCondition` escalates *before* the trap check, which reads as "SYNTAX preempts the
trap" -- and the oracle does the opposite. The mechanism is `RexxActivation::trapOn`
(`execution/RexxActivation.cpp:1547`): arming a trap **disables** the matching `::OPTIONS ... SYNTAX`
flag for that activation, so by the time a condition is raised the escalation is already gone. That
is why the flag lives on `Activation` here and not on the package.

## The 2 rows that did not move

Both are `::REQUIRES`, and both stay a `Loud` refusal (`rexx-exec: ::REQUIRES is not implemented
(Phase 5)`, rc 120) against an oracle that refuses them for its own reasons.

* **`::REQUIRES zzznolib LIBRARY`** -- oracle rc 158, `98.903 Unable to load library "ZZZNOLIB".`
  Making the row agree means answering 98.903, and answering it *correctly* means knowing which
  libraries load. Measured on this build: none of `rxmath hostemu orxncurses rxregexp orxclassic
  external_methods rexxutil regutil` loads (all 98.903), and `::requires rexx library` is rc 0
  because `REXX` is the built-in package. So a blanket 98.903 with a `REXX` exception would agree
  here and would be a loud wrong answer on a machine where a native package really is installed.
  **This is `::ROUTINE EXTERNAL`'s subject** -- loading a shared library -- and that row of this same
  table is filed under **Phase 7**. `::REQUIRES LIBRARY` is filed under 5c only because 5a's handover
  sentence put it there. Recommendation for the plan: re-file it under 7 beside `::ROUTINE
  EXTERNAL`, or own the library loader.
* **`::REQUIRES 'zzznofile.rex' NAMESPACE ns`** -- oracle rc 213, `43.901 Could not find file
  "zzznofile.rex" for ::REQUIRES.` The probe fails on the *file*, not on the namespace. Making it
  agree means implementing enough of `::REQUIRES` to look for the file, which means reproducing the
  oracle's search rules. Measured, the oracle finds a required file through **four** routes -- the
  current directory, the requiring program's own directory, `REXX_PATH` and `PATH` -- and appends an
  extension (`::requires 'dep'` finds `dep.cls`). A search narrower than that answers 43.901 for a
  file the oracle runs, which trades an honest refusal for a loud wrong answer. And `::REQUIRES` with
  the file *present* is package loading -- the prolog runs, public classes and routines are imported,
  `ns:name` resolves -- which is a phase of its own and is a declared exclusion
  (`docs/superpowers/plans/phase-4-exclusions.txt`).

**So this task did not implement `::REQUIRES` in any form.** Its two rows are named above with what
each needs.

## LOSTDIGITS -- ruled against, and closed

**Superseded 2026-09-03.** The first version of this task accepted
`::OPTIONS LOSTDIGITS SYNTAX` and let a program that lost digits answer rc 0.
The controller ruled that trades an honest loud refusal for a silent wrong
answer, which is the one direction this project does not go, and asked for a
loud refusal on the arithmetic path instead. That is what now ships, and the
paragraphs below are what building it measured.

### The cheap version is disqualified by the brief's own criterion

The refusal was specified as needing only the per-package flag -- "the refusal
only needs to know the flag is set". Measured, that fires on a program the
oracle answers cleanly, which is exactly the disqualifier the brief named:

```text
$ cat p.rex
numeric digits 3
say 1 + 1
::options lostdigits syntax

oracle   rc 0   stdout "2"
```

`::OPTIONS LOSTDIGITS SYNTAX` changes nothing for a program that never loses
digits, so a flag-only refusal would refuse `1 + 1`, `abs(1.23456789)`,
`1.23456789 == 1` and every other row the table below marks `Agrees`. **So the
refusal needs a real digit-count test after all** -- it is just a much smaller
one than the brief and my own earlier report both assumed, because the count is
already sitting in the operand this crate has converted.

### What the oracle actually checks, and what it does not

47 single-operation programs at `NUMERIC DIGITS 3`, each with the directive,
run from a fresh empty directory:

| raises 98.972 | does not raise |
|---|---|
| `+ - * / // %`, both operands, **left named first** | strict `==` `>>` |
| `**`'s **base** (its exponent fails 26.8 first) | `abs` `trunc` `format` `max` `sign` |
| prefix `+` and `-` | concatenation |
| non-strict `= > < >= <= \=`, **only where both operands are numeric** | a comparison that falls through to the string rule |
| a controlled `DO`'s `Initial`, `TO` and `BY` | a whole-number argument conversion (`substr`, `word`) |
| | an assignment or a `SAY` with no arithmetic |

Two boundaries worth having: the count is the **stored** digit count including
trailing zeros (`1.20 + 0` at DIGITS 3 does not raise, `1000 + 0` does), and
`DIGITS` at its default 9 leaves `1.23456789 + 0` alone -- so the directive is
inert until the precision is actually narrowed.

### Where this crate now stands: 46 of 47

Every case the oracle raises, this crate refuses at rc 120 naming LOSTDIGITS;
every case it does not raise, this crate answers and its three descriptors
match. **One residue**, and it is loud on both sides rather than silent:

```text
numeric digits 3 / do 123456789 / leave / end     with the directive
  oracle       rc 158  98.972
  this crate   rc 120 -> no; rc 230  26.2 "repetition count ... must be zero or
                        a positive whole number"
                        WITHOUT the directive, BOTH sides answer that same 26.2
```

A bare `DO n` converts its count rather than computing with it, so this crate
reaches the whole-number refusal first where the oracle's LOSTDIGITS preempts
its own. Different loud answers, and this crate's is the one it gives with or
without the directive.

### `CONDITION` was checked and has no problem

`::OPTIONS LOSTDIGITS CONDITION` is the default spelling: it turns the
escalation *off*. Measured, `numeric digits 3 / say 1.23456789 + 0 /
::options lostdigits condition` is rc 0 printing `1.23` on the oracle and on
both engines here. Nothing to refuse, and nothing refused.

### What the refusal does not cover, unchanged from before

`SIGNAL ON LOSTDIGITS` still arms nothing, so a program that traps the
condition still answers the rounded number where the oracle runs its handler.
That is deliberate rather than missed: arming a trap **disables** the
`::OPTIONS` escalation (`RexxActivation::trapOn`), so the refusal declines
exactly where the trap route takes over, and refusing there would be refusing
a program whose directive is no longer in force. It is the LOSTDIGITS
*condition* gap, reachable with no directive at all, and it is pinned as a
`Diverges` row so it goes red when someone closes it.

### The measured cost, which is not zero

**This is the figure the brief asked me to stop and report rather than
absorb.** `instructions:u`, interleaved round-robin over three binaries built
in isolated target directories on real disk, medians:

| program | engine | BASE `7cbabaf1b` | Task 6 `82a8ae86d` | with the gate |
|---|---|---|---|---|
| `arith.rex` | ir | 1.0000x | 1.0001x | **1.0026x** |
| `arith.rex` | tree-walker | 1.0000x | 0.9998x | **1.0019x** |
| `varlookup.rex` | ir | 1.0000x | 1.0000x | **1.0000x** |
| `varlookup.rex` | tree-walker | 1.0000x | 1.0000x | **1.0000x** |

So: **`::OPTIONS` itself costs nothing measurable** (Task 6's commit is 1.0001x
and 0.9998x against BASE), and **the arithmetic gate costs +0.26% (ir) and
+0.19% (tree-walker) of `arith.rex`, and nothing at all on `varlookup.rex`.**

It is small but it is real, not noise: over 12 interleaved rounds the gated
binary is above the ungated one in **12/12** rounds on ir and **11/12** on
tree-walker, where run-to-run spread within one binary is 0.18-0.42%. A first
shape that gated per *operand* rather than per *operator* cost +0.33% / +0.24%;
the committed one pays the gate once per operator and is the figure above.
`varlookup.rex` is 0.0000x because its `x = x + 1` never leaves the tagged
fast path, and that path already declines an operand wider than `DIGITS`
(`small_int_arith`'s own `within_digits` guard) -- so the check needed no site
there and got none.

**The trade, stated for the decision:** +0.2-0.3% on the arithmetic axis buys
the removal of a silent wrong answer. Reverting it is one commit if that is
not the call.

### A follow-up that is now much closer than my first report claimed

My earlier report argued the correct 98.972 was out of reach because it needed
detection on the hot path. Having built the detection, two things are true that
were not measured then:

* **The substitution is the operand's plain rendering.** Measured,
  `1.23456789e2 + 0` reports `Number 1.23456789E2 ...` -- the operand as `SAY`
  would print it, not the source text and not the rounded value. So the message
  needs `string_value_text` and nothing new.
* **Where the refusal fires, 98.972 is unambiguous.** Arming a trap clears the
  escalation, so no trap can be competing at that point, and there is no
  condition-versus-syntax ordering left to get wrong.

So converting each refusal into the oracle's own answer is a small change on
top of this one. Closing the **trap** route as well is the same three-way
`Interp::novalue_raised` already has -- trap first, escalation second,
otherwise carry on -- and would retire the whole gap. Neither is done here.

## What is accepted and inert, and why that is not the same defect

Four of the six condition options and one whole keyword are stored and never read today. Each is
inert for a stated reason, and the reason is what separates them from `LOSTDIGITS`:

| option | why nothing reads it | what happens to a program that would need it |
|---|---|---|
| `ERROR`, `FAILURE` | this crate raises neither condition, because `ADDRESS` and a bare command clause are both `Loud` Phase 7 refusals | refused loudly at the command, before any divergence |
| `NOTREADY` | stream I/O is a `Loud` refusal (`the LIBRARY REXX entry point "stream_init" is not implemented`) | refused loudly at the stream operation |
| `PROLOG`/`NOPROLOG` | its only reader is a required package's prolog, and `::REQUIRES` is a `Loud` refusal | refused loudly at the `::REQUIRES` |
| `LOSTDIGITS` | this crate never raises the condition, so the arithmetic path **refuses** instead | refused loudly at the operand that loses digits -- the section above |

`LOSTDIGITS` is the row that needed its own machinery, because its raise site
is silent rather than loud; the other three are already loud at the event.

`ERROR`, `FAILURE` and `NOTREADY` are wired through the same
`Interp::condition_raises_syntax(condition)` the two live ones use, so the escalation is already in
place for whoever makes those conditions raisable. **The handover that matters is 5d's**: the moment
`Stream` reads land, `::OPTIONS NOTREADY SYNTAX` needs a raise site that consults
`condition_raises_syntax` before it resumes, exactly as `required_string_dispatch` does for NOSTRING.

## Tests, and the mutation controls that show they can fail

* **A 35-row table** in `lostdigits_is_refused_where_the_oracle_raises_and_nowhere_else`, one row
  per operation, each marked `Refuses`, `Agrees` or `Diverges` -- the `Agrees` rows are the
  over-fire control and are the larger half.
* **12 corpus programs** under `corpus/lang/directive_options*.rex`, run by a new binary
  `crates/rexx-exec/tests/directive_options.rs` against the oracle on both engines, all three
  descriptors raw. The binary reads the directory rather than a committed list, so a program added
  later cannot be silently unrun. It is a binary of its own for the reason `variable_reference.rs`
  states -- 5c has no `corpus/phase-5c.txt` yet and creating one reddens
  `every_closed_phase_this_table_owns_rows_for_is_gated` -- and the flip task should move these lines
  into that file and delete the binary.
* **Four unit tests** in `options.rs`, each asserted over `ESCALATABLE` rather than over a list
  written beside it: that `ALL` writes all six and nothing else, that each condition writes only its
  own slot, that a trap clause clears the right ones for all six -- which is where the `CALL`/`SIGNAL`
  split is witnessed for `LOSTDIGITS` and `NOSTRING`, the two the corpus has no `CALL ON` program
  for -- and that `trapOff`'s extra `NOVALUE` guard fires.

**Nineteen mutations, each applied alone, rebuilt, and reverted from a copy** (never
`git checkout --`). All nineteen redden a test; the twelfth row is the only one the corpus binary
does not catch. The first twelve are the `::OPTIONS` machinery:

| mutation | caught by |
|---|---|
| `start_from_package` does not apply the numeric settings | `every_directive_options_program_answers_the_oracle` |
| the package `TRACE` is not applied | same |
| the routine `>I>` route removed | same |
| the NOVALUE escalation removed | same |
| the NOSTRING escalation removed | same |
| `trapOn`'s escalation-disable removed | same |
| `trapOff`'s escalation-disable removed | same |
| the `CALL`/`SIGNAL` split removed for **NOVALUE** (with NOSTRING) | same |
| an internal call does not inherit the escalation state | same |
| a bare `NUMERIC DIGITS` ignores the package default | same |
| `NUMERIC INHERIT` ignored | same |
| the DIGITS/FUZZ cross-check dropped | same |
| the `CALL`/`SIGNAL` split removed for **NOSTRING alone** | `options::tests::a_trap_clause_disables_the_escalation_its_own_condition_asked_for` -- the corpus binary stays green, because its only `CALL ON` witness reads an unset variable |

The other seven are the LOSTDIGITS refusal, all caught by
`lostdigits_is_refused_where_the_oracle_raises_and_nowhere_else`: the gate never armed; the
per-activation escalation check dropped; the digit-count test widened from `>` to `>=` (the
over-fire control); and each of the four sites removed in turn -- comparison, `DO` header, prefix,
`**` base.

**The `**` base site was found by a mutation that was not planned as one.** Consolidating the two
per-operand gates into one per-operator gate moved the check into the non-`Power` branch and
silently dropped it for `1.23456789 ** 1`; the `power-base` row went red on the next run. That is
the row table earning its place -- the shape was measured on the oracle before the code existed, so
the witness was there before the regression was.

**Three of the twelve were green on the first pass**, and each named a real hole in the witness
set rather than a redundant mutation:

* `trapOn`'s disable was invisible because a `SIGNAL ON NOVALUE` that fires is trapped either way.
  The shape that separates it is a read **inside the handler**, where the trap has been used up:
  `directive_options_novalue_trap_wins.rex` now has one.
* the `CALL`/`SIGNAL` split had no witness at all; `directive_options_call_on_keeps_it.rex` is it.
* internal-call inheritance had none either; `directive_options_internal_call.rex` is it.

### The sweep behind those tables

116 probe programs, oracle and both crate engines, three descriptors, from two fresh directories.
22 disagree with the oracle and every one is accounted for: 7 are `::REQUIRES` and its search-rule
probes, 5 are `ADDRESS` and bare commands (Phase 7, loud), 2 are streams (Phase 7, loud), 2 are
`Package~digits`-shaped readbacks (loud), 3 are parse-error rendering (the committed deferral, two of
them controls I wrote to check that `::options zzzjunk` renders like every other parse error), 2 are
LOSTDIGITS and 1 is the concatenation NOSTRING gap below. Measured with
`target/release/rexx-run` at sha256
`9ec1b1d990e68b636f7c3743e3e63d4338ae37523e9e6a7c1c9e2b4c19397f28`, rebuilt after the last mutation
revert and byte-identical to the binary the gates ran on.

## Other findings, none of them mine to fix

* **`.object~new || 'a'` does not raise NOSTRING here and does on the oracle.** Measured at BASE and
  now, `signal on nostring; o = .object~new; x = o || 'a'` is rc 0 printing `no nostring` here and
  `NOSTRING trapped` on the oracle -- a pre-existing silent divergence on the concatenation operand
  path. `say .array` reaches the protocol correctly, so this is one operand position and not the
  protocol. Unrelated to `::OPTIONS`; not filed anywhere I could find.
* **`Package~digits`, `~fuzz`, `~form` and `~trace` are the readback the oracle offers for these
  settings** and are `Loud` refusals here (`method "DIGITS" of class "Package" is not implemented`).
  They are table C `Package` rows, already `agree` through the `hasMethod` instrument, so no gate row
  asks for the bodies. Worth knowing for whoever writes the method-body spec: they are the natural
  witnesses for `::OPTIONS`.
* **`::options zzzjunk` is the parse-error-rendering deferral, not an `::OPTIONS` defect.** Oracle
  rc 231 with a full report; this crate rc 120 `rexx-exec: 25.924: Invalid subkeyword found.` --
  the same shape `::class q class` and an unterminated `DO` give, and the deferral
  `deferred-parse-error-rendering` already owns.

## What I did not do

* **`::REQUIRES`, in any form.** Both its rows are still red. Named above with the measurement.
* **The LOSTDIGITS condition.** The refusal covers the escalated route; `SIGNAL ON LOSTDIGITS`
  still arms nothing. Named above, with the two measurements that make closing it smaller than my
  first report claimed.
* **The oracle's own 98.972 in place of the refusal.** One change on top of this one, also named
  above. Not taken because the brief asked for the refusal and because a refusal has no message
  substitution to get wrong at four sites.
* **`do 123456789` still answers 26.2 where the oracle answers 98.972.** Loud on both sides, and the
  same answer this crate gives without the directive.
* **Only two benchmark axes measured.** `arith.rex` and `varlookup.rex`, `instructions:u` only --
  no cycles, no wall clock, and none of the other nine bench programs. Task 10 owns the axes and the
  pinned binary; these two were chosen because the change is on the arithmetic path and
  `varlookup.rex` is the control that should not move.
* **No `Package~digits` readback**, which is the reflection half of the same subject.

## Two process failures worth recording

Mid-task I measured an internal-`CALL` "divergence" that did not exist. `cargo test --release`
rebuilds the workspace's **binaries** as well as its test harnesses, so the mutation runs relinked
`target/release/rexx-run` from mutated source, and the probe script I ran afterwards used it while
`git status` was clean and `crates/` matched the backups byte for byte. Caught by the binary's
SHA-256: `bbcae08eaa37373ccc4d93ecbe4963cfac6ed809484f388a265dc0cbc632791e` where the correct build is
`73a6d4656c3a94a4651621635d8287866be5aa19ef4ee3ff62cde46bca6b55e3`. The brief's rule is right and is
narrower than it needs to be: **a mutation run is a revert**, and the rebuild is owed after it even
though nothing that looks like a revert happened. (It bit a second time, harmlessly, during the
follow-up: a probe ran against a binary predating a message-text fix, which changed only the words
in a refusal I was not reading.)

**Two builds of different revisions came out byte-identical**, and the reason was a shared
`CARGO_TARGET_DIR`. Extracting `7cbabaf1b` and `82a8ae86d` into separate source directories and
building both with one target directory produced `Finished in 0.02s` for the second and the same
sha256 for both binaries -- cargo considered the tree fresh. Had I not hashed them I would have
reported a perf comparison between a binary and itself, and it would have read as a clean 1.0000x.
**Separate revisions need separate target directories**, and the hash is what says whether they
got them.

## Scratch directories created (not deleted -- the controller sweeps)

* `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/ae5f3c99-11ad-4b29-9dfe-f3d991a1a398/scratchpad/task-6/`
  -- the probe programs and their three descriptors (`probe/`, `probe2/`), the gate logs, the
  `srcgen/` sourceline driver, and `backup-task6/` (the five source files copied before each
  mutation).

No build tree was created on real disk: every build reused `rust/target/`, and `df -h /tmp` read
19% used with 51 G free at the start.
