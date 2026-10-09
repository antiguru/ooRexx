### Task 11a: Deferred items, small

**Evidence:** `.superpowers/sdd/2026-10-07-phase-6-1/scout-e1-report.md` (items 2, 3, 4) and
`scout-e2-report.md` (items 7, 8, 9, P1, P2, P3), each with its probe, both engines' output and the
crate site at `f7170918c`. Ruling R11. Scratch patches for P1 and P2/P3:
`/tmp/claude-1000/p61/se2/copies-doubling.diff`, `collections-scratch.diff` (measurement builds, not
reviewed).

**Rule:** each item answers as the oracle answers, witnessed against the oracle (5 runs for any
concurrent probe); each fix lands with a crate or corpus test that fails at the task's base. A user
STRING answering a non-Array, non-string object is a loud refusal with a deviation row (R11).

**Files:** `ir/drive.rs` (`Op::LoopNext`), `run/loops.rs` (`flat_loop_step`, `run_repeating`),
`run.rs` (`clock_stale`); `dispatch/object_protocol.rs` (`dispatch_held_message`), `scheduler.rs`
(`halt_message`), `lib.rs` (the sender record, rooted or pruned in `collect_now`);
`dispatch.rs` (`release_method_activation`); `dispatch/reqstr.rs`; `dispatch/hash.rs`
(`not_this_task`), `dispatch/buffer.rs`, `dispatch/object_protocol.rs` (Message SEND, START,
REPLY wrong-type rows); `dispatch/construct.rs`, `tests/assertions.rs`; `builtin/string.rs`
(`copies_bytes`); `dispatch/collection.rs` (`last_item`, `slots_of`),
`dispatch/collection/list.rs`, `dispatch/array/surface.rs`, `dispatch/array/sort.rs`,
`dispatch/collection/queue.rs`; `phase-6-1-gate.md`.

- [ ] **Step 1: Empty-body `TIME('E')` loop.** A loop with no body clause (plain WHILE, controlled
      WHILE, UNTIL, and inside INTERPRET) re-reads the clock each pass as a body clause would; the
      scout's probes end. Measure emptyloop and rexxcps (Task 2's callgrind command, +0.5% budget):
      this is a store on the hottest loop path.
- [ ] **Step 2: `Message~halt` after `~send`.** The sending activity is recorded as the oracle's
      `startActivity` (`MessageClass.cpp:432`); `m~halt` answers `1` and halts it with 4.1. The
      record is rooted or pruned in `collect_now`. Witness under default mode and `every`.
- [ ] **Step 3: `EXIT` in a CALL ON handler at the REPLY clause's end** raises 91.999 to the sender
      (oracle rc 165).
- [ ] **Step 4: User STRING answering an Array** is joined as the oracle joins it (the existing
      array-joining path), so `say`, `length()`, concatenation and R9's truth judgment see the joined
      text; another non-string object answer is a loud refusal and a deviation row. Add an Array
      case to `tests/truth/values`.
- [ ] **Step 5: Wrong-type native rows.** `ITEMS` of the hash collections, MutableBuffer `LENGTH`,
      Message `SEND`, `START` and `REPLY` on a receiver of the wrong type refuse through
      `Loud::receiver_class` (owner none, D3), not a Phase 9 refusal. Re-derive the refusal tables.
- [ ] **Step 6: `Literals` rows.** `.routine~new(name, code, package)` inside a test-case class
      answers as the oracle (`AB A`); the `Literals` assertion rows gain a test case for `self` in
      `tests/assertions.rs`, and their `EXEMPT` rows go.
- [ ] **Step 7: Collections without copying every slot.** `Array~append`, `items`, `last` and
      `List~append` read the length and last element without cloning the slots (keep R10's
      `charge_growth`); SYNTAX unwinding through deep frames becomes linear. Scaling table (n = 1e3,
      1e4, 1e5 appends; unwind depth 2000, 4000, 8000) before and after, both engines.
- [ ] **Step 8: COPIES** fills by doubling. Callgrind with libc included (the scout shows the
      libc-excluded column hides this) on a COPIES program, plus the eight perf programs.
- [ ] **Step 9:** The per-task check; the seeded gate in release; perf against the task's base and
      base61 on the eight programs, recorded under `## Task 11a`. Each queued item fixed gets a
      first line `RESOLVED by <commit>: ...` as earlier resolved items have. Commit.

