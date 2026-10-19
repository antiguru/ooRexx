# Task 6.11 review: classes, methods and packages

Reviewed `55681370e..8f749b048`. The tree is at `95d22fbdd`, whose only change past the range is
two lines in `progress.md` (`git diff --stat 8f749b048 95d22fbdd`). I built `rexx` from it into the
scratchpad (`rev611-target`, `cargo build --release -j 4 --bin rexx`, exit 0). Every probe ran
from its own fresh `mktemp -d`: the oracle under the standard wrapper, and the crate under
`memcap 2G timeout 20`. I compared stdout, stderr and rc separately. The parent binary is
`/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.10-r3/rexx-head`, which carries `55681370e`'s code.
I used it to sort each remaining difference into "changed by this task" or "pre-existing". The
probe sources were in `scratchpad/rev611/probes.kmWt/`, which is not durable, so the deciding lines
are quoted below.

## Spec compliance

Compliant for the seven clusters. Every witness and every cluster test the report names is
green: the corpus gate gives 1121 of 1121, and the witnesses' oracle stdout prints each path the
report claims. Report Concern 2 does not hold: `FLOAT` then `OBJECT` for one name lists the same
entries as the oracle (Strengths). Report Concern 3 does hold, and it is wider than the report
says (Important 1).

## Strengths

* **The install order matches the oracle's `resolveDependencies` on every shape I built.** The
  programs that printed activate order all agree with the oracle on stdout, stderr and rc:
  * A forward `SUBCLASS` chain and an `INHERIT` of a mixin declared later. Also a class that
    subclasses a later class, which itself subclasses an earlier `INHERIT`-bearing class. That gives
    `c e d a b m x z` on both, where the parent printed `e d a b c m x z`.
  * `METACLASS` naming a later metaclass whose superclass is declared later still, including the
    metaclass's `INIT` running per class: `init:K init:K meta1 meta2 k plain`.
  * A 30-class reverse `SUBCLASS` chain.
  * Names given as quoted strings in mixed case.
  * Classes in a `::REQUIRES` file that subclass each other forward, plus a main-file class that
    subclasses one of them: `r1 r2 ly lx lz`.
  * `.Package~new` (`pk  b a d c e`), a `.Package~new` built under INTERPRET (`pk3  r q`),
    `.Method~new` with classes (`meth  t s`), and `.Routine~new` with classes (`act b`, `act a`).
  * Cycles: mutual `SUBCLASS` behind an independent class, a cycle through `INHERIT`/mixin, and a
    class naming itself. Each gives 98.911 blamed on the same directive line. A missing superclass
    gives 98.909, and duplicate names give 99.901, on both.
  * The `::CONSTANT` blame target now follows the new order. `::class a subclass c` / `::class b`
    / `::class c` / `::constant x (1/0)` blames `2 *-* ::class a subclass c` on both. The parent's
    stderr differed.
* **Method identity and `methods`/`instanceMethods` match as multisets** on a class with a mixin,
  an `inherit`/`uninherit` round, `define` of a new name, `define` hiding (`hide FOO 2 0 The NIL
  object 2`), `delete`, and a redefinition on the superclass (`redef base FOO 2 1 1`). The parent
  differed on the mixin, inherit, hide, delete and redefinition lines. The class-side `methods` and the mixin's class method also
  match.
* **Report Concern 2 is settled by running it.** An instance that gets `setMethod('BAR', ...,
  'FLOAT')` and then `setMethod('BAR', ..., 'OBJECT')` lists `BAR@T BAR@T` among its 37 entries on
  both sides. The `.nil`/`.t` filters, the hide (`bar hidden The NIL object 0`), enhanced objects
  (`enh nil 2: E1@NIL FOO@NIL`, `enh ident 1 1`), `copy`, and `unsetMethod` are all identical too.
* **`Package~prolog`'s synthetic-directive test** answers like the oracle on all the shapes I
  tried. A Routine for `.Package~new` with an empty-bodied `::method`, `::routine`, `::attribute` or
  `::constant`, with no code, or with `::requires`. `.nil` for `.Routine~new`/`.Method~new` with
  empty, array and directive-bearing sources, and for `~newFile` of both kinds. A Routine for a
  required file and under INTERPRET.
* **`>I>`/`<I<` names** match the oracle for `~newFile` routines and methods, a routine defined
  inside a `.Routine~new` source (`in package "withroutine"`), a `.Routine~new` under INTERPRET, a
  `.Package~new` class method, an empty name, a name with spaces, and an external `CALL`.
* **UNINIT through `define` runs where the oracle runs it.** Collection timing is not compared
  (licensed), only whether each UNINIT runs at all. Both sides run the same set: a grandchild
  (`uninit C`, which the parent never ran), instances created before and after the `define`
  (`uninit Second TAG` / `uninit Third TAG` once each on both), and an override in the subclass
  deleted after the superclass gains one (`d1 uninit D2`). A `delete` before collection gives none
  on both. `inherit`/`uninherit` of a mixin with UNINIT gives the same set on both.
* **The sortWith level** matches for the plain call, under INTERPRET, from an internal routine,
  from a class method, nested (a comparator that sorts with another comparator), with a SYNTAX in
  the comparator trapped (`condition('o')~stackFrames`) or untrapped (the `Compiled method` line in
  the traceback), with `RAISE SYNTAX` in the comparator, with NOMETHOD, with a `.nil` comparator,
  with `TRACE I` inside `compare`, with `EXIT` inside `compare`, and with `REPLY` inside `compare`.
* **Perf reproduces, and the libc attribution holds.** I ran `callgrind.sh -r 1 -B 0.5 -p
  "startup interpret sendloop dispatch" pin=... parent=t6.10-r3/rexx-head head=<my build>`, exit 0:

  | program | parent | head | raw own | -libc own |
  |---|---|---|---|---|
  | startup | 119146081 | 119168563 | +0.0189 | +0.0058 |
  | interpret | 4115983576 | 4132899573 | +0.4110 | +0.0002 |
  | sendloop | 14555023156 | 14595030847 | +0.2749 | 0.0000 |
  | dispatch | 22035816964 | 22075819329 | +0.1815 | 0.0000 |

  In sendloop, all of the +40.0M raw is one glibc function: `__memcmp_avx2_movbe` goes from
  180,533,034 to 220,534,283 Ir, and the other libc rows are equal to within 1k. The libc-subtracted
  sendloop and dispatch figures are equal, so the method-identity change adds no per-send cost. The
  only call that pushes a level is `sort_with`. This supports the controller's ruling.
* **Tests.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1155 passed.
  * `REXX_CORPUS_GATE=1 ... --test corpus`: exit 0, `1121 of 1121 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.
  * `-p rexx-classes`: exit 0.

## Issues

### Critical

None.

### Important

1. **A `sortWith`/`stableSortWith` reached through a subclass's override gets the wrong scope and
   the wrong executable** (report Concern 3, measured). `in_method_level` takes its scope from
   `lookup(receiver, name)`. That finds the subclass's Rexx `SORTWITH`, not the `Array` native
   that is running. The traceLine is wrong, and so is `executable`, which the report's Concern
   does not mention. Probe: `::class myarr subclass array` / `::method sortWith` / `forward class
   (super)`, with a comparator recording `.context~stackFrames`. Frame 2:
   * oracle: `sub METHOD SORTWITH The NIL object array 1        *-* Compiled method "SORTWITH" with scope "Array".`
   * crate:  `sub METHOD SORTWITH The NIL object array 1        *-* Compiled method "SORTWITH" with scope "MYARR".`
   * `stableSortWith` gives the same pair with `STABLESORTWITH`.
   * `f~executable == .Array~method('SORTWITH')`: oracle `1`, crate `0`. `f~executable ==
     .myarr~method('SORTWITH')`: oracle `0`, crate `1`. `f~executable~scope~id`: oracle `Array`,
     crate `MYARR`.

   The untrapped error's traceback is right on both (`Compiled method "SORTWITH" with scope
   "Array".`), which comes from a different path. So the frame and the traceback disagree within
   one crate run. `.other~define('SORTWITH', .array~method('SORTWITH'))` gives `scope "OTHER"` on
   both, so the scope has to come from the method that was actually dispatched, not from a
   constant `Array`. Fix: hand `in_method_level` the scope the dispatch resolved for the running
   native, or look up from the method object being run, instead of re-resolving `name` from the
   receiver. Add a witness line using the forwarding subclass.

### Minor

2. **`instanceMethods` on a class object omits the class's own `setMethod` entries**, though
   `instanceMethod` now finds them. `own_method_entry` reaches a class's own methods through
   `class_own_entry` (`dispatch.rs:2199`), but `object_level_entries` (`introspection.rs`) only
   matches `Body::Instance`. Probe: `::class sub` / `::method sm class` / `forward message
   'SETMETHOD'`, then `.sub~sm('OWN', 'return 2')`:
   * `.sub~instanceMethod('OWN')`: `Method return 2` on both (the parent raised).
   * `cnt(.sub~instanceMethods, 'OWN')` / `cnt(.sub~instanceMethods(.nil), 'OWN')`: oracle `1 1`,
     crate `0 0`, parent `0 0`.
   * The same shape on an instance gives `instance own 1 1` on both.

   This is not a regression, but it sits in the cluster's own new code, and the fix is to read
   the pool owner the way `own_method_entry` does.
3. **The UNINIT witness's stdout puts the UNINIT line where the oracle happens to collect it**
   (`uninit ran for Second` before `after first collection`, with the `pad` line keeping the
   object out of the last allocations). Collection timing is unspecified under the GC-ordering
   licence. In my other shapes the crate ran the same UNINITs later than the oracle. For example,
   `u4` (a grandchild) prints `uninit C` before `end` on the oracle and after `end` on the crate.
   The witness matches today, but a collector change that does not alter what runs could turn it
   red. Consider printing the set at exit instead, or comparing it as a multiset after several
   forced collections.
4. **Divergences that predate the task, all present in the parent, for the queue:**
   * A cycle inside `.Package~new('pk2', ('::class x subclass y', '::class y subclass x'))`: the
     oracle says `Error 98 running pk2 line 1` / `Cyclic inheritance in program "pk2".`, the crate
     names the main file in both lines. This is another instance of
     `2026-10-18-install-error-names-caller-file.md`.
   * `b~sendWith('SORTWITH', .array~of(c))`: the oracle lists `METHOD SENDWITH ... Compiled method
     "SENDWITH" with scope "Object".` between the SORTWITH frame and the program frame, and the
     crate has no such frame. This is the same family as the report's fexe3 `METHOD SEND` gap.
   * `.Routine~new(...)~call` around a sort: the oracle has `ROUTINE rr` plus `METHOD CALL`, the
     crate has `ROUTINE CALL`. This is the queued `2026-10-17-routine-call-frame-name.md` (q4).

## Assessment

**Task quality:** Good, with one targeted fix needed.

**Reasoning:** The install order, method identity, the every-scope listings, prolog, package
names and the UNINIT cascade match the oracle on every edge I could build around them, mixins,
enhanced objects, INTERPRET, `::REQUIRES` and cycles included. The libc-subtracted perf shows no
cost. The sortWith frame is right except when a subclass forwards to the native. There the frame's
scope and executable name the subclass. The report flagged that case as unmeasured. Measured, it
is wrong on both fields, and the fix is local to `in_method_level`'s scope source.

**Verdict: CHANGES REQUESTED** (Important 1. Minor 2 is a few lines in the same area and could ride
along. Minors 3 and 4 can be queued.)
