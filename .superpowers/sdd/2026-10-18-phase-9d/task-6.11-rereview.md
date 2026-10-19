# Task 6.11 re-review: fix round 1

Scoped re-review of `bb0a7ed1f..ef78325e5`. The code is `feeca3405`, and `ef78325e5` adds only
records. I built `rexx` from HEAD into the scratchpad (`rev611r1-target`, exit 0). Every probe ran
from a fresh `mktemp -d`: the oracle under the standard wrapper, and the crate under
`memcap 2G timeout 20`. The comparison binaries were
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.11/rexx-head`, the round's parent `a22a686d0`
(its mtime matches that commit's), and `t6.10-r3/rexx-head`, the task's parent. The probe sources
were in `scratchpad/rev611/r1.sJvD/`, which is not durable, so the deciding lines are quoted below.
One slip: I ran a single six-line probe on `t6.10-r3/rexx-head` without `memcap`. It finished at
once with rc 120.

## Scope

* Important 1 (the frame scope): fixed for the shapes the round names, but not for the general
  rule (Important 1 below).
* Minor 2 (a class object's own `instanceMethods`): closed. `m5`: `all 1 nil 1` on both the oracle
  and the crate, where the round's parent gave `all 0 nil 0`. My earlier `m4` probe and the new
  witness `class_setmethod_instance_methods` are identical to the oracle.
* Minor 3 (the UNINIT witness): closed. Eight oracle runs and eight plain crate runs all print
  `defined: 1 deleted: 0`. So do 20 seeds under `REXX_SWITCH_MODE=sim:S,gc=0.01` and 5 under
  `gc=0.5`. The task's parent `t6.10-r3/rexx-head` prints `defined: 0`, so the witness can still go
  red.

## Where "highest-priority primitive" and "the method dispatched" differ

Each case ran in its own program, and every line below is the frame `.context~stackFrames[2]` that
the comparator recorded.

| case | oracle | crate | |
|---|---|---|---|
| subclass forwards to `super`, two deep (`deep2` -> `deep1` -> Array) | scope `Array`, executable `.Array~method` | same | match |
| `.other2~define('SORTWITH', .array~method(...))` at run time; `fsub2 subclass other2` forwards with `forward class (super)` (STABLESORTWITH) | `OTHER2` | `OTHER2` | match |
| same, through an intermediate `mid` override forwarding to super | `OTHER2` | `OTHER2` | match |
| same, `fsub2`'s `sortWith` does `forward class (.array)` | `Array`, executable is Array's (`arrayexe 1`) | **`OTHER2`, `arrayexe 0`** | differs |
| mixin `mx` that defines Array's SORTWITH, inherited by `withmix` | `WITHMIX` | `WITHMIX` | match |
| instance `setMethod('SORTWITH', .array~method(...), 'OBJECT')` | `MYARR2` | `MYARR2` | match |
| instance `setMethod('SORTWITH', .array~method(...))` (FLOAT) | traceLine `scope ".NIL"`, executable answers | traceLine matches; **`f~executable` aborts** | differs |
| `self~run(.array~method('SORTWITH'), 'I', c)` on an Array subclass | **`METHOD *UNNAMED*`, `scope ".NIL"`** | `METHOD SORTWITH`, `scope "Array"` | differs |
| `.array~of(2,1)~sendWith('SORTWITH', ...)` and on `.other` | `Array` / `OTHER` | same | match |

A caution for anyone repeating these: the oracle's own behaviour hides some shapes. A `define`
run in a superclass's `::METHOD ACTIVATE CLASS` replaces a subclass's own Rexx override with the
primitive, scoped to the subclass. This happens on both sides: `.fsub~methods` lists `FSUB 0
OTHER 0 Array 0`, and the forwarder never runs. A `forward` probe therefore has to `define` from
main code after install, as above.

## Issues

### Critical

None.

### Important

1. **`forward class (.array)` past a class that defined Array's primitive names the wrong
   method.** `primitive_scope` takes the first primitive entry in the receiver's dictionary. A
   scoped forward starts its lookup at `.array`, so the method it runs is Array's, and the oracle's
   frame names it. The probe is `.other2~define('SORTWITH', .array~method('SORTWITH'))` from main
   code, then `::class fsub2 subclass other2` / `::method sortWith` / `say 'forwarder ran'` /
   `forward class (.array)`:
   * oracle: `fwd .array METHOD SORTWITH |        *-* Compiled method "SORTWITH" with scope "Array". | arrayexe 1 0 scope Array`
   * crate:  `fwd .array METHOD SORTWITH |        *-* Compiled method "SORTWITH" with scope "OTHER2". | arrayexe 0 0 scope OTHER2`
   * round's parent: `... with scope "FSUB2". | arrayexe 0 0 scope FSUB2`

   The round's doc comment ("the one a send or a `forward class (super)` from an override
   reached") is true only when the lookup starts at the receiver or at its immediate super. Any
   rule that re-derives the method from the receiver's dictionary can be wrong for a scoped send.
   The frame needs the method the dispatch actually ran. That is the same fix the `run` case
   (Minor 2) needs.

### Minor

2. **`run` of Array's method gives the wrong name and scope**, unchanged by this round (the
   round's parent gives the same). The probe is `self~run(.array~method('SORTWITH'), 'I', c)`
   inside an Array subclass's method:
   * oracle: `run METHOD *UNNAMED* |        *-* Compiled method "*UNNAMED*" with scope ".NIL". | arrayexe 0 0 scope NIL`
   * crate:  `run METHOD SORTWITH |        *-* Compiled method "SORTWITH" with scope "Array". | arrayexe 1 0 scope Array`

   The level's name is hard-coded in `sort_with` (`b"SORTWITH"`), and the oracle uses the message
   name the native was invoked under. The fix for Important 1, threading the dispatched method and
   message name into the level, closes this too. Otherwise queue it.
3. **`f~executable` aborts for a FLOAT `setMethod` of the primitive**, also unchanged by this
   round. The probe is `y~sm('SORTWITH', .array~method('SORTWITH'))` on an Array subclass
   instance, then `y~sortWith(c)` and `c~frames[2]~executable`. The oracle answers a Method whose
   scope is `.nil`. The crate prints `rexx-exec: method "EXECUTABLE" of class "StackFrame" is not
   implemented (Phase 9)`, rc 120. The traceLine and the condition's frames match
   (`cond METHOD SORTWITH        *-* Compiled method "SORTWITH" with scope ".NIL".` on both).
   `build_method_level_frame` looks the executable up by `(scope, name)`, and a `.nil` scope has
   no dictionary. The own entry's `executable` is the object to answer.
4. **Pre-existing, for the queue** (the task's parent `t6.10-r3` gives the same):
   * `.k~define('SORTWITH', .array~method('SORTWITH'))` on a non-Array class, then
     `.ksub~new~sortWith(...)`: the oracle raises `98.975 Missing array element at position 2.`,
     the crate aborts with `a message send to a value that is not an array is not implemented`,
     rc 120.
   * `.OrderedCollection~method('SORTWITH')~source~items`: oracle `0`, crate `1` (`forward
     message('STABLESORTWITH') -- just use the stable variety`). The crate's OrderedCollection
     `SORTWITH` is a Rexx stub where the oracle's is native.

## Checks

* The new witnesses `stackframes_sortwith_dispatched_scope` and
  `class_setmethod_instance_methods`, and the rewritten UNINIT witness: identical to the oracle on
  stdout, stderr and rc.
* `cargo test --release -p rexx-exec --lib`: exit 0, 1155 passed.
* `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1123 of 1123 matching`.
* `-p rexx-parse --test sourceline_oracle`: exit 0.
* I did not repeat the perf check. The round's change is confined to the `#[cold]`
  `primitive_scope` on the sort path and to introspection.

## Assessment

**Verdict: CHANGES REQUESTED.** The fix is Important 1. Minors 2 and 3 share its cause, which is
that the level does not carry the method it runs. Minor 4 can be queued.
