# Task 6 re-review: fix round 1 (`ea4a6d194`)

Reviewer: review-s6. Scope: the diff `b06b278ce..ea4a6d194` and my findings from `task-6-review.md`, checked
against the controller's rulings. HEAD is `ea4a6d194` on a clean tree. The release `rexx-run` was built into
my own target dir, with a `Compiling rexx-exec` line. The forge was rebuilt with its `build.sh`: NEEDED
libgcc_s.so.1, libc.so.6 and ld-linux, and no undefined Rexx symbol. The instrument is unchanged: `$S/cmp.sh`,
the standard oracle wrapper, each side run from a fresh `mktemp -d`, with stdout, stderr and rc compared
separately. `$S` is `scratchpad/review-s6/`. Each prediction was written before its run and is appended to
`$S/predictions.txt` under "Re-review of ea4a6d194".

## Verdict

**Clean: every finding is fixed or recorded as ruled, and the fix adds no defect.** Two Minor notes follow;
neither blocks.

## Findings against the rulings

| Finding | Ruling | Status | Evidence |
|---|---|---|---|
| I1 | match the oracle | **fixed** | `opts_user.rex` and `opts_notready.rex` are now identical on all three descriptors, rc 0. The fix round's `opts_error`, `opts_callback`, `opts_both` and `opts_failure` are identical too. |
| I2 | make the entry true, or match | **entry made true** | The ledger (`phase-4-exclusions.txt`, the "A REDIRECTING HANDLER'S LINES ..." entry) quotes `interleave.rex`. Re-run: oracle `out o:a`, `err e:a`, `out o:b`, `err e:b`; crate both `out` lines, then both `err` lines. Both rc 0, stderr 0 bytes, as quoted. |
| M1 | match, or make the sentence true | **sentence made true** | The same entry now excepts a target that is also the input source. Re-run `conflict_syntax.rex`: stem line oracle `stem 93.900 2 s1 s2`, crate `stem 93.900 1 before S.2`; array line `array 93.900 1 before The NIL object` on both, as quoted. `CommandHandler.cpp:148` is the one `ioContext->cleanup()`, after `activity->run`. |
| M2 | (fix) | **fixed** | The `Popped` doc is back on `struct Popped` (`dispatch/library.rs`). |
| M3 | (fix) | **fixed** | "One of the write members." |
| M4 | (fix) | **fixed** | `redirect.rs` now cites the ledger. |
| M5 | distinct names, then re-derive | **fixed** | One constructor, `escalated` (`command.rs:408`, confirmed by grep). The method is `handler_escalation` and returns `Failure`, so it is not a constructor by the table's definition. No two constructors share a name any more. The refusal rows are checked below. |
| M6 | record as a divergence | **recorded** | New entry "A NULL COMMAND ENVIRONMENT NAME REGISTERS NOTHING". I checked both of its citations: `InterpreterInstance.cpp:915` is `new_upper_string(name)`, and `StringClass.hpp:887` is `strlen(string)` with no guard. |
| M7 | queued only | **queued** | `.superpowers/sdd/queued/2026-09-27-stream-write-visibility.md` exists. |

## Escalation order against the oracle, with SIGNAL ON traps

The existing probes arm only `CALL ON`, so I wrote three new ones. Each arms `SIGNAL ON SYNTAX` plus one
`SIGNAL ON <trap>` inside an internal routine, then issues the handler command. All are identical on all three
descriptors, rc 0, and every line came out as predicted:

- `sig_error.rex` (`::options error syntax`):
  - USER under `SIGNAL ON USER` is 98.970. A trap on the raised condition does not disarm ERROR SYNTAX.
  - USER under `SIGNAL ON ERROR` returns `res 0`. The ERROR trap disarms ERROR SYNTAX, and the USER is
    untrapped and silent.
  - ERROR is trapped by `SIGNAL ON ERROR`, and FAILURE by `SIGNAL ON FAILURE`.
  - FAILURE under `SIGNAL ON ERROR` is trapped as ERROR.
  - NOTREADY under `SIGNAL ON NOTREADY` is 98.970.
  - USER with no RESULT under `SIGNAL ON USER` is 98.970.
- `sig_ready.rex` (`::options notready syntax lostdigits syntax nostring syntax`):
  - NOTREADY, LOSTDIGITS and NOSTRING under their own `SIGNAL ON` are trapped. The trap disarms each one's
    SYNTAX.
  - NOTREADY under `SIGNAL ON USER` is 98.974.
  - ERROR under `SIGNAL ON ERROR` is trapped.
- `sig_both.rex` (`::options error syntax notready syntax`):
  - NOTREADY under `SIGNAL ON ERROR` is 98.974. The callback arm comes first on the oracle, and the clause
    arm comes first here.
  - NOTREADY under `SIGNAL ON NOTREADY` is 98.970. NOTREADY SYNTAX is disarmed, and
    `RexxActivation::command`'s ERROR SYNTAX arm then fires ahead of the trap.
  - USER under `SIGNAL ON NOTREADY` is 98.970.

`sig_both`'s second line is the case that needs both checks, in the order the code has them. The clause arm
at `command.rs:808` tests the held name. `raise_handler_condition` then tests ERROR SYNTAX ahead of the
traps. Both answer as the oracle does.

## The refusal rows

The tsv diff has exactly two rows changed and none added:
- `escalated`: moves to `command.rs:408`, which is the free function. It stays `body`/`off-send-surface`, and
  every construction site is in `command.rs`.
- `nostring_syntax`: `send` becomes `body+send`. This is right: `escalated` in `command.rs`, a body file, now
  constructs it.

`lostdigits` (error.rs:472), `notready_syntax`, `failure_syntax` and `error_syntax` each gain a construction
site in `command.rs`. They were already `body`, so their rows are unchanged, which is right. The gates,
including `refusal_sites`, are green per the appended status.

## Other reruns

With the fixed binary, `$S/cmp.sh` over every `task-6-forge/probes/*.rex` and the three
`rust/corpus/lang/library_command_*.rex` gives identical results everywhere except four probes, all
recorded: `address_trace` (stderr), `stream_state` (all three descriptors), `interleave` and
`conflict_syntax`. The new corpus program `library_command_options.rex` is identical. Its sourceline
expectation is `count 28`, and the file has 28 lines.

## Minor notes (new, not blocking)

- **N1.** Two constructors' docs describe only their original caller. `Raised::notready_syntax`
  (`error.rs:1585-1588`) says "`name` is the stream's name as the program wrote it", and `Raised::lostdigits`
  (`error.rs:469-471`) says "an arithmetic operand". Both now also take a handler condition's description
  from `escalated` (`command.rs:408`).
- **N2.** The new ledger sentence "Any stream-object target with a side effect sees it, the FUNCTION group's
  ArrayOutputStream shape among them" overreaches. It comes from my own review's wording. The FUNCTION
  group's `ArrayOutputStream` appends to its own array, so it cannot show the order on its own. What shows the
  order is a pair of targets whose side effects land in one place, as `interleave.rex`'s `SAY` does. Suggested
  wording: "an OutputStream subclass target is enough; interleave.rex's pair both SAY".

## Cleanup

`$S/target` was deleted after this re-review.
