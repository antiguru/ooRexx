# Task 6.9 review: conditions

I reviewed `31b57465c..046fa0ac5`. HEAD was built into the session scratchpad (`rev69-target`, `cargo build --release --bin rexx`, exit 0). The parent binary is `/home/moritz/dev/repos/ooRexx-p9-pin/cg-9d/t6.8-r2/rexx-head`, built from `f48b4d866`. `git diff --stat f48b4d866 31b57465c -- rust/crates` is empty, so that binary carries BASE's code. Every probe ran on all three sides: the oracle under the standard wrapper, and HEAD and the parent under `memcap 2G timeout 20`. Each run started from the same fixed run directory, holding the probe's `.rex` files and nothing that could be mistaken for an external routine. I compared stdout, stderr and rc separately. The probe sources were in `scratchpad/rev69-probes.AnbT/`, which is not durable, so the deciding lines are quoted below.

## Spec compliance

Compliant for the five clusters and for most of the handed-over items. Every cluster test named in the report passes. The five witnesses match the oracle in the gated corpus (1101 of 1101). Two items fall short, both on paths the task changed:

* The 98.936 after a `REPLY` is implemented for a plain `RAISE ... RETURN value`, but not when the RAISE runs inside an `INTERPRET` (finding 1).
* The cluster `signal-trap-missing-label` now traps the 16.1 in every context I probed, with one exception: a NOTREADY raised inside a `LIBRARY REXX` stream method builds a wrong 16.1 object (finding 2).

## Strengths

* **The RETURN-tail sender rule matches the oracle everywhere off the REPLY path.** These all match on stdout, stderr and rc:
  * plain `RAISE USER ... RETURN` in a routine;
  * the same inside `INTERPRET` under `CALL ON`, under `SIGNAL ON` and with no trap;
  * `EXIT` inside `INTERPRET`;
  * nested `INTERPRET`;
  * `INTERPRET` in an internal call, with and without the call's own handler;
  * `call ic` from inside a fragment, where `ic` raises with `RETURN`;
  * methods: own handler through `INTERPRET`, `EXIT` through `INTERPRET`, plain `RETURN`, and `SIGNAL ON` through `INTERPRET`;
  * DESCRIPTION, ADDITIONAL, ARRAY, RETURN and EXIT in combination;
  * SYNTAX `RETURN`/`EXIT` RESULT.

  Ten of these 19 programs differ on the parent, so they do exercise the change. The `x = r()` and `call r2` RESULT readbacks after a trapped RETURN match.
* **`RAISE PROPAGATE` with a tail matches.** These all match:
  * `interpret "raise propagate return 'pv1'"` from a CALL ON handler;
  * `interpret "raise propagate"`;
  * a SIGNAL handler that installs its own CALL ON and propagates through INTERPRET;
  * `raise propagate description 'newd' additional ('nad') return 'second'`;
  * SYNTAX `interpret "raise propagate return 'p1'"` followed by a second-level `raise propagate exit 'p2'`;
  * HALT and NOMETHOD propagated from INTERPRET, trapped and untrapped (`Error 97.1 ... line 8` at the interpreted `raise propagate return 'nv'`);
  * under TRACE R/I, the `>K>   "RESULT" => "cd"` lines.

  After a REPLY, `raise propagate return 'pp'` matches both plain and through INTERPRET, on 8 runs each.
* **Condition-object fields match on 30 programs.** Each program dumped CONDITION, DESCRIPTION, ADDITIONAL, RESULT, RC, CODE, ERRORTEXT, MESSAGE, INSTRUCTION, PROPAGATED, POSITION, PROGRAM, PACKAGE, STACKFRAMES (type, line, name) and TRACEBACK. The conditions were SYNTAX, NOMETHOD, NOSTRING, NOTREADY (BIF and method), HALT, USER, LOSTDIGITS and ERROR, under CALL ON and SIGNAL ON, both direct and propagated from a routine's handler. Every program matches the oracle. Eleven differ on the parent.
* **NOTREADY frame leading holds across 22 stream operations.** Under `CALL ON NOTREADY` I tried CHARIN, CHARS, LINES, LINEIN (with and without a position), LINEOUT, CHAROUT, `STREAM(..., 'c', 'open read')`, and the Stream methods `open`, `charin`, `lineout`, `command`, `arrayin`, `makearray`, `supplier`, `lines`, `linein(1,1)`, `arrayout` and `say`. I also tried `pull`, `parse linein`, and two NOTREADYs in one clause. Each handler saved the object, ran four more native sends (`reverse`, `items`, `query`, `pos`), and dumped the object again. All 22 match, so no owed object waits for a later native's frame.
* **NOMETHOD and NOSTRING ADDITIONAL match** across 30 programs: SIGNAL ON ANY, SYNTAX, NOMETHOD and NOSTRING, each against a string receiver, `.nil` with arguments, NOSTRING concatenation, INTERPRET, a routine, and a method with its own trap, each followed by a second NOMETHOD in the handler.
* **CALL ON trap argument.** All of these match:
  * a label (`lab args 1 Directory USER A d1`, `parse arg` giving `a Directory`);
  * a `PROCEDURE` label with `use arg`;
  * an external `.rex` routine, a `::routine`, and `CALL ON ANY`;
  * ERROR;
  * the builtins `reverse`, `length`, `datatype`, `condition` and `linein`;
  * the trap set inside `INTERPRET`, and a raise from inside `INTERPRET`;
  * `call on user i name arg` giving `40.12 ARG argument 1 must be a whole number; found "a Directory"` at the `call rz` line.

  The queued item's three shapes (`qcallarg`, `qcoorder`, and my `coorder2`, which mutates `arg(1)` and reads it back through `condition('O')` and `.context~condition`) match on every descriptor. The item is rightly closed.
* **SIGNAL ON with a missing label.** All of these match, with SIGL, `condition('I')` and POSITION:
  * NOVALUE, HALT under `SIGNAL ON ANY`, and USER, in the main program, a routine and a method;
  * inside `INTERPRET`, and from an internal call;
  * CALL ON with a missing label (43.1).
* **Error-report line ends.** All of these match:
  * CR, LF and CRLF inserts in the main program, a REPLY thread and a `::REQUIRES` file, unredirected and through `.TraceOutput~destination`;
  * INTERPRET 13.1 reports;
  * raw CR inside a source literal;
  * a TRACE R clause echo ending in `'0a'x` through a redirected trace output;
  * `rexx -e` with an LF insert, a CRLF INTERPRET error, and a redirected destination (`L 4572726F722039382E3930303A2020610A622E`).

  Four of these differ on the parent.
* **Concern 3 is not reachable from Rexx code.** `display_condition` is called only from `rexx-api/src/ffi.rs` (`DisplayCondition` in the native thread-context table). No C source under `ootest/` calls `DisplayCondition`. On the oracle side, its callers are the `rexx` launchers, `rexxpaws`/`rexxhide` and `testbinaries/provoke_locks.cpp`. The crate's `rexx` launcher path is `lib.rs`, which this task changed and which the main-program probes above cover. So the native API needs a C extension, which is Phase 10. Out of scope.
* **`.context` and library frames.** A user `supplier` method run by `Table~putAll` (CoreClasses.orx) matches on these:
  * `.context~stackframes`, including the `Method PUTALL ... (no source available).` traceline, `executable` and `target`;
  * `.context~name`, `line`, `package` and `executable`;
  * an untrapped 42.3 traceback through the library frame;
  * a trapped one, dumped in full;
  * `putAll(1)`'s 93.948, trapped and untrapped;
  * a USER raise inside the supplier;
  * a NOMETHOD caught by `appendAll`'s own `signal on nomethod`.

  So do `.context` reads in the main program, a routine, a method and INTERPRET. Three of these programs differ on the parent.
* **Tests.**
  * `cargo test --release -p rexx-exec --lib`: exit 0, 1152 passed.
  * `REXX_CORPUS_GATE=1 ... --test corpus --test refusal_sites --test licensed_divergences`: exit 0, `1101 of 1101 matching`.
  * `-p rexx-parse --test sourceline_oracle`: exit 0.
* **Perf reproduces.** `callgrind.sh -r 1 -j 4 -B 0.5 -p "interpret sayloop" pin=... parent=t6.8-r2/rexx-head head=<my build>`, exit 0:
  * `interpret 4207377815 4116396189 4116418620 -2.1624 -2.1619`;
  * `sayloop 179667781 179747954 179745333 +0.0446 +0.0432`.

  Head against parent is +0.0005% and -0.0015%, both within layout noise.

## Issues

### Critical

None.

### Important

1. **After a `REPLY`, a `RAISE ... RETURN value` inside `INTERPRET` does not give 98.936, and with a CALL ON trap in the method it runs the handler instead.** `raise_sender` sets `replied = !in_fragment && ...`, so a fragment raise never sees the reply. The `Some(trap) if trap.call` arm and the `None if replied && result.is_some()` arm both miss it. On the oracle, the interpret activation's `returnFrom` checks the same `reply_issued` flag as its parent. So a CALL ON trap is queued, and then `returnFrom` raises 98.936 before the handler can run. A SIGNAL ON trap unwinds first, and a valueless RETURN is quiet. The crate gets both of those right. Probe `reply_frag.rex`, with method `a` doing `reply 'ra'; call on user u name h; interpret "raise user u description 'ia' return 'x'"`, method `b` the same with `return` and no value, and method `e` using `signal on user u name h`. Run 8 times, every run byte-identical per side:
   * oracle stdout `a ra` / `b rb` / ` b own h ib` / `e re` / ` e own sig h ie` / `done`, and stderr `14 *-* raise user u description 'ia' return 'x'`, `14 *-* interpret "..."`, `Error 98.936:  RETURN cannot return a value after a REPLY.`;
   * HEAD stdout `a ra` / ` a own h ia` / `b rb` / ... and stderr empty;
   * parent stdout without either handler line and stderr empty.

   In `reply.rex`, a method with no trap doing `reply 'rb'; interpret "raise user u description 'ib' return 'x'"` gives the same 98.936 at line 24 on the oracle and nothing on HEAD or on the parent, over 8 runs. This is the handed-over item ("`raise user x return 5` inside a replied method"), reached through INTERPRET. The fix is to compute `replied` for the fragment's own activation as well, and to check it before queueing a CALL ON trap. A non-fragment sender is filtered out, which is already right. Add a witness line for `call on` plus `interpret ... return 'x'` after a REPLY.

2. **A 16.1 from a missing `SIGNAL ON NOTREADY` label, raised inside a `LIBRARY REXX` stream method, builds a corrupted condition object.** When a SIGNAL ON SYNTAX trap takes the 16.1, it inherits the NOTREADY's pending `ADDITIONAL` (the Stream) and its `RESULT`. It also gets the native method frame, PROGRAM `REXX`, and no POSITION. The cause is that `offer_to_trap` hands the 16.1 over at the native level, with the NOTREADY's `pending_additional` and `pending_result` still set. Probe `signal on syntax; signal on notready name gone_lbl; x = linein('/nonexistent/qq')`, with the handler dumping every entry:
   * oracle: `ADDITIONAL Array`, `POSITION = 3`, `PROGRAM = nr_dump.rex`, no `RESULT`, one `sf PROGRAM 3`, one `tb      3 *-* x = linein(...)`;
   * HEAD: `ADDITIONAL Stream`, no POSITION, `PROGRAM = REXX`, `RESULT = `, `sf METHOD The NIL object LINEIN` ahead of the program frame, and `tb        *-* Compiled method "LINEIN" with scope "Stream".` ahead of the program line.

   The same happens for `signal on any name gone_any`, for `chars(...)`, for `.stream~new(...)~linein`, and in a routine and a method (`16.1 The NIL object REXX Stream 1 2` against the oracle's `16.1 3 /nr2.rex Array 0 1`). On the parent the 16.1 went untrapped, so this is the cluster's own new path. Fix: clear `pending_additional` and `pending_result` and raise the 16.1 at the Rexx level that holds the trap, so it gets that level's POSITION, PROGRAM and frames. Add a witness with a NOTREADY from LINEIN.

### Minor

3. **The untrapped report of the same 16.1 carries an extra traceback line.** `call r` with `r: signal on notready name gone_lbl; x = linein('/nonexistent/qq')` makes HEAD print `       *-* Compiled method "LINEIN" with scope "Stream".` before `     4 *-* x = linein(...)`, which the oracle does not. The parent prints the same, so this predates the task, but it is in the `signal-trap-missing-label` cluster. Fixing finding 2 at the Rexx level should remove it too.
4. **Pre-existing, in the clusters' neighbourhood: when a trap handler call fails and SIGNAL ON SYNTAX takes the error, the oracle runs the next clause before the SIGNAL.** Probe `signal on syntax name s; call on user u name queued; call rz` (where `rz` raises USER), then `say 'line4'` and `say 'line5'`:
   * oracle `line4` / `s 40.4 3 4`;
   * HEAD `s 40.4 3 3`.

   A missing handler label behaves the same: `call on user u name say` gives 43.1 on the oracle after `after RESULT` is printed, and HEAD and the parent agree with each other. `call on notready name gone_call` gives the same. Untrapped, both sides report at line 2 and run nothing further. This looks like an oracle defect: a clause runs after a SYNTAX error and SIGL names it. Do not copy it. It becomes more reachable now that builtins get an argument. Queue it as a Deviation candidate for Moritz's batch.
5. **Pre-existing: a non-SYNTAX `RAISE ... RETURN|EXIT expr` has no `RESULT` entry on either side.** `call on user u name mh; x = r()`, where `r` does `raise user u description 'd' return 'first'`, gives `mh hasResult 0 The NIL object` on both. The reference (`condtra.xml`, RESULT: "Any result specified on the RETURN or EXIT options of a RAISE instruction") says it should be there. The cause is `RexxActivation::raise` (`RexxActivation.cpp:1827`), which passes the member `result` and not `resultObj` to `createConditionObject`. That is an oracle bug the crate reproduces, and it predates the task (the parent prints the same). The task added RESULT for the SYNTAX path only, which is correct. Queue it as a Deviation candidate rather than leave it silently copied.
6. **Pre-existing, outside the clusters: clause order of translation errors.** `x = 1 + ;` followed by a later clause holding an invalid character (`'01'x` in source, or LF inside an INTERPRET string): the oracle reports `35.1 ... "+"` at the first clause, and the crate reports `13.1` for the later character. The parent is the same. Queue candidate (`rexx-parse` lexes ahead of parsing).
7. `deliver_pending_traps` keeps a `_code` parameter that its own doc says is unused. Drop it, or keep it only if a caller needs the signature.

## Verdict

CHANGES REQUESTED: 2 Important (REPLY plus INTERPRET 98.936; the 16.1 object for a NOTREADY in a native stream method) and 5 Minor (findings 4 to 6 are queue or Deviation candidates, not blockers).
