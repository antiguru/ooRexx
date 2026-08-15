# Task 5 review: `PROCEDURE`, `PROCEDURE EXPOSE`, `USE ARG`, `USE LOCAL`

Reviewed: `32a667ac..8b87195b`, 2,079 insertions across 16 files.
Oracle: `/home/moritz/dev/repos/ooRexx/build/bin/rexx`, every probe run from a
directory created for this review (`rev5a`..`rev5h` under the scratchpad), each
wrapped `( ulimit -v 1048576; LD_LIBRARY_PATH=... rexx FILE )` with stdout,
stderr and exit status read as three separate descriptors.
Tree restored after every experiment; `git status --porcelain` empty.

---

## Verdicts

**1. Spec compliance: MET WITH ONE EXCEPTION.** Every step of the brief is
built or deliberately and correctly deviated from, and nothing beyond the
brief's scope was added. The exception is Step 5's instruction *"Take every
error number and text from the oracle"*: `USE ARG >name` implements two of the
oracle's three refusals (88.928, 88.931) and omits the third (98.995), which
turns four measured programs into silent wrong answers. See Critical C1.

**2. Task quality: GOOD, with one correctness defect and one false test
comment.** The measurement discipline is unusually good — every claim in §1-§4
of the report that I re-ran reproduced exactly, including the two the brief
singled out as unverified. Comment accuracy is high but not perfect: three
statements are wrong, one of them about a test's own discriminating power.

**Counts: 1 Critical, 1 Important, 3 Minor.**

---

## The highest-value check: `use strict arg p = 1, q`

**Measured answer: minimum expected is 2 — the implementer's inference is
correct.** Oracle, `rev5a/inv1.rex`, `call sub 7` into `use strict arg p = 1, q`:

```
rc 216
Error 40.3:  Not enough arguments in invocation of SUB; minimum expected is 2.
```

The implementation answers the identical bytes (rc 216, same two stderr lines).

I did not stop at the one shape the report flagged, because a single
confirmation cannot separate "position of the last target with no default"
from other rules that agree on it. Three further shapes, all byte-identical
between oracle and crate:

| program | oracle | crate |
|---|---|---|
| `use strict arg p = 1, q`, 1 arg | 40.3, minimum 2 | same |
| `use strict arg p = 1, q`, 2 args | rc 0, `[7][8]` | same |
| `use strict arg p = 1, q, r = 3`, 2 args | rc 0, `[7][8][3]` | same |
| `use strict arg p = 1, q, r = 3`, 1 arg | 40.3, **minimum 2** | same |
| `use strict arg p, q = 1, r`, 2 args | 40.3, **minimum 3** | same |

The last two are the discriminating pair: a rule counting targets-without-a-
default would answer 2 for `p, q = 1, r`, and the oracle answers 3. The
implemented rule — `rposition` of the last target with no default, plus one
(`run.rs:1884-1898`) — is the measured rule, not merely a consistent one.
`use strict arg p, , r` (an omitted target in the middle) also agrees at
`[1][3]`, and `use strict arg` with one argument gives 40.4 maximum 0 on both.

**Report §6 concern 5 is now closed by measurement, and the implementation is
right.** The concern was correctly raised and correctly resolved.

---

## Findings

### C1 (Critical). `USE ARG >name` is missing the oracle's 98.995 refusal, and silently gives wrong answers instead

`crates/rexx-exec/src/run.rs:1927-1940` (`bind_use_target`, the `target.alias`
branch).

The oracle requires a `>` target to be **currently unset**. When it is not, it
raises:

```
Error 98 ... Execution error.
Error 98.995:  Unable to reference variable "Q"; it must be an uninitialized local variable.
```
rc 158. The implementation performs no such check: it calls `slot_of`, then
`alias_slot`, unconditionally.

**Four measured programs where the crate returns rc 0 with wrong output where
the oracle raises** (all under `rev5f/`, oracle and crate transcripts kept):

| program | oracle | crate |
|---|---|---|
| `e1` — shared pool, target `q` already holds `'q-orig'` | rc 158, 98.995 | rc 0, `after call p: via-alias q: via-alias` |
| `e2`/`g2` — `sub: procedure`, `q = 1` then `use arg >q` | rc 158, 98.995 | rc 0 (e2 prints `q after use: orig`, `p: via-alias`) |
| `f3` — `procedure expose q` where the exposed `q` holds a value | rc 158, 98.995 | rc 0, `p: via-alias q: q-in-caller` |
| `f4` — `use arg >q` twice onto the same target | rc 158, 98.995 | rc 0, `p: via-alias r: r-orig` |
| `g3` — stem target `q.` already holds a tail | rc 158, 98.995, names `"Q."` | rc 0, no output |

This is the outcome shape this project ranks worst: not a loud gap naming what
is missing, but a wrong answer found only by chasing a wrong value.

**The rule, characterised by measurement rather than guessed.** The trigger is
*only* "the target currently has a value". It is **not** about exposure and not
about locality, despite the message's wording:

* `f1` — shared pool, target unset → rc 0 on both, alias installed. Legal.
* `f2` — `q = 'local'` then `drop q` then `use arg >q` → rc 0 on both. `DROP`
  restores the uninitialised state.
* `g1` — `procedure expose q` where the exposed `q` is **unset** → rc 0 on
  both, `p: via-alias q: Q`. So `f3` fires because the value is there, not
  because the name is exposed. This is the pair that separates the two
  hypotheses, and without `g1` the fix would very likely be written as an
  exposure check and be wrong.
* `f4` fits the same rule: after the first `use arg >q`, `Q` resolves to `P`
  which holds `'p-orig'`, so `Q` "has a value" at the second.

**Concrete fix.** In `bind_use_target`, between `let index = self.slot_of(&name);`
and `self.roots.alias_slot(...)`:

```rust
let frame = self.activation().frame;
if self.roots.slot(frame, index).is_some() {
    return Err(Raised::variable_reference_not_uninitialised(&name).into());
}
self.roots.alias_slot(frame, index, slot);
```

`RootSet::slot` already resolves through any alias, which is exactly what `f4`
needs. Add to `error.rs` beside the other two `USE ARG >` refusals:

```rust
/// 98.995: `USE ARG >name` whose target is not currently unset. ...
pub(crate) fn variable_reference_not_uninitialised(name: &[u8]) -> Raised {
    Raised::syntax(98, 995, vec![String::from_utf8_lossy(name).into_owned()])
}
```

One substitution, the target's own spelling as `use_target_name` already
produces it — `Q` for a simple variable and `Q.` for a stem, both measured.
The stem case needs no separate handling: a stem lives in one slot, and `g3`'s
`q.1 = 'local'` makes that slot `Some`.

Tests to add: `g2` (assigned then aliased) and `g1` (exposed but unset, which
must still *succeed*) as a pair — `g1` alone cannot fail against a wrong fix,
and `g2` alone does not pin that the rule is about the value rather than
exposure. `f2` (dropped, must succeed) is the third worth having.

### I1 (Important). The frame-release test's "control" does not discriminate the mutant its comment names

`crates/rexx-exec/src/run.rs:8507-8516`, in
`an_isolated_callees_frame_is_released_on_both_paths`.

The comment states:

> The control: a callee with no PROCEDURE pushes no frame at all (D9r's shared
> pool), so this arrives at the same answer for a different reason. **Without
> it, an implementation that never pushed a callee frame would pass both
> assertions above.**

The bolded claim is false. All three assertions in the test compare
`live_frames()` against the same value, `1`, so an implementation that never
pushes a callee frame satisfies the control exactly as it satisfies the two
assertions the control is said to backstop.

**Verified by mutation, not by reading.** I built precisely the named mutant —
`exec_procedure` sets `inner = outer`, `owns_frame = false`, and never calls
`push_slots`:

```
cargo test -p rexx-exec --lib an_isolated_callees_frame_is_released_on_both_paths
test result: ok. 1 passed; 0 failed
```

The whole test passes under the mutant, control included. (The mutant is caught
elsewhere — five other tests fail on it, including
`procedure_isolates_and_expose_aliases_the_caller_entry` — so this is a false
statement about coverage rather than a coverage hole.)

The test is otherwise sound: with `pop_slots` removed on the `owns_frame`
path it fails as intended, so it does catch a genuine leak, which is the
property its name claims.

**Fix.** Either delete the third block and its comment, or make it actually
discriminate. The cheapest honest version keeps the block and corrects the
claim to what it does pin — that a *shared-pool* callee must not push a frame
it then leaks. If a control against "never pushes a frame" is wanted here
rather than left to the isolation tests, it has to assert a *different* number
from inside the callee, e.g. that `live_frames()` is 2 while an isolated callee
is running and 1 after it returns.

This is the third could-not-fail construct in this task, after the two the
implementer caught themselves. I re-verified both of those and both now
genuinely fail under their own mutants: swapping the two `USE LOCAL` errors
fails `use_local_as_a_programs_first_instruction_raises_98_993`, and removing
the `pop_slots` fails the frame test.

### M1 (Minor). `RootSet::iter`'s doc asserts something the code currently permits

`crates/rexx-core/src/roots.rs:391`:

> A write through an alias lands in the target's storage (`set_slot` resolves
> first), so **an aliasing slot's own entry stays `None` for its whole life**
> and the `filter_map` drops it.

Reachable today and false: `rev5f/g2.rex` (`sub: procedure` / `q = 1` /
`use arg >q`) leaves `slots[Q] == Some(1)` *and* `aliases[Q] == Some(..)`
simultaneously, so `iter` yields a value no name can reach. The consequence is
over-retention only, never a collected live object, which is why this is Minor
rather than part of C1's severity — but the statement is offered as a
guarantee and is not one.

The same sentence appears in `alias_slot`'s own doc at `roots.rs:260` ("The
aliased slot's own storage stops being reachable, and stays `None`").

**Fix:** both become true once C1 lands, since the 98.995 check is exactly the
guarantee that the slot is `None` at the moment `alias_slot` is called. Land
C1, then either leave these as-is or add the half-sentence that says *why*
they hold ("`bind_use_target` refuses a target that is not unset, 98.995"),
which is what makes them checkable by the next reader rather than assertions.

### M2 (Minor). `slot_ref`'s "there is no chain here to walk" is literally false

`crates/rexx-core/src/roots.rs:249`:

> One step suffices for all depths precisely because every alias this type
> records was produced from a `SlotRef` and so was chased when it was made;
> **there is no chain here to walk**, by induction on the order the frames were
> pushed.

A chain does form. `rev5h/chain.rex`:

```rexx
r = 'r-val'
call s1 >p, >r
say 'q:' q 'p:' p 'r:' r
exit
s1:
use arg >q          /* aliases Q -> P */
use arg xx, >p      /* aliases P -> R, while Q -> P still stands */
q = 'written-through-q'
return
```

At the third instruction `aliases[Q] = Some(P)` and `aliases[P] = Some(R)` are
both set — a two-link chain in one frame, which the shared pool makes possible
because `bind_use_target` aliases into `self.activation().frame` and that frame
is the caller's.

**The single step is nevertheless correct, and I confirmed it against the
oracle**: both interpreters print `q: written-through-q p: r-val r: r-val`,
byte-identical. The reason is not the absence of a chain, it is that a `>`
binding attaches to *storage* at bind time and re-aliasing the intermediate
does not retarget it — which is the oracle's own semantics, and is what a
single step implements.

**Fix:** replace the induction argument with the measured reason. The claim to
make is "one step is not a shortcut for walking a chain, it is the semantics:
a binding is to the storage the target named when it was made, and the oracle
agrees — `chain.rex`". The current wording is a proof of the right conclusion
from a false premise, and the premise is the part a later reader would rely on.

(Verified alongside: `slot_ref` at `roots.rs:252` is the only producer of a
`SlotRef` in the workspace, so the "only producer" half of the type's doc is
correct.)

### M3 (Minor). The `expose a.1` gap is reachable through the indirect form too, which neither the comment nor report §6 records

`crates/rexx-exec/src/lib.rs:519` (`Loud::compound_expose`) and report §6
concern 1 both describe the gap as `procedure expose a.1`, the direct spelling.
The indirect form reaches it as well — `rev5e/c3.rex`, `v = 'A.1'` /
`procedure expose (v)`, oracle rc 0 printing `changed`, crate rc 120 with
`PROCEDURE EXPOSE of the single compound tail "A.1" is not implemented`.

The behaviour is right and consistent; only the disclosure is narrower than the
gap. **Fix:** one clause in `compound_expose`'s doc noting that
`expose_names`'s indirect expansion lands here too, so the next reader sizing
the gap does not have to find it.

---

## The two brief disagreements — rulings

### The redirect design: the implementer is right, and the brief's `Interp` route is not implementable as described

**Ruling: the `RootSet` choice is correct. Accept it.**

The load-bearing claim is about `Interp::read`, and it is **verified**.
`crates/rexx-exec/src/lib.rs:1299-1313`:

```rust
fn read(&mut self, code: &Code<'_>, id: SymbolId) -> (ObjRef, Novalue) {
    let slot = match code.slots.get(&id) {
        Some(slot) => *slot,
        None => self.slot_of(code.symbols.name(id).as_bytes()),
    };
    let frame = self.activation().frame;
    match self.roots.slot(frame, slot) {
```

The hot read resolves through `code.slots` — the plan's `by_symbol` map — and
goes straight to `self.roots.slot(frame, slot)`. `slot_of` is reached only on
the fallback miss. So a pair-returning `Plan::slot_of`, which is what the brief
specified, **would not be on the hot read path at all**; the `Interp` route
would have meant hand-writing the alias check at each of the 15 production
sites, with the identical instruction count on the hot path and fifteen places
to get it wrong. The implementer's characterisation is accurate and the
conclusion follows.

Two things make the ruling firmer than "the argument is sound":

* **The report's §1d refutation of the brief's own stated shape is real, and I
  reproduced it.** `rev5b/trans.rex` gives `bee sees: set-by-cee set-by-cee-m`
  / `a sees: set-by-cee from-a-m`. One `PROCEDURE` (`cee: procedure expose n m`)
  exposes two names resolving to two *different* frames, so D9r's "a bitset over
  slot indices plus one target `SlotFrame`" cannot represent it. The per-slot
  `Vec<Option<usize>>` gets it for free. This is a genuine correction to the
  plan, not a preference.
* The cost was measured and reported honestly, including the part that does not
  flatter it (+2 % on stem-heavy code, explicitly labelled small-signal). §6
  concern 3 declines to claim more precision than the method supports.

The one thing I would not carry forward unexamined is the induction argument
the design leans on for single-step resolution — see M2. The design is right;
that particular justification for it is not.

### Step 2 was not built: the implementer is right, and the brief's premise is false of execution

**Ruling: correct deviation. `plan.rs` should stay unchanged.**

The brief's reasoning is *"`PROCEDURE` must be a routine's first instruction,
so the list is a property of the body."* **I measured the 17.1 claim and it
holds** — `rev5b/`, all against the same `sub:` label text:

| program | oracle | crate |
|---|---|---|
| `say 'main'` / `sub:` / `procedure` (fallen into) | rc 239, 17.1 | byte-identical |
| `call sub` / `sub:` / `nop` / `procedure` | rc 239, 17.1 | byte-identical |
| `call sub` / `sub:` / `lbl2:` / `procedure` | rc 0, runs | byte-identical |

The same body text is legal when called and 17.1 when fallen into, so no
property of the body decides it — the brief's premise is true of the grammar
and false of execution, exactly as reported. I added a shape the report did not
measure: `sub:` / `if 1 = 1 then procedure` is also 17.1, rc 239, and the crate
matches byte for byte, which confirms the "taken at the top of `step`, so
nested stepping sees `false`" mechanism rather than just the four tabulated
cases.

`Activation::first_instruction_pending` plus `entered_by_call` is the right
place for this, and the ordering it enables — resolve exposed names while the
caller's frame is still top, *then* push the callee's — is what keeps
`grow_slots`'s invariant true. That is a better outcome than Step 2 would have
produced, not merely a permissible substitute.

---

## Everything else the brief asked me to verify by running

All measurements below are mine, from directories I created; every "matches"
means stdout, stderr and exit status all compared.

**Exposure is transitive** — `rev5b/trans.rex`, the two-frame version.
Oracle `bee sees: set-by-cee set-by-cee-m` / `a sees: set-by-cee from-a-m`;
crate byte-identical. This is the case that cannot pass by accident, since
`n` must chase two levels and `m` must stop at one, from a single `PROCEDURE`.

**The indirect form is plural and exposes its own selector** — both halves
reproduced. `list = 'ALPHA BETA'` gives `a-set b-set g-in-caller` (GAMMA the
control), and `v = 'zzz'` gives `callee v: zzz callee zzz: z-in-caller` /
`caller v: v-set-in-callee caller zzz: zzz-set-in-callee`. Crate identical on
both. Duplicate names in one list (`expose n n`) also match.

**The five stem transcripts** — all five pass in-crate
(`an_exposed_stem_aliases_the_callers_entry_not_the_object`), including the
`drop` pair; `keep. = a.` then a callee `drop a.` leaves the caller printing
`A.1 orig` on both interpreters. The whole-stem case also appears in the corpus
program, which I ran differentially myself (below).

**`USE ARG`** — every value in the brief reproduced against the oracle *and*
the crate: extra arguments ignored in the lax form; `use strict arg p` with
three arguments is 40.4 rc 216; `call sub 1,,3` gives `[1] [Q] [3]`;
`use arg >q` with a plain symbol is 88.928 rc 168 and with `>p` aliases. Beyond
the brief, all byte-identical: `use arg p.1` and `use arg st.` targets,
`use strict arg` with no targets (40.4, maximum 0), `use arg` at top level
(`[P]`, so an empty `call_context` is the right answer), and `use strict arg`
reached through the *expression* call form `f(1)` (40.3, `invocation of F`).
`use arg >q.1` and `call sub >st.1` are rejected at parse time with the right
numbers (20.931 / 20.930) under the standing, pre-existing parse-error
rendering limitation — unchanged by this task.

**The new loud gap** — verified and **the choice is right**. Oracle,
`rev5b/ct1.rex`: with `a.1 = 'kept'` and `a.2 = 'other'`, `sub: procedure
expose a.1` writing both tails leaves the caller printing `changed other`.
Tail 1 is shared; tail 2 is the callee's own. That is aliasing *inside* a stem
object at one tail, and this task's mechanism aliases whole slots. The two
available approximations both lose: exposing the whole stem silently shares
`a.2`, and ignoring the expose silently isolates `a.1`. A named loud failure is
the correct third option, and it carries no owner string because nothing has
been scheduled to build it — which is the honest label. Only the disclosure's
scope is narrow (M3).

**Corpus is 36 of 36, and both new witnesses are genuinely compared.**
`REXX_CORPUS_GATE=1 cargo test -p rexx-exec --test corpus` reports
`mode: STRICT (the gate)` and `36 of 36 matching`. I did not take that on
trust: `corpus.rs` invokes the real oracle binary per program at test time
(there are no stored expected bytes to go stale), and I re-ran both new
programs myself from a fresh directory:

| program | oracle stdout | crate stdout | stderr |
|---|---|---|---|
| `lang/call_procedure_expose.rex` | 142 bytes | 142 bytes, identical | both empty, identical |
| `lang/use_arg_forms.rex` | 188 bytes | 188 bytes, identical | both empty, identical |

Both rc 0 on both interpreters. The outputs are substantive, not degenerate —
`isolation: caller-v callee-w` / `a sees: set-by-cee from-a-m` /
`plural: a-set b-set g-in-caller` / `stem: stem-changed`, and the `USE ARG`
program's eight lines including `dropped: PRESET` and
`alias: set-through-alias`. Both headers state per block what a wrong
implementation would print, and the values are chosen so no literal equals its
own derived name, which is what the brief's Step 1 asks for.

**`grow_slots`'s panic and its `should_panic` test both survive** — confirmed,
and I verified the *substance* of the new claim rather than its presence. Two
halves:

* An exposed name binds via `alias_slot`, which writes `self.aliases[frame.start
  + index]` in the **callee's** frame and only reads the target's position
  (`roots.rs:268-270`). Nothing is allocated in the caller. Correct.
* The computed-`expose (v)` case really is resolved while the caller's frame is
  on top: `exec_procedure` (`run.rs:1698-1730`) runs `expose_names`, then the
  `slot_of` loop, *then* `frame_len(outer)` and `push_slots`. The grow happens
  before the push. `rev5c/ex2.rex` exercises it end to end —
  `nm = 'ZQXW'` / `procedure expose (nm)` / `interpret "zqxw = ..."`, with a
  second run-time binding (`AAA`) present in the caller to check the `extra`
  write-back does not clobber it — and matches the oracle byte for byte
  (`caller aaa: caller-aaa zqxw: set-in-callee`). The `extra` write-back at
  `run.rs:1715-1722` is safe because `resolve_and_run_call` clones the caller's
  `extra` **into** the callee at `run.rs:2137-2140`, so the map written back is
  a superset, not a replacement. That comment is accurate.
* `collect.rs`'s 28 changed lines are the doc correction, the `should_panic
  (expected = "4a invariant")` attribute is untouched, and the panic string
  still contains `4a invariant`. The correction's own correction — that the
  old comment said the caller prints `9` where the measured value is `5` — is
  right; `rev5b` reproduces `5`.

**§6 concern 2, the unreached `USE LOCAL` 99.910 arm** — **keeping it is
right, and the reasoning generalises, but only in the narrow form the report
gives it.** The arm is genuinely unreachable today: `rexx-parse` intercepts
every placement at parse time. Collapsing it to an unconditional 98.993 would
be a silent wrong answer the day that check moves, and the alternative — an
`unreachable!` — is precisely what this project's failing-loudly rule excludes.
The report is also correct that a test for it would have to route through the
parse-time path and so could not fail if the arm were wrong; writing one would
add a test that cannot fail, which is a defect here, so *not* writing it is the
right call. What keeps this from being a general licence for untested arms is
that the reachable sibling **is** tested and the two are one `if`/`else`: the
98.993 arm's test pins the condition, so an inverted condition fails.
I confirmed that by mutation — swapping the two errors fails
`use_local_as_a_programs_first_instruction_raises_98_993`. An unreached arm
whose condition no test pins would not be defensible on this reasoning.

**No pre-existing expected-byte literal changed.** Every deletion in the diff
under `tests/` and `corpus/` is a comment, a witness row for a variant that
moved in scope (`Procedure`, `Use`, `VariableReference`), or a count literal
moved in step with those rows. No `.rex` program and no expected-output file
was modified; the two `sourceline_oracle/*.txt` files are new, and their
`count` lines match `wc -l` on their sources (73 and 76). All 34 previously
passing corpus programs still match, which the 36-of-36 gate run establishes
directly.

**Global constraints.** `cargo fmt --all --check` exit 0.
`cargo clippy --workspace --all-targets -- -D warnings` exit 0.
`cargo test --workspace` **919 passed, 0 failed, 4 ignored**, matching the
report's figure against a 901 baseline. No `unsafe` (workspace forbids it, and
clippy is clean). The C++ tree is untouched — the diff's 16 files are all under
`rust/`. No `NUMERIC DIGITS` above 1000, no `.Package~new`, no SF #2018 shape
in any probe.

---

## Ranked summary

| # | Severity | Location | Finding |
|---|---|---|---|
| C1 | **Critical** | `run.rs:1927-1940` | `USE ARG >name` omits the oracle's 98.995 refusal; five measured programs get silent wrong answers instead of rc 158 |
| I1 | **Important** | `run.rs:8507-8516` | The frame-release test's "control" passes under the exact mutant its comment says it catches; verified by building the mutant |
| M1 | Minor | `roots.rs:391`, `roots.rs:260` | "an aliasing slot's own entry stays `None` for its whole life" is reachable-false today; becomes true once C1 lands |
| M2 | Minor | `roots.rs:249` | "there is no chain here to walk" is false — `chain.rex` builds a two-link chain; the single step is still correct, for a different reason |
| M3 | Minor | `lib.rs:519`, report §6.1 | The `expose a.1` gap is also reachable through `expose (v)`; disclosure is narrower than the gap |
