# Task 6.9 re-review: fix round 1

I reviewed `93d76b864..bd88a6239`, the code commits `856b6c7f1` and `7936f3ca1`. HEAD was built into the session scratchpad (`rev69r1-target`, `cargo build --release --bin rexx`, exit 0); the target dir is deleted. The first-round binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.9/rexx-head`, built from `94b09da09`, the code that `046fa0ac5` carries. Probes ran on the oracle (standard wrapper), HEAD and that binary, each under `memcap 2G timeout 20`, from one fixed run directory. stdout, stderr and rc were compared separately. Probe sources were in `scratchpad/rev69-probes.AnbT/r1a`, `r1b` and `r1c`, which are not durable, so the deciding lines are quoted below.

## Spec compliance

Compliant. Important 1, Important 2 and Minor 3 from `task-6.9-review.md` are closed, tested with my own probes. Minor 7 is done. The parameter removal is a pure removal.

## Closed findings

* **Important 1 is closed.** My `reply_frag.rex` and `reply.rex` now match the oracle on all three descriptors. Each was run 8 times, and each side gave one distinct result. The oracle's stderr is `Error 98.936:  RETURN cannot return a value after a REPLY.` at line 14 (and line 24 in `reply.rex`). Neither ` a own h ia` nor any other handler line is printed for the CALL ON case. The new witness `raise_after_reply_interpreted.rex` also gave one distinct result per side over 8 runs, and the results are identical.
* **Important 2 and Minor 3 are closed.** These match on stdout, stderr and rc:
  * `nr_dump.rex`, which gives `ADDITIONAL Array`, `POSITION = 3`, the program as PROGRAM, no RESULT, one `sf PROGRAM 3`, and one traceback line;
  * `nr2` (`signal on any name gone_any`);
  * `nr4` (CHARS);
  * the untrapped `nr_untrapped` and `nr_untrapped_m`, which no longer print `Compiled method "LINEIN"`;
  * six main, routine and method variants (BIF and method LINEIN).

## Hunt

### (a) Replied-fragment state

I wrote a 30-program matrix. Each program sends `.c~new~m`, sleeps 0.4 s, and prints `done`. Each case ran three ways: with `call on user u name h`, with `signal on user u name h`, and with no trap (`nop`). The cases:
* nested INTERPRET after REPLY;
* `interpret "nop"`, then `reply`, then an interpreted RAISE;
* `call rr` after REPLY, where the routine `rr` has its own trap or none and interprets `raise ... return 'x'`;
* an internal call after REPLY, interpreted and plain (`ic2: raise user u description 'ic2' return 'x'`);
* a method on a second object sent from the replied method;
* a valueless `return` through INTERPRET.

Some shapes are translation errors on both sides (`99.924` for REPLY inside INTERPRET, `99.919` for REPLY in an internal call), and they match. Every program matches on all three descriptors. The 21 concurrent programs were each run 8 times, and every one gave one distinct result per side, identical across the sides. Deciding lines from the oracle, all reproduced by HEAD:
* nested INTERPRET under CALL ON gives `got r` and then `Error 98.936` at line 11. Under SIGNAL ON it gives ` m handler nest`. With no trap it gives `Error 98.936`.
* INTERPRET before REPLY behaves the same way: 98.936, ` m handler before`, 98.936.
* The routine called after REPLY is not replied itself, so it gives ` rr handler rr`, ` m after` under both kinds of trap, and ` m after` with none. There is no 98.936.
* The internal call after REPLY, under CALL ON in `m`, gives ` m handler ic` (and ` ic2`), with no 98.936.
* A method sent from the replied method gives ` dm handler dm` and ` m got x`. Under SIGNAL ON it gives `91.999 Message "DM" did not return a result.`
* A valueless `return` gives ` m handler noval` under both kinds of trap.

`reply_prop.rex` (propagate RETURN after REPLY, plain and through INTERPRET) still matches.

### (b) The 16.1 at the program's clause

I wrote 120 programs. Each sends a Stream method to `/nonexistent/dir/qq`: `linein`, `lineout('a')`, `charin`, `charout('a')`, `chars`, `lines`, `arrayin`, `open('read')`, `command('open read')` and `query('size')`. Each ran under `signal on notready name gone_lbl` and under `call on notready name gone_lbl`. Each ran in the main program, in a method and inside INTERPRET, with a SIGNAL ON SYNTAX handler that dumps every entry, frame and traceback line, and also with no SYNTAX trap.

All 60 untrapped programs match. Of the trapped ones, 24 differ, in two classes. Neither class is a defect of this round:
* **CALL ON with a missing label, main and method (16 programs).** The oracle prints `after op` (the next clause) before the SIGNAL. This is first-round Minor 4, which is queued. The first-round binary gives HEAD's output exactly.
* **SIGNAL ON inside INTERPRET (8 programs).** The only difference is the frame's name: the oracle has `sf PROGRAM 3 []`, HEAD has `sf PROGRAM 3 [s5.rex]`. Every entry of the condition object matches, as does the traceback line. This is finding 1 below.

### (c) The parameter removal

The diffs of `clause.rs`, `run.rs`, `run/loops.rs` and `ir/drive.rs` only drop the `code` argument and its declaration from these functions:
* `in_clause`, `in_counted_clause`, `leave_clause`;
* `deliver_pending_traps`, `end_promoted_branch`, `offer_to_trap`.

The diffs also drop the `Code` import in `clause.rs`. No other token changes. Perf: `callgrind.sh -r 1 -j 4 -B 0.5 -p "interpret sayloop nop emptyloop"` with pin, r0 (`t6.9/rexx-head`) and my build of HEAD, exit 0:

| program | r0 | head | head vs r0 |
|---|---|---|---|
| interpret | 4116424328 | 4117114703 | +0.0168% |
| sayloop | 179751209 | 179748595 | -0.0015% |
| nop | 9241484648 | 9241487097 | +0.00003% |
| emptyloop | 7747194550 | 7747196090 | +0.00002% |

For interpret, the libc-subtracted figures are identical (1702536829 on both), so the +0.0168% comes from libc and not from the crate's code. The `Op::Clause` arm is flat on `nop` and `emptyloop`.

### Regression sweep

I reran 185 first-round probe programs on HEAD. The only differences left are the known queued ones: the clause order of translation errors (`rinterp_*`, first-round Minor 6) and the failing-trap-call ordering (`b_*`, `ord`, first-round Minor 4). HEAD and the first-round binary give the same output on those.

Tests:
* `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
* `REXX_CORPUS_GATE=1 ... --test corpus --test refusal_sites`: exit 0, `1103 of 1103 matching`.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **The frame name of a 16.1 raised from a SIGNAL trap inside INTERPRET.** The oracle reports one frame, `PROGRAM 3` with an empty name. HEAD reports the same frame with the program's name. The probes:
   * `signal on syntax; interpret "signal on novalue name gone_x; y = qq"`: the oracle gives `sf PROGRAM 2 []`, HEAD `sf PROGRAM 2 [i2.rex]`;
   * the same with a NOTREADY from `linein` inside the fragment;
   * `signal on notready name gone_x` set before the INTERPRET.

   The first-round binary gave two frames here, `sf INTERPRET 2 []` and `sf PROGRAM 2 [i2.rex]`, so HEAD is closer. For an ordinary failure, `interpret "x = 1/0"`, both sides give `sf INTERPRET 2 []; sf PROGRAM 2 [i1.rex]`. A PROGRAM frame with no name looks like an oracle artefact: the frame is labelled PROGRAM but takes the INTERPRET activation's name. Do not copy it without a ruling. Queue it as a Deviation candidate, next to first-round Minor 4.

## Verdict

APPROVED
