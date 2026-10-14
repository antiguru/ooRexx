# Phase 9b1 spec review

Spec: `docs/superpowers/specs/2026-10-14-phase-9b1-uniform-object-model.md` (untracked). Tree
`ce3793971`. Read only. Code citations are against HEAD; the crate binary
`rust/target/release/rexx-run` is 2026-10-13 22:19, after the last `rust/crates` commit
(`1590a1cea`, 22:18:19). Oracle `/home/moritz/dev/repos/ooRexx/build/bin/rexx` under the standard
wrapper. Five probes of my own ran once per side from a fresh directory each
(`scratchpad/review/run.sh`, sources in `scratchpad/review/probes/`); their outputs are quoted where
used. The five decisions Moritz approved are taken as given.

**Verdict: not approved as written.** One criterion is green before any work starts and cannot be
checked where it says to look (H1). Five decisions the implementation needs are missing or
half-stated (M2, M3, M4, M5, M6), and one factual claim is false (M1). The +0.5% budget is realistic
for the send, call and variable paths and not shown for four programs that 9b1 also touches (M7).
Nothing in the approved direction is in question.

## High

**H1. Criterion 2 (section 6) is green at HEAD and is not checkable on the TSV it names.**
`rust/corpus/refusal-sites.tsv` records constructors, surfaces and probes, not the per-site noun
(its header, lines 5-14; the scout says the same, scout-leaks "(a)"). At HEAD,
`grep -E "did not build|does not keep" rust/corpus/refusal-sites.tsv` matches nothing, while the
source has the wordings at `dispatch/package.rs:129`, `:1163`, `:1179`, `dispatch/class_protocol.rs:436`,
`:501`, `:543`, `:918`, `:1132`, `dispatch/executable.rs:110`, `:580`, `:620`, `dispatch/context.rs:1024`,
`:1025`, `:1110`, `dispatch/object_protocol.rs:398`, `:654`, `dispatch/construct.rs:231` and
`lib.rs:556`, with "no longer live" at `environment.rs:1340`, `construct.rs:206`,
`object_protocol.rs:448`, `:467`, `:677` and `dispatch.rs:1824`. The check has to be the source grep
(the scout's command (a), line 32 of scout-leaks). The criterion must also say which nouns go
(package, executable, routine object, method object, stack frame, "an object") and which stay with
a citation: `lib.rs:556` is Deviation 32 and "no longer live" is a dead-handle guard. "a value or
package" does not decide whether "a method object this crate did not build" counts. The second
clause ("the only identity refusal left") has no column to check either; name the constructor
(`object_name_of_a_value`, `lib.rs:566`, TSV row 176).

## Medium

**M1. Section 5's "except the registry's `by_name`" is false.** `ClassRegistry::expunge`
(`rust/crates/rexx-classes/src/registry.rs:289-308`) runs `by_name.retain(|_, class| !dead.contains(class))`
at `:306`, and `collect_now` calls it with the freed classes (`rust/crates/rexx-exec/src/lib.rs:3205-3207`).
The scout's sentence ("Pruned at sweep through `freed_classes`, except `by_name`", scout-leaks
line 424) is wrong in the same way. Delete the exception, or a planner budgets a task that has no
subject.

**M2. `constant_values` and `package_tables` are not "immutable compiled data".**
`constant_values: FxHashMap<(ProgramId, usize), ObjRef>` (`lib.rs:1469`) holds objects and is a
global root under `constant_root_key` (`lib.rs:3021-3022`); `package_tables` (`lib.rs:1438`) is a
global root under `package_table_root_key` (`lib.rs:3008-3009`) and is in neither section 3's field
list nor its "may stay keyed" list. Section 3 bullet 2 files constant values under state 3. Left
there, every dead package leaves one global root per `::CONSTANT`, so a heap constant outlives its
package and the row is never pruned, while "a package nothing references is collected" reads green.
Both belong in the package's fields, traced through it.

**M3. Program lifetime and `ProgramId` reuse are undecided, and section 3 states a mechanism that
does not exist.** "the handle keeps the program alive exactly as long as some object needs it" is
not true of the tree: `programs: Vec<Rc<Program>>` (`lib.rs:1304`) is itself a strong, append-only
holder (`install.rs:690-691`, `:994-995`, `:1729-1730`, `:1864-1865`, `trace.rs:860-861`), and
`plans` (`:1323`), `chunks` (`:1352`), `package_options` (`:1308`), `routines` (`:1360`),
`compiled_method_names` (`:1504`) and the rest key by `ProgramId`
(`plan.rs:57-62`, `BodyKey { program: ProgramId, .. }`). `Rc<Program>` has no drop hook the
collector sees. The spec has to decide: whether `programs` becomes weak or goes; that `ProgramId`s
are never reused (a reused id makes a stale `plans`/`chunks` entry answer for a new program, a wrong
answer, not a leak); and what prunes the id-keyed maps when the last handle drops. Section 4
bullet 1 states the outcome; the 80000-loop criterion is decided by the mechanism.

**M4. L1 decides the held case only.** "never cleared while it is reachable" fixes W13-W16 and W18.
The dropped case is open: W3 (`'ab'||'c'` dropped) and W5 (`1000+1` dropped) agree today at
"cleared" (scout-deviations table), and the scout's fix (treat a non-heap target as alive at
`heap.rs:282`) flips both to "never cleared" (scout-leaks L1, "The other way"). State that an
inline referent is never cleared and license W3/W5 under the GC-ordering licence by name; "never"
is a limit of timing only once the spec says it is. The alternative the implementer will otherwise
reach for, materialising at `WeakReference~new`, changes `w~value`'s identity (scout-leaks L1).

**M5. L12's licence is not one of the approved decisions.** Section 2 records L12 "licensed as
unspecified (a new Deviation)". The principle makes a representation-only difference a defect
"unless Moritz has ruled a Deviation for it" (`rust/CLAUDE.md:14-15`); the five approved decisions
do not include it. Mark it proposed, or cite the ruling.

**M6. Section 1's guarantees have no criterion and no witness.** Bullet 3 (subclass and enhanced
strings keep full identity and accept `objectName=`) and bullet 4 (a held value's `identityHash` is
stable) are rules the L1, L2 and L4 work can break, and section 6 checks none of them; criterion 1
covers the scout's leak rows only. Measured today, every one holds on both sides: probes `e1`,
`e2a`, `e2b`, `e2c` (rc 0 both sides, byte-identical): enhanced long and short strings and a
`::class mystr subclass string` instance take `objectName=`, `~copy` carries the name, `use arg`
names the caller's value, `IdentityTable` membership follows the object, `(e || '')` answers the
receiver (`E04`, `E16`, `R9`, `S4` all `1`), and a weak reference to a held enhanced or subclass
value is not cleared after `gc 'force'` (`W1 0 0`). Add a corpus program pinning them as a criterion.
Bullet 4's list also omits collection round trips (scout-deviations C13-C16, stem, array,
directory); add them.

**M7. The +0.5% budget is realistic for the hot paths and not shown for four programs.** Against the
pin, the 9b close (`docs/superpowers/records/2026-10-13-phase-9b/close2/callgrind.log`) already
stands at hashnew +0.3065%, startup +0.2930%, pingguard +0.2696%, alloc +0.2272%, sayloop +0.1926%.
9b1 touches what those measure: `startup` builds the REXX and bootstrap packages (now heap objects
traced by the collector instead of global roots), `alloc` and `hashnew` run the trigger (section 4
bullet 3 changes what is charged) and the sweep. The call, send and variable programs have room
(fibcall −3.4351%, fibfunc −3.5383%, extcall −13.2913%, dispatch −0.0664%, sendloop −0.1004%,
varlookup +0.0027%, rexxcps −0.0316%). The spec should quote the per-program headroom from that
table, name the pin by path (the close log records `/home/moritz/dev/repos/ooRexx-p9-pin/`; "the
Phase 9 pin" is not one), and say what a `startup` overrun means at 119 M Ir, where a few thousand
instructions are 0.1%.

Where the indirection lands (brief item 5). CALL resolution is cached per site
(`ir.rs:753` `resolved_call`, keyed by `routine_generation`; `ir/drive.rs:2013`, `:2139`, `:4090`),
so `find_routine` (`environment/identities.rs:930-945`, walks `routines` and
`merged_public_routines` up `package_lineage`) is cold. The per-entry reads on the call and send
paths are: `options_of` → `package_options.get(&program_id)` (`lib.rs:2445`, from
`start_from_package` at `:2458` and the entry at `:2537`); `Rc::clone(&self.programs[..])`
(`run/call.rs:966`, `dispatch.rs:3209`, `lib.rs:2561`); and `plans.get(&BodyKey)` / `chunks.get`
(`plan.rs:736`, `:781`). Variable lookup reads no package state. Environment symbols read
`package_classes`/`merged_public_classes`, then `rexx_class_cache`, then `package_locals.get`
(`environment.rs:596-612`). A heap package turns the `FxHashMap<ProgramId, _>` lookups into
resolve-plus-field, equal or cheaper, provided the activation keeps `program_id` (`activation.rs:262`)
or an `Rc<Program>` and does not resolve the package per clause. One structural risk the spec does
not name: `Body` lives in `rexx-core`, which depends on `rexx-num` and `rustc-hash` only
(`rust/crates/rexx-core/Cargo.toml`), and `Program` is `rexx-parse`'s (`rexx-parse/src/lib.rs:64`),
so no `Body` variant can hold `Rc<Program>` by name, and `NativeObject` (`body.rs:916-922`) carries a
class, rendered text and an entries table. Either `rexx-core` gains the dependency (or an erased
payload), or the package body holds an id and the program table stays, which is M3 again.

**M8. Criterion 1 names section 3 and not section 4.** L8 and L9 are section 4's, so under the
criterion's own wording they are neither fixed, licensed nor dissolved. "dissolved by section 3 or
4".

## Low

**L1.** "renamed back in" (section 3): nothing renames `TESTRECLAIM01` out statically. The harness
renames out the test running when the timeout ends (`rexx-exec/tests/suite.rs:20-22`), and the 9b
close records it as `blocked: timeout after 60 s` (`close2/suite/tests.tsv:9891`). The criterion
is that it passes inside the timeout; there is no list to edit.

**L2.** Section 1 bullet 1 is two-directional ("whether two equal values are one object is
unspecified"); the narrowing of Deviation 4 is one-directional ("may be one object"). C17 (literal
versus an unset symbol of the same text: oracle one object, crate two) is covered by the first and
not the second. "may or may not be one object".

**L3.** Scope the no-op licence to value-based values. For enhanced and subclass strings the
receiver-answering no-ops are visible through methods, both sides agree today (`E04`, `E16`, `R9`,
`S4` above), and bullet 3 gives those objects full identity; an unscoped "plus the no-op set" lets
an implementation regress them.

**L4.** Say that `.true` and `.false` are value-based. The deviations scout lists "`.true`/`.false`
stay distinct from every computed `1`/`0`" as a must; the crate keeps them distinct today
(`e1` `T1`: `.true~identityHash == 1~identityHash` is `0` on both sides), the crate's DO does not
depend on it (Deviation 25 judges by string value, exclusions `:1739-1752`), and
`.true~objectName = 'TRUE'` is named on the oracle (`e2d`: `TRUE TRUE a String`) and refused here
(rc 120), which section 1 licenses. The spec is silent on a case the scout flagged.

**L5.** Section 4 bullet 3: a trigger counts what is charged at allocation
(`Heap::charge_body_bytes`, `rexx-core/src/heap.rs:172`), not "the weight ... it frees". Classes
built by `~subclass`, `~mixinclass` or `enhanced` are charged already
(`2026-10-12-gc-trigger-blind-to-class-weight.md`, measured at `70d314bac`); `::CLASS` directives,
packages and programs are not. Say which.

**L6.** "1.66 GB": `/usr/bin/time -f %M` reports KiB; 1658768 KiB is 1.70 GB (1.58 GiB).

**L7.** "No ooTest test, corpus program, sample or extension names a plain string" holds. Derived:
`grep -rn -iE "objectname *=|~objectname\(" ootest/ rust/corpus/ samples/ extensions/` (receivers:
arrays, `.Object~new`, `.context`, packages, routines, stems, MutableBuffer, Directory, classes,
`.RexxInfo`; `MethodArgs.testGroup:148` passes a non-string argument to an Object's `objectName=`).
Record the command beside the claim.

**L8.** Section 7 cites `66b0f6854` as closing the scope move; that commit is "Phase 6.1 Task 1:
Activation layout for the scope move", the layout. The fields are on `Activation` at HEAD
(`activation.rs:375` `elapsed_anchor`, `:583` `random_seed`, `:586` `locals`) and the scout's
three probes agree, but the REPLY-seam variants were not re-probed (scout-leaks "(e)"). "closed
it" is what the queued-file check will establish.

**L9.** Criterion 5's "measures its programs at the task" is not mechanical. Name them: fibcall,
fibfunc, extcall, dispatch, dispatchclass, sendloop, varlookup, pkglocal, startup.

**L10.** Section 3 bullet 3: the REXX package has no program (`Package::Rexx`), so "shares the
program" needs "where there is one".

**L11.** Wording. No em-dash and no `--` in the file. Semicolons at lines 17, 75 and 101 join
independent clauses; lines 89-91 end list items. Line 3's "Sits between 9b and 9c" and section 3's
"the handle keeps the program alive exactly as long as some object needs it" describe a future
mechanism as present (M3).

## Per check

1. Claims: true except M1 (`by_name`), M2 (constant values as compiled data), M3 (the handle
   sentence), L1 (renamed out), L6 (1.66 GB), L8 (the commit). Verified true: `heap.rs:282`
   (L1's cause); `eval.rs:843-846` and `run/loops.rs:1062-1066` (L2, `to_number` before the receiver
   test); `dispatch/reqstr.rs:198` (L3, `Primitive::Instance` only); `object_packages` (`lib.rs:1431`)
   and the package global roots (`lib.rs:3003-3005`); `PackageManager::addToRequiresCache` builds a
   `WeakReference` (`interpreter/package/PackageManager.cpp:921-929`, cleared entry removed at
   `:909`); the Phase 9 spec row (`2026-10-10-phase-9-core-conformance.md:332`); Deviations 4, 31 and
   33 at `phase-4-exclusions.txt:929`, `:1945`, `:2022`; the close record
   `close2/suite/tests.tsv`; `callgrind.sh -B` judges each program's whole-process delta against the
   first binary (`rust/bench-programs/callgrind.sh:5-19`); the close command is the Phase 9 spec's
   G4 (`:343-344`).
2. Criteria: 1 mechanical (a 12-row table), wording gap M8. 2 not mechanical and green today, H1.
   3 mechanical. 4 mechanical and meaningful (the crate's floor is about 22 MB by the scout's
   `interpret` row, so 2x of 20 MB leaves room only if the programs are collected). 5 mechanical
   once the pin is a path; headroom M7; L9. 6 mechanical.
3. Coverage: all twelve ranked leaks are placed (L1, L2, L3, L10, L11 fixed; L4 licensed; L12
   licensed, M5; L5, L6, L7 by section 3; L8, L9 by section 4). The deviations scout's "extend
   literal interning to C17's scope" is out of scope by section 7 and consistent with section 1.
   `rexx-api` handle tables fall under section 5. Nothing dropped silently.
4. Section 1 against the probes: held-value stability (C01-C16) is bullet 4 less the collection
   rows (M6); weak references are M4; subclass and enhanced identity holds today and is unwitnessed
   (M6, L3); `.true` is L4. The rule licenses no wrong answer about one held value.
5. Package-as-object: M2, M3, M7 and the `rexx-core` dependency.
6. Wording: L11.
