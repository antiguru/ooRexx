# Task 8.5 re-review, fix round 4

Reviewed `15181b196..8249ccef1` (code `6a8290bb6`). HEAD was built from the checkout (only `progress.md`
modified) with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t85-rr4-target memcap 8G cargo build
--release -j 4 --bin rexx`, exit 0. Its `.text` section is byte-identical to the measured
`cg-9d/t8.5/rexx-head-6a8290bb6` (`objcopy -O binary --only-section=.text`, md5 `4b4f0777...` both). Previous
answers come from the pinned `rexx-head-4a6321376` (whose `.text` round 3's re-review matched to a build) and
base from `rexx-base-6e0bf5e5b`. Every crate run is `memcap 2G timeout -k 5 20` in a fresh `mktemp -d` under
`t85-rereview4-scratch/`; stdout, stderr and rc compared separately. The oracle ran only acyclic programs
(`acy/a1.rex` to `a4.rex`, `subchain.rex ... leaf`, `subchain2.rex ... leaf`, the round's `ac5.rex` and
`ac7.rex`). Probe paths are relative to `t85-rereview4-scratch/p/`.

## Verdict

**CHANGES REQUESTED.** Critical none, Important 1, Minor 2. New oracle defects none.

## Verdicts on the fix brief

1. **A cycle through a stem's default is 11.1: PARTIAL.** Every cycle terminates loudly, HALT interrupts,
   memory is flat; but the fix made an acyclic path quadratic (Important 1), and it raises 11.1 on a
   forwarded message whose reference is not cyclic (Minor 1).
   * Cyclic shapes, all on `s. = >s.` under SIGNAL ON SYNTAX unless noted (`cyc/c01.rex` to `c60.rex`,
     results `res-pin/`). `SYNTAX 11.1` for `say s.x`, `length(s.x)`, `procedure expose s.` then `say s.`,
     `use arg q` then `say q`, `say s.zz` under SIGNAL ON NOVALUE (11.1, no NOVALUE), `s.~foo` via
     `unknown`, `s.[1]`, `s. == s.`, `s. = s.`, `s.~equals(s.)`, `q. = s.; say q.`, `interpret 'say s.'`,
     `value('s.')`, `value('s.x')`, `parse var s. a`, `datatype(s.)`, `abbrev(s., 'x')`, `left(s., 3)`,
     `address command s.`, `.message~new(s., 'LENGTH')~send`, `s.~send('LENGTH')`, `s.~copy` then `say`,
     `s.~makestring`, `(s.)~length`, `s.~length`; for the reference-free `t[] = t` (Stem subclass):
     `t[1]`, `t~request('STRING')`, `t~makestring`. Ending normally: `drop s.` (`dropped Stem`), `drop s.x`,
     `.context~variables~items` (`1`), `s.~allIndexes~items s.~items` (`0 0`), `s.~makeArray`,
     `s.~supplier`, `s.~hasIndex`, `s.~request('ARRAY')`, `s.~toDirectory`, `do i over s.`, `s.~string`
     (`S.`), `.array~of(s.)~toString` (`S.`), `s.~hashcode`, `s.~objectname`, `t~items`, `t~allIndexes`,
     `t~string` (empty, the stem's name). `trace i; x = s.` traces `>V> S. => "S."`, `>>> "S."`;
     `trace i; x = s.y` traces `>C> S.Y => "S.Y"`, `>V> S.Y => "S."`; `trace i; x = t` traces `""`. No run
     overflowed the stack or reached the timeout.
   * `call sub >s.` with `use arg >q` is `SYNTAX 88.930` (`cyc/c10.rex`), at the argument binding, not a
     cycle path.
   * Long cyclic chains end: `subchain.rex N cyc` (N Stem subclass instances, each the previous one's
     default, the last pointing back at the first) is `SYNTAX 11.1` at 1000 and 20000 links in 0.05 s;
     `mixchain.rex N cyc` (instances and references to stems alternating, closed into a cycle) is
     `SYNTAX 11.1` at 10 and 1000.
   * HALT: `halt.rex` (each iteration traps 11.1 from `say s.`, `length(t)` over `t[] = t`, and `s.~foo`)
     under `memcap 2G timeout -s TERM -k 5 3`: `HALT HALT after 88343`, rc 124.
   * Memory: `halt.rex 20000` peaks at 36444 KB (0.72 s), `halt.rex 200000` at 36008 KB (6.79 s), rc 0.
   * `cargo test --release -p rexx-exec --lib -- reference cyclic`: `17 passed; 0 failed`, including
     `a_cyclic_reference_is_insufficient_stack_in_every_conversion`.
   * `oracle-crashes.txt` entry 61 is in the file's format and holds the re-review's `sd1.rex` verbatim.
2. **Minor 1, the doc blocks: ADDRESSED.** `value.rs:1033-1035`: "What a value's text has to be read from
   somewhere other than the object holding it." sits above `#[derive(Copy, Clone)] enum Redirect`;
   `Referent` (`value.rs:1009-1010`) and the new `Link` (`value.rs:1024`) carry their own.

**Concern 1, the TRACE line: confirmed, but it is not the only acyclic answer that moved** (Minor 2).
`ac5.rex` line 22 (`q = >t.`, `t. = .array~of('one', 'two')`, `trace r; x = q`): head `>>> "an Array"`,
oracle `>>> "an Array"`, 4a6321376 and base `"one` / `two"`. rexxref (`instrc.xml:3697-3699`): traced
results other than strings "appear in the string representation obtained by sending them a STRING message",
and the oracle's `q~string` is `an Array` (`acy/a4.rex` line 3), so the new line follows both rexxref and
the oracle. Comparing my acyclic set (`acy/a1.rex` to `a3.rex`, the round's `ac1.rex` to `ac10.rex` and
`n.rex`) against 4a6321376, the only other difference is the same shape (`acy/a2.rex` line 22). `acy/a4.rex`
found the stdout moves listed under Minor 2.

**Concern 2: both findings confirmed.**
* The bare stem with an array default: `ac5.rex` line 21 traces `"one` / `two"` on head and base and
  `"an Array"` on the oracle; `say t.`, `'p' t.`, `length(t.)` are the items joined here (`length` 7) and
  `an Array` (8) on the oracle. Also on a Stem subclass instance with an array default (`acy/a1.rex` line
  C, `acy/a4.rex` line 4 and line 13). **A queue candidate**, widened below: `t.~string` and `u~string`
  answer the items joined here and `an Array` on the oracle, which is what rexxref's STRING rule gives the
  TRACE line too. Since this round a reference to the stem answers `an Array` while the stem itself answers
  the items joined (`acy/a4.rex` lines 2 and 3), so the crate is now inconsistent with itself on one value.
* NOSTRING through a stem (`ac7.rex`): head names the Stem for 3, 4, 7 and 9; the oracle names the
  VariableReference for 3, 4 and 7 and the Directory for 9; base raised no NOSTRING for 3, 4 and 7 and
  41.1 for 9. Head follows the ruled rule (NOSTRING names the referenced value; for 2, 6 and 8 the oracle
  itself names the Stem, and 1 names the Directory against the oracle's VariableReference). **Not a queue
  candidate**: it belongs to the NOSTRING Deviation candidate already going to Moritz, whose text should
  name the through-stem shapes.

**Perf: confirmed.** `r4final/` holds 99 head and 99 base `.vg` files; every head `Command:` line names
`cg-9d/t8.5/rexx-head-6a8290bb6` and every base line `rexx-base-6e0bf5e5b`, each program three times, over
the 29 programs in `bench-programs/`, `rexxcps` and the three ping programs. That binary's `.text` matches my
HEAD build. In `r4/callgrind.txt`, libc subtracted, the largest rises are fibfunc `+0.0213`, fibcall
`+0.0209` and define `+0.0004`; every program is under +0.05%, and the spread columns are at most `0.0023`.
None of those programs builds a chain of Stem subclass instances, so the benchmark cannot see Important 1.

## Issues

### Critical

None.

### Important

1. **An acyclic chain of Stem subclass instances became quadratic.** `instance_conversion`
   (`dispatch/reqstr.rs:464-472`) now calls `chain_end(store)`, which walks the whole rest of the chain, and
   then `classify_string_conversion(store)`, whose stem arm takes the default, the next instance, back into
   `instance_conversion`: one full walk per link. `stem_subclass_text`, `stem_subclass_text_len` and
   `stem_subclass_try_text` (`value.rs:1266-1295`) do the same for the renderings. The report's "a chain
   mixing stems and references costs one walk per conversion" holds only where a reference sits between the
   instances (`reference_conversion` jumps to the end); `mixchain2.rex` 10000 is 0.0020 s here and 0.0008 s
   on 4a6321376.
   * `subchain2.rex N leaf` (N instances, each the previous one's default, the last's default `'leaf'`),
     `length(first)` timed alone:

     | N | head | 4a6321376 |
     |---|---|---|
     | 5000 | 0.163 s | 0.00009 s |
     | 10000 | 0.676 s | 0.00015 s |
     | 20000 | 2.56 s | 0.00028 s |
     | 40000 | 11.54 s | 0.00060 s |

   * `subchain.rex N leaf` (`length(first)` and `first~string`): 20000 links take 8.02 s here and 0.08 s on
     4a6321376 and base; 100000 links are rc 137 here (killed at 25 s, nothing printed) and `len 4`, `leaf`,
     `end` in 0.12 s on 4a6321376 and 0.13 s on base.
   * The oracle prints `len 4`, `leaf`, `end` at 1000 links and `SYNTAX 11.1` at 20000 (its own recursion
     limit), so long chains are rare; the brief still says to keep the acyclic paths at their current cost.
   * Fix: answer from where the chain ends, as `reference_conversion` does: `instance_conversion` classifies
     `chain_end(store)`'s value directly, and the three `stem_subclass_*` renderings render it.

### Minor

1. **A message forwarded through a reference that is not cyclic raises 11.1 when the stem it reaches has a
   cyclic default.** `chain_from` answers `(Referent::Cycle, Referent::Cycle)` on meeting a cycle
   (`value.rs:1203`) even after it has recorded where the references end, so `referent` and
   `referenced_receiver` (`dispatch.rs:4346`) see a cycle past the stem. `cyc2/d01.rex`
   (`s. = >s.; q = >s.; say q~items`): head `SYNTAX 11.1`, 4a6321376 `0`; `s.~items` itself is `0`
   (`cyc/c06.rex`). Head raises the same 11.1 for `q~allIndexes~items` (`d03`), `q~put(1, 'a')` (`d04`) and `q~hasIndex(1)`
   (`d05`). The reference `q` ends at the stem in one hop, and the stem answers these messages without its
   default, so this is not a position where `r = >r` raises. Fix: forwarding takes the reference end;
   whatever needs the cycle (the conversions, and the Stem subclass `t[] = t` case through
   `conversion_subject`) asks `chain_end`.
2. **Acyclic stdout answers moved that the report does not name.** `acy/a4.rex` (`zz = .array~of('one',
   'two')`, `t. = zz`, `q = >t.`, `u` a Stem subclass instance with `u[] = zz`, `r = >u`): `say q~string`
   and `say r~string` print `an Array` at head and `one` / `two` at 4a6321376 and base; `trace r; x = r`
   moved the same way. The report says only the TRACE line of `q` moved. The new answers match the oracle
   and rexxref's STRING rule, so the move is not wrong, but the report's acyclic claim ("identical ... on
   every run but one TRACE line") is false, and the stem's own `t.~string` and `u~string` still answer the
   items joined (queue candidate below). Fix: name these moves in the report.

## Out of scope

* `c10.rex`: `call sub >s.` with `use arg >q` is 88.930 regardless of the cycle; not examined further.
* Queue candidate 1 of earlier rounds (a plain stem whose default has no string value in an arithmetic
  operand) stays queued.

## Queue candidates

1. **A stem's string value with an array default** (`string_value_text` for a Stem and a Stem subclass
   instance). `t.~string`, `u~string`, `trace r; x = t.` and `trace r; x = u` answer the items joined here
   (`acy/a4.rex` lines 2, 4, 13; `ac5.rex` line 21) and `an Array` on the oracle, which rexxref's TRACE rule
   (`instrc.xml:3697-3699`) agrees with. A reference to the same stem answers `an Array` since this round.
   Base answers as head, so the stem side predates the task.

## New oracle defects

None. The oracle ran no cyclic program. Its `SYNTAX 11.1` on the 20000-link acyclic chain is its recursion
limit raising the documented error, not a hang or crash.
