# Task 6.4 fix round 1: re-review

Verdict: APPROVE. No open findings against the fix. Two minor observations, neither blocks.

Method: crate built at HEAD (`aa9bc799c` code, records on top) and, as a control, at `8b0400019` (`git archive`, own target dir).
Every program ran on the oracle, the fix and the control from fresh empty dirs.
"SAME" means stdout, stderr and rc each identical, compared as separate streams.

## I1: closed

* `g2/nested_halt_none`: SAME on the fix, DIFF on the control. Error 4.1 at line 18, rc 252.
* Own variations, all SAME on the fix:
  * untrapped propagated NOMETHOD in a routine: stdout, rc 159 and the 97.1 message match.
    The stderr traceback differs, see observation 1. Control differs the same way, so not a regression.
  * NOMETHOD propagated to a caller's SIGNAL ON, and in a user-class method with `unknown`.
  * HALT propagated through three routines to no trap (control DIFF, fix SAME), and to a top-level `SIGNAL ON HALT`.
  * HALT with `CALL ON HALT` in the caller, for a routine and for a method.

## I2: closed

* `g2/reply_*` (18 programs): all SAME on the fix. `reply_error_none`, `reply_notready_none`, `reply_halt_*` were DIFF on the control in merged order only.
* Own variations, all SAME: propagate after REPLY with no RESULT (USER, NOVALUE) via SIGNAL ON,
  `raise propagate return 9` through SIGNAL ON and CALL ON, a propagate from a routine and from an internal call of the replied method,
  ERROR under CALL ON in the replied method.

## No regression

The whole propagate matrix `rev64/g` and `rev64/g2` (180 programs: method, routine, nested, INTERPRET, each over six conditions and three trap styles): 180 SAME, 0 DIFF.

## M3 to M6

* M3: `p/st1` SAME. `p/st2` differs only at `CHAROUT y` and the following `CHARS` (oracle `ERROR:29`), the queued oracle FIFO defect.
* M4: `QUERY EXISTS` on `/dev/null`, `/dev/zero`, a missing path, an open-for-read and a closed file all match.
* M5: `LINES` with `n N c C Count x '' 2.5` matches (probe `rrv/x1`).
* M6: `p5/v1`, `v2`, `v3`, `v7`, `v8`, `v10` to `v12` SAME. `charin(f,1,1E10)` returns 3 in the crate with no big allocation.
  The oracle itself dies with Error 5 there, the licensed OOM divergence, not a finding.
* Remaining known: `p5/v5`, `v6`, `v9`, `v13` (Concern 3), `p5/v4` (OOM), unchanged.

## Observations

1. Untrapped NOMETHOD traceback: the oracle's first trace line is the `raise propagate` site (line 12 in `rrv/v1_nomethod_untrapped`).
   The crate shows the original `o~nosuch` line (7). HALT now shows the propagate line in both. Stderr only, one line, present before the fix.
2. `raise user x return 5` inside a replied method: the oracle raises 98.936 at the RAISE itself.
   The crate stays silent, before and after the fix. It is a RAISE RETURN after REPLY rule, not the propagate path, so it is outside this round.

Scratch files: `rev64/rrv/*.rex` in the session scratchpad. Target dirs deleted.
