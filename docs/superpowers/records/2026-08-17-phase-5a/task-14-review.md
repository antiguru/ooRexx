# Task 14 review -- required string values

**Spec compliance: PASS.**
**Task quality: CHANGES REQUESTED.**

Reviewed at `127fb74b8` over `df799fee0..127fb74b8`. The five gates were verified independently by the
controller and are taken as established (`fmt` 0, `clippy` 0, corpus 169 of 169, 98 `test result: ok`,
no `FAILED`, no `panicked`). Everything below is my own running, at that revision, from a fresh empty
directory with absolute paths and three descriptors read separately.

---

## Verdicts

### Spec compliance: PASS

The brief's "Done when" has three parts.

* **The `makeString` program agrees at rc 0 on both engines.** Re-run by me: oracle rc 0 `K says
  hello`, `REXX_ENGINE=ir` rc 0 `K says hello`, `REXX_ENGINE=tree-walker` rc 0 `K says hello`, empty
  stderr on all three.
* **Every context the `reqstr` section names has a program.** Nine corpus programs cover every context
  the phase can reach. Three contexts -- a command, an `ADDRESS` command, and `OPTIONS` -- have no
  corpus row because the instruction itself is refused; they are pinned instead by
  `a_reqstr_context_whose_instruction_is_another_phases_is_still_loud`, which asserts rc 120 with
  empty stdout and the message on stderr for each, with a `makeString` receiver in place. The
  owners check out: `OPTIONS` is 5c by the spec's amended **D31**
  (`docs/superpowers/specs/2026-08-17-phase-5-object-model.md:1133`), and the other two are Phase 7.
  An in-crate test as the only instrument is stated and is acceptable under the plan's own rule.
* **Both controls are recorded as run.** Both are, and control 1's blast radius is not collateral --
  see below.

### Task quality: CHANGES REQUESTED

One live behaviour defect of exactly the shape the task exists to remove, one instrument that does not
cover half of what it is claimed to cover, and a set of prose counts that do not trace to a command.

---

## Finding 1 (behaviour, must fix) -- `VALUE`'s second argument is converted where the oracle never converts it

`builtin::run` puts **every** supplied argument through `Interp::required_string_arguments`
(`rust/crates/rexx-exec/src/builtin.rs:786`), and `datatype::value` then reads position 2 out of the
converted list (`rust/crates/rexx-exec/src/builtin/datatype.rs:487`, `let newvalue = arg(args, 2);` --
`arg` reads `Args::values`).

The oracle does not convert that position at all. `BUILTIN(VALUE)`
(`/home/moritz/dev/repos/ooRexx/interpreter/expression/BuiltinFunctions.cpp:1810`) reads

```c
    RexxString *variable = required_string(VALUE, name);
    RexxObject *newvalue = optional_argument(VALUE, newValue);
    RexxString *selector = optional_string(VALUE, selector);
```

`optional_argument` is a bare `stack->peek` -- no `requiredStringArg`, so no `requestString`, so no
`makeString` and no NOSTRING. The new value is stored as the object the expression produced.

Two witnesses, both **rc 0 on both sides, empty stderr on both sides, differing on stdout alone**.

*What is stored.* Latch armed by a directive that installs a `makeString` on an unrelated class:

```rexx
r = value('a', .Array)
say 'cls=' a~class
::class K
::method makeString class
  return 'arm'
```

```
oracle       rc 0   cls= The Class class
crate/ir     rc 0   cls= The String class
crate/tw     rc 0   cls= The String class
```

*When the side effect happens.*

```rexx
say 'before'
r = value('a', .K)
say 'after'
say 'stored' a
::class K
::method makeString class
  say 'K asked'
  return 'converted'
```

```
oracle     rc 0   before / after / K asked / stored converted
crate/ir   rc 0   before / K asked / after / stored converted
crate/tw   rc 0   before / K asked / after / stored converted
```

The defect is gated on the latch and did not exist before this task: with no `makeString` installed
anywhere and no NOSTRING trap, `required_string_arguments` answers `None` and the same first program
prints `cls= The Class class` on both engines. So this is introduced by this task, invisible to the
corpus because no corpus program combines an armed latch with `VALUE(name, object)`, and it is the
silent-divergence shape the task's own mutation 3 exists to catch. A NOSTRING trap alone is enough to
arm it: with `signal on any` and no `makeString` anywhere, `value('a', .Array)` raises NOSTRING at the
call where the oracle stores the object.

It also falsifies two committed statements:

* `rust/corpus/lang/required_string_builtin_arguments.rex`'s header comment -- "An argument the
  builtin does not use is converted anyway" -- is stated as a property of builtin argument lists. It
  is a property of `SUBSTR`'s pad, which the oracle *does* fetch (`optional_pad`). `VALUE` position 2
  is a position the oracle fetches raw.
* the report's "Two structural consequences" -- "An unread argument is converted too ... so converting
  the whole list is right rather than merely convenient" -- generalises from that one measurement.

Fix shape: the conversion has to be per-position rather than list-wide, or `VALUE` position 2 has to be
exempt, and the pair needs a corpus program (an armed latch plus `value(name, object)`), because
nothing in the tree currently reddens for it.

## Finding 2 (prose, must correct) -- concern 1's residual is enumerable, and enumerating it is what found finding 1

The report says "I did not find such a builtin and I did not enumerate them, so this is a residual and
not a cleared question." The set is finite and lives in one file. I enumerated it.

Every builtin is a `BUILTIN(x)` block in
`/home/moritz/dev/repos/ooRexx/interpreter/expression/BuiltinFunctions.cpp` -- 81 of them, and
`/bin/grep -rln "^BUILTIN(" --include=*.cpp .` from `interpreter/` names that file and no other. Parsing
each block's `const size_t x_name = N;` declarations and then the order of its
`required_string`/`optional_string`/`required_integer`/`optional_integer`/`required_big_integer`/
`optional_big_integer`/`optional_pad` invocations gives, for every one of the 81, a **non-decreasing**
position sequence. So no builtin that exists today fetches position 3 before position 2, and the
ordering residual as the report states it is hypothetical rather than live.

Two things the same pass established that the report does not say:

* **Each position is converted at most once, whatever the fetch pattern.** `requiredStringArg` and
  `optionalStringArg` call `replace(position, newStr)` on the expression stack
  (`expression/ExpressionStack.cpp:142`, `:165`), so a second fetch of the same position finds a
  string. That matters for `XRANGE`, whose loop makes two passes over its arguments; I probed
  `xrange('a', .K, 'x', 'z')` and `xrange('alpha', .K)` with a saying `makeString` and both sides print
  `K asked` exactly once and agree byte for byte at rc 0.
* **One position in one builtin is never converted at all**, which is finding 1. The report's concern
  was aimed at ordering and the "never converted" case is the one that is live.

## Finding 3 (test/instrument, and prose) -- the debug latch check is blind to one of the latch's two arming routes

The latch's failure direction is a **lie, not a miss**: with the latch wrongly clear,
`required_string_value` returns the value the caller already holds, so limb 1 answers the old string
where a `makeString` would have answered a different one, and limb 3 renders where a NOSTRING trap
should have raised. Both are wrong answers, not slow ones. Release correctness therefore rests on the
completeness of the two arming sites -- `Interp::arm_reqstr_for` (`lib.rs:4049`, called from both
directive installers at `:3976` and `:4026`) and the `NOSTRING`/`ANY` arm of
`Interp::exec_condition_trap` (`run.rs:3952`) -- and not on the check, which only detects. Monotonicity
holds: those two sites are the only writes, the initialiser is `false`, and nothing clears it.

The report was never inverted, so I inverted it. Both mutations applied to one file at a time, debug
build, restored from a copy and rebuilt afterwards (`git status --porcelain -- crates` clean, and the
restored binary re-verified against both probes -- note that `cp -a` preserves mtime and cargo will not
rebuild without a `touch`).

*Mutation A, `arm_reqstr_for` made a no-op.* `say .k` with a class-side `makeString`:

```
thread 'rexx-interp' panicked at crates/rexx-exec/src/dispatch.rs:1792:9:
the required-string latch is off where the protocol would render differently
```

on **both** engines, rc 101. The check is live for the `makeString`-shaped route.

*Mutation B, the `NOSTRING`/`ANY` arming replaced by `if false`.* `signal on nostring name h; say .array`:

```
crate/ir debug   rc 0   The Array class / after
crate/tw debug   rc 0   The Array class / after
oracle           rc 0   trapped NOSTRING
```

A silent wrong answer, and the `debug_assert` says nothing. `required_string_latch_holds` compares only
the conversion limbs' bytes; its `None` arm takes `string_value_text`, and it never asks whether a
NOSTRING trap is armed -- its own doc says the third limb "cannot fire while the latch is off", which is
the thing under test.

So two claims are over-broad:

* `Interp::reqstr_armed`'s doc, "a third arming route added without setting this reddens the debug gate
  rather than answering the old string" -- true for a route that installs a `MAKESTRING`, false for one
  that arms a trap.
* concern 6, "The latch's debug check is the only thing that would catch a third arming route" -- for
  the trap route the check catches nothing and the corpus (`required_string_nostring.rex`) is the
  instrument.

The report's stand-in for an inversion -- "the first version of this check compared identities and
reddened eleven tests in the debug gate, which is the check working" -- is a bug in the check firing,
not a wrong latch being caught, and it says nothing about which arming routes the check covers.

Applying the "say what a check could not see" test: what the debug check would have done had the claim
been false, for a trap-shaped route, is *the same thing* -- nothing. That half of the claim is
unsupported.

One latent route worth recording rather than fixing: a user class that inherits a native `MAKESTRING`
by `::CLASS X SUBCLASS String` installs no `MAKESTRING` name, so `arm_reqstr_for` never fires. It is
unreachable in this phase -- `.MyStr~new` is `rexx-exec: method "NEW" of class "String" is not
implemented (Phase 5)`, rc 120, against oracle rc 163 -- so it is not a defect now.

## Finding 4 (prose) -- counts that do not trace to a command

* "**171 call sites name it**" (`to_text`, in the rejected-alternative argument). At `df799fee0`,
  `git grep -c "to_text(" -- 'rust/crates/*'` sums to **149**; at `127fb74b8`, **158**. No quoted
  command produces 171.
* "**It replaced `args: &[Option<ObjRef>]` in 82 signatures.**" The diff removes **79** lines matching
  `args: &[Option<ObjRef>]` and adds **79** matching `args: Args<'_>`. At head there are 80 parameters
  of type `Args<'_>` and 40 remaining occurrences of the old slice type.
* `rust/corpus/lang/required_string_builtin_arguments.rex` header: "**The three raises below** are one
  per message shape". The `SELECT` has four raising branches (`n = 1` through `n = 4`); three *message
  shapes* over four raises. The count as written is wrong about the file it sits in.

## Finding 5 (prose) -- the sitting: the own-contribution table is exact, the accumulated table drops a column

Re-derived from `rust/bench-baselines/phase-5a-arms.tsv`, task `14`, commit label `6f7899514`.

* Seven axes: `alloc4c`, `arith`, `compound`, `dispatchclass`, `emptyloop`, `strings`, `varlookup`.
  **`bench-rexxcps/rexxcps.rex` is absent**, which is correct for this task.
* Instrument column present, 287 `instructions:u` rows and 287 `cycles:u`. Every figure the report
  rests on is `instructions:u`.
* Own contribution, `per_pass` `instructions:u`, head over base: **every row in the report's table
  reproduces exactly**. `compound`/ir `1929.480073 / 1914.479906 = 1.007835` is the widest own
  contribution, and `varlookup`/tw `1754.000159 / 1773.000019 = 0.989284` is the widest movement of any
  kind. Both headline claims hold.
* The duplicate-key check from `bench-baselines/README.md` prints `0`. The pin's `sha256sum` is
  `141c3fa9...074b`, matching `PINNED.md:16`.
* The byte-identity claim is corroborated rather than reproduced: `127fb74b8` touches only
  comments in `rust/crates`, and both files keep their exact line count across it (`dispatch.rs` 4042,
  `run.rs` 12007), so a line-number-stable comment edit is consistent with an identical binary.

Two prose defects in the same section:

* **The accumulated table drops the `size` column and is labelled "widest per axis".** Every figure in
  it is the `small` row. For two axes the `large` row is wider: `arith`/ir `pinned>head` is `0.993016`
  (large) against the quoted `0.993066` (small), and `dispatchclass`/tw is `1.012133` (large) against
  the quoted `1.012110` (small). The direction understates rather than overstates, but the label does
  not describe the projection.
* **The `cycles:u` sentence mislabels which row it is quoting.** `0.585463` and `1.794598` are the
  `value_min`/`value_max` of `dispatchclass | pinned>base | across_builds | tw | small | cycles:u`
  (line 8528). The report calls them "`dispatchclass`/tw `cycles:u` for the **base** build alone". They
  are an across-builds ratio, not a build's own figure; the `scope` column is dropped. The point the
  sentence makes -- that a `cycles:u` figure from this bench is not a result -- stands on the numbers
  either way.

**The `emptyloop` control is valid in shape, and the conclusion drawn from it is stronger than it
supports.** It varies nothing about the change, and the premise that its per-pass path is untouched
checks out at the code, not just at the program: `Interp::accept_header_value`'s `Initial`, `To` and `By`
arms contain no `required_string_value` (`run.rs:7415`, `:7431`, `:7432`), and only `For`/`OverFor`
(`:7451`) and `Count` (`:7461`) call the protocol -- `do i = 1 to n` uses the first three.
`bench-programs/emptyloop.rex` has one `say` and it is outside the loop, so it is fixed cost that the
per-pass fit cancels. With `pinned>base` at exactly `1.000000` on both arms, the `-0.8%` on
`pinned>head` cannot be added work, so it is layout by elimination. What that establishes is that
**layout moves this sitting by up to about 0.8%**, which makes a `+0.78%` own contribution
uninterpretable rather than clean -- it does not establish that `compound`/ir's `+0.78%` *is* layout.
The report's "layout, and the sitting carries its own control for that" reads as the second. The
sentence that follows -- "The mechanism is inlining: the protocol's gate is `#[inline]` and sits inside
`Interp::say_evaluated`, which is itself `#[inline]` and reached from `step`" -- is an unverified
mechanism claim about an axis whose per-pass path contains no `SAY` at all; nothing was run at that
site. The observable ruling (direction rules out added work) is the part that holds.

## Finding 6 (prose) -- a wrong C++ citation, in the report and in a committed doc comment

The report and `Interp::required_string_arguments`'s doc comment both cite
`ExpressionStack::requiredStringArgument`, `expression/ExpressionStack.cpp:152`. The function is
`ExpressionStack::requiredStringArg` and it is at **`:142`**; `requiredStringArgument` does not exist
anywhere in the C++ tree. (`optionalStringArgument` does, in `runtime/MethodArguments.hpp` -- a
different function for method arguments, not builtin arguments.) The mechanism the doc describes is
right; the name and line are not.

## Finding 7 (prose) -- comments still name the size of a repo set, after the self-review pass removed others

Self-review finding 1 removed "a string, a number and an array" and "the two internal callers" for
naming the members or the count of a set the code can enumerate. Two of the same shape survive in the
same task:

* `Interp::reqstr_armed`'s doc: "**The protocol has two arming routes** and this latches on either."
  The count is precisely the thing at risk here -- finding 3 is about what happens when the set grows.
* `Interp::parse_strings`: "the **four sources** that build their own bytes never had an object to
  convert" (`parse_template.rs`).

## What checked out

* **No `unsafe` added**, no em-dashes in any added comment or corpus line, no `Op::Generic` and no
  promotion-to-follow. Both engines are exercised for every construct: `ir_dual` holds them to each
  other over the corpus and `corpus.rs` holds the default engine to the oracle.
* **Control 1 is not over-broad.** Exactly eight corpus programs install a `::METHOD makeString`
  directive, and they are exactly the eight the report names as differing at `161 of 169`.
  `required_string_default_name.rex` installs none and is correctly unaffected -- the pair works, and
  there is no collateral in the set.
* **Control 2 shows the pair rather than asserting it**: rc 0 for oracle and both engines, `--- stderr`
  empty on all three, stdout `wrong` against `K says hello`. It asserts rather than quotes the gate
  table C line for this control, where control 1 quotes it; the three-descriptor dump is the stronger
  evidence anyway.
* **Verdict-cell spelling is clean.** Nothing in the diff authors a verdict cell. The two in the report
  are harness output and both carry `loud=no`; neither divergence is a refusal. The three refused
  contexts are pinned as `(120, "", message)` -- all three descriptors move -- rather than as any
  `diverge-*` spelling.
* **The re-derivation deleted nothing.** `object_operand_tests` gains a module doc explaining that every
  case in it is the *left* operand, which `reqstr` names nowhere, and every case stays;
  `corpus/lang/message_send_argument_object_not_a_string.rex` is not in the changed-file set. So the
  brief's "a question to answer rather than to delete" is satisfied by answering.
* **Two grep-backed claims re-run and confirmed.** `requestStringNoNOSTRING()` matches only its
  declaration (`classes/ObjectClass.hpp:390`) and its definition (`classes/ObjectClass.cpp:1302`).
  `arith_general|apply_binary|compare_values` in `run.rs` matches four lines, all comments or docs.
* **The "no reachable site" claim for String's operator-method half is true.** `NATIVE_METHODS` gives
  `String`'s own scope `LENGTH`, `MAKESTRING` and `REVERSE`, all `Arity::Fixed(0)`, so no String method
  takes an argument. The operator path itself is right: `RexxString::stringComp`,
  `primitiveStrictComp` and `concatRexx` all open on `otherObj->requestString()`, which is the full
  protocol including NOSTRING, and that is what `apply_binary` now calls.

## Minor

* `6f7899514`'s subject is "Set the counter before the trap that reports it" and its body describes only
  the corpus programs, but it also carries a pure doc-comment hunk in `dispatch.rs` that the message
  does not mention.
* The `OPTIONS` refusal message says `(Phase 5)` where the owner is 5c. Coarser than the report's own
  claim, not wrong.
* The context table maps `DO`'s `exprr`/`exprf` to three arms (`Count`/`For`/`OverFor`) while the sitting
  section says "the two it did are `exprr` and `exprf`". Both readings are defensible; they are not the
  same sentence.
* "`REXX_CORPUS_GATE=1 cargo test --release --test ir_dual` is `9 passed; 0 failed`" sits directly under
  "ir_dual runs every corpus program on both engines", where 9 is a count of test functions, not of
  programs.

## What I did not verify

* The five gate commands (established by the controller).
* The `cmp` byte-identity of the sitting's `head` binary against a rebuild -- corroborated by the
  line-count argument above, not reproduced.
* The per-site "which operand an error names" and trace-line tables. Those are measured rows I spot-read
  rather than re-ran.
