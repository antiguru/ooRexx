# Task 8.7b re-review, fix round 3

Reviewed `57ec30209..f08f35538` (code `9cb989e54`, `376052dd5`; records `f08f35538`) against
`task-8.7b-fix-r3.md` and the controller's ruling on Minor 2 there.

**Verdict: APPROVED.** Critical 0, Important 0, Minor 0. No new oracle defect.

| brief item | verdict |
|---|---|
| 1. Debug pause input drop (Important 1) | ADDRESSED |
| 2. SIGNAL ON release POSITION and TRACEBACK (Minor 1) | ADDRESSED |
| 3. NOTREADY under a CALL ON ANY delayed at the clause (Minor 2, as ruled) | ADDRESSED |
| 4. Entry 67 and the `rust/CLAUDE.md` sentence | ADDRESSED |

## What I ran

* Builds: `git archive` of `d1f8b387f` and `376052dd5` (`rust/`, `interpreter/`) into fresh scratchpad
  directories, `CARGO_INCREMENTAL=0 memcap 8G cargo build --release -j 4 --bin rexx` into
  `/tmp/claude-1000/t87b-rr3-target-base` and `-head`, exit 0 each. `.text` (`objcopy -O binary
  --only-section=.text`, `cmp`): base identical to `ooRexx-p9-pin/cg-9d/t8.7b/rexx-head-d1f8b387f`, head
  identical to `rexx-head-9cb989e54`, so the probes below ran on those two pins. Both target dirs deleted
  at the end.
* Probes: `t87b-rr3-scratch/<name>/`, runner `t87b-rr3-scratch/run.sh`, sides `o` oracle (standard
  wrapper, only where named), `b` `d1f8b387f`, `h` `9cb989e54`, crate under `memcap 2G timeout -k 5 20`,
  stdin `/dev/null`, stdout, stderr and rc compared separately. Nothing shaped like entries 64 to 67 ran
  on the oracle: the oracle ran only `s7o`, `s9o`, `n2`, `n3` (no held condition, and in `n2`/`n3` each
  of the two conditions has its own trap).
* Tests at head, from the archived tree with `build`, `oodocs`, `ootest` linked in:
  `memcap 8G cargo test --release -j 4 --no-fail-fast -p rexx-exec --lib --test signals --test
  refusal_sites`: exit 101; lib `1206 passed; 1 failed`, the failure
  `a_move_across_file_systems_copies_and_unlinks` ("needs ... on different file systems": the archive is
  on `/tmp`, environmental); `a_condition_raised_while_its_handler_runs_is_held ... ok`; `refusal_sites`
  `5 passed`; `signals` `42 passed`.
* Case-file witnesses against `d1f8b387f`'s code: the base archive's case file replaced by each case
  alone (the changed `SIGNAL ON` case, the new NOTREADY case, the new debug case, cut from head's file),
  `cargo test --release -p rexx-exec --lib -- a_condition_raised_while_its_handler_runs_is_held`, exit
  101 each, `0 passed; 1 failed`. Actual lines: signal `out> sg 10     10 *-*   signal on user sig name
  sg`; notready `out> ha 2 NOTREADY no_such_file_zz.txt`, `out> after a, m = 2`; debug `out> after 6`.
  The base file was restored and `cmp`'d.

## Items

**1. Debug pause input.** `dp2` (rr2's): `b` rc 137 after 210840 stdout lines; `h` rc 0, `after 1`.
The drop is limited to rows queued during the `LINEIN` send (`run/interpret.rs:384`-`:393`,
`drop_queued_for_delayed`, `run/condition.rs:176`). Conditions raised elsewhere while a pause exists
keep the delayed rule:

* `dq1`: `trace ?r` in force, the handler's own clause `call rp 'again'` raises the condition again,
  the reader answers `''`. `b` and `h` identical on all three descriptors: `h 1 first`, `h 1 returns`,
  `h 2 again`, `h 2 returns`, `after 2`, the same stdout as the untraced control `dq1c`.
* `dq2`: the reader's answer is the line `call rp 'frominput'`, so the condition comes from executing
  the input, not from the read. `b` and `h` identical on all three.
* `dq7`: a pause in a subroutine the handler calls, which also queues a different, trapped condition
  (`other`); the reader raises `ping`. `h`: `h 1 first`, `ho x`, `h 1 returns`, `after 1`; `b` runs `h`
  four times. The `other` row queued before the read survives the drop.
* `dq3` (`CALL ON ANY`, reader raises USER): `h` `after 1`, `b` `after 2`.
* `dq4` (reader raises `other` while `ho` runs for `other`, `trace ?r` inherited): `h` `ho from input`
  once, `b` twice. Note 4 covers both: the trap is delayed and the input is being processed.
* `dq6` (reader raises SYNTAX by `return 1/0`): `b` and `h` identical, `Error 42.3` at rc 214; the drop
  runs before `answer?`, so a failing send is still the pause's failure.

Witness red at `d1f8b387f` (above), green at head. NOTREADY ignored while delayed: `ignored_while_delayed`
unchanged; rr2's `d1`, `p1`, `p2`, `p3`, `p5`, `p6`, `p7` re-run at `h`: stdout, stderr and rc each equal
to rr2's recorded oracle output.

**2. SIGNAL ON release.** `p8`: `h` `sg two 12 27     12 *-*     raise user s2 description (dd)
return` (POSITION and TRACEBACK the raise's, SIGL the `SIGNAL ON`); `b` `sg two 27 27     27 *-*
signal on ...`. Concern 2, things set between hold and release or around the trap, all `h`:

* `s1`: the released `SIGNAL ON` names a missing label; `SIGNAL ON SYNTAX` takes 16.1. `sx SYNTAX 16.1
  9  9`, traceback the `SIGNAL ON` clause: the SYNTAX trap gets its own object, not the held USER's.
  `b` identical.
* `s8`: a SYNTAX trapped by `SIGNAL ON SYNTAX` in the handler between hold and release. `sx SYNTAX 9`,
  then `sg USER S2 two SIGNAL 4 12`, traceback line 4.
* `s9`: a `RAISE PROPAGATE`d SYNTAX from a subroutine between hold and release. `b` and `h` identical;
  the same program without the hold (`s9o`) is identical on `o`, `b`, `h` (the propagated SYNTAX ends
  the program, rc 214), so the release is never reached there. `s7`/`s7o` (a propagated USER) the same:
  `o`, `b`, `h` identical, `hs2 one` only.
* `s2` (release inside `INTERPRET`): `sg two SIGNAL 4 8 two-add`, traceback line 4. `s3` (release in a
  subroutine the handler calls): `sg two SIGNAL 4 12`. `s4` (`SIGNAL ON ANY` over a held USER under
  `CALL ON ANY`): `sg USER S2 two SIGNAL 4 8`, then a later `1 / 0` gives `sx SYNTAX 42.3 15`. `s5`
  (held HALT): `sg HALT two SIGNAL 4 8`. `s6` (a CALL ON condition and a SYNTAX after the release): `h3
  three 16 CALL`, then the 42.3 report names line 14; equal to `b` except `sg 4` against `sg 7`.

No probe reached `Delivery::held` set with `reraised_object` empty (which would build the object with
POSITION 0); none of the above takes `reraised_object` between `release_delayed` and the trap.

**3. NOTREADY under a CALL ON ANY delayed at the clause.** `p4`: `h` `ha 1 USER U one`, `after a, m =
1`; the second clause, NOTREADY first, still runs the handler for each (`ha 2 NOTREADY`, `ha 3 USER U
two`). `n1` (crate only): two NOTREADYs under ANY, `ha 3 NOTREADY no_such_file_zz5.txt`, `after c, m =
3`; USER, NOTREADY, USER in one clause: `ha 4 USER U three`, `ha 5 USER U four`, NOTREADY ignored.
`n5` (p4's clause under `INTERPRET`): `h` `m = 1`, `b` `m = 2`. `p5`: `b`, `h` identical. Adjacent
successes, where each condition has its own trap: `n2` (`CALL ON USER` and `CALL ON ANY`) `hu 1 USER
U one`, `ha 2 NOTREADY ...`, `m = 2`, and `n3` (`CALL ON NOTREADY` and `CALL ON ANY`) `ha 1 USER U one`,
`hn 2 NOTREADY ...`, `m = 2`: `o`, `b`, `h` identical on all three descriptors. Witness red at
`d1f8b387f` (above).

**4. Entry 67.** Its program is `t87b-rr2-scratch/p4/p4.rex` byte for byte (`cmp`), and so is
`fix-r3/p4/p4.rex`; the effect line matches rr2's `p4/out/p4.rex.o.*` (stdout `ha 1 NOTREADY
no_such_file_zz.txt`, stderr empty, rc 139). The `rust/CLAUDE.md` sentence states what that run shows.
**Refusal sites**: every changed row of `refusal-sites.tsv` is an `error.rs` row whose line moved by
+4 and nothing else (`git diff 9cb989e54 376052dd5 -- rust/corpus/refusal-sites.tsv`, old and new rows
compared with the `error.rs:N` line masked: no other difference, the only shift 4); `error.rs` gained
four lines above every such row, and the test is green at head (above).

## Queue candidate (predates the task)

`n4`: `x = ru('one') || rn()` under `CALL ON ANY`, `rn: return linein(...)`. `b` and `h` both run the
handler for the NOTREADY first and then for the USER (`m = 2`): the NOTREADY is raised and delivered in
`rn`'s activation, so `notready_trap_delayed_at(trapping)` does not see the USER queued at main. The
same-name form `n7` (`linein(...) || rn()` under `CALL ON NOTREADY`) does the same at `b` and `h`
(`hn 1 ...zz2`, `hn 2 ...zz`). Whether Note 2's inherited DELAY state reaches an internal routine
called in the clause after the first condition queued is for the controller; it is outside this
round's ruling, which names one clause.

## Perf

The implementer's `fix-r3/callgrind-final.txt`: every program, exit 0, no OVER or SPREAD flag; -libc
d% at most +0.0366% (pingmsg); every `cmd:` in `cg-9d/t8.7b/fix-r3-final/*.cg` names
`rexx-base-504ff0743` or `rexx-head-9cb989e54` (99 each, `grep -h '^cmd:' | awk '{print $2}' | sort |
uniq -c`), and the latter's `.text` is my `376052dd5` build's. SPREAD: pingguard head spread 0.0001%,
-libc -0.0162%; pingsem 0.0001%, -0.3563%.

Re-measured: `bench-programs/callgrind.sh -B 0.5 -j 4 -r 3 -o
/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t8.7b/rr3 -p "pingmsg" base=.../rexx-base-504ff0743
head=.../rexx-head-9cb989e54`, exit 0, log `cg-9d/t8.7b/callgrind-rr3.log`, `cmd:` lines 3 per binary:

| program | head d% | -libc d% | spread base / head |
|---|---|---|---|
| pingmsg | +0.0057 | +0.0365 | 0.0002 / 0.0003 |

Within +0.05% and within 0.0001 points of the implementer's figure.

## Strengths

The drop is scoped to the rows the send itself queued, which is what keeps `dq1` and `dq2` on the
delayed rule. The SIGNAL release reuses the raise's object, so every condition entry agrees with the
CALL ON and RETURN releases, and `s1` shows the missing-label path does not leak it.
