# Task 9 review -- sends, argument positions and the other resumable entries

Reviewer: p6-t9-review. Range `4b0128186..8d65d9406`. Built from `git archive 8d65d9406` into
`$R/tree` (touched), `CARGO_TARGET_DIR=$R/target`, one `Compiling rexx-exec` line;
`R` = `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/p6-t9-review`.
`$R/cmp.sh FILE` runs the oracle (wrapped as CLAUDE.md says) and `rexx-run` from one fresh
`mktemp -d` directory in turn and compares the three descriptors.

## Witnesses (check 4)

```
send_in_argument stdout=same stderr=same rc=0/0
send_callee_allocates stdout=same stderr=same rc=0/0
send_resumable_entries stdout=same stderr=same rc=0/0
send_new_init_fails stdout=same stderr=same rc=214/214
send_guarded stdout=same stderr=same rc=0/0
```

Oracle stdout read: every path the report names prints (`note init first-object`,
`HELLO THERE 1 0`, `STARTED MESSAGE 1`, `trapped SYNTAX 1`, the three routine lines, `replied first`
then `continuation tagged`; `send_guarded` `1 0` / `1`; the `TRACE I` block carries `>M>`, `>A>`,
`>F>`). `send_callee_allocates` holds 17-, 12-, 17-, 10- and 10-byte values across the allocating
method, and phase-8.txt is in `collect_stress.rs`'s subset, so it runs under collect-on-every-allocation.

## Findings

### Critical

**C1. Pins lost at sites that still recurse: an `Op::Send` now reaches a
synchronous Rexx body with no pinned frame for five shapes that were pinned on `4b0128186`.**
Measured with the `pinning` feature on both commits, one scratch-only probe test
(`zz_reviewer_probe_pins`, appended to `concurrency_tests.rs`' `measured` module in each archive
copy; `$R/pin3.log`, `$R/pinbase.log`), parking on `call SysSleep 0` inside the Rexx body:

| program (plus classes) | `4b0128186` | `8d65d9406` |
|---|---|---|
| `x = .kd~new~m` (`::method m delegate d`, `d` a `.c` whose `m` sleeps) | `[[TreeEval]]` | `[[]]` |
| `x = .dd~new` (`::class dd subclass directory`, INIT sleeps) | `[[TreeEval]]` | `[[]]` |
| `x = .aa~new` (subclass of `Array`) | `[[TreeEval]]` | `[[]]` |
| `x = .tt~new` (subclass of `Table`) | `[[TreeEval]]` | `[[]]` |
| `x = .kk~new('Z')` (subclass of `Class`) | `[[TreeEval]]` | `[[]]` |

Each of these still runs the Rexx body on the Rust stack: `send_to_delegate`
(`dispatch.rs:2424`, `send_message` -> `complete_send` -> `run_activation`),
`native_directory_new` -> `Interp::complete_native` (`dispatch/construct.rs:75`,
`dispatch.rs:2303`), and the synchronous `send_message(.., INIT, ..)` in
`native_array_new`/`hash::native_hash_new`/`native_new_class` (`dispatch/hash.rs:1480`,
`dispatch/class_protocol.rs:572`). On the base they sat inside the
tree evaluation's `TreeEval` pin; Task 9 compiled the enclosing expression to `Op::Send`, and
nothing below it pins. Two mechanisms: (a) `send_to_delegate` has never pinned for itself;
(b) `PinKind::native` (`pinning.rs:111`-`126`) exempts every native **named** `NEW`, `SEND`,
`START`, `CALL`, ... -- which was meant for the resumable entries, but `Directory`/`Array`/`Table`/
`Class`/... `NEW` rows are `NativeBody::Run` natives that still recurse, so the name list now
states something false about them.
Failure scenario: Phase 6's scheduler reads "pinned iff a Rust frame sits between scheduler and
driver"; a park inside `INIT` of a `Directory` subclass created as `x = .dd~new` reports no pin
while `native_directory_new`'s, `complete_native`'s and a nested `drive`'s frames are live on the
Rust stack. `a_park_inside_a_stackless_entry_is_under_no_pinned_frame` cannot see it: its only
`~new` shape is a plain `::class` (`Object~new`, the one that did become resumable).
Fix: (1) delete `NEW`, `SEND`, `SENDWITH`, `START`, `STARTWITH`, `CALL`, `CALLWITH` from
`RESUMABLE_OR_PARKING` -- the `Begin` rows no longer pass through `pinned!` at all
(`dispatch.rs:2109`-`2114` wraps only `NativeBody::Run`), so the list now only exempts `Run` rows
that recurse; (2) pin `send_to_delegate`'s send (a `Delegate` kind, or `Forward`); (3) add the
five shapes above to the pinning test as pinned (non-empty frames) so the lie reddens.
Other Op::Send-reached recursions were checked and keep a pin: `UNKNOWN` (`[[Unknown]]` for
`o~zzz`, `o~send('ZZZ')`, `Message~send`, `~start`, `x~m = 5` with no `M=`), `FORWARD`
(`[[OpExec, Forward]]`). The security manager's `check_protected_method` runs a manager
method synchronously from `seam::clear` with no pin of its own either; not probed (it needs a
manager setup), same class as (2).

### Important

**I1. The compiled and the tree form of one builtin call now answer differently (check 6).**
`$R/probes/p1_minf.rex`, `f: return 5`, `o = .c~new`:

```
say min(3, f())                   oracle 3    8d65d9406 3
say min(3, f()) (o~~m == o)       oracle 3 1  8d65d9406 5 1
```

The second line's slot contains a cascade, so it falls to `Op::EvalExpr` and
`invoke_builtin_call`, which still loses `3`. Before Task 9 both forms printed 5 (one defect);
now whether a program gets the right answer depends on whether some *other* term in the same
expression compiles -- an engine-internal divergence. The report and the queue note say the
tree form "was not probed"; it reproduces with one line. Fix: fix the tree path (the queued
item's `enter_clause` clearing the shared argument stack under `invoke_builtin_call`), or at
minimum record this reproducer in `.superpowers/sdd/queued/2026-09-30-min-with-function-arg.md`
and delete "was not probed for this".

### Minor

**M1. `Activity::native_tails`' rooting has no witness.** Mutant: `activity.rs`'s
`for tail in native_tails { tail.object_roots(out); }` replaced by `let _ = native_tails;`,
rebuilt (one `Compiling rexx-exec` line), `cargo test --release -p rexx-exec --test
collect_stress`: `32 passed; 0 failed` (`$R/mut1.log`). Every object a tail holds is rooted by
something else in each witnessed shape: `Then::Started`'s message by `native_instance`'s
`push_temp` (`environment.rs:1162`), `Then::Answer`'s object as INIT's `SELF`, `Then::Held`'s
message and the blame's receiver/args by the sender's registers and argument stack. Not a defect
(the rooting is right to have), but "rooted for GC" is asserted, not witnessed. Fix: a
collect-stress case where the tail is the only root, or a sentence in `NativeTail`'s doc saying
the roots are defensive -- not both.

**M2. Disclosed scope gap, now a pinning fact too.** The report's concern that `Message~new`,
`String~new`, `Stem~new`, the collections' and `Class~new` still send INIT recursively is true
(`dispatch/string.rs:1979`, `dispatch/hash.rs:1480`, `class_protocol.rs:572`, ...). Their
recursion is legitimate only if pinned; C1 covers that.

## Checks that held

1. **One begin/finish.** `CallTail` is built in two places (`run/call.rs:976`, `CallTail::method`)
   and popped only by `finish_call`; methods, labels, routines and the REPLY resume all finish
   there. `invoke` = `begin_invoke` + `complete_send`, `send_message` = `begin_send` +
   `complete_send`; `~new`, `Object~send/sendWith/start/startWith`, `Message~send/sendWith`,
   `Routine~call/callWith/[]` go through `begin_send`/`begin_call` and `NativeStarted`, with
   `complete_native` for the two non-send callers. No copy of invoke's logic found. The seam fix
   `54c4c28e5` is a real single seam: `method_is_protected` has one caller, inside
   `seam::clear`; `Clearance::Checked` carries the fact that the manager ran, which is what
   invalidates the cached behaviour.
2. **native_tails with the activity.** A field of `Activity`, so it moves with the activity and
   holds no `RegFrame`; `finish_send` reads `call_tails.len()` before `finish_call` pops and
   drains all tails at that depth innermost first (read; the nested `x = .c~new~send('SEND', 'M')`
   answered and parked with no pin, which exercises two tails at one depth). Rooting: M1.
3. **Pinning.** The shapes the report lists park unpinned and really are stackless (my probe
   reproduces `[[]]` for `x = .c~new(1)`, `.c~new~send('SEND','M')`, `Routine~call` of a
   `.routine~new` body, sends in arithmetic and arguments). `UNKNOWN` and `FORWARD` keep their
   pins. The `TreeEval` pins in `begin_invoke_call`/`invoke_builtin_call`/
   `arguments_before_failure` stay. Lost pins: C1.
4. **Witnesses.** Above; also probed against the oracle, all three descriptors identical:
   message-assignment evaluation order (`o~v(t('arg')) = t('val')`, value first), the cascade in
   the assignment form, 91.999 for a value send answering nothing (in an argument and in an
   assignment, rc 165), `UNKNOWN` through `Op::Send` and `Object~send`, `EXIT` inside a method
   reached by send/`Message~send`/`Object~send`/`~start`, `EXIT` inside a routine reached by
   `Routine~call`, `TRACE I` over clause/assign/cascade-assign/nested sends/lists/DO header
   sends/IF, NOVALUE-trapped `.foo`, ISGUARDED through `Op::Send`, `Object~send` and
   `Message~send` (`q4_guarded_wait.rex`). `q5_deep_send.rex` (20,000-deep recursion by send)
   agrees on rc 245 and the error lines; the traceback length differs, which is the licensed
   depth-cap divergence.
5. **alloc / heapshape.** Same stdout on `prev` (`4b0128186`, sha `9cc21fe6...`) and
   `8d65d9406`: `21000000`, `gc_forced= 1`. Callgrind on reduced copies (`n = 20000`, 30 outer
   rows): `Interp::alloc_with` 20,258 vs 20,235 for alloc; 60,354 vs 30,301 for heapshape, the
   difference being `Interp::literal`'s 30,053 calls -- the tree path re-built the `"element-"`
   literal per evaluation where the compiled stream loads an interned constant. Same work, named
   reason.
6. **min(3, f()).** I1.
7. **Structural tests.** `golden_tests.rs`' "outside the native set" rows moved from `.nil`
   (now native) to a cascade, which is still outside; the `call zsub length('x')` row became a
   positive golden and `>zp` still pins `Op::Call`. `corpus_shape_tests.rs`' route widths match
   `native_shape`'s (`k + 1` steps for list child `k`, one per operator). `concurrency_tests.rs`'
   `FRAME_PROBES` moved to cascades, which still evaluate through the tree. Not loosened.
8. **Op.** `const _: () = assert!(size_of::<Op>() == 16)` (`ir.rs:61`); `Clause` is the first
   variant; `Send`, `List` are last; no `Op::Generic`; `unsafe` only in `ffi.rs`, `load.rs`,
   `bytes.rs`, `frame.rs`.

Tests on the review build: `--lib` 850 passed, 15 failed, every failure
`the worktree's build/lib is three directories above this crate` (an archive has no `build/`);
`collect_stress` 32 passed; `corpus` (`REXX_CORPUS_GATE=1`) 29 passed 1 ignored; `dispatch_seam`
6 passed. `pinning` `measured::` 6 passed, `pinned_parks_over_the_derived_list` fails for the
absent `ootest/`.

## Verdicts

- **Spec compliance: not yet.** Steps 1-4 are met except that Step 3's stackless entries were
  delivered by removing the enclosing pin without re-pinning what still recurses below them (C1);
  the brief's pinning rule is the task's own invariant.
- **Task quality: approve after C1 and I1.** The begin/finish split is one mechanism, the seam
  fix is genuine, witnesses are oracle-identical and print what they claim.

## Addendum (team-lead's extra check): every `~new` INIT path, pinned or stackless

Scratch-only probes in `concurrency_tests.rs`' `measured` module, `--features pinning`, release,
at `8d65d9406` (`$R/pin4.log`, `$R/pin5.log`, `$R/pin6.log`). Two instruments per class `B`:
(a) `x = .k~new` with `::class k subclass B` whose INIT does `call SysSleep 0`, reporting the
pinned frames at the park; (b) INIT recursion `N` deep (a `.local` counter, `y = .k~new` inside
INIT), reporting `Outcome::stack.bytes` at `N = 50` and `N = 500`. Stackless means (b) is flat.

| base class | park frames | stack 50 / 500 | verdict |
|---|---|---|---|
| Object | `[[]]` | 0 / 0 (1,344 / 1,344 with args) | stackless, truthful |
| List, Queue | `[[]]` | 1,344 / 1,344 | stackless, truthful |
| Supplier, EventSemaphore, MutexSemaphore | `[[]]` | (same `native_new` row) | stackless by the row, not measured in (b) |
| Array | `[[]]` | 316,256 / 3,174,656 | **recursive, unpinned** |
| Directory | `[[]]` | 340,000 / 3,400,000 | **recursive, unpinned** |
| Table, Set, Bag, StringTable | `[[]]` | 312,000 / 3,120,000 each | **recursive, unpinned** |
| IdentityTable, Relation | `[[]]` | (same `native_hash_new` row) | **recursive, unpinned** by the row |
| Class | `[[]]` | 348,256 / 3,494,656 | **recursive, unpinned** |
| MutableBuffer | `[[]]` | 320,256 / 3,214,656 | **recursive, unpinned** |
| WeakReference | `[[]]` | 342,656 / 3,438,656 | **recursive, unpinned** |
| CircularQueue | `[[]]` | not measured | its `NEW` is Rexx in the `REXX` package, so the send is `Op::Send`-shaped |
| Stem, String, Message | none | -- | a subclass's `NEW` is refused loudly (`not implemented (Phase 9)`), so no INIT runs |

The `OF` factories that send INIT (`Array~of`, `Bag~of`, `Set~of`, `Queue~of`, `List~of`) are
pinned, `[[Native("OF")]]`: truthful. `Table~of`/`Directory~of` are `MapCollection~OF` in Rexx.

So: (1) **every still-recursive `~new` is unpinned**, because `PinKind::native`'s
`RESUMABLE_OR_PARKING` exempts the name `NEW` for all classes (C1 mechanism (b)); this widens C1
from the five shapes above to every row in bold. (2) **The brief's "`~new` into INIT" is a gap**:
it holds for the `native_new` row only (Object and the five classes sharing it); Array, Directory,
the hash collections, Class, MutableBuffer and WeakReference still enter INIT recursively, about
6.2-7 KB of native stack per level. The report discloses Message/String/Stem/collections/Class; it
omits MutableBuffer and WeakReference and does not say the recursive ones are unpinned. Fix
either make those `NEW`s begin halves too, or (minimum, for truthfulness) drop `NEW` from the
exemption list so they pin as `Native("NEW")`, add rows for Array, Directory, Table, Class,
MutableBuffer and WeakReference to the pinning test expecting a non-empty frame, and record the
remaining gap against the brief in the report.
