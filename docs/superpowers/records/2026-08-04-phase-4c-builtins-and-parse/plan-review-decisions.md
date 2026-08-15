# Adversarial review of D-R and D-P

Reviewed: `docs/superpowers/plans/2026-08-04-phase-4c-builtins-and-parse.md`, sections
D-R (`:161`-`:188`) and D-P (`:190`-`:221`), plus the tasks that implement them
(Task 1 `:318`, Task 13 `:671`, Task 15 `:749`).

**Verdicts**

| | Verdict |
|---|---|
| D-R (`::routine` is 4c's) | **STANDS-WITH-CORRECTION** -- six defects, none fatal to the ruling, two of which turn a loud gap into a silent wrong answer if shipped as written |
| D-P (`+++` is Phase 7's) | **STANDS-WITH-CORRECTION** -- the ruling survives every attack and one probe strengthens it decisively; three factual corrections to the supporting prose |

## Method and safety

Every oracle invocation was wrapped exactly as instructed:

```
( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib \
  /home/moritz/dev/repos/ooRexx/build/bin/rexx ABSOLUTE_PATH )
```

with stdout, stderr and the unpiped exit status read as three separate
descriptors, never `2>&1`. Every probe ran from a directory I `mkdir`ed under
`.../scratchpad/dr-probes/<name>/`, never the scratchpad root -- this review is
precisely the one the root's stale `.rex` files corrupt, and probe R7 below
demonstrates the mechanism deliberately. Nothing in either repository was
modified; this file is the only write. No `NUMERIC DIGITS` above 1000, no
`select`/`when` segfault shape, no `.Package~new`. No symbol named `x` or `b`
appears before a quoted string.

---

# D-R -- `::routine` is assigned to Phase 4c

## Axis 1: is the contradiction real, or is the plan over-reading?

**Real. The plan is not over-reading, though it cites imprecisely.**

`2026-07-30-phase-4a-executor-design.md:71`, read directly:

> `Message`, `Guard`, `Reply`, `Forward`, **every directive**, and environment
> symbols beyond `.nil`, `.true` and `.false` are Phase 5's

`phase-4-exclusions.txt:88` (the row header):

```
  >I> / <I<   4c, deferred alongside ::routine dispatch itself.
```

and `:112`-`:116` of the same row:

> So a ::routine IS reachable in 4b for any non-builtin name; 4b declines to
> implement it because getting builtin-colliding names right needs 4c's table,
> and a wrong answer there would silently run the wrong routine rather than
> fail loudly.

and `:131`:

> THREE MEASURED WAYS A ::ROUTINE ACTIVATION IS NOT AN INTERNAL LABEL'S, which
> 4c will have to meet

Three separate sentences in the exclusions file place `::routine` dispatch in
4c: "deferred alongside", "needs 4c's table", "which 4c will have to meet". A
directive is assigned to 4c and to Phase 5 by two documents in the tree. The
contradiction is genuine and the ruling has to resolve it in one direction or
the other.

**Minor citation defect.** D-R at `:165` attributes the words *"which 4c will
have to meet"* to `phase-4-exclusions.txt:88`. Line 88 is the row header quoted
above; the phrase is at `:131`. Task 1 Step 6 (`:390`) says to "correct
`phase-4-exclusions.txt:88`'s row", which is the right target for the row as a
whole, so this is a prose-precision defect, not a scheduling one.

**The direction of the resolution is defensible but should be stated as such.**
The spec is the governing document and the exclusions file is a later
amendment ledger; the plan resolves in favour of the later, more specific,
measured document. That is the direction that risks the project's own
"phased work rots its own prose" failure. It is mitigated correctly: Task 1
Step 6 amends the *spec* rather than leaving the two to disagree, which is the
honest form of the amendment. No change needed.

**`:540`'s `QualifiedCall` row is unaffected**, and I checked this rather than
assuming it. The row's citation is the same "every directive" sentence, but its
independent reason ("namespaces come from `::REQUIRES`") survives the
`::ROUTINE` carve-out intact. Task 1 Step 6 already says to annotate it.

## Axis 2: is `::routine` separable from the rest of the directive machinery?

**Separable on the happy path. NOT separable on the failure path, and this is a
defect the plan does not record.**

### Probe R1 -- a `::routine` and a `::class` in one file

`/tmp/.../dr-probes/r1.rex`:

```rexx
say 'start'
call zorkolo
say 'after' result
exit
::routine zorkolo
  return 'ROUTINE-RAN'
::class foo
```

Oracle:

```
--- stdout
start
after ROUTINE-RAN
--- stderr
--- rc
rc=0
```

So a `::class` sitting beside a `::routine` costs nothing at run time, provided
the class installs. Probe I3 (below) repeats this with `::class` + `::method`
and gets `main ran / R / rc 0`.

### Probe R14 -- a `::routine` body is a full `CodeBody`

`/tmp/.../dr-probes/s4/r14.rex`:

```rexx
call zorkolo
say 'ret=' result
call quaffle
say 'ret2=' result
exit
::routine zorkolo
  call helper
  say 'routine got' result
  signal onward
  say 'skipped'
onward:
  return 'ZDONE'
helper:
  return 'HELPER'
::routine quaffle
  call zorkolo
  return 'Q+' || result
```

Oracle:

```
--- stdout
routine got HELPER
ret= ZDONE
routine got HELPER
ret2= Q+ZDONE
--- stderr
--- rc=0
```

A routine body carries its own label table, `CALL` into it works, `SIGNAL`
within it works, and routine-to-routine dispatch works **with no `::REQUIRES`**.
So the "does `::routine` drag in `::requires`" worry is answered: within one
file, no.

### The defect: directive INSTALLATION runs before `main`, and the crate does none of it

`/tmp/.../dr-probes/s6/i1.rex`:

```rexx
say 'main ran'
call zorkolo
say result
exit
::routine zorkolo
  return 'R'
::class foo subclass zzznotaclass
```

Oracle:

```
--- stdout
--- stderr
     7 *-* ::class foo subclass zzznotaclass
Error 98 running .../s6/i1.rex line 7:  Execution error.
Error 98.909:  Class "ZZZNOTACLASS" not found.
--- rc=158
```

`i2.rex`, same shape with `::requires 'zzz_no_such_file_here.rex'`:

```
--- stdout
--- stderr
     7 *-* ::requires 'zzz_no_such_file_here.rex'
Error 43 running .../s6/i2.rex line 7:  Routine not found.
Error 43.901:  Could not find file "zzz_no_such_file_here.rex" for ::REQUIRES.
--- rc=213
```

Note `say 'main ran'` never runs on either. Installation precedes the main body.

Now the same two programs through `rust/target/debug/rexx-run` **today**:

```
##### rust i1.rex
--- stdout
main ran
--- stderr
rexx-exec: routine "ZORKOLO" is not implemented (4c)
--- rc=120
##### rust i3.rex          (::class foo + ::method bar, both benign)
--- stdout
main ran
--- stderr
rexx-exec: routine "ZORKOLO" is not implemented (4c)
--- rc=120
```

The crate already prints `main ran` where the oracle prints nothing. It is
saved from being *scored* as a wrong answer only because the `call` then hits
`Loud::unresolved_call`. **Task 13 removes exactly that fallback.** After Task
13, `i1.rex` gives `main ran / R / rc 0` in Rust against `Error 98.909 / rc 158`
on the oracle -- a silent divergence, where today there is a loud one.

I verified there is no existing guard: `rexx-exec` reads `program.directives`
in exactly one place, `activation.rs:414` inside `body_of`
(`grep -rn "DirectiveKind" crates/rexx-exec/src/*.rs` returns three hits, two of
them comments). Nothing installs, validates, or refuses a directive.

**What the plan should say.** Task 13 gains a step: *before* dispatching to a
`::routine`, fail loudly if the program contains any directive other than
`::ROUTINE`. The message keeps `owned_message`'s shape with owner `Phase 5`, so
`loud.rs`'s `ends_with` contract still holds. Without it, D-R's own stated
motive -- "a wrong answer there would silently run the wrong routine rather than
fail loudly" -- is defeated one level up, by the program running at all.

## Axis 3: the variable-pool claim, and what else differs

### The claim is correct

`run.rs`'s `exec_procedure` (read at `:1948`-`:2013`), the three load-bearing
lines:

```rust
let slot = self.slot_of(&name);                     // index in the CALLER's plan
...
let len = self.roots.frame_len(outer);
let inner = self.roots.push_slots(len);
for (_, slot, target) in &bindings {
    self.roots.alias_slot(inner, *slot, *target);   // same index in the CALLEE's frame
}
...
let plan = Rc::clone(&self.activation().plan);      // callee reuses the caller's plan
```

The alias is installed at *the caller's slot index* into the callee's frame, and
the callee keeps the caller's `Plan`. That is only sound because an internal
label's activation runs the same `CodeBody`. A `::routine` has its own
`CodeBody`, hence its own `Plan` from `plan_for`, hence a different name-to-slot
map, and the identity fails. The plan's claim, including the "silently wrong"
characterisation, is right.

### The claim is INCOMPLETE -- three more things differ, and the internal-call path inherits all three

`run.rs:3304`-`:3313`, the internal-call setup, inherits four things from the
caller:

```rust
let plan = Rc::clone(&caller.plan);
let frame = caller.frame;
let settings = caller.settings.clone();
let trace_mode = caller.trace_mode;
let extra = caller.extra.clone();
let traps = caller.traps.clone();
```

Measured, a `::routine` inherits **none** of `settings`, `trace_mode` or `traps`.

**Probe R11 -- NUMERIC and ADDRESS are not inherited, and do not leak back**

```rexx
numeric digits 5
numeric fuzz 2
numeric form engineering
address 'ZORKENV'
call zorkolo 'A1', 'A2'
say 'ret=' result
say 'caller digits=' digits()
exit
::routine zorkolo
  say 'digits=' digits() 'fuzz=' fuzz() 'form=' form()
  say 'address=' address()
  say 'argc=' arg() 'a1=' arg(1) 'a2=' arg(2)
  numeric digits 12
  return 'RET'
```

```
--- stdout
digits= 9 fuzz= 0 form= SCIENTIFIC
address= sh
argc= 2 a1= A1 a2= A2
ret= RET
caller digits= 5
--- stderr
--- rc=0
```

The routine sees the package defaults (`9`/`0`/`SCIENTIFIC`) and the initial
address environment (`sh`), not `5`/`2`/`ENGINEERING`/`ZORKENV`. Its own
`numeric digits 12` does not reach the caller. Arguments *do* cross.

This one matters for scheduling as well as correctness: `ADDRESS` environment
tracking is Task 9's and NUMERIC is 4a's, so Task 13 must reset a field two
other tasks own. Task 13 already sits after Task 9, so the ordering is fine --
but the requirement is unwritten.

**Probe R12 -- `trace r` does not cross (confirms the plan's third fact)**

```rexx
trace r
say 'caller-1'
call zorkolo
say 'caller-2'
exit
::routine zorkolo
  say 'routine-1'
  zz = 3 * 3
  return 'R'
```

```
--- stdout
caller-1
routine-1
caller-2
--- stderr
     2 *-* say 'caller-1'
       >>>   "caller-1"
     3 *-* call zorkolo
       >>>   "R"
     4 *-* say 'caller-2'
       >>>   "caller-2"
     5 *-* exit
--- rc=0
```

**Probe R15 -- condition traps are not inherited (discriminating form)**

R13 (a caller trap firing on a division by zero inside the routine) cannot tell
"the trap was inherited and its label was missing" from "the condition
propagated to the caller". R15 puts a label of the trap's own name *inside the
routine* so the two hypotheses predict different bytes:

```rexx
signal on syntax name shared
call zorkolo
say 'never'
exit
shared:
  say 'CALLER trap, sigl=' sigl
  exit 0
::routine zorkolo
  say 'in routine'
  zz = 1 / 0
  return
shared:
  say 'ROUTINE trap, sigl=' sigl
  return
```

```
--- stdout
in routine
CALLER trap, sigl= 2
--- stderr
--- rc=0
```

`CALLER`, at the caller's own `call zorkolo` line. The trap was not inherited;
the condition propagated. (R13, the non-discriminating version, is retained in
`/tmp/.../dr-probes/s3/` and agrees.)

**What the plan should say.** D-R's "Three measured facts" becomes six, and
Task 13 Step 2 gains: *`Activation::nested`'s `Inherited { settings, trace_mode,
traps }` must be constructed from package defaults for a `::routine`, not cloned
from the caller.* An implementer who copies the internal-call setup and only
fixes the pool gets three wrong answers instead of one, and two of them
(`digits()`, `address()`) are silent.

## Axis 4: resolution order -- two measured omissions

### What the plan says

Task 13 Step 3 (`:690`): "Internal label, then builtin, then `::routine`, then
Error 43.1."

### Probe R2 -- internal label beats `::routine`; builtin beats `::routine`

```rexx
call zorkolo
say 'result=' result
call max 1, 9
say 'maxresult=' result
say 'fn=' zorkolo()
exit
zorkolo:
  return 'INTERNAL'
::routine zorkolo
  return 'ROUTINE'
::routine max
  return 'ROUTINE-MAX'
```

```
--- stdout
result= INTERNAL
maxresult= 9
fn= INTERNAL
--- stderr
--- rc=0
```

Both hypotheses were discriminable: `INTERNAL` vs `ROUTINE` and `9` vs
`ROUTINE-MAX` are different bytes. The plan's order is right for an unquoted
symbol target, in both the `CALL` and the expression form.

### Probe R3 -- the unresolved case, from a clean directory

```rexx
call zorkolo
say 'never'
```

```
--- stdout
--- stderr
     1 *-* call zorkolo
Error 43 running .../r3.rex line 1:  Routine not found.
Error 43.1:  Could not find routine "ZORKOLO".
--- rc
rc=213
```

`43.1`, rc `213` -- matching the brief's stated clean-directory figure and not
the scratchpad-root figure.

### Omission 1 -- a QUOTED target skips step 1 but still reaches step 3

**Probe R5:**

```rexx
call 'ZORKOLO'
say 'quotedupper=' result
call 'zorkolo'
say 'quotedlower=' result
call 'MAX' 1, 9
say 'quotedmax=' result
exit
zorkolo:
  return 'INTERNAL'
::routine zorkolo
  return 'ROUTINE'
::routine max
  return 'ROUTINE-MAX'
```

```
--- stdout
quotedupper= ROUTINE
quotedlower= ROUTINE
quotedmax= 9
--- stderr
--- rc=0
```

Three findings in one output. A quoted target (a) skips the internal label --
`ROUTINE`, not `INTERNAL`; (b) still finds the `::routine`; and (c) still loses
to the builtin. So there are **two** orders, not one:

* `CallTarget::Symbol`: label -> builtin -> `::routine` -> external -> 43.1
* `CallTarget::Literal`: builtin -> `::routine` -> external -> 43.1

The crate's `search_labels: bool` already encodes the difference, so an
implementer may get this right by accident, but the plan states one order for
both. It also silently invalidates the exclusions file's `:534`-`:536` claim
that `CallTarget::Literal`'s "measured answer" is 43.1 -- that measurement was
taken with no `::routine` in the file.

### Omission 2 -- the external search sits between `::routine` and 43.1, and Step 3 turns a loud gap into a silent wrong answer

**Probes R6/R7**, run from `/tmp/.../dr-probes/ext/` which contains
`zorkolo.rex` = `return 'EXTERNAL-FILE'`:

`r6.rex` (a `::routine zorkolo` present):

```
--- stdout
withroutine= ROUTINE
--- rc=0
```

`r7.rex` (no `::routine`):

```
--- stdout
noroutine= EXTERNAL-FILE
--- rc=0
```

The `::routine` beats the external file, and the external file beats 43.1. The
plan's "then Error 43.1" is true only in a directory with no matching file.

This is not academic. Task 13 Step 3 says **"Replace `Loud::unresolved_call`
with a real 43.1 raise."** `eval.rs:507`-`:511` argues in advance against
exactly that:

> 4b has not built the builtin/external steps that answer would need to tell
> "not a label" apart from "not anything", so this stays the same loud `4c`
> fallback ... not a fabricated 43.1.

4c builds the builtin step and the `::routine` step. It does **not** build the
external step. So after Task 13 the crate still cannot tell "not anything" from
"not anything I search", and `call zorkolo` next to a `zorkolo.rex` yields
Rust 43.1 rc 213 against oracle `EXTERNAL-FILE` rc 0.

There is also an unresolved in-tree disagreement about who owns that step,
which D-R does not notice:

* `lib.rs:479`-`:481` (`Loud::unresolved_call`'s doc): "the next steps are the
  builtin table and then external resolution, and **both are 4c's**"
* `eval.rs:483`-`:484`: "internal routine first (4b), builtin second (4c),
  **external third (Phase 7)**"
* `phase-4-exclusions.txt`: no owner row for external routine resolution at all
  (`grep -n external` returns only `:61` (`VALUE`) and `:514`).

**What the plan should say.** Either (a) keep `Loud::unresolved_call` and let
Task 13 raise 43.1 *only* after a `::routine` miss when the program is known to
have no external candidate -- which is not buildable in 4c -- or, preferably,
(b) raise 43.1 as planned **and** add an EXCLUSIONS row for "external routine
resolution", owner Phase 7, carrying the R6/R7 transcripts, plus a corpus rule
that no 4c corpus program may call a name matching a file in its own directory.
Task 1 is the natural place for the row, since it is already editing the
exclusions file. And the `lib.rs` / `eval.rs` disagreement about the owner has
to be settled in the same commit, since `unresolved_call`'s doc is about to be
rewritten anyway.

### Omission 3 -- routine name lookup is case-insensitive, including for a quoted directive name and a quoted call

**Probes M1/M2/M3**, each with `::routine 'MiXeD'` at the end:

| call form | result |
|---|---|
| `call 'MiXeD'` | `MIXEDROUTINE` |
| `call MiXeD` | `MIXEDROUTINE` |
| `call 'MIXED'` | `MIXEDROUTINE` |

`DirectiveParser.cpp:2277` (`RexxString *internalname = commonString(name->upper());`)
confirms the mechanism. This is **not** the convention `CodeBody::labels` uses
-- that map is documented (`ast.rs:598`-`:608`) as "upcased for a symbol label,
verbatim for a literal one". An implementer who reuses the label key convention
for the routine table breaks `call 'zorkolo'`. Worth one line in Task 13.

## Axis 5: cost -- task-sized, and the "package object" is one field

**Does `::requires` travel with it?** No -- R14 shows routine-to-routine
dispatch inside one file needs nothing. `PUBLIC` on a `::routine` only matters
across a `::REQUIRES` boundary.

**Do namespaces travel with it?** No -- `ns:name(...)` is `ExprKind::QualifiedCall`,
Phase 5's for an independent reason.

**How much is "the package object"?** `traceEntryOrExit`
(`RexxActivation.cpp:3678`-`:3712`) needs exactly two substitutions for the
routine form:

```cpp
info = new_array(context->getExecutable()->getName(),
                 context->getPackage()->getProgramName());
```

the routine's name and the program's path. The name is already in the AST. The
path is *not* on `Interp` today -- `execute` (`lib.rs:1694`) takes `path: &str`
as a local and uses it only on the `Failure::Raised` arm, to build `ClauseSite`.
So the cost is **one `String` field on `Interp`, set in `execute`**. That is the
whole of "the package object" for `>I>`/`<I<`. The plan's cost position is
correct and can be made concrete.

**Remaining machinery**, all present: `plan_for` already takes a `BodyKey` +
`&CodeBody` + `&SymbolTable`; `body_of` (`activation.rs:410`) already resolves
`Some(i)` to `DirectiveKind::Routine`'s body; `roots.push_slots` /
`Activation { owns_frame: true }` already exist for `PROCEDURE`. The new code is
a directive scan, a defaults-constructed `Inherited`, and two trace lines.

**Verdict on cost: task-sized.** One task, provided the four additions above
(loud non-`::ROUTINE` directives, defaults not inheritance, the external-search
exclusion row, the case-insensitive key) are written into it.

**Two file-list omissions, both concrete.** Task 13's file list is
`src/run.rs`, `src/plan.rs`, `src/lib.rs`, `src/error.rs`,
`tests/trace_oracle.rs`, `corpus/phase-4c.txt`. Missing:

1. `crates/rexx-exec/tests/coverage.rs`. It carries
   `assert_program_has_no_directives` (`:150`), applied to every program in the
   subset (`:632`), whose message is *"has a `::` directive, which this walker
   does not follow into"*. D6 says the harnesses read the union of all three
   subset files, and Task 15 Step 4 adds `phase-4c.txt` -- at which point the
   `::routine` witness program Task 13 commits makes this assertion fire.
   Task 15 lists `coverage.rs` but says nothing about this assertion.
2. The same walker **does not descend into directive bodies**, so once routine
   bodies execute, criterion 5's coverage silently under-counts anything whose
   only corpus occurrence is inside a `::routine`. Either widen the walker or
   record the limit.

**Cross-check on benefit, since D-R's reason 3 is weak.** Reason 3 ("Phase 4
closes with a construct the oracle runs at rc 0 still failing loudly") proves
too much -- it is equally true of `.array~new` and of `::class`, and would drag
Phase 5 into 4c if taken seriously. The load-bearing reasons are 1 and 2. As
supporting evidence I measured how much of the L1 table `::routine` unblocks:
`corpus/keyword-exempt.txt` (850 lines) contains 2 rows whose derived blocker
names a routine, and their names are `DIGITS` and `CHARIN` -- both builtins, not
`::routine`s. So `::routine` unblocks approximately nothing in `base/keyword`.
Its value is the prefix coverage (`>I>`/`<I<`, two of the three prefixes 4c
claims) and the resolution-chain boundary, not L1 rows. D-R should say that
rather than lean on reason 3.

## Axis 6: is `>I>`/`<I<` reachable without `::options`?

**Yes, and more cheaply than the plan supposes. `::options` is genuinely not
needed.**

**Probe R8 -- the routine's own `trace l`:**

```rexx
say 'caller-start'
call zorkolo
say 'caller-end'
exit
::routine zorkolo
  trace l
  say 'routine-body'
  return 'R'
```

```
--- stdout
caller-start
routine-body
caller-end
--- stderr
       >I> Routine "ZORKOLO" in package ".../tl/r8.rex".
       <I< Routine "ZORKOLO" in package ".../tl/r8.rex".
--- rc=0
```

**Probe R10 -- `trace r` reaches it too, which the plan does not say:**

```rexx
call zorkolo
exit
::routine zorkolo
  trace r
  yy = 1 + 1
  return 'R'
```

```
--- stdout
--- stderr
       >I> Routine "ZORKOLO" in package ".../tl3/r10.rex".
     5 *-* yy = 1 + 1
       >>>   "2"
     6 *-* return 'R'
       >>>   "R"
       <I< Routine "ZORKOLO" in package ".../tl3/r10.rex".
--- rc=0
```

`traceEntry` (`RexxActivation.cpp:3624`-`:3640`) explains it: the
`earlyTraceEntry` gate is `nonDynamicTracingLabels() && isMethodOrRoutine()`,
and its own comment says *"TRACE A or TRACE I or TRACE L or TRACE R"*. So any
`::routine` whose first instruction is a non-dynamic `trace a/i/l/r` emits both
lines.

This matters more than a footnote. A 4c corpus witness for `PARSE` or a builtin
inside a `::routine` under `trace r` -- the natural shape for a differential
program -- emits `>I>`/`<I<` whether or not anyone intended it. `::routine` and
these two prefixes are not merely scheduled together; they are *coupled*, which
strengthens D-R.

**Probe R9 -- the caller's `trace l` does not cross (confirms the plan):**

```rexx
trace l
say 'caller-start'
call zorkolo
call inner
say 'caller-end'
exit
inner:
  say 'internal-body'
  return
::routine zorkolo
  say 'routine-body'
  return 'R'
```

```
--- stderr
     7 *-*   inner:
--- rc=0
```

Only the internal label echoes. Nothing from the routine.

**What the plan should say.** Task 13 Step 5's bullet becomes: *`::options trace
labels` is one route and is out of scope; the reachable in-scope route is the
routine's own non-dynamic `trace a`, `trace i`, `trace l` **or** `trace r` as
its first instruction (after an optional `EXPOSE`), per `traceEntry`'s
`earlyTraceEntry` gate.* Recording it in the exclusions file, as `:718` asks,
is then a one-line note that `::options` is not needed rather than a limitation.

---

# D-P -- `+++` is Phase 7's, not 4c's

## Axis 1: is there a third emission site?

**No third `TRACE_PREFIX_ERROR` site. But there are two more producers of a
line beginning `+++`, and the plan's own quoted evidence comes from one of
them.**

### Exhaustive search, four ways

By enum name:

```
$ grep -rn "TRACE_PREFIX_ERROR" --include=*.cpp --include=*.hpp --include=*.h interpreter/
interpreter/execution/RexxActivation.hpp:93:        TRACE_PREFIX_ERROR    ,         //  1
interpreter/execution/RexxActivation.cpp:3570:  "+++",                               // TRACE_PREFIX_ERROR
interpreter/execution/RexxActivation.cpp:4024:    buffer->put(PREFIX_OFFSET, trace_prefix_table[TRACE_PREFIX_ERROR], PREFIX_LENGTH);
interpreter/execution/RexxActivation.cpp:4468:            traceValue(rc_trace, TRACE_PREFIX_ERROR);
```

By the literal, across the whole tree (build/ and test/ excluded):

```
interpreter/messages/RexxErrorMessages.h:725:    MESSAGE(Message_Translations_debug_error, "+++ Interactive trace.  Error")
interpreter/messages/RexxErrorMessages.h:726:    MESSAGE(Message_Translations_debug_prompt, "+++ Interactive trace. \"Trace Off\" to end debug, ENTER to continue. +++")
interpreter/execution/RexxActivation.cpp:3570:  "+++",                               // TRACE_PREFIX_ERROR
```

**By prefix passed through a variable** -- the case the brief warns a name-grep
cannot find. Every function that indexes `trace_prefix_table[]` with a
parameter: `traceEntryOrExit`, `traceValue`, `traceTaggedValue`,
`traceOperatorValue`, `traceCompoundValue`, `traceClause`, and the inline
`traceIntermediate(RexxObject*, TracePrefix)`. I enumerated their callers rather
than counting them:

```
$ grep -rn "traceIntermediate(\|traceClause(\|traceResultValue(\|traceEntryOrExit(" \
    --include=*.cpp --include=*.hpp interpreter/ | grep -v RexxActivation.hpp:3..
interpreter/classes/IntegerClass.cpp:1915:      ... TRACE_PREFIX_LITERAL);
interpreter/classes/NumberStringClass.cpp:3905: ... TRACE_PREFIX_LITERAL);
interpreter/classes/StringClass.cpp:2080:       ... TRACE_PREFIX_LITERAL);
interpreter/instructions/ParseTrigger.cpp:285:  ... TRACE_PREFIX_DUMMY);
interpreter/execution/RexxActivation.cpp:716,1484,3665: TRACE_PREFIX_INVOCATION[_EXIT]
interpreter/execution/RexxActivation.cpp:4453:  traceClause(current, TRACE_PREFIX_CLAUSE);
interpreter/instructions/AddressInstruction.cpp:158 / CommandInstruction.cpp:85: traceResultValue
```

Every one passes a literal, and none passes `TRACE_PREFIX_ERROR`. The
`RexxActivation.hpp:339`-`:371` inline wrappers all bind their own literal. So
**two sites for the enum, confirmed by four instruments.** D-P's central factual
claim survives.

### But the string `+++` has four producers, and the plan mis-attributes two lines

1. `RexxActivation.cpp:4024` -- `traceSourceString()`, enum, interactive-debug entry banner
2. `RexxActivation.cpp:4468` -- command `RC(n)`, enum
3. `RexxActivation.cpp:4237` -- `processTraceInfo(..., Message_Translations_debug_prompt, TRACE_OUTPUT, ...)`, the debug *prompt*
4. `Activity.cpp:1496` and `:1507` -- `Message_Translations_debug_error`, in
   `Activity::displayDebug`, whose one caller is `RexxActivation.cpp:2484`,
   inside `if (debugPause)`

D-P at `:205`-`:210` quotes both banner lines under the `:4024` bullet:

```
+++ "LINUX COMMAND <absolute path>"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
```

The **second** line is producer 3, not producer 1. The plan's text reads as if
`traceSourceString` emits both. Producers 1, 3 and 4 are all interactive debug
and producer 2 is command dispatch, so **the ownership ruling is unaffected** --
but the enumeration should say "two `TRACE_PREFIX_ERROR` sites; four producers
of a `+++`-prefixed line, the other two being the debug prompt at `:4237` and
the debug error at `Activity.cpp:1496`". The `trace_oracle.rs` table's unit is
the enum entry, so "two" is right *there*; the exclusions-file prose is where
"four" belongs.

### Verification of the plan's own `RC(3)` measurement

```rexx
trace r
address sh
'exit 3'
say 'rc=' rc
```

```
--- stdout
rc= 3
--- stderr (cat -A)
     2 *-* address sh$
     3 *-* 'exit 3'$
       >>>   "exit 3"$
       +++   "RC(3)"$
     4 *-* say 'rc=' rc$
       >>>   "rc= 3"$
--- rc=0
```

Exact, including the three-space gap. The plan's measurement holds.

### One precision correction on "neither is a 4c construct"

`RexxActivation::command` has exactly two callers:

```
interpreter/instructions/AddressInstruction.cpp:163: context->command(environment, _command, getIOConfig());
interpreter/instructions/CommandInstruction.cpp:89:  context->command(context->getAddress(), command, OREF_NULL);
```

`AddressInstruction` is the `ADDRESS` instruction -- **4c's instruction** -- in
its `ADDRESS env 'cmd'` form. So a 4c-owned instruction is one of the two
syntactic doors to producer 2. The *emitter* is command dispatch, which is
Phase 7's under D18, and the split table already carves `ADDRESS` at exactly
this line ("the `Address` instruction's environment-name tracking" is 4c's;
"the rest of `Address`" is Phase 7's). So say **"the emitter is command
dispatch, which is Phase 7's"** rather than "neither is a 4c construct" -- the
latter invites a reader to check whether `ADDRESS` is 4c's, find that it is, and
reopen the decision.

## Axis 2: can a 4c construct emit `+++`?

**No. Five probes, none produced it.** All run with stdin at `/dev/null`, from
`/tmp/.../dr-probes/p2/`.

`a.rex` -- `trace e`, `ADDRESS` environment only, `raise error 7`:

```
--- stdout
ZORKENV
--- stderr
--- rc=0
```

`bb.rex` -- `trace f`, `signal on error`, `raise failure 5`:

```
--- stdout
--- stderr
--- rc=0
```

`c.rex` -- `trace r`, `signal on syntax`, `INTERPRET` raising 42.3:

```
--- stdout
trapped 42
--- stderr
     2 *-* signal on syntax name sh
     3 *-* interpret "zz = 1/0"
       >>>   "zz = 1/0"
     3 *-* zz = 1/0
     5 *-* sh:
     6 *-* say 'trapped' rc
       >>>   "trapped 42"
--- rc=0
```

`d.rex` -- `trace i`, `PARSE VALUE` with a `.` placeholder, `ARG`, `PULL`:

```
--- stdout
a 
--- stderr
     2 *-* parse value 'a b' with p1 . p2
       >L>   "a b"
       >K>   "VALUE" => "a b"
       >>>   "a b"
       >=>   P1 <= "a"
       >.>   "b"
       >=>   P2 <= ""
     3 *-* say p1 p2
       ... (>V>, >O>, >>>)
     4 *-* arg aa
       >>>   ""
       >=>   AA <= ""
     5 *-* pull nn
       >K>   "PULL" => ""
       >>>   ""
       >=>   NN <= ""
--- rc=0
```

(Incidentally this confirms `>.>` is a `PARSE` placeholder prefix, i.e. genuinely
4c's, which D-P's gate arithmetic depends on.)

`e.rex` -- `trace a`, builtin call, `CALL length`, `raise syntax 40.1`:

```
--- stdout
3
--- stderr
     2 *-* rr = max(1,2)
     3 *-* call length 'abc'
     4 *-* say result
     5 *-* raise syntax 40.1 array('X')
     5 *-* raise syntax 40.1 array('X')
Error 40 running .../e.rex line 5:  Incorrect call to routine.
Error 40.1:  External routine "X" failed.
--- rc=216
```

No `+++` anywhere. **D-P survives axis 2.**

### One route the plan does not mention, and it is not a construct at all

`RXTRACE=ON` in the environment sets `Trace ?R` (`TraceSetting.hpp:282`-`:287`,
`setTraceResults()` + `setDebug()`; `platform/unix/SysInterpreterInstance.cpp:63`).
Measured, on a program containing **no `TRACE` instruction**:

```rexx
say 'hello'
zz = 2 + 2
```

```
$ RXTRACE=ON ... rexx rx.rex   < /dev/null
--- stdout
hello
--- stderr
       +++ "LINUX COMMAND .../p3/rx.rex"
     1 *-* say 'hello'
       >>>   "hello"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
     2 *-* zz = 2 + 2
       >>>   "4"
--- rc=0
```

So the second site is reached through the *environment* as well as through
`TRACE ?`. Still Phase 7 (the platform layer), so the ruling is unaffected, but
D-P's "Interactive debug, reached through `TRACE ?`" should read "reached
through `TRACE ?` **or `RXTRACE=ON`**". This is also the one shape in which a
corpus program with no `TRACE` in it could diverge, if the differential harness
ever ran with `RXTRACE` set.

## Axis 3: the `TRACE ?` half

### Reproduction of the exclusions file's measurement (`:989`-`:1011`)

```rexx
trace ?r
say 'hello'
zz = 1 + 1
```

stdin at `/dev/null`:

```
--- stdout
hello
--- stderr
       +++ "LINUX COMMAND .../p1/p1.rex"
     2 *-* say 'hello'
       >>>   "hello"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
     3 *-* zz = 1 + 1
       >>>   "2"
--- rc=0
```

Reproduced. Two details the exclusions row does not record and an implementer
would need: the first line carries the standard 7-column prefix indent, the
second is flush left at column 0 (it goes through `processTraceInfo` as raw
message text with no `INSTRUCTION_OVERHEAD` padding); and the prompt appears
**once**, gated by `settings.setDebugPromptIssued(true)` at `:4238`.

### The decisive probe: `TRACE ?` consumes stdin and issues each line as a command

The exclusions row says *"stdout and the exit code match; only stderr differs."*
That is true **only** with stdin at `/dev/null`. With real stdin:

`q.rex` (`trace ?r`) and `n.rex` (`trace r`), identical otherwise:

```rexx
trace ?r          /* n.rex: trace r */
pull vv
say 'got[' || vv || ']'
pull ww
say 'got2[' || ww || ']'
```

stdin for both: `LINE1\nLINE2\nLINE3\nLINE4\n`.

```
##### trace ?r
--- stdout
got[LINE1]
got2[]
--- stderr
       +++ "LINUX COMMAND .../p4/q.rex"
     2 *-* pull vv
       >K>   "PULL" => "LINE1"
       >>>   "LINE1"
       >>>   "LINE1"
+++ Interactive trace. "Trace Off" to end debug, ENTER to continue. +++
/bin/sh: 1: LINE2: not found
/bin/sh: 1: LINE3: not found
/bin/sh: 1: LINE4: not found
     3 *-* say 'got[' || vv || ']'
       >>>   "got[LINE1]"
     4 *-* pull ww
       >K>   "PULL" => ""
       >>>   ""
       >>>   ""
     5 *-* say 'got2[' || ww || ']'
       >>>   "got2[]"
--- rc=0

##### trace r (no ?)
--- stdout
got[LINE1]
got2[LINE2]
--- stderr
     2 *-* pull vv
       >K>   "PULL" => "LINE1"
       ...
     4 *-* pull ww
       >K>   "PULL" => "LINE2"
       ...
     5 *-* say 'got2[' || ww || ']'
       >>>   "got2[LINE2]"
--- rc=0
```

Three consequences, all measured:

1. **`stdout` differs**, not only stderr: `got2[]` vs `got2[LINE2]`.
2. The debug pause **drains stdin**, so every later `PULL`/`PARSE PULL` in the
   program reads different data.
3. Each drained line is **issued as a command** to the current `ADDRESS`
   environment -- `/bin/sh: 1: LINE2: not found`. Interactive debug is therefore
   *literally built on top of command dispatch*, the other `+++` site and D18's
   Phase 7 subsystem.

### Judgement, and the cost estimate the brief asks for

**Assigning `TRACE ?` to Phase 7 is right, and point 3 above is a stronger
argument than the one the plan gives.** The plan argues from honesty ("claims
interactivity the crate cannot deliver"). The measurement upgrades that to a
dependency argument: a correct `TRACE ?` needs a stdin-reading pause loop that
issues each line as a *command*, which is Phase 7's by D18. It cannot be built
in 4c even if someone wanted to.

**Cost of reproducing just the two banner lines**, stated concretely so the
trade is visible: roughly 30-60 lines. `mode_from_setting` retains the `?`
instead of dropping it; one `in_debug` bool on the trace setting; one
`source_traced` once-per-activation flag; the `Interp` path field that D-R's
`>I>` needs anyway; two format strings (7-space-indented `+++ "LINUX COMMAND
<path>"`, and the flush-left prompt). Genuinely cheap -- half a day.

**Would doing it be a correctness improvement or a new lie? A new lie**, and the
probe above is the proof rather than the intuition. A banner-only implementation
matches the oracle byte for byte at stdin `/dev/null` and diverges on **stdout**
the moment stdin has content -- which is exactly the condition every
`PULL`/`PARSE PULL` corpus program runs under. It would convert a currently
*disclosed* stderr-only gap into an undisclosed stdout gap that only appears in
combination with another 4c feature. That is the worst shape this project has a
name for. The plan's ruling is correct; only its stated reason should be
upgraded.

### Gate arithmetic

Checked. `PREFIX_COVERAGE` (`trace_oracle.rs:527`-`:547`) has 19 rows, 13
`Witnessed`, 6 `Owned`. The six: `+++` (`:529`), `>.>` (`:531`), `>M>` (`:539`),
`>I>` (`:542`), `>N>` (`:543`), `<I<` (`:546`) -- the plan's `:529,542,546`
citations are exact. 4c adds `>.>`, `>I>`, `<I<` -> **16 of 19**, with `+++`
(Phase 7), `>M>` and `>N>` (Phase 5) remaining. `OWNER_PHASES` already contains
`"Phase 7"` (`:559`), and `phase-4-exclusions.txt:492` spells it `"Phase 7"`, so
`Coverage::Owned`'s "spelled exactly as the exclusions file's owner column
spells it" contract is satisfied by the flip. Task 1 Step 4's citation of
`phase-4-exclusions.txt:84` for "Four of the six -- +++ and >.> (4c)" is exact.

---

# Summary of required amendments

## D-R (STANDS-WITH-CORRECTION)

| # | Defect | Amendment |
|---|---|---|
| R-1 | `:185` / Task 13 Step 1: "`None` at its one construction site (`plan.rs:631`)". `plan.rs:615` is `#[cfg(test)]`; `:631` is the test helper `activate`. There are 7 `BodyKey` constructions; the **production** one is `lib.rs:1422` in `Interp::run`. | Cite `lib.rs:1422`; note the six test constructions also need the new arm. |
| R-2 | Task 13 Step 3's order is stated once but is two orders. Measured (R5): a quoted target skips the internal label and still reaches the `::routine`. | State both orders; note `search_labels` already encodes the split; note this invalidates `phase-4-exclusions.txt:534`'s `CallTarget::Literal` measurement. |
| R-3 | Task 13 Step 3 replaces `Loud::unresolved_call` with 43.1, but 4c does not build the external search that sits in front of 43.1. Measured (R6/R7): an external `zorkolo.rex` runs at rc 0. `eval.rs:507` argues against exactly this. | Raise 43.1 **and** add an EXCLUSIONS row "external routine resolution -- Phase 7", with the R6/R7 transcripts and a corpus rule. Settle `lib.rs:479` ("both are 4c's") against `eval.rs:484` ("external third (Phase 7)") in the same commit. |
| R-4 | "Three measured facts" is six. Measured (R11, R15): NUMERIC settings, the ADDRESS environment, and condition traps are **not** inherited, and the internal-call path (`run.rs:3304`-`:3313`) inherits all three. | Task 13 Step 2: construct `Inherited` from package defaults, not from the caller. List all six differences. |
| R-5 | Implementing `::routine` removes the loud fallback masking the fact that no directive is ever installed. Measured: oracle aborts before `main` on `::class foo subclass zzznotaclass` (98.909, rc 158) and on a missing `::requires` (43.901, rc 213); `rexx-run` prints `main ran`. | New Task 13 step: fail loudly (owner `Phase 5`) if the program holds any directive other than `::ROUTINE`. |
| R-6 | Task 13's file list omits `tests/coverage.rs`, whose `assert_program_has_no_directives` (`:150`, applied at `:632`) fires on the first `::routine` corpus program, and whose walker does not descend into routine bodies. | Add the file; widen the walker or record the coverage limit. |
| R-7 | Routine-name lookup upcases both sides, unlike `CodeBody::labels`. Measured (M1/M2/M3): `::routine 'MiXeD'` is found by `call MiXeD`, `call 'MiXeD'` and `call 'MIXED'`. `DirectiveParser.cpp:2277`. | One line in Task 13. |
| R-8 | Step 5 says `::options trace labels` **or** the routine's own `trace l`. Measured (R10): `trace r` reaches it too, per `traceEntry`'s `earlyTraceEntry` comment ("TRACE A or TRACE I or TRACE L or TRACE R"). | Widen the sentence; note the coupling (any routine-body `trace r` corpus witness emits both lines whether intended or not) as reason 4 for D-R. |
| R-9 | D-R reason 3 ("a construct the oracle runs at rc 0 still failing loudly") proves too much -- equally true of `::class`. Measured: `keyword-exempt.txt` has 2 routine-named rows, both builtins, so `::routine` unblocks ~0 L1 rows. | Demote reason 3; the real benefit is prefix coverage plus the resolution-chain boundary. |
| R-10 | `:165` attributes "which 4c will have to meet" to `:88`; it is at `:131`. | Trivial. |

**The ruling itself is not refuted.** `::routine` is separable, task-sized, does
not drag in `::requires` or namespaces, and its "package object" cost is one
`String` field on `Interp`. The `>I>`/`<I<` coupling (R-8) makes co-scheduling
close to forced.

## D-P (STANDS-WITH-CORRECTION)

| # | Defect | Amendment |
|---|---|---|
| P-1 | "exactly two emission sites" is right for the *enum* and wrong for the *string*: there are four producers of a `+++`-prefixed line. The plan's own quoted banner pair comes from `:4024` **and** `:4237`, not from `:4024` alone. | Task 1 Step 4's replacement paragraph: "two `TRACE_PREFIX_ERROR` sites (`:4024`, `:4468`); two further `+++`-prefixed lines from `Message_Translations_debug_prompt` (`:4237`) and `Message_Translations_debug_error` (`Activity.cpp:1496`), both interactive debug. All four are Phase 7's." |
| P-2 | "Interactive debug, reached through `TRACE ?`". Measured: `RXTRACE=ON` produces both banner lines with no `TRACE` instruction in the program. | "reached through `TRACE ?` or `RXTRACE=ON`". |
| P-3 | "Neither is a 4c construct" invites a reader to notice that `AddressInstruction.cpp:163` -- the `ADDRESS env 'cmd'` form of a 4c instruction -- is one of the two callers of `RexxActivation::command`. | "The emitter is command dispatch, Phase 7's under D18; the `ADDRESS` instruction is one of its two doors and the split table already carves `ADDRESS` at exactly that line." |
| P-4 | The `TRACE ?` KNOWN GAP row says "stdout and the exit code match; only stderr differs". Measured: with non-empty stdin, stdout differs (`got2[]` vs `got2[LINE2]`), the pause drains stdin, and each drained line is issued as a shell command. | Correct the row when Task 1 Step 5 assigns its owner, and use the dependency argument (debug pause -> command dispatch -> D18) as the reason, ahead of the honesty argument. |

**No third `TRACE_PREFIX_ERROR` site exists** (four search instruments,
including an enumeration of every caller of every prefix-parameterised trace
helper), and **no 4c construct emits `+++`** (five probes across `trace
e`/`f`/`r`/`i`/`a`, `ADDRESS` environment-only, `PARSE`, `ARG`, `PULL`,
`INTERPRET`, `RAISE` and condition traps). Reproducing the `TRACE ?` banner
without the pause is a new lie, and the stdin probe proves it rather than
arguing it.
