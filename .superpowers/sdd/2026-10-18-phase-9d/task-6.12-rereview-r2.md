# Task 6.12 re-review: fix round 2

**Scope.** This review covers `729cc5fe0..19692e83a`. The code commit is `45bd0c3ba`.

**Build.** I built HEAD into a fresh target dir in the scratchpad (`rev612r2-target`,
`cargo build --release --bin rexx`, exit 0, sha256 `616c83fa...`). The round-1 binary for comparison
is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.12-r1/rexx-head`, the report's `parent`
(`c5a73f61...`).

**Method.** Probes ran as in the earlier reviews:
* the oracle under the standard wrapper;
* the crate under `memcap 2G timeout 20`;
* each run from a fresh directory;
* stdout, stderr and rc compared separately.

The probe sources were in `scratchpad/rev612-probes.lQEo/`, which is not durable, so the lines that
decide each point are quoted below.

## Earlier findings

I reran every probe from round 0 and round 1 against HEAD (32 programs). They match the oracle on all
three streams, with three known exceptions:
* `t_nil` and `t_object`: the oracle segfaults at exit (entry 54).
* `w2`: the queued `defineMethods` supplier gap.
* `u2`: UNINIT counts, which depend on GC timing.

| finding | status | deciding probe |
|---|---|---|
| r1 Important 1: existing instances | closed | `cache2`: `.b0~put(2)`, `.b1~put(2)` and `.b2~put(2)` each give `events: METHOD PUT`. `cache3`: `existing bag put after setPrivate: 97.2`, and `.pb~items events: METHOD ITEMS`. All three are identical to the oracle. |
| r1 Important 2: shared-state window | closed | `lift_protection`/`restore_protection` are gone, and no method's flags change during a run. The window shapes `w1`, `g2`, `rec` and `u2` (its `.local~z` stays 0) still match round 1. The new mark is part (b) below. |
| r1 Minor 3: entry 55 | closed | The heading now names the measured shape: untrapped, `.object~new`. The trapped `3.1 an Object` case is recorded. I reproduced SIGSEGV 4 of 4 on the entry's program in round 1. |

## (a) Dispatch tables updated in place

`repoint_instance_method` repoints only the entries whose scope is the asked class, in its dictionary
and in every behaviour, retired ones included.

* **The other six classes stay unaffected.** Probe `a1` builds one instance each of Directory, Table,
  IdentityTable, Relation, Set, StringTable, Bag and a `MyBag` subclass. All are built before
  `.Bag~method('PUT')~setProtected` and `.Bag~method('REMOVE')~setPrivate`. Results, identical to the
  oracle:
  * Managed `put`: Directory through StringTable each give `put events:` with nothing after it. Bag and
    MYBAG give `METHOD PUT`.
  * `remove`: the six give `remove ok`. Bag and MyBag give `97.2`.
  * Round 1 gave `Bag put events:` empty and `Bag remove ok`.
* **Enhanced objects and object-level methods.** A Bag enhanced before the setters asks the manager
  (`.enh~put('X') events: METHOD PUT`) and refuses `remove` (`97.2`). A `MyBag` that has its own
  `setMethod` entry asks for `put`. Its own `OWNM` entry sends nothing. All of this matches the oracle.
* **Undoing a setter.** Neither `setPublic` nor `setUnprotected` exists on either side: both give 97.1.
  `Setup.cpp` defines neither, so no undo path needs checking.
* **An object in use on another thread.** In probe `mt2`, a managed activity (started by `reply`)
  loops 400 times. Each pass reads `.Bag~method('PUT')~isProtected`, sends `put` to a Bag built before
  the setter, and counts the manager's events. Meanwhile the main program sends `setProtected` after
  10 ms. A pass is "bad" when it read 1 and asked no manager.

  | binary | result (`bad asked protectedIters`) |
  |---|---|
  | oracle (2 runs) | `0 390 390` |
  | HEAD (2 runs) | `0 390 390` |
  | round 1 | `390 0 390` |

  HEAD also gives `0 390 390` under `sim:1`, `sim:3,uniform:0.2`, `sim:11,uniform:0.2`,
  `sim:21,uniform:0.2`, `sim:5,pct:3,k=40`, `sim:13,pct:3,k=40`, `sim:29,pct:3,k=40` and
  `sim:31,pct:3,k=40`, rc 0 each time. A native `put` cannot be interrupted part-way, so the repoint
  happens between two sends, and every send that follows it asks.

## (b) The one-shot "not a send" mark

* **Where the mark lives.** `Interp::unasked_run` is a single field for the whole interpreter. It is
  shared by every activity, so it is not per call.
* **Why another send cannot consume it.** `invoke_armed_executable` sets it immediately before
  `begin_invoke`. The first thing `begin_invoke` does is call `invocable()`, a lookup in five tables
  that sends nothing. It then enters `seam::clear`, which consumes the mark with `std::mem::take`
  straight after the `security_armed` test, and does so whether or not the method is protected. No
  other send and no activity switch can come between the set and the take. Whatever the outcome, the
  next statement after `begin_invoke` writes `false`, with no `?` in between. So the mark is never
  carried over to a later send, even on a path that skips the seam (the `Loud::native_method` error
  from `invocable`).
* **Probe `mark`.** A managed routine performs one marked call, then sends `.Array~superclass` and a
  protected Rexx `k~prot`. All five cases are identical to the oracle, and each one's follow-up sends
  ask the manager (`events: METHOD SUPERCLASS | METHOD PROT`):

  | marked call | outcome |
  |---|---|
  | `Object~run` of an ABSTRACT protected method | 93.965 before any body |
  | a primitive run on a class (`.Class~method('SUPERCLASS')`) | ran |
  | a native run with too many arguments | 93.902 at the arity check |
  | a Directory entry holding the ABSTRACT method | 93.965 |
  | a protected Rexx method run with `Object~run` | ran |

* **The other two exits.** A protected method run by `Object~run` that sends itself matches the oracle
  (`rec` and the new `directory_entry` lines). HALT and REPLY cannot land between the set and the
  reset: no Rexx clause runs there.

## Outside this round

These are not findings against this round.

* **A known panic.** `.Message~new(aRoutine, 'CALL')~start` panics with `a live activation`, rc 101,
  with or without a manager, on HEAD, round 1 and `b41fc9257`. The oracle prints `result: ran`. This
  is the queued `2026-10-14-routine-start-call-panic.md`, which my first `mt` probe ran into.
* **A loud refusal.** `Object~run` of `.Class~method('SUPERCLASS')` on a receiver that is not a
  class stops with the loud refusal `a message send to a value that is not a class object is not
  implemented` (rc 120). This is not specific to the security manager.

## Issues

* **Critical:** none.
* **Important:** none.
* **Minor:** none.

## Checks

From `rust/`, with `CARGO_INCREMENTAL=0`, the fresh target dir, and `memcap 8G -j 4 --release`:

| command | result |
|---|---|
| `cargo test -p rexx-exec --lib` | exit 0, 1155 passed |
| `-p rexx-exec --test security_replacements --test dispatch_seam` | exit 0, 6 and 2 passed |
| `-p rexx-parse --test sourceline_oracle` | exit 0 |
| `REXX_CORPUS_GATE=1 -p rexx-exec --test corpus` | exit 0, `1133 of 1133 matching` |
| all twelve `security_manager_*` witnesses | identical to the oracle on stdout, stderr and rc |

**Perf.** I ran `callgrind.sh -r 1 -B 0.5 -p "sendloop dispatch rexxcps startup" pin=...
r1=<round 1> head=<this build>`, exit 0. With libc and ld-linux subtracted, head minus round 1 is:

| program | head minus round 1 |
|---|---|
| sendloop | -188 Ir of 13.79 G |
| dispatch | -188 Ir |
| rexxcps | -464 Ir |
| startup | +30 Ir |

Every delta rounds to 0.0000%, which reproduces the report.

## Assessment

**Task quality:** Ready.

**Reasoning:** Existing instances now see the marked copy. The six other classes sharing the donated
row, enhanced objects, subclass instances and an object being sent to from another thread all behave
as the oracle does. The "not a send" decision is now a mark that the same call's own pass through the
seam consumes, and it is reset unconditionally afterwards. No probe shape left it behind for a later
send.

**Verdict: APPROVED** (none open).
