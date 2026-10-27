# Task 8.5 re-review, fix round 2

Reviewed `b91dcf0f1..390f7509c` (code `b21493ec6`). HEAD was built from the checkout (only `progress.md`
modified) into `/tmp/claude-1000/t85-rr2-target-head`. BASE `6e0bf5e5b` was built from a `git archive`
(`rust/`, `interpreter/`) into `/tmp/claude-1000/t85-rr2-target-base`. Both used `CARGO_INCREMENTAL=0 memcap 8G
cargo build --release -j 4 --bin rexx` and exited 0. Every probe ran three ways (oracle under the standard
wrapper, base and head under `memcap 2G timeout 20`), each engine in its own fresh `mktemp -d` run directory
under `t85-rereview2-scratch/`. stdout, stderr and rc were compared separately, with the run directory
normalised to `RUNDIR` in stderr. Probe paths below are relative to `t85-rereview2-scratch/`, and the deciding
lines are quoted.

## Verdict

**APPROVED.** Critical none, Important none, Minor 4. Queue candidates 2, new oracle defects 1.

## Verdicts on the fix brief

1. **Important 1 (every required-string position): ADDRESSED.**
   * The report's position table lists every position the re-review measured, and block D's measurement
     names the same list.
   * I ran the unit test's program as a file (`i/unit_positions.rex`). Head prints `cmp 1 1 1`, `meth 2 x-x
     abkmake 0 1 1`, `methn 88.909`, `do ddddd`, `interpreted`, `address 3`, `push KMAKE`, `tail X.kmake`,
     `trace R`, `msg x kmake y`, `nostring a Directory Directory`, `signal kmake`. Each matches the table's
     head column. The oracle column matches too: `cmp 0 0 1`, `methn 88.909`, `address 127`, `push A KMAKE`,
     `tail X.a KMAKE`, `trace A`, `msg an Arrayx a KMAKE y`. Base gives `meth 0 xkmakex aba KMAKE 1 0 0` and
     `methn 97.1`, as the table says.
   * `cargo test --release -p rexx-exec --lib` at head: `1194 passed; 0 failed`.
     `r2/witnesses/unit-tests-base-6e0bf5e5b.txt` shows all three reference tests FAILED at base.
   * The corpus program `lang/variable_reference_forwarded_string.rex` (`h/`): head and oracle agree on all
     descriptors (`arith 6 10 2`, `digits 5`, `NOSTRING nstr NSTR`, `NOSTRING a Directory Directory`,
     `NOSTRING nstr NSTR`, rc 0). Base gives rc 215 with `Error 41.1: Nonnumeric value ("a KNUM")`. The
     round-1 pin binary `rexx-head-074ada1da` gives `NOSTRING a NSTR VariableReference`, so the program is
     also red against fix round 1.
   * The rexxref citations hold. provide.xml:727-731, :739, :740, :743, :745, :746, :751-752, :758,
     :798-800 and :807-810 each hold the quoted text (`sed -n`). instrc.xml around :3511 reads "only the
     first capitalized letter is needed".
2. **Important 2 (block D's cause, NOSTRING object): ADDRESSED.**
   * D now scopes the `stringValue` sentence. It names the positions where the oracle forwards: arithmetic,
     NUMERIC DIGITS, and a comparison's right operand where the value has no string value. It also names
     the strict comparison exception (`"x" == (>d)`). I measured that exception: the oracle prints
     `nostring a Directory VariableReference`.
   * The NOSTRING object and description are stated in D and in the commit. They are the referenced value and
     its `STRING` answer, citing condtra.xml (ADDITIONAL "The object that caused the NOSTRING condition",
     DESCRIPTION "The readable string representation of the object causing the NOSTRING condition", lines
     581 and 633). `a_reference_raises_nostring_only_where_its_value_has_no_string_value` now expects
     `NOSTRING a Directory Directory`.
   * See Minor 2 for one parenthesis in D.
3. **Minor 1 (re-wrap): ADDRESSED.** `awk 'length > 76'` over `local_novalue_hook.rex` finds nothing.
   `cargo test --release -p rexx-parse --test sourceline_oracle` at head: `1 passed`. The report's "its line
   count is unchanged" is false (Minor 1 below).

## What I checked by running (the hunt)

* **Plain objects (no reference) are unchanged** (`a/plain.rex`). Oracle, base and head are byte-identical:
  `untrapped say nstr a Directory`, `untrapped cat nstrx xa Directory`, `untrapped len 4 11`. Under SIGNAL
  ON NOSTRING, CONDITION('C'), ('D') and ('A'), and `condition('O')~description` give `h1 NOSTRING |nstr| NSTR
  1 nstr NSTR`, `h1 NOSTRING |a Directory| Directory 1 a Directory Directory`, and the same in arithmetic (`h2`)
  and in a comparison (`h3`). Under `::OPTIONS NOSTRING SYNTAX` (`e/opt.rex`), `say n` is `s3 98.973 Object
  "nstr" does not have a string representation.` on all three.
* **References, CONDITION('D'), ('A') and ('O')** (`c/refs2.rex`):
  * For `r = >n` and `rr = >r` (a reference to a reference), head gives `h1 r |nstr| NSTR 1 0 0 |nstr| 1`
    in SAY, arithmetic and strict comparison alike. The `|nstr|` field is `condition('O')~description`, and
    the final `1` is `condition('O')~additional` being the object CONDITION('A') answers. Per condtra.xml
    that object is the one causing the condition. In arithmetic the oracle agrees (`h2 r |nstr| NSTR`), and
    in SAY and strict comparison it names the reference (`h1 r |a NSTR| VariableReference`), as D states.
  * `rd = >d` gives `|a Directory| Directory` on head in every position.
* **References to stems.** `rs = >s.` and `rrs = >rs`, with `s.` defaulting to an object with no string value,
  give `|a NSTR| Stem ... 1` on head. That is the same line head and base give for the plain stem `s.`, so a
  reference to a stem stands for the stem, as `conversion_subject` intends.
  * The oracle forwards the plain stem's arithmetic conversion to the default value (`h2 s. |nstr| NSTR`),
    where base and head name the stem. That predates the task (queue candidate 1).
* **CALL ON** (`f/callon.rex`, `g/callany.rex`). `call on nostring` is `Error 25.1` on all three. `call on any`
  does not catch the conversion: head prints `say nstr a Directory`, then `Error 41.1: Nonnumeric value ("a
  Directory")` with rc 215. The oracle and base print the same, except that they print `say a NSTR a
  Directory`.
* **`::OPTIONS NOSTRING SYNTAX` with a reference.** In `e/opt.rex`, head gives `s1 98.973 Object "nstr" ...`
  for `say r`, against the oracle's `"a NSTR"`. In `m/optarith.rex`, head and oracle agree on `s1 98.973
  Object "nstr"` for `1 + r` and for `'a' = r`. This follows the ruling, but no test covers it (Minor 3).
* **A STRING method that reassigns the referenced variable and allocates** (`d/misc.rex`, through a reference
  held in `.local`). Head gives `h nstr NSTR orig` and `hu wiped WIPE`, with no crash. The condition object
  survives the variable's reassignment (`pending_additional` is rooted, `activity.rs:644`, `:704`).
  `r~string` and `rr~string` stay `a NSTR` on all three.
* **The fallback `while` is unwitnessed.** I built a mutant of `b21493ec6` from a `git archive` in its own
  target dir, with `reqstr.rs:492` `while let` made `if let`. It changes `c/refs2.rex`: `h1 rr |a NSTR|
  VariableReference` replaces `h1 rr |nstr| NSTR`, and `rrs` names the reference instead of the Stem.
  * `cargo test --release -p rexx-exec --lib reference` on the mutant: `15 passed; 0 failed`.
  * The full lib run fails 68 tests, all native library, rexxutil file move, `run::tests::routines`
    library rows and scheduler native/pool/callback/lent rows. Those need artifacts the archive build
    lacks. None is a reference test.

  Minor 4.
* **Cyclic references** (`j/self.rex`, `k/cycle.rex`, and a three-line `r = >r; say 'made'; say length(r)`).
  * Head hangs, ignores SIGTERM and runs until SIGKILL (rc 137 under `timeout -k 5 10`). gdb shows the
    interpreter thread in `reference_conversion` (`reqstr.rs:479`, inside the function at :477) under `required_string_arguments`.
  * Base aborts with `thread 'rexx-interp' has overflowed its stack` (rc 134).
  * The oracle prints `made` and is killed at 25 s by `timeout -k 5 20` (rc 137).
  * `do forever; end` on head halts on SIGTERM (`Error 4.1`, rc 124), so the loop is one HALT cannot reach.
  * The pin binary `rexx-head-074ada1da` hangs the same way. So this is fix round 1's `reference_conversion`
    loop, not `b21493ec6`. `conversion_subject` would loop on a cycle too, but it is reached only after
    the classifier answers none, which a cycle never does.

  Queue candidate 2 and new oracle defect 1.
* **Perf.**
  * `r2/final/callgrind.txt` measures `b21493ec6`. `rexx-head-b21493ec6` has mtime 11:01, the commit time
    of `b21493ec6` (11:01:48). Run on the corpus witness, it prints `NOSTRING nstr NSTR` where
    `rexx-head-074ada1da` prints `NOSTRING a NSTR VariableReference`.
  * Every one of the 99 head `.vg` files in `r2final/` names `rexx-head-b21493ec6` on its `Command:` line,
    and every base file names `rexx-base-6e0bf5e5b`.
  * With libc subtracted, the largest rise is fibfunc `+0.0213`, then fibcall `+0.0209`. Both are under
    +0.05%, with no SPREAD or OVER flag.
  * The code change is in `required_string_dispatch`, which is `#[cold] #[inline(never)]` and past the
    point where an answer was found, so it touches no hot path. That matches the unchanged figures.

## Issues

### Critical

None.

### Important

None.

### Minor

1. **The report's Minor 1 paragraph says "its line count is unchanged". It is not.**
   `git show b21493ec6^:rust/corpus/lang/local_novalue_hook.rex | wc -l` gives 71, and the file at head has
   72 lines. The regenerated sourceline file passes, so only the sentence is wrong.
2. **Block D lists "String method arguments (where it raises 88.909)" among the positions where the oracle
   raises NOSTRING "for every reference".** There the oracle raises no NOSTRING, even under SIGNAL ON
   NOSTRING. `l/dsent.rex`, oracle: `SYNTAX x = 'xplainx'~pos(>v) 88.909` and `SYNTAX x =
   'xkmakex'~pos(>v) 88.909`, where `=`, `==`, INTERPRET and PUSH give `NOSTRING ... VariableReference`.
   Fix: move String method arguments out of the NOSTRING clause and say the oracle raises 88.909 there.
3. **The report's "The ruled positions of fix round 1 (...) are unchanged" is false for a value with no string
   value.** In those positions `b21493ec6` changes the answer from the reference's `string` to the value's
   `STRING`. Head prints `untrapped nstr nstr` for `say r rr` (`b/refs.rex`), where 074ada1da's unit test
   expected `a N`. The changed expectation `n nstr| nstr| ...` is in the same commit. Item 2 of the report
   states this, but the sentence in item 1 contradicts it. The `::OPTIONS NOSTRING SYNTAX` message changes
   the same way (`e/opt.rex` head `s1 98.973 Object "nstr"`, oracle `"a NSTR"`), and no test or corpus
   program covers it.
4. **`conversion_subject`'s loop over every reference is unwitnessed.** The mutant `while let` → `if let`
   (`reqstr.rs:492`) passes every reference test (`15 passed`). It changes a reference to a reference
   (`h1 rr |a NSTR| VariableReference` against head's `|nstr| NSTR`) and a reference to a reference to a stem.
   Fix: add `rr = >r` to `a_reference_raises_nostring_only_where_its_value_has_no_string_value`, or to the
   corpus witness in a position the oracle forwards (`1 + rr` gives oracle `NOSTRING nstr NSTR`, `c/refs2.rex`
   `h2 rr`).

## Out of scope

* **Queue candidate 1.** A plain stem whose default has no string value names the stem in an arithmetic
  operand, where the oracle forwards to the default. `c/refs2.rex`, oracle `h2 s. |nstr| NSTR`, base and head
  `h2 s. |a NSTR| Stem 0 0 1`. This predates the task. Proposed owner: stems.
* **Queue candidate 2.** A cyclic variable reference (`r = >r`, or `x = >a; y = >b; a = y; b = x`) hangs any
  string conversion with no HALT check. It needs SIGKILL. Base overflowed the stack instead. The cause is
  `reference_conversion` (`reqstr.rs:477`, from 074ada1da) recursing through `classify_string_conversion`
  as a loop. The oracle fails too (new oracle defect 1), so there is no oracle answer to match. A depth guard
  answering a resource error (Error 11-like) or the reference's own name would end it. Proposed owner:
  variables.

## New oracle defects

1. **A variable reference that refers to itself, directly or through a cycle, hangs the oracle in any string
   conversion and ignores SIGTERM.** Program:

       r = >r
       say 'made'
       say length(r)

   Oracle: prints `made` and is still running at `timeout -k 5 20`'s SIGTERM, so it is SIGKILLed at 25 s (rc
   137, stderr empty). The two-variable cycle (`k/cycle.rex`) does the same after `made`. I did not trace the
   C++ cause. It is presumably `VariableReference::stringValue` (VariableReference.cpp:258-261) asking the
   referenced value, which is the reference itself. A search of `defects.json` for cycle, recursion and
   self-reference finds no entry for it.
