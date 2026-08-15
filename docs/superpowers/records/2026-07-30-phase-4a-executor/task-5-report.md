# Task 5 report: stems and compound variables

Status: **DONE.**

## Pre-flight reading

`task-5-brief.md`; D15a in full; Task 4's report and the ruling on test
placement (applied here by analogy, confirmed correct once the plan's
"Global constraints" picked up the same rule generally). `rexx-parse`'s
`ExprKind::Stem`/`Compound`, `Tail`, `compound_parts` (`ast.rs`); `rexx-core`'s
`Body::Stem`, `RootSet` (`roots.rs`, in full); the current
`rexx-exec/src/lib.rs` (`Plan`, `Activation`, `Code`, `slot_of`, `read`).

The brief gives seven oracle transcripts and states the tombstone/replace/
name/verbatim-key rules in prose, but gives a full signature for only one of
six functions (`tail_key`), and that one signature does not compile as
written. Rather than guess at the other five, I derived the read/write
algorithm from the seven transcripts plus five more oracle probes (below),
and I'm sending the design before writing ~400 lines of `stem.rs` around a
misreading, the same shape as Task 4's pre-flight.

## Five more oracle probes, beyond the brief's seven

All under `( ulimit -v 1048576; build/bin/rexx FILE )`.

```
s1: say never_touched.5                     -> NEVER_TOUCHED.5
s2: t1=q.; t2=q.; say t1; say t2; say(t1==t2) -> Q. / Q. / 1
s3: x.='d'; x.1='one'; drop x.; say x.1; say x. -> X.1 / X.
s4: say w.9; w.='wd'; say w.9                -> W.9 / wd
s5: q.1='x'; y=q.; say y; say q.2; say q.     -> Q. / Q.2 / Q.
```

`s1` shows a tail read on a **completely untouched** stem needs no
`Body::Stem` object at all: the derived name is stem-name-plus-key,
constructible directly. `s3` shows `drop x.` (the whole stem) leaves the
variable behaving **exactly as if never touched** -- both the bare read and
a tail read afterward match `s1`/an uninitialised bare stem exactly, so
nothing distinguishes "dropped" from "never touched" observably. `s5` is
the one that pins down what `Body::Stem.name` is actually *for*: `q.1='x'`
never bare-assigns `q.`, so the object it auto-vivifies has `default: None`;
aliasing it into `y` and reading `y` gives `Q.` (the object's own name), not
the tail value and not anything derived from `y`'s own identity -- proving
the name has to live on the object because by alias-read time the original
site (`q.`) is gone from context.

## The algorithm this pins down

**Reading a bare stem** (`ExprKind::Stem(id)`): go through the *exact same*
slot read every variable uses (`self.read`/`slot_of`, keyed by
`code.symbols.name(id)`, which already includes the trailing period).
Unset -> derive the name the same way an uninitialised simple variable
does, **no `Body::Stem` object involved**. Set -> the slot holds a
`Body::Stem`; rendering it (`to_text`, extended) is `default` if `Some`,
else the object's own `name`.

**Reading a tail** (`ExprKind::Compound(id)`): `compound_parts` on
`code.symbols.name(id)` gives the stem name and the `Tail` pieces.
Resolve each piece to bytes -- `Constant` verbatim, `Variable(name)` by
reading *that* name's current value through the ordinary variable path
(deriving its own name if unset, same as any read) and converting with
`to_text` -- and join with `.` for the key (this is `tail_key`). Resolve the
stem's slot: unset -> derived name `stem_name + key`, no object needed
(matches `s1`). Set -> look up `tails.get(&key)`: `Some(Some(v))` -> `v`;
`Some(None)` (tombstone) -> derived name, **not** the default (D15a's
`u.1`/`u.2` pair); absent -> the stem's `default` if `Some`, else derived
name.

**`stem_assign`** (`stem. = expr`): evaluate `expr`, build a **new**
`Body::Stem { name, default: Some(value), tails: {} }`, and rebind the
stem's slot to it (`slot_of` + `set_slot`) -- the old object, if any, is
left alone for whatever aliased it (the `r.`/`u` transcript).

**`stem_set`** (`stem.tail = expr`, D15a's "a tail assignment mutates"):
resolve the key, evaluate `expr`; if the stem's slot already holds a
`Body::Stem`, mutate its `tails` map in place
(`heap.get_mut` + `tails.insert(key, Some(value))`); if the slot is unset,
**auto-vivify** `Body::Stem { name, default: None, tails: {key: Some(value)} }`
and bind it (this is the only place a stem object is created from a read
that was never a bare assignment, and is what makes `s5` possible: `q.1='x'`
with `q.` never itself assigned).

**`stem_drop_tail`** (`drop stem.tail`): if the stem's slot holds an object,
insert the tombstone (`tails.insert(key, None)`); if unset, a genuine no-op
-- an absent key and a tombstone already render identically when there is
no default, so there is nothing to record yet.

**`stem_drop`** (`drop stem.`, whole stem): **rebind to a new**
`Body::Stem { name, default: None, tails: {} }`, the same shape
`stem_assign` uses but with `default: None`. This is forced, not merely
consistent with the brief's "replace the Stem object" wording:
`RootSet::set_slot` takes an `ObjRef`, not `Option<ObjRef>` (checked in
`roots.rs`), so **there is no way to write "unset" back into an existing
slot at all** through the current API. A general `DROP` on a *simple*
variable will hit the identical wall and needs a `rexx-core` amendment
(an `unset_slot`, or `set_slot` taking `Option<ObjRef>`) -- out of scope
for this task (`rexx-core` is not in Task 5's Files list) but flagged here
since it will block whichever later task builds plain `DROP`.

## Two things that don't compile or don't exist as given

* `tail_key(&mut self, &Plan, frame, SymbolId)`: neither `Plan` nor `frame`
  (a bare `SlotFrame`) is enough on its own to reach a *named* variable's
  value -- resolving a `Tail::Variable` piece needs `self.read`, which needs
  `&Code<'_>` (body + symbols + the id-to-slot map), not `&Plan` alone, and
  the compound's own name needs `&SymbolTable` to turn `SymbolId` into text
  in the first place. Proposed: `fn tail_key(&mut self, code: &Code<'_>, id: SymbolId) -> Vec<u8>`,
  reusing the exact bundle `read` already takes -- this is the same fix
  shape as Task 3/4's `Code<'_>` discipline, not a new idea.
* `stem_get`, `stem_set`, `stem_drop_tail`, `stem_assign`, `stem_drop` have
  no signatures at all in the brief. Proposed, all taking the stem's own
  **name in bytes** rather than a `SymbolId`:
  - `stem_get(&mut self, code: &Code<'_>, stem_name: &[u8], key: &[u8]) -> ObjRef`
  - `stem_set(&mut self, code: &Code<'_>, stem_name: &[u8], key: &[u8], value: ObjRef)`
  - `stem_drop_tail(&mut self, code: &Code<'_>, stem_name: &[u8], key: &[u8])`
  - `stem_assign(&mut self, code: &Code<'_>, stem_name: &[u8], value: ObjRef)`
  - `stem_drop(&mut self, code: &Code<'_>, stem_name: &[u8])`
  `&[u8]` rather than `SymbolId` because `ExprKind::Compound`'s `SymbolId`
  names the *whole* dotted spelling, and there is no separate `SymbolId` for
  just its stem half -- `compound_parts` only ever hands back a borrowed
  `&str` slice of the same interned name, never a new id. A bare
  `ExprKind::Stem(id)`'s name (`code.symbols.name(id)`, already including
  the period) fits `&[u8]` just as directly. The eventual call sites
  (Task 6/7's `eval.rs`, not this task) get `key` from `tail_key` and
  `stem_name` from `compound_parts` directly -- both are cheap, allocation-
  free operations on the same interned string, so nothing needs computing
  twice in one meaning of the word: `tail_key` decomposes once internally
  to build the key, and a caller that also wants the stem name calls
  `compound_parts` itself rather than `tail_key` returning a tuple, which
  would be a wider contract than the brief's stated `-> Vec<u8>`.

## The brief's seven transcripts, verified against the oracle

All under `( ulimit -v 1048576; build/bin/rexx FILE )`, done while waiting
for the design question above (does not depend on its answer).

| transcript | result | matches D15a |
|---|---|---|
| `u.='d'; u.1='one'; drop u.1; say u.1; say u.2` | `U.1` / `d` | yes |
| `a.=1; b.=a.; a.1=2; say b.1` | `2` | yes |
| `r.='rd'; u=r.; drop r.; say u` | `rd` | yes |
| `s.='def'; t=s.; s.='other'; say t` | `def` | yes |
| `say q.` | `Q.` | yes |
| `i='abc'; v.i='val'; say v.i v.ABC` | `val V.ABC` | yes |
| `i=1; j=2; a.i.j='deep'; say a.1.2` | `deep` | yes |

All seven match exactly. No spec defect found in the transcripts themselves;
the open questions above are entirely about the missing/non-compiling
signatures, not about the semantics.

## Question sent to the team lead, and the answer

Sent before writing `stem.rs`. Answer: the test-placement question was
confirmed general (the plan's Global constraints now say so for every
module task, `2ef9dd7a`); the algorithm and signature design were not
individually contested, and the team lead said to carry on, naming the
tombstone as the one an obvious implementation gets wrong -- which matches
what this report already had. Implemented with one refinement found while
writing the code (below): `stem_get`/`stem_set`/`stem_drop_tail`/
`stem_assign`/`stem_drop` ended up **not** taking `code: &Code<'_>` at all,
only `stem_name: &[u8]`/`key: &[u8]`/`value: ObjRef` -- none of the five
actually touch `code.body`/`code.symbols`, only `tail_key` does (to turn a
`SymbolId` into text in the first place), so passing `Code` to the other
five would have been an unused parameter.

## A second design question, found while implementing, resolved without
## re-asking (verified against the oracle rather than blocked on)

`stem_assign`'s "replace the object" rule (D15a, and this report's own
draft above) is not quite right as stated: it cannot explain `a.=1; b.=a.;
a.1=2; say b.1` -> `2` by itself. If `b. = a.` *wrapped* `a.`'s current
value as `b.`'s new `default`, then a later tail write through `a.` (which
mutates `a.`'s own object) would never be visible through `b.`, whose
object is a separate wrapper. The only model that reproduces `2` is that
`b.` ends up referencing the *same* `Body::Stem` object `a.` does -- which
is also the literal reading of D15a's own parenthetical, "one shared
object". So `stem_assign` checks whether the value being assigned is
itself already a `Body::Stem`: if so, it shares that object directly (a
plain slot rebind, the same as any ordinary variable assignment); if not,
it wraps it as usual. Verified with a probe beyond the seven given
transcripts, specifically to rule out the alternative "wrap, and have
`stem_get` chase through a nested default" hypothesis, which is a rule
D15a never states:

```
a. = 1 ; b. = a. ; a.1 = 2 ; say b.1     -> 2
a. = 9 (a fresh reassignment, afterward)
say b.                                   -> 1
say b.1                                  -> 2
```

`b.` keeps answering `1`/`2` even after `a.` is rebound to a brand new
object -- proof that `b.` was never holding a reference to `a.`'s *slot*
(which would have followed the rebind) but to the object that used to be
there, which only "sharing" produces. Did not re-block on this: the
transcript already given (`a.=1; b.=a.; a.1=2; say b.1 -> 2`) already
discriminates the two hypotheses once thought through, and the extra probe
above is confirmation rather than a fork requiring a ruling. Recorded in
`stem_assign`'s doc comment so a future reader does not re-derive it.

## Implementation

`rust/crates/rexx-exec/src/stem.rs` (new), `impl Interp` block:

* `tail_key(&mut self, code: &Code<'_>, id: SymbolId) -> Vec<u8>` --
  `compound_parts` on the compound's own name, each `Tail::Constant`
  verbatim, each `Tail::Variable` resolved through `read_by_name` (below)
  and `to_text`, joined with `.`.
* `read_by_name(&mut self, name: &[u8]) -> ObjRef` -- the same slot
  machinery `read` (lib.rs) uses but keyed purely by name, with no
  `SymbolId` fast path, because a tail piece from `compound_parts` is a
  borrowed `&str` with no id of its own. Doubles as a bare stem's raw read
  in this module's own tests, since that is exactly the same operation.
* `stem_get`/`stem_set`/`stem_drop_tail`/`stem_assign`/`stem_drop(&mut
  self, stem_name: &[u8], ...)` -- the five D15a operations, plus two
  private helpers, `replace_stem` (the shared "build a fresh object and
  rebind" body of `stem_assign`'s wrap branch and `stem_drop`) and
  `derived_tail_name` (stem name + key, text). `is_stem` is the private
  check `stem_assign` uses for the share-vs-wrap decision above.

`rust/crates/rexx-exec/src/value.rs`: extended `to_text` with a
`Body::Stem` arm -- `default` if `Some` (a fresh recursive call, since a
default can itself be a stem through the sharing rule), else the object's
own `name`. Hit a real borrow-checker wrinkle here: `name.as_ref()` on the
match-ergonomically-bound `&mut Box<[u8]>` failed to borrow-check (E0515,
"cannot return value referencing local variable `name`") even though the
identical shape works one arm up (`text.get_or_insert_with(...).as_slice()`
for `Body::Num`); `&**name` (an explicit double deref) sidesteps whatever
`AsRef` impl `.as_ref()`'s method resolution was landing on. Recorded in
the code with the exact error, since it is the kind of thing worth
recognising a second time rather than re-debugging.

`rust/crates/rexx-exec/src/lib.rs`: added `mod stem;`. **Did not** touch
`step`'s `Assignment` arm, whose comment ("`addVariable` builds only
`Variable`, `Stem` or `Compound` here; the spike takes the first and Task 5
takes the others") reads as though it names this task directly. Read
Task 5's own Files/Interfaces list instead as the authoritative scope: it
asks for `stem.rs` as a self-contained library of primitives (which is
what every one of the six functions is, exercised entirely through direct
calls in this module's own tests, not through the instruction loop) and
says nothing about wiring `ExprKind::Stem`/`Compound` into `eval_node`'s
dispatch or `step`'s target-shape match, both of which do not exist yet
and are squarely Task 6 (the resolution plan, which has to learn to walk
`Stem`/`Compound` at all) and Task 7 (expression evaluation, whose own
brief lists `Stem`, `Compound` as forms it evaluates) territory. Flagging
this reading rather than guessing silently: if the team lead intended
Task 5 to also wire the `Assignment` arm, that is a small, mechanical
follow-up now that `stem_assign`/`stem_set` exist.

## Tests

`cargo test -p rexx-exec`: **16 unit tests total**: `stem::tests` has 8 new
(one per measured behaviour from the seven transcripts, plus a dedicated
discrimination test for the tombstone-vs-tag shape and the multi-level/
untouched-stem cases), `value::tests` still has its original 8 unchanged
from Task 4, plus the 10 pre-existing `tests/spike.rs` integration tests
and 2 doctests, all still passing. `cargo clippy -p rexx-exec --all-targets
-- -D warnings`: clean, exit 0, checked unpiped per the team lead's own
caught mistake (`clippy | tail` reports `tail`'s status). `cargo fmt -p
rexx-exec -- --check`: clean after one `cargo fmt` pass.

Two `#[allow]`s needed, both following Task 4's established precedent:
`dead_code` on the whole `impl Interp` block in `stem.rs` (nothing outside
this module's own tests calls any of these six functions yet, and since
dead-code analysis is transitive, allowing it once at the block level
covers the connected unreachable component rather than repeating the same
reason ten times over individual functions).

Did **not** create `rust/crates/rexx-exec/tests/stem.rs`; the plan's Files
list still names it but the Global constraints (added alongside this
task's answer) now say unit tests beside the subject for every module task.

## Oracle verification for the two new probes beyond the seven given transcripts

Both under `( ulimit -v 1048576; build/bin/rexx FILE )`, already quoted
above but summarised here for the record:

* `q.1='x'; y=q.; say y; say q.2; say q.` -> `Q.` / `Q.2` / `Q.` -- pins
  down that `Body::Stem.name` is read for a tails-only stem's bare render,
  through an alias, not just for a wholly-untouched variable.
* `a.=1; b.=a.; a.1=2; say b.1; a.=9; say b.; say b.1` -> `2` / `1` / `2`
  -- the sharing-vs-wrapping discriminator above.

## Commit

`28a62383`, "Stems: tombstones, aliasing, and a name the object carries
itself". Staged and committed directly (`rust/crates/rexx-exec/src/stem.rs`,
`src/lib.rs`, `src/value.rs` only -- confirmed via `git status --porcelain`
before staging that nothing from the concurrently active `rexx-parse`/
`rexx-extract` agents was in the index).

## Review follow-up

Team lead approved the algorithm, both signatures, and the "share vs wrap"
correction; confirmed all five decisive probes reproduce independently; and
asked for two things: move the `q.1='x'` justification for `Body::Stem.name`
into the type's own doc comment in `rexx-core` (reports are read once, code
comments are read every time), calling it a *better* justification than the
one already in the spec since it never bare-assigns the stem at all, so
there is no reference site's value to argue the name "travelled with"; and
add an explicit line contrasting "dropped whole stem" (indistinguishable
from never-touched) against "dropped tail" (a tombstone, which is *not*
indistinguishable from absent), since the two sit one level apart in the
same file and invite conflating them.

Both applied: `rust/crates/rexx-core/src/body.rs`'s `Body::Stem` doc comment
now leads with the tails-only case, and `stem_drop`'s doc comment carries
the explicit tombstone contrast. Also corrected in the same pass, since it
had gone stale: `stem_drop`'s comment previously said writing "unset" back
into a slot was impossible through the current `RootSet` API, which was
true when Task 5 was implemented but stopped being true when the team lead
added `RootSet::clear_slot` (for plain `DROP`'s own future task) shortly
before this review round -- the comment now explains why stems still use
`replace_stem` rather than `clear_slot`, rather than repeating a claim the
tree had already outgrown.

Verified after editing: `cargo build -p rexx-core -p rexx-exec`, `cargo test
-p rexx-core -p rexx-exec` (all green, unchanged counts), `cargo clippy
-p rexx-core -p rexx-exec --all-targets -- -D warnings` and `cargo fmt
--check` for both, all clean, all checked unpiped.

These two doc-only edits landed in `9b11836d` ("Audit fixes..."), not a
commit of my own: I staged them, and the team lead's own concurrent audit
commit swept them in before I ran `git commit`. Confirmed via `git show
9b11836d -- rust/crates/rexx-core/src/body.rs rust/crates/rexx-exec/src/
stem.rs` that the diff is byte-for-byte what I staged, nothing more and
nothing less.

## Full review, second round: one real defect in `stem_get`

`stem_get` used `stem_name` (the read site's own spelling) for two jobs:
finding the slot, and deriving an unresolved tail's name. Once aliasing
exists those want different names -- deriving needs the **object's own**
`name` field, the same rule `to_text` already applies to a bare read. Six
oracle probes (`a.1='x'; b.=a.; say b.2 -> A.2` and five more) confirmed
it. No existing test reached the aliased-and-unresolved combination: the
aliasing test's every read resolved (through a shared default or a
written tail), and `derived_tail_name` is reachable only when a tail does
*not* resolve, so the two tests never met each other.

Fixed in `stem_get`: the object's `name` field is cloned out alongside
the resolution decision (before any further `&mut self` call), and the
"no value found, object exists" branch derives from it instead of
`stem_name`. The "no object at all" early return still uses `stem_name`,
correctly -- there is no object to ask.

My session hit a usage limit mid-fix, leaving one `unreachable!` site
converted to call a not-yet-written `body_variant_name` helper (also
requested in review, replacing `{:?}` on a whole `Body::Stem`, which can
dump an arbitrarily large tails map -- the same message-size class Task 3d
bounded). The team lead finished it: wrote the helper, converted the other
two sites, verified unpiped, and committed as `7a628261`, crediting the
diagnosis and the doc comment (the four transcripts and the explanation of
why no test reached it) as mine. One placement lesson from that recovery,
worth keeping: a free function placed between an `#[allow(dead_code)]`
attribute and the `impl` block it governs makes the allowance land on the
new function instead, and every method in the `impl` becomes a dead-code
error -- an attribute and its target are a unit, nothing may sit between.

Tree verified clean at `7a628261`: `cargo test -p rexx-exec` 20 unit + 10
integration + 2 doctests, `clippy -D warnings` and `fmt --check` both
clean, both checked unpiped.
