# Task 6.11 re-review r2: fix round 2

Scoped re-review of `7de7d5a6c..3f5eaef22`. The code is `7efdf2dc5`, and the two commits after it
are records and report wording. I built `rexx` from HEAD into the scratchpad (`rev611r2-target`,
exit 0). Every probe ran from a fresh `mktemp -d`: the oracle under the standard wrapper, and the
crate under `memcap 2G timeout 20`. The comparison binary is
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.11/rexx-head` (`a22a686d0`). The probe sources
were in `scratchpad/rev611/r2.G0Wf/`, which is not durable, so the deciding lines are quoted
below.

## The earlier findings

All closed. I re-ran every probe from the review and the round-1 re-review on this build, and each
one is identical to the oracle on stdout, stderr and rc, except the queued frame gaps listed under
Minor 2.
* **Round-1 Important 1** (`forward class (.array)` past a run-time `define`): `fwd .array METHOD
  SORTWITH |        *-* Compiled method "SORTWITH" with scope "Array". | arrayexe 1 0 scope Array`
  on both sides.
* **Minor 2** (`run`): `run METHOD *UNNAMED* |        *-* Compiled method "*UNNAMED*" with scope
  ".NIL". | arrayexe 0 0 scope NIL` on both. That covers `run` from an Array subclass and from the
  class that defined the primitive.
* **Minor 3** (a FLOAT `setMethod` and `f~executable`): `setmethod float METHOD SORTWITH | ...
  scope ".NIL". | arrayexe 0 0 scope NIL` on both, rc 0. Before the round it was rc 120.
* The rest of the round-1 matrix still matches: two-deep `super`, `define` in `ACTIVATE`, a mixin
  (`WITHMIX`), `OBJECT` `setMethod` (`MYARR2`), `sendWith` on Array and on the defining class.
  So do the first review's frame probes: INTERPRET, an internal routine, a class method, nesting,
  SYNTAX trapped and untrapped, NOMETHOD, a `.nil` comparator, `TRACE I`, `EXIT` and `REPLY`
  inside `compare`.

## The new built-in kind

For `SORTWITH` and `STABLESORTWITH`, with `SORT` and `APPEND` as controls, every line below is
identical on the oracle, the round's parent and this build:
* `isPrivate`, `isGuarded`, `isProtected`, `isAbstract`, `isAttribute`, `isConstant`,
  `package == .nil`, `source~items` (`0`), `.Array~method(n) == .Array~method(n)` (`1`),
  `scope`, `hasMethod`, `instanceMethod` identity, `class`, `string`.
* `copy`: a distinct object, guarded, scope `The Array class`. After `setPrivate`,
  `setProtected` and `setUnguarded` on the copy it gives `1 1 0`, and the original still gives
  `0 0 1`.
* `.Method~new('x', m)`: 93.961 on all four names.
* A `setPrivate` copy `define`d into a subclass and sent: 97.2. A `setProtected` copy without a
  manager sorts (`1,2,3`). A `setGuarded` copy sorts.
* Listings: `cnt(.array~methods, 'SORTWITH')`, `cnt(.array~new~instanceMethods,
  'STABLESORTWITH')` and the subclass with the private copy give `2 2 3`.
* Arity, through the leveled arm's own check: `93.902 Too many arguments in invocation of method;
  1 expected.` and `93.903 Missing argument in method; argument 1 is required.`

The other send paths reach the leveled arm too, each with the same frame as the oracle:
* `.message~new(a, 'SORTWITH', 'I', c)~send`, `a~start('STABLESORTWITH', c)` followed by `wait`
  and `result`.
* `b~sortWith:.array(c)`, and `forward message 'SORTWITH' array (c)` (`METHOD VIAFORWARD`
  frame).
* The answer is the receiver (`result Array 2`). `~sortWith:.object(c)` gives 97.1.

`NativeBody::Leveled` is matched only at `dispatch.rs:2599`/`:2604` (the ordinary arm excludes it
and falls back to it) and at `:2850` (`run_other`). `rg NativeBody::` finds no other match.

## Errors in the comparator, and the frames after the sort returns

All identical to the oracle:
* Under `::options novalue error`, a local `signal on novalue` inside `compare`:
  `local novalue UNDEFINED_VAR 3`. The 26.903 that follows has frames `METHOD SORTWITH` and
  `PROGRAM`. Afterwards `.context~stackFrames~items` is `1`, a second sort's comparator sees `3`
  frames with `STABLESORTWITH` second, and the count is `1` again after it returns. An untrapped
  NOVALUE in `compare` gives `98.986 Reference to unassigned variable "STILL_UNDEFINED2".` with
  the `Compiled method "SORTWITH" with scope "Array".` traceback line.
* `RAISE USER` in `compare`, trapped by `signal on any`: `USER BOOM kaboom` with frames
  `COMPARE`, `SORTWITH`, `PROGRAM`.
* A non-numeric answer gives 26.903. A builtin error inside `compare` gives 93.924 with the same
  traceback.

## Perf

`callgrind.sh -r 1 -B 0.5 -p "churn sendloop dispatch"` with pin, the task's parent
(`t6.10-r3/rexx-head`), round 1 (`t6.11-r1/rexx-head`) and this build, exit 0:

| program | parent -libc | r1 -libc | head -libc | round own (head - r1) | task own (head - parent) |
|---|---|---|---|---|---|
| churn | 379070660 | 379073837 | 379194571 | +0.0319 | +0.0327 |
| sendloop | 13788018100 | 13788020876 | 13788021263 | 0.0000 | 0.0000 |
| dispatch | 20643708714 | 20643711790 | 20643712304 | 0.0000 | 0.0000 |

The raw figures are churn +0.0199 (round) / +0.0244 (task), and sendloop and dispatch +0.0000
(round). churn's round delta, +120,798 Ir with libc excluded, sits entirely in `begin_invoke`'s
inclusive cost: 39,133,475 to 39,254,097. By line it is attributed to `begin_invoke`'s inlined
`body.rs` and `scheduler.rs` lines, with `dispatch.rs`'s own lines at -40,566. So it is
instructions the send path executes after codegen moved, not an unrelated layout effect. That is
consistent with the report's account, and it is under the +0.05% budget. sendloop and dispatch
pay nothing.

## Tests

* `cargo test --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1123 of 1123 matching`.
* `-p rexx-parse --test sourceline_oracle`: exit 0.

## Issues

### Critical

None.

### Important

None.

### Minor (pre-existing, for the queue)

1. **A security manager is not consulted for a protected primitive.** This holds for every
   primitive, not only the leveled kind: `.Array~method('APPEND')~copy~setProtected` `define`d
   into a subclass and called from a `.Routine` with `setSecurityManager` gives `mgr method APPEND`
   on the oracle and nothing on the crate. The parent is the same. `SORTWITH` behaves the same way.
2. **The `Message~send` frame is missing.** `msg~send` of `SORTWITH` lists `METHOD SEND ...
   Compiled method "SEND" with scope "Message".` on the oracle and not on the crate. This is the
   queued fexe3 `METHOD SEND` family, together with `SENDWITH` and `Routine~call` from the earlier
   reviews.

## Assessment

**Verdict: APPROVED.** The round closes the round-1 Important and both of its Minors. The new
built-in kind is invisible to every method-kind inspection I could run. The round adds +0.032% on
churn with libc subtracted and nothing on sendloop or dispatch.
