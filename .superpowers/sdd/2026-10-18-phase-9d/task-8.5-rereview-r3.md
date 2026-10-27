# Task 8.5 re-review, fix round 3

Reviewed `2c450f98c..b43c6c6e8` (code `0136aeb82`, `ffadc7da7`, `4a6321376`). HEAD was built from the
checkout (only `progress.md` modified) with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/claude-1000/t85-rr3-target
memcap 8G cargo build --release -j 4 --bin rexx`, exit 0. Its `.text` section is byte-identical to the
measured `cg-9d/t8.5/rexx-head-4a6321376` (`objcopy --only-section=.text`, md5 `aa60cb20...` both). Base
answers come from the pinned `cg-9d/t8.5/rexx-base-6e0bf5e5b` and the round-2 binary `rexx-head-b21493ec6`.
Two mutants were built from `git archive 4a6321376 rust interpreter`, each in its own target dir. Every crate
run is `memcap 2G timeout -k 5 20` in a fresh `mktemp -d` under `t85-rereview3-scratch/`; the oracle used the
standard wrapper, never on entries 59 and 60. stdout, stderr and rc are compared separately. Probe paths are
relative to `t85-rereview3-scratch/`.

## Verdict

**CHANGES REQUESTED.** Critical none, Important 1, Minor 1. New oracle defects 1 (a further shape of
`cyclic-variable-reference-hangs`).

## Verdicts on the fix brief

1. **A cyclic variable reference terminates loudly: PARTIAL.**
   * Every reference-only cycle I built ends in 11.1 and is trappable. `p/pos.rex N` under SIGNAL ON SYNTAX
     on `r = >r`: `SYNTAX N SYNTAX 11.11.1` for `r~request('STRING')`, `-r`, `r * 2`, `r == 'a'`,
     `'a' == r`, `'a' < r`, `r~makestring`, `numeric digits (r)`, `interpret r`, `x.r = 1`, `datatype(r)`,
     `do i = 1 to r`, `\r`, `if r`, `address value r`, `push r`, `'x'~pos(r)`, `(r)~length`, `r~foo`,
     `signal value r`, `.string~new(r)`, `s.r`, `r~equals(r)` and `r = r`. EXPOSE (`procedure expose r`,
     then `say r`) and USE ARG (`use arg >q` bound to `>r`, then `say q`, `q/usearg.rex`) give 11.1 at the
     conversion, not at the binding. DROP, `.context~variables` (`vars 2|k WHICH|k R`), `trace r` (`>>> "R"`),
     `r~name` (`R`), `r~value~class`, `r~string` (`R`), `r~class~id`, `.array~of(r)~items`, `r~hashcode`,
     `r~objectname`, a Table index and `r~isa(.string)` end normally.
   * A three-variable cycle, and a chain leading into a self-cycle (`p = >q; q = >w; w = >w`), give 11.1 in
     SAY, `length` and `+` (`q/c6.rex`: `e1 11.1`, `e2 11.1`, `e3 11.1`). An instance variable `v = >v` read
     in another method gives `SYNTAX 11.1` (`q/inst.rex`).
   * A cycle formed later by assignment is found (`p/later.rex`: `before x y`, `mid y`, then `SYNTAX 11 11.1`).
   * A long acyclic chain works: `p/chain.rex 1000` gives `len 4 leaf 1`, `arith 6`, `ok` on head and oracle;
     50000 links on head give the same in 1.65 s.
   * HALT interrupts a loop that raises and traps the cycle's 11.1 (`p/halt.rex` under `timeout -s TERM 3`:
     `HALT HALT after 1`, rc 124).
   * No growth: `p/leak.rex` (a trapped 11.1 in a subroutine, for each cycle shape) peaks at 35972 KB for
     20000 iterations and 36588 KB for 200000.
   * A condition object carrying a cyclic reference ends: untrapped `raise syntax 40.1 array(r)` gives
     `Error 40.1:  External routine "R" failed.` rc 216 (`q/c3.rex`); `o~message` reads `R.`, and `say
     o~additional[1]` is 11.1 (`q/c4.rex`).
   * `cargo test --release -p rexx-exec --lib -- reference cyclic` at head: `17 passed; 0 failed`. With
     the cycle check disabled (`if false && slow.is_some() && slow == fast`, `value.rs:1156`), the cyclic
     test fails rather than hangs: `"r = >r\n" then "say r" did not end`, `test result: FAILED`, 20.00 s,
     exit 101.
   * Error choice: errnums.xml:289-299 holds Error 11 "Control stack full", naming "a recursive subroutine that
     does not end correctly". The citation holds.
   * **A cycle through a stem's default does not terminate.** See Important 1.
2. **Minor 4, the chain chase witnessed: ADDRESSED.** I built my own one-hop mutant (`referent_from` answers
   `first` when `first` is itself a reference, cycle detection kept). On the unit test's program
   (`q/unit_n.rex`) head prints `nstr` and `NOSTRING a Directory Directory`, the mutant `a N` and `NOSTRING a
   Directory VariableReference`. On `corpus/lang/variable_reference_forwarded_string.rex` the last line is
   `NOSTRING nstr NSTR` on head and oracle and `NOSTRING a NSTR VariableReference` on the mutant. Both
   witnesses are red against it, matching `r3/mutant-one-hop-*.txt`.
3. **Minor 3: ADDRESSED.** The sentence now says what changed (report line 361-364).
   `r3/probes/nostring_syntax.rex`: head `Error 98.973:  Object "nstr" does not have a string representation.`
   rc 158, oracle `Object "a N"` rc 158, base `a N` rc 0. Block D states the position inside the ruled
   Deviation candidate's ORACLE BUG paragraph (report line 401-402), and names the witness
   `tests::a_references_nostring_syntax_names_its_values_string`, which passes.
4. **Minors 1 and 2: ADDRESSED.** `git show aeb913cff:rust/corpus/lang/local_novalue_hook.rex | wc -l` is 71,
   the file at head 72, as the sentence now says. Block D no longer lists String method arguments among the
   NOSTRING positions and says the oracle raises 88.909 there (line 403-404).

**Perf: confirmed.** `r3final3/` holds 99 head and 99 base `.vg` files; every head `Command:` line names
`rexx-head-4a6321376` and every base line `rexx-base-6e0bf5e5b`, over 33 programs (the 29 in `bench-programs/`,
`rexxcps` and the three ping programs). That binary's `.text` matches my HEAD build. In
`r3/final/callgrind.txt`, libc subtracted, the largest rise is decrender `+0.0386`, then fibfunc `+0.0213` and
fibcall `+0.0209`; all are under +0.05%, and the spread columns are at most `0.0024`.

## Issues

### Critical

None.

### Important

1. **A cyclic reference through a stem's default still hangs every conversion and ignores SIGTERM.** A stem
   whose default is a reference to that stem (directly, or through a variable holding the stem) is a chain
   that never ends at a value, but `referent_from` (`value.rs:1138`) stops at the stem, a `Value`, and the
   classifier's stem arm (`dispatch/reqstr.rs:423-426`) follows the default back into
   `reference_conversion` (`reqstr.rs:483`). The two calls are in tail position, so the release build loops
   without a HALT check, exactly as round 2's `r = >r` did.
   * `q/sd1.rex` (`s. = >s.`, then `say s.`): head spins. Run under `timeout 20` without `-k` it was still at
     state `Sl` with 2:10 of CPU time in 2:10 when I SIGKILLed it; gdb shows `referent_from` (`value.rs:1147`)
     under `reference_conversion` (`reqstr.rs:484`), `required_string_answer` (`:342`), `say_evaluated`.
   * The same on head, rc 137 under `timeout -k 5 20` with nothing on stdout: `p/stemdef.rex 1` (`say s.`),
     `2` (`r = >s.; s. = r; say length(s.x)`), `4` (`say 1 + s.`), and `q/sd_via.rex` (`t = >u; s. = t;
     u = s.; say length(u)`). `rexx-head-b21493ec6` spins on `sd1.rex` the same way.
   * `trace r; x = s.` (`q/sd_tr.rex`) and `s.~foo` (`q/sd_msg.rex`) abort with `thread 'rexx-interp' has
     overflowed its stack` (rc 134) on head.
   * Base `6e0bf5e5b` aborts with the stack overflow on `sd1.rex`, `sd_tr.rex`, `sd_msg.rex` and
     `sd_via.rex`. So the SIGTERM-ignoring spin is this task's, from the conversion loop it introduced, and
     the round's fix covers only chains made of references.
   * `s. = >s.` alone is fine on both sides (`q/sd0.rex`, head and oracle `made`, `Stem`).
   * Fix: the chain walk takes a stem's default as the next link, so a cycle through any mix of references
     and stem defaults is `Referent::Cycle` and 11.1 everywhere `r = >r` is, including the renderings and
     forwarded messages; add `s. = >s.` to `a_cyclic_reference_is_insufficient_stack_in_every_conversion` and
     to `oracle-crashes.txt` (the oracle hangs on it, below).

### Minor

1. **The new `Referent` enum took `Redirect`'s doc block.** `value.rs:1009-1013` reads "What a value's text
   has to be read from somewhere other than the object holding it." then "Where a chain of variable
   references ends", both above `pub(crate) enum Referent`, and `enum Redirect` (`value.rs:1024`) is left with
   no doc. At `2c450f98c` the first sentence sat directly above `enum Redirect` (line 1037-1040). Fix: move
   the two lines back above `#[derive(Copy, Clone)] enum Redirect`.

## Out of scope

* Queue candidate 1 (a plain stem whose default has no string value in an arithmetic operand) stays queued, as
  the brief says.

## New oracle defects

1. **A stem whose default is a reference to that stem hangs the oracle in a string conversion and ignores
   SIGTERM**, a further shape of `cyclic-variable-reference-hangs` (defects.json:1863). `q/sd1.rex`:

       s. = >s.
       call lineout "made.txt", "made"
       say s.
       say "end"

   Oracle: killed at 25 s by `timeout -k 5 20` (rc 137, stdout and stderr empty). `s. = >s.` followed by
   `say "made"; say s.~class~id` without a conversion prints `made` and `Stem`, rc 0 (`q/sd0.rex`), so the
   hang is in the conversion. I ran the oracle on `sd1.rex` once; it should be listed in `oracle-crashes.txt`
   before anyone runs it again.
