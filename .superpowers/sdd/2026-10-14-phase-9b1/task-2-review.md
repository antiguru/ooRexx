# Phase 9b1 Task 2 review (8db45ca6d..7c756e734)

Reviewer binary: release `rexx-run` built from a `git archive 7c756e734` copy at `/tmp/t2rev/src`
(own `CARGO_TARGET_DIR=/tmp/t2rev/target`, `Compiling rexx-exec` seen). Probes ran from fresh
empty directories under `/tmp/t2rev/run-*`, oracle under `ulimit -v 1048576` and `timeout -k 5 20`,
crate under `memcap 2G` and `timeout 20`. The copy was deleted after the review.

### Spec Compliance

- ❌ Issues found: one L2 site is still converting first. `run/loops.rs:2679`
  (`controlled_step_wide`) calls `to_number` on the control variable before
  `operator_message_receiver`, so a loop step on a control variable that holds a String subclass
  instance computes instead of sending `+` (see I1). Everything else the brief asks for is present:
  L1 at `rexx-core/src/heap.rs:281-287`, L2 at `eval.rs:843` and `run/loops.rs:1062` via
  `value.rs:729`, L3 at `dispatch/reqstr.rs:199-202`, three witnesses in `corpus/phase-9.txt` with
  their `sourceline_oracle` files, the `OWNERS` entry removed.
- ⚠️ Cannot verify from diff: the queued file
  `.superpowers/sdd/queued/2026-10-13-class-side-string-method-not-sent.md` is left for the
  controller to retire (report says so). `value.rs` is outside the brief's file list; the change
  there is the receiver-mode conversion, which is where the fix has to live, and it is small.

### Strengths

- The receiver-mode conversion (`value.rs:721-742`) keeps `to_number` unchanged for every
  non-receiver caller and threads one flag through the stem and variable-reference redirects, so the
  stem default and `>`-reference shapes follow the referenced value. The fallback paths
  (`operator_message_receiver` answers `Some` for every `Body::Instance`, `eval.rs:1203`) mean no
  receiver can fall from "converted" to an error.
- Callgrind reproduced to the instruction on a reviewer-built binary (below). The first form's
  +0.996% was caught, recorded and replaced, not argued away.
- The report's mutation table reproduced (mutants A, B, E, F below).

### Checks run

1. **L1.** `Decoded` has four variants (`rexx-core/src/handle.rs:90-99`). The new arm keeps the
   `Heap` branch byte-for-byte, so a dead heap referent still clears and nothing new is marked: no
   leak. `NIL` stays `NIL`. A handle's representation is fixed when it is made (`value.rs:194`,
   `:203`); no site converts a held value between inline and heap (`grep -rn materiali
   crates/*/src` finds no string materialisation), so a held value cannot change representation
   under its WeakReference. Long string: heap, marked while held (witness line 3; stress program 3
   below). Under collect-every-alloc, a dropped `copies('z', 40)` and a dropped `.object~new` clear
   (rc 0), which is the licensed unspecified case.
2. **L2.** Probe `ops.rex`: a String subclass with a method for every operator. Oracle rc 0 and
   crate rc 0, stdout identical: every arithmetic operator, prefix `- + \`, every comparison
   operators, `& | &&`, `||`, blank and abuttal concatenation (including `s x`, `s||x`, `s''x`),
   right-operand subclass (`1 + s`), and a subclass with only `STRING` overridden. Probe `vref.rex`
   (stem default, `use arg >r`, DIGITS 3 on a non-overriding subclass) and `vref2.rex` (a
   VariableReference object as receiver): identical. Probe `errs.rex` (41.1 text from a
   non-numeric subclass on left, right, prefix, DO initial, DO TO, step): identical. The
   implementer's `operator_receivers.rex` and `mb.rex` (MutableBuffer): identical, rc 0 and 159.
   Probe `dos.rex` against the oracle's `DoBlockComponents.cpp:129-173` and `DoBlock.cpp:194-213`:
   setup `+` on initial, TO and BY, BY's `<` sent to BY's result, the control's `>` sent, FOR and
   a repetitive count not sent `+`: identical. **The step `+` on a subclass control differs (I1).**
3. **operand_number.** Plain text, `Body::Num`, small integer and inline arms are `to_number`'s.
   Only the `Body::Instance` text arm differs (`value.rs:832`). On the hot path: it is
   `arith_left_operand`'s conversion for every non-small-integer left operand. Callgrind,
   `bench-programs/callgrind.sh -B 0.5 -j 6 -p "arith fibcall sendloop" -o /tmp/t2rev/cg
   pin=/home/moritz/dev/repos/ooRexx-p9-pin/rexx-run head=<reviewer binary>`, exit 0:
   arith +0.1194% (-libc +0.1088%), fibcall -3.4488%, sendloop -0.1004%, spread at most 0.0001%.
   Equal to the report's figures. Against 9b close2's arith (-libc 11486773876) this task adds
   +0.033%.
4. **L3.** Probe `l3.rex`, oracle rc 0 and crate rc 0, identical: `say .k`, `length(.k)`,
   `'x' || .k`, `'x' .k`, `'x'.k`, `PARSE VALUE .k WITH a`, `PARSE VALUE .k 'z' WITH a b`,
   `PARSE VAR`, `pos`, `c2x`, `substr`, `reverse`, a CALL argument, an inherited class-side
   `STRING`, and `.k || 'x'`.
5. **GC rooting.** The diff adds no root and no new held value: the class receiver in
   `reqstr.rs` is a class (never collected, D59), and the L2 change routes subclass receivers
   onto the existing `ArithOperand::Send` and DO header object paths. A temporary test in the
   archive copy ran four programs plain and under `run_program_collect_every_alloc`: fresh heap
   right operands held across an allocating `+`/`*` on a subclass, a fresh subclass receiver for
   prefix `-`, a stem default receiver, an allocating class-side `STRING` under `||`, `length`
   and `PARSE VALUE`, and a DO whose initial, TO and BY are fresh heap values held across
   allocating `+` and `>` sends. 24, 14 and 20 collections, stdout and exit identical to the plain
   run. There is no new root, so no root-deletion test is owed.
6. **Mutations** (archive copy, release build each, three witnesses diffed against the HEAD
   binary):

   | mutant | weak | operators | class_side |
   |---|---|---|---|
   | A: `heap.rs` `_ => false`, plus `reqstr.rs` instance-only | RED (lines 1, 2 `The NIL object`) | green | RED |
   | B: stem redirect `to_number(default)` (`value.rs:848`) | green | RED (`4 -3`) | green |
   | C: VarRef redirect `to_number(referenced)` (`value.rs:870`) | green | **green** | green |
   | D: `loops.rs:2679` `to_number` to `operand_number` (the I1 fix) | green | green | green |
   | E: `loops.rs:1062` back to `to_number` | green | RED (`sent + 0` lines gone, `k 3 k 4`) | green |
   | F: `eval.rs:843` back to `to_number` | green | RED (`4 2 6 1.5 1 1 9`) | green |

   Mutant D makes `dos.rex` identical to the oracle. Mutant C is caught only by `vref2.rex`
   (oracle `VariableReference plus pfx- times`, mutant `VariableReference 4 -3 6`).

### Issues

#### Critical (Must Fix)

None.

#### Important (Should Fix)

- **I1. The loop step `+` is not sent to a String subclass control value.**
  `rust/crates/rexx-exec/src/run/loops.rs:2679` (`controlled_step_wide`) converts with
  `to_number` before asking `operator_message_receiver`, the L2 order this task exists to remove.
  The oracle sends `+` to the control value every step (`DoBlock.cpp:194`,
  `result->callOperatorMethod(OPERATOR_PLUS, by)`). Probe:

  ```rexx
  do i = 1 to 3; say 'm' i; if i = 1 then i = .c~new('1'); end
  do i = 1.5 to 3; say 'w' i; if i = 1.5 then i = .c~new('1.5'); end
  ::class c subclass string
  ::method '+'; say 'c+' arg(); forward class (super)
  ```

  Oracle rc 0 prints `c+ 1` after `m 1` and after `w 1.5`; the crate rc 0 prints neither. Fix: the
  same `operand_number` at `:2679` (mutant D: probe identical, all three witnesses still pass).
  Add the shape to `corpus/lang/string_subclass_operators.rex`. It is spec section 2's L2
  ("`eval.rs` and `loops.rs` convert to a number before the operator-receiver test") and the
  file is in the brief.

#### Minor (Nice to Have)

- **M1. The variable-reference arm of receiver mode has no witness.** `value.rs:870`: mutant C
  leaves every witness green. The report claims the receiver mode holds "through ... a variable
  reference". Add `call f >s` / `f: use arg r; say (r + 1) (-r)` to the operators witness (oracle
  `plus pfx-`).
- **M2. The budget record omits the collector-path program set.** The weak pass is the collector
  path, so the global constraint's list (fibcall, fibfunc, extcall, dispatch, dispatchclass,
  sendloop, varlookup, pkglocal, startup, alloc, hashnew) applies; the report measured five. I
  ran the other seven (`-p "fibfunc extcall dispatch dispatchclass varlookup pkglocal startup
  alloc hashnew"`, `-o /tmp/t2rev/cg2`, exit 0): all within +0.5% (largest hashnew +0.3093%,
  startup +0.2882%, alloc +0.2535%), and each -libc figure equals 9b close2's within 0.0001%,
  so the task adds nothing there. The records directory does not hold these figures.

### Assessment

**Task quality:** Needs fixes

**Reasoning:** L1 and L3 are correct, minimal and witnessed, and L2's receiver-mode conversion is
sound and within budget, but L2 leaves the loop step's `+` converting first (`loops.rs:2679`),
a one-word fix with an oracle-checked probe. The variable-reference arm and the full budget set
also lack records.
