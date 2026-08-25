# Task 10 review: the send path -- `resolve`'s inputs, and D24's three surviving constraints

Reviewed `fc31fb82a..207756aae` (three commits: `64c78d0c6` roadmap, `b55a721d1` code, `207756aae`
sitting rows). Read-only on the checkout apart from this file. Everything below that says "measured"
or "ran" is my own run, not the report's.

## Spec compliance

**Issues found**, all in the "the code is right, what is said about it is not" and "the acceptance
does not reach the subject" classes rather than in the "a requirement is missing" class. Each of the
brief's four build items is present:

* `resolve` gained the sending side as one `Caller` value (`dispatch.rs:375`-`:410`, `:648`-`:658`),
  and `send_message` forwards it (`:942`-`:953`). The brief says "the caller's **scope**"; the
  implementation carries the caller's **receiver**, and that correction is right. `checkPrivate`
  reads `activation->getReceiver()` and the only scope it touches is the resolved method's own --
  verified line for line at `classes/ObjectClass.cpp:609`, `:612`, `:616`, `:617`-`:620`,
  `:622`-`:626`, `:628`, and `checkPackage` at `:659`, `:665`-`:669`, `:671`.
* `Primitive::SmallInt` is its own arm (`dispatch.rs:555`), folded to `String`'s behaviour at both
  consumers (`:618`, `:1366`).
* The receiver is a `CallContext` field (`lib.rs:3035`-`:3046`), filled by `enter_method_body`
  (`dispatch.rs:831`-`:845`) and read back for `MethodIdentity` and `SELF`.
* Selectors are interned by the parse (`rexx-parse/src/selector.rs`, `expr.rs:864`, `:891`-`:894`),
  and `ExprKind::Message.name` is a `Selector`.
* The roadmap sentence is amended at `docs/superpowers/plans/2026-07-27-rust-rewrite.md:492`-`:494`,
  and the spec's citation of `:492` still lands on it.

The three in-crate tests are ordinary `#[test]` in `src` modules with no gate environment variable in
them, so all three failures are structural as the brief requires. The sitting is recorded per axis on
`instructions:u`, and no axis reaches 1% (highest is `strings`/`ir`/small at 1.009685).

**Is the sitting adequate acceptance for a task whose subject is the hottest path?** No, and the
remedy was available at this commit. The report is right that none of the six axis programs sends a
message -- I re-derived that independently: `arith`, `compound`, `emptyloop`, `strings` and
`varlookup` contain no `~` and no `[` at all, and `alloc4c.rex`'s tildes are all inside its opening
block comment. But the report attributes the gap to `bench-programs/dispatch.rex` being rc 120 on
`~new`, which invites the reading that no send-exercising program can run yet, and that is false. A
native-method send loop and a user-defined class-method send loop both run at rc 0 on both engines at
HEAD. I measured the missing figure, interleaved pinned/head within each of three rounds, on
`perf stat -e instructions:u`:

| program | arm | pinned (median) | head (median) | ratio |
|---|---|---|---|---|
| `do 200000; t = t + s~length` | `ir` | 513,883,452 | 514,681,607 | 1.00155 |
| `do 200000; t = t + s~length` | `tree-walker` | 549,887,461 | 550,730,555 | 1.00153 |
| `do 100000; t = t + .K~m` (class method) | `ir` | 680,173,197 | 680,787,453 | 1.00090 |
| `do 100000; t = t + .K~m` (class method) | `tree-walker` | 688,347,900 | 689,166,423 | 1.00119 |

So the answer the brief actually wanted is green -- the send path is not measurably slower at HEAD
than at the pin, on either engine, well under the 1% bar. The finding is that the task did not
produce it and instead recorded "nothing I added is on any executed path", which is a weaker claim
than the evidence supported. This is plan-mandated: `global-constraints.md` fixes the guard at those
six axes, so the brief's letter was met.

**What I could not verify.** I did not re-run the suite, so the five gate exit codes, the 1818/1819
test counts, the corpus 143 of 143, and both tables' 5a row counts (C 135/117, D 36/7) are
unverified. I did not re-apply the three mutations, so 1815/3, 1817/1 and 1817/1 are unverified --
but I did verify the reasoning behind the third one's uniqueness claim, and all five assertion
citations. The staleness list holds 41 commits; I spot-checked 5 against `progress.md` rather than
all 41.

## Strengths

* **Every C++ citation I checked is exact to the line**, which is unusual enough to say: ObjectClass
  `:609`, `:612`, `:616`, `:617`-`:620`, `:622`-`:626`, `:628`, `:659`, `:665`-`:669`, `:671`;
  RexxActivation `:2342`-`:2349`, `:2344`-`:2347`, `:2348`; IntegerClass `:2066`; LanguageParser.cpp
  `:2269`, `:3309`, `:3320`-`:3321`, `:3369`, `:3391`; LanguageParser.hpp `:481`. The Rust-side ones
  hold too: `ir.rs:1037`-`:1040` and `:1060` (`Op::Message` with no `site`), `ir.rs:1509`
  (`struct CallSite`), `dispatch.rs:23`-`:27`, spec `:1123`.
* **The receiver-versus-scope correction is the right correction, made for the right reason.** The
  brief and D27's amendment both said "the caller's scope"; the C++ says receiver, and `:628` reads
  scope off the method, which `Resolution` already carries. That is a case of a task pushing back on
  its own brief with evidence.
* **Mutation 3's uniqueness reasoning is sound and I confirmed it.**
  `self_and_super_are_bound_before_the_bodys_first_instruction` (`dispatch.rs:1946`) sends `.K~m` to
  a method defined on `K`, so receiver and scope are the same object and a `SELF` re-derived from the
  resolution still prints `The K class`. The new test's `.J~m` with `m` at `K` separates them, and
  the oracle answers `The J class` at rc 0 with empty stderr -- measured.
* **Both self-corrections are genuinely fixed.** I keyed both row sets by (axis, arm, size) and
  compared `value_median` at `build=pinned>head`, `scope=across_builds`, `instrument=instructions:u`:
  exactly three of the 24 shared cells differ, and they are exactly `strings`/`tw`/small,
  `alloc4c`/`ir`/small and `strings`/`ir`/large. `alloc4c`/`ir`/large is identical in both, as the
  report now says. All five assertion citations resolve at HEAD: `selector.rs:121`,
  `dispatch.rs:1865`, `dispatch.rs:1915`, `expr/tests.rs:909`, `expr/tests.rs:931`.
* **Every budget figure reproduces from the TSV.** `strings`/`ir` per-pass pinned 5369.518229, head
  5421.517944, delta 51.999715, 1% of pinned 53.69518229, headroom 1.695467; the `9-fixround-1` pair
  is 5369.518137 and 5421.518032 for a delta of 51.999895, so the task's own contribution is
  -0.000180. The six-by-four ratio table matches the TSV cell for cell.
* **The pin passes its own staleness test, checked here rather than taken from the report.** sha256
  of `bench-baselines/pinned/rexx-run-15a1ffa98` is `141c3fa9...1d074b`, which is what
  `PINNED.md:16` records; `15a1ffa98` is an ancestor of HEAD; the crate-source commit list is 41
  commits, all recognisable as this plan's.
* **The stale-binary hazard was genuinely handled.** `target/release/rexx-run` has sha256
  `2fed9841...617ce`, the head sha the report names and not the mutant's, and I confirmed
  behaviourally: `.J~m` answers `The J class` on both engines, and `dispatch.rex` is rc 120 with
  exactly the quoted `method "NEW" of class "Object" is not implemented (Phase 5)`.
* **The interning's identity is soundly contained.** No `Selector` and no `Selector::same` appears
  anywhere in `rexx-exec` or `rexx-classes` (grepped both crates for the type name), so
  identity-by-allocation cannot leak into dispatch, and `Deref` is what carries the bytes across.
  The per-parse pool matches the oracle for the right reason and I checked the reason: `strings` is
  the global pool only during an image build (`parser/LanguageParser.cpp:788`-`:793`), and
  `globalStrings` is `OREF_NULL` at run time (`memory/RexxMemory.cpp:164`), so an `INTERPRET`
  fragment's pool really is fresh.
* `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` both exit 0.
  I forced a cold clippy pass by touching all thirteen changed files; the run re-checked `rexx-parse`
  and `rexx-exec`, and a second invocation finished in 0.04s with nothing to do, which is what says
  the first one did the work.

## Issues

### Important

**I1. `run.rs:5437`-`:5443`: `receiver: None` is wrong for `Entered::Label`, and the comment says it
is right.** `invoke_call_over` fills the convention's receiver with `None` for both of its arms, and
the new comment reads "Nothing this function enters was reached by a message send, so there is no
receiver to carry -- `RexxActivation::getReceiver`'s `OREF_NULL` for a frame that is not a method's
(`execution/RexxActivation.cpp:2348`)". The oracle does not do that for an ordinary internal call:
`RexxActivation::internalCall` passes the *caller's* receiver down
(`execution/RexxActivation.cpp:3313`, `return newActivation->run(receiver, name, ...)`), and
`RexxActivation::run` assigns it unconditionally at `:474`. It is `internalCallTrap` that passes
`OREF_NULL` (`:3343`), so the oracle distinguishes the two and this crate does not. Measured on the
oracle, with a control on each side:

* `CALL inner` inside a class method, `inner: return self~priv` with `priv` marked `private` -- rc 0,
  prints `private reached`. So that frame's `getReceiver()` is the method's receiver.
* the same private send from a `CALL ON ERROR NAME handler` handler in the same method -- rc 159,
  `Error 97.2: Object "The K class" cannot accept private message "PRIV" from this context`.
* the same private send from the top-level program -- rc 159, the same 97.2. This is the control that
  proves the check fires at all.

Why it matters: nothing reads `Caller::receiver` today, so no answer is wrong yet, but the brief's
stated reason for landing the signature in this task is "landing the signature here keeps Tasks 12
and 13 from rewriting each other". Task 13 will read this field, at one of its two production fill
sites it will read a value the oracle does not have, and the comment at that site tells the reader
the value is correct. A false comment at the fill site is worse than a missing one.

Fix: give `Entered::Label(_)` the enclosing convention's receiver and `Entered::Routine(_)` `None`,
citing `:3313` and `RexxCode.cpp:187` respectively; and either route a condition-trap handler call so
it gets `None` per `:3343`, or say in the comment that the trap case is not yet distinguished and
name the instrument that will catch it. Say explicitly that no instrument in this tree can catch a
wrong value here until an access scope reads it -- the corpus cannot, because the field is unread.

**I2. `dispatch.rs:381`-`:398` and `:648`-`:658`: `Caller`'s two absences are both `None` on types
that already mean something else, guarded only by an assertion that cannot fail.** Two separate
problems with one fix.

* `Caller::package` is `Option<crate::plan::ProgramId>` where `None` means "no activation is
  running". `Interp::package_objects` is `HashMap<Option<ProgramId>, ObjRef>` where `None` means
  "the `REXX` package" (`lib.rs:2354`, read that way at `environment.rs:627` and `:669`, rendered
  `The REXX Package` at `:637`). Confirmed: nothing today conflates them, because `Caller::package`
  has no reader outside the `debug_assert`. So the trap is real and confined exactly as the report
  says -- but it is confined by nothing except that no code has been written yet, and it is recorded
  only in an untracked report and a doc comment.
* The `debug_assert!(caller.receiver().is_none() || caller.package().is_some())` at `:652`-`:655`
  cannot be falsified by any path in this tree. `Interp::caller` (`:439`) is the only production
  constructor, and when it yields `Some(receiver)` the frame is a method activation, so
  `running_program()` is `Some` by construction; the test's `no_caller` (`:1614`) yields
  `(None, None)`. `caller.package()` has no other caller at all -- it is the assertion that keeps the
  accessor out of `dead_code`, which the report correctly identifies as its real job. Calling it "a
  tripwire" overstates what it can do: it is a contract statement compiled into a debug-only branch,
  not an instrument.

Fix both at once by making the absence unrepresentable: `enum Caller { NoActivation, In { package:
ProgramId, receiver: Option<ObjRef> } }`. That removes the assertion's subject rather than asserting
it, makes the "no activation" state impossible to compare against a `package_objects` key, and gives
`package` a reader in the match that consumes it. This project's own record is that a prose warning
about a type-level ambiguity does not hold; the type-level fix does.

**I3. Plan-mandated: the sitting cannot see this task's subject, and the missing measurement was
achievable.** Detailed under Spec compliance. The six-axis guard is fixed by `global-constraints.md`,
so the task complied; the acceptance the brief named ("the one task in the plan whose subject is the
shape of the hottest path") was not produced, and a send-exercising program runs at rc 0 on both
engines at this commit. My own interleaved measurement is above and comes back under 1% on all four
arms, so the outcome is green -- but it should have been the task's number, not the reviewer's. Fix:
record a send-exercising `instructions:u` comparison beside the sitting, interleaved, with the axis
programs' zero-send property as the control it already has.

### Minor

* **`dispatch.rs:288`-`:303`: the `SmallInt` arm's stated reason is not a property the change
  buys.** The doc says "the reason is the route rather than the answer: the tag alone decides this
  receiver's behaviour, without reaching the arena or testing the class-identity range". That was
  already true of the arm at base, which was `Decoded::SmallInt(_) | Decoded::Text(_) =>
  Ok(Primitive::String)` -- also tag-only, also no arena. Splitting it changes neither the route nor
  the answer: every consumer folds it (`:618`, `:1366`), and no consumer matches `Primitive::SmallInt`
  alone. The arm is a pure no-op today. That is not a defect -- D24 asks for it structurally and the
  report says plainly that no answer changes -- but the rationale in the comment should say what the
  split actually is, which is a distinct kind held ready for a distinct behaviour, not a cheaper
  route. I also settled the fidelity half: on the oracle a small-integer receiver is indistinguishable
  from the equivalent string across `~class~id`, `~class` object identity (`(i~class == s~class)` is
  1), `~length`, `~isA(.String)`, `~hasMethod` for names in and out of `String`, `~reverse`,
  arithmetic, `=` and `==`, and the two behaviours enumerate the same 150 methods; and our engine
  matches the oracle byte for byte on all of that on both arms. So the fold is not a divergence.
* **Report, budget section: "The difference is smaller than the pinned build's own run-to-run
  spread on this figure" is false on the numbers it cites.** The difference is 0.000180 and the
  pinned build's movement between the two runs is 5369.518229 - 5369.518137 = 0.000092, so the
  difference is about twice the spread, not smaller than it. It happens to be almost exactly the sum
  of the two builds' own movements (the head figure moved 0.000088 the other way), which is the
  statement that is true and is the one worth making. The conclusion -- nothing measurable was added
  -- is unaffected.
* **Report: `expr.rs:867` and `:933` are off by one.** The two `ExprKind::Message` constructors are
  at `crates/rexx-parse/src/expr.rs:866` and `:932`; `:867` and `:933` are both
  `target: Box::new(target),`. The substance holds -- those are the only two constructors in
  non-test parser code, and no third one exists in `rexx-exec` either.
* **Report: "all four `~`" is a line count presented as an occurrence count.**
  `/bin/grep -c '~' bench-programs/alloc4c.rex` really does answer 4, but that is four *lines*; the
  file holds eight `~`. All eight are inside the opening block comment, so the conclusion is right.
  This is the exact hazard the surrounding paragraph is warning about.
* **Report, gate table: the +1 test in gate 5 is not what is claimed.** "one more, because a
  `debug_assert` this workspace compiles out of every `--release` gate is what the debug run is for"
  -- a `debug_assert` adds no test. The extra test is `rexx-core`'s pre-existing
  `#[cfg(debug_assertions)] fn the_bytes_past_len_are_never_part_of_the_value`
  (`crates/rexx-core/src/bytes.rs:320`), which predates this task, so the 1818/1819 gap is not this
  change's.
* **Roadmap `:494`: "nothing reads it" is wider than the code.**
  `crates/rexx-classes/tests/behaviour_wiring.rs:807`-`:825` reads `version_at` six times.
  `dispatch.rs:25`-`:26`'s narrower "this module reads it nowhere" is the accurate form and the one
  the roadmap should have copied.
* **`dispatch.rs:840`-`:845`: the read-back's guarantee is not enforced.** The
  `.expect("the calling convention replaced directly above carries the receiver")` cannot fire, and
  the function argument `receiver` stays in scope and is still used two lines earlier at
  `super_scope_for(receiver, resolution)`. So "all name the same value by construction rather than by
  each taking its own copy of this function's argument" describes the current text, not a property
  the code enforces -- a later edit can still reach for `receiver`. Worth keeping, worth not
  overclaiming.
* **`docs/superpowers/specs/2026-08-17-phase-5-object-model.md:1179` now quotes roadmap text that no
  longer exists**, as an open item ("needs amending or a recorded reason it survives") that this task
  closed. Not this task's file to rewrite, but the ledger should note it so the next reader of that
  row does not re-open the work.

## Assessment

**Task quality: Needs fixes.** The four build items all landed, the citations are unusually accurate,
both self-corrections check out, and the three constraint tests are real and structural. What blocks
it is that the one value the task exists to make durable is wrong at one of its two production fill
sites and carries a comment asserting it is right (I1), that both of `Caller`'s absences are
overloaded `None`s held apart only by prose and a debug-only assertion that no path can falsify (I2),
and that the acceptance the brief called this task's real one was neither produced nor honestly
bounded, when a send-exercising measurement was available and comes back green (I3).
