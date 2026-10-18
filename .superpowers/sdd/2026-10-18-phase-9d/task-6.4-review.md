# Task 6.4 review: stream names and states

Reviewer of `0e7e18cc4..a735033de` (crates, `rust/corpus/lang`, `phase-9.txt`). The crate was built at
`8b0400019`, whose `rust/crates` tree is identical to `a735033de` (`git diff --stat a735033de HEAD --
rust/crates` is empty). Every probe below ran on the oracle (standard wrapper, fresh dir) and on the
crate (`memcap 2G`, `timeout 20`), each side in its own empty directory.

## Spec compliance

Compliant. The four witnesses exist. One witness uses a name holding a `'80'x` byte
(`stream_notready_names.rex`). Each of the brief's three clusters has a root cause in the diff. The
`RAISE PROPAGATE` change goes beyond the brief, but it was needed: the cluster's `STREAM 'C' OPEN` rows
go through the orx `Stream~command`'s `raise propagate`, and the report flags the side effect
(Concern 1).

Cannot be verified from the diff: whether `raise-propagate-from-method`'s own instance still needs to
run for its INTERPRET and REPLY witnesses (Review Focus 4). The REPLY finding below says it does.

## Strengths

* All `NOTREADY` raises now go through one `#[cold]` helper, `notready` (`dispatch/stream.rs:400`). It
  sets the state, `ADDITIONAL` and `RESULT` together, which removed six copies of the same block.
* `read_setup` and `write_setup` follow the oracle's `readSetup` and `writeSetup` by name.
* `propagate_unreportable` cites the oracle lines it implements.
* The propagate matrix below matched on 73 of 90 programs with natural conditions. Every difference
  traces to the two findings or to thread scheduling.

## Probes run

1. **Propagate matrix.** `$S/gen2.sh` generates 90 programs: conditions ERROR, FAILURE, NOTREADY,
   USER X, HALT and NOVALUE, crossed with the method, routine, INTERPRET, nested-two-levels and
   REPLY'd-method settings, crossed with `SIGNAL ON`, `CALL ON` and no trap in the caller. The callee
   raises each condition naturally (a failing command, `chars` on a missing file, `undefined_v`,
   or a `RAISE` from an internal call) and its own `SIGNAL ON` handler runs `raise propagate`. The
   caller prints `CONDITION` C, I and D, plus `PROPAGATED`, `RESULT` and `INSTRUCTION`, and then the
   returned `RESULT`. A first matrix (`gen.sh`) raised with a tail-less `RAISE` in the callee. That
   leaves the activation before the callee's own trap runs, so it exercised only the nested
   setting. All 86 of its matching programs are matches, and it is not counted above.
   * 77 of 90 match: every method, routine and INTERPRET program, and every nested program except
     the HALT one with no trap.
   * The 13 differences are `nested_halt_none` (finding 1); `reply_{error,failure,notready}_*`
     (finding 2); and `reply_halt_*`, where only the order of `R replied` and the HALT traceback on
     two threads differs.
   * Targeted programs (`$S/p3`) match: a `CALL ON` handler in the callee that propagates from a
     further internal call (`e_callon_callee`); top-level propagate of ERROR, where the exit rc is
     the object's RESULT; propagated NOTREADY `RESULT` through `linein` and `chars`; HALT
     propagated into a caller's `SIGNAL ON`.
2. **Streams.** `$S/p/st1.rex` covers a directory, a missing file, `'wbad'||'80'x||'name'`,
   `/dev/null`, a mode-444 file, and a missing parent directory. Each runs LINEOUT, CHAROUT, CHARS,
   LINES, LINES C, LINEIN with and without a position, CHARIN with and without a position, both
   closes, OPEN READ, OPEN WRITE, QUERY EXISTS and QUERY SIZE. Each row prints the answer, S, D and
   the condition's D, `ADDITIONAL` class, `RESULT`, `INSTRUCTION` and `PROPAGATED`. One row
   differs (finding 4). `$S/p/st2.rex` covers a FIFO and differs (finding 3).
3. **LINEOUT and CHAROUT with no data create a missing file.** The oracle does too: `$S/p4/a_create.rex`
   matches for both BIFs, both methods, `charout(n,,1)`, a mode-444 file, and a stream opened READ
   then closed by `lineout(n)` and `charout(n)`.
4. **Argument edges.** `$S/p5/v1..v13` run `'2.5'`, `'-1'`, `''`, `'1E10'`, `' 2 '`, `'1E1'`, `'1.0'`,
   `'0'`, `'+2'`, `'.5'`, `'-0'`, `'0.0'` and `'2e0'`. Each goes to every position and count
   argument of the four BIFs and the four methods. The 40.12, 88.907 and 93.907 codes and messages
   all match. The differences that remain:
   * the bare-93 `MESSAGE`, already in Concern 3;
   * a `charin` read after a buffered BIF write on another stream object, which is oracle buffering;
   * `.stream~new(f)~lines(v)` (finding 5);
   * an OOM after a `charout` at `1E10` (finding 6).

## Issues

### Critical

None.

### Important

1. **An untrapped propagated HALT is not reported.** Site: `run/condition.rs`,
   `propagate_unreportable`, the `None => Ok(Flow::Exit(result))` arm, and the early return when
   there is no caller. `RexxActivation::raise` (`RexxActivation.cpp`, the `if (!trapped)` block
   between `:1866` and `:1890`) reports 4.1 for an untrapped HALT and 97 for an untrapped NOMETHOD
   before it calls `returnFrom`. The new function cites `:1801`-`:1805` and `:1895` and skips that
   block. Probe `$S/g2/nested_halt_none.rex`: the oracle prints a traceback at `raise propagate`
   (line 18) and `Error 4.1`, rc 252. The crate prints `R none`, `end`, rc 0. The behaviour predates
   the task (the old code answered `Flow::Exit(None)`), but the new function claims to implement
   this oracle routine. The non-propagate path a few lines up already has the `None if halt` arm to
   reuse.
2. **A propagate after `REPLY` returns a value without 98.936.** Site: the same function's
   `Ok(Flow::Exit(result))` arms, which bypass the check `exec_return` does at `run.rs:2513`. The
   oracle's `returnFrom(resultObj)` raises 98.936 when the propagated object has a `RESULT`. Probes
   `$S/g2/reply_{error,failure,notready}_{sig,call,none}.rex`: the oracle prints `raise propagate`
   at line 21 and `Error 98.936: RETURN cannot return a value after a REPLY.`, and the crate
   prints nothing. This is new: the old `Exit(None)` carried no value. A `USER` condition carries
   no `RESULT`, so `reply_userx_*` match. No witness covers it. The `raise-propagate-from-method`
   instance's REPLY witness (Review Focus 4) would catch it.

### Minor

3. **FIFO writes (pre-existing, outside the diff).** Probe `$S/p/st2.rex`:
   * `lineout(fifo, 'x')`: the oracle answers 0 READY, and the crate answers 1 with `NOTREADY`
     `ERROR:29 Illegal seek`.
   * `chars(fifo)`: the oracle answers 1, and the crate answers 0.
   * `lineout(fifo, 'z', 1)`: the oracle raises 93.958, and the crate raises `NOTREADY`.

   The write and transient paths are not in this diff.
4. **`QUERY EXISTS` on `/dev/null` (pre-existing).** After `OPEN WRITE`, the oracle answers a name and
   the crate answers `''`. Probe: `$S/p/st1.rex`, row `QEXISTS` under `/dev/null`.
5. **`Stream~lines(option)` (pre-existing).** The method accepts any option, where the oracle raises
   93.915. Probe: `.stream~new('wf')~lines('2.5')` in `$S/p5/v1.rex`.
6. **An OOM reading a sparse 10 GB file (licensed OOM area).** After `charout('wf','q',1E10)`, both
   sides make a 10 GB sparse file. The crate's next `linein` holds the whole line in memory and is
   OOM-killed at 2G (`$S/p5/v4.rex`). The oracle answers `''`. The task widened the accepted
   spellings to include `1E10`, but the digit spelling reached this before. A count of `1E10`
   (`charin(f, 1, 1E10)`) has the same shape: `read_from` allocates `vec![0; len]` before reading.
   The oracle answers 5.0 there. A cap at the remaining file size is cheap.
7. **The NOMETHOD propagate position (pre-existing, reportable path).** Probe `$S/p3/b_nomethod.rex`:
   the oracle's traceback names the `raise propagate` line (9), and the crate's names the original
   send (6).

## Witnesses

The witnesses catch these plausible wrong fixes:
* returning nothing instead of `RESULT` (the `missing` and `method returns` rows);
* not marking `INSTRUCTION CALL` or `PROPAGATED` (the `show` rows);
* an untouched SIGNAL path (the `signalled` row).

The witnesses miss these plausible wrong fixes:
* skipping the HALT and NOMETHOD report (finding 1);
* ignoring REPLY (finding 2).

## Verdicts

* **Spec compliance:** compliant.
* **Task quality:** needs fixes. Findings 1 and 2 are both in the new `propagate_unreportable`.
  Finding 2 is new behaviour this task introduced. Each fix is small and can reuse existing checks.

`$S` is `/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/91ae65d5-ff7c-4420-b551-a1575c2ba797/scratchpad/rev64`.
