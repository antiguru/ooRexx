# Task 6.6 re-review: fix round 1

Scope: `41e8051dd..1c5f27e68`, read as the code diff (`install.rs`, `lib.rs`, `directive_options.rs`,
`block.rs`, `clause.rs`) plus the corpus and TSV changes. I built HEAD (`rexx`, `rexxc`) into
`scratchpad/rerev66-target`. I built the parent `41e8051dd` (`rexx`) from a detached scratch worktree
into its own target dir, to tell a regression from an old divergence. Every probe ran each side in its
own fresh `mktemp -d` copy, and stdout, stderr and rc were compared separately (paths replaced by
`DIR`). The probe trees were in `scratchpad/rr66probes.l3td/`, which is not durable, so the deciding
sources and lines are quoted below.

## Findings from the review

1. **NUMERIC INHERIT constant: closed.** `f1a` (required file), `f1b` (CALL) and `f1c` (INTERPRET of
   the CALL) match the oracle: `0.3333`, `0.3333 4`, `0.3333 4`.
2. **Leading label: closed.** `a: if 1 = 1 then b: nop` gives `found "A"` on both sides, and so do
   the five new TSV rows. Twelve more shapes of my own match the oracle in rc, stdout and stderr:
   * `a:; if ...` gives `"IF"`, `a:` then a newline then `if ...` gives `"IF"`, and
     `a: b:; if ... c: nop` gives `"IF"`. So the adjacency test stops at a clause end;
   * `a: /* c */ if ...` gives `"A"`, `a: ;; b: if ...` gives `"B"`, and `a:,` then a newline then
     `if ...` gives `"A"` at line 2;
   * `say 1; a: if ...` gives `"A"`, and `a: if 1 then if 1 then b: nop` gives `"A"`;
   * the controls `a: if ... then nop; else b: nop` (`"ELSE"`), `a: select; when ...` (`"WHEN"`),
     `a: if 1 = 1 then` then `b: nop` on the next line (`"B"`, line 2), and `a: do 1; if 1 then b: nop; end`
     (`"IF"`).
3. **`phase-5a.txt`: closed.** `awk 'length>80'` gives 0 lines.
4. **`directive_options`: closed.** `cargo test --release -p rexx-exec --test directive_options
   --test rexxc` exits 0, with `24 passed` and `4 passed`.

## The new TSV rows

I decoded rows 1222-1226 and ran each through both `rexxc FILE -s`. All five are identical in full
output and rc. For example, `R3 SAME expect=47.3 line 3 | 3 *-* if 1 = 1 then b: nop | ... line 3 ...
found "A". | rc=209`, and `R5 SAME ... found "WHEN". | rc=209`.

## The wider change: what matches

These all match the oracle on HEAD:
* **Prologue under the requirer's settings.** The main file has `::options digits 4 fuzz 2 form
  engineering`. The required file:
  * with no `::OPTIONS` gives `prolog 9 0 SCIENTIFIC` (p1);
  * with `numeric inherit` gives `prolog 4 2 ENGINEERING` (p2, which the parent got wrong);
  * with `digits 6` gives `prolog 6 0 SCIENTIFIC` (p3);
  * with `numeric inherit digits 7` gives `prolog 4 2 ENGINEERING` (p4);
  * under a requirer with no options gives `9` (p5).
* **A called file that requires an inheriting file.** It gives 6 under a caller's `numeric digits 6`
  when the callee inherits (p6), and 9 when it does not (p7).
* **Cached packages.** A file required by `a.cls` (DIGITS 5) and then by `b.cls` (DIGITS 6) runs its
  prologue once, at 5. Its constant is `0.33333` (q1). In q2, `call "c.rex"` under 4, then under 6,
  then `.package~new` give `prolog 4` / `callee 4` / `callee 6` / `0.3333`. So the second requirer's
  settings do not leak, as on the oracle.
* **Nested requires** (n1): main DIGITS 4, `a` DIGITS 5, then `b` inherit, then `c` inherit give `c 5`,
  `b 5`, `a 5`.
* **External CALL starts at defaults.** e1 calls under `numeric digits 4 / fuzz 1 / form
  engineering`, and the callee prints `9 0 SCIENTIFIC`. A caller's `::options digits 4` does not reach
  it (e2). A function call (e3) prints 9, and a callee's non-inheriting requirement prints 9 (e4).
* **INTERPRET** (i1).
* **`.Package~new(name)` and `loadPackage`.** l1 and l2 match, and so does l4, where a loaded file
  with no options that requires an inheriting file gives `z 9`, and one with DIGITS 7 gives `z 7`.
* **Condition and TRACE settings do not leak.** `signal on novalue` in the caller does not trap in a
  callee (t1) or in a prologue (t2), and neither does a requirer's `::options novalue syntax` (t3).
  `trace r` does not reach a callee (t4), and neither does a requirer's `::options trace r` (t5). An
  inheriting constant under a caller's `signal on novalue` gives `0.3333` and does not trap (t7).
* **`.Routine~newFile` and `.Routine~new` of an inheriting package** (r1, r3) now give `prolog 4`
  as on the oracle. The parent gave 9.

## New findings

### Important

1. **A run-time load that is not an install drops the requiring package's own `::OPTIONS`.** This
   covers `.Routine~newFile`, `.Method~newFile`, `.Routine~new(src)` and `.Method~new(src)`. In
   `find_requires`, `(Some(from), None)` falls to the `_` arm and hands the required file the
   running activation's settings. That arm ignores `from`. On the oracle, the prologue starts from
   `from`'s own start settings, which inherit from the running activation only if `from` itself says
   `NUMERIC INHERIT`. The fix is to answer `self.start_settings(from, running.as_ref())` for that
   pair. Then r1, r3 and these all match.
   * **Regressions (the parent matched the oracle).** In each case the required file `q.cls` is
     `say "prolog" digits()` / `::options numeric inherit`, plus a `::constant` where shown, and
     the main file runs `numeric digits 4` first:
     * r8: `.routine~newfile("r.rex")`, where `r.rex` is `say "rtn" digits()` / `::requires
       "q.cls"`. The oracle and the parent print `prolog 9`, and HEAD `prolog 4`.
     * r2: the same, but `q.cls` has `::constant k (1/3)` and `r.rex` prints `.c~k`. The oracle and
       the parent print `prolog 9` / `rtn 9 0.333333333`. HEAD prints `prolog 4` / `rtn 9 0.3333`.
     * r7: `.method~newfile("m.rex")` requiring `q.cls`. The oracle and the parent print `prolog 9`,
       and HEAD `prolog 4`.
     * r4: `.method~new("m", .array~of("say 'meth' digits() .k~v", "::requires 'q.cls'"))`. The
       oracle and the parent print `prolog 9`, and HEAD `prolog 4`.
   * **Still divergent, differently.** When the requiring package has `::options digits 6`:
     * r5 (`.routine~new` with `"::options digits 6"`): the oracle prints `prolog 6` / `r 6
       0.666667`. HEAD prints `prolog 4` / `r 6 0.6667`, and the parent `prolog 9` / `r 6
       0.666666667`.
     * r6 (the same through `.routine~newfile`): the same three answers.

   A witness should hold r2's shape (constant and prologue) and r5's.

### Minor (old, to queue)

2. **`.Package~new(name, source)` of an inheriting package ignores the running settings.** This
   predates the round: the parent and HEAD agree. l3 runs `numeric digits 4`, then `.package~new("q.rex",
   .array~of("say 'src' digits()", "::requires 'z.cls'", "::options numeric inherit"))`, where z.cls
   is inheriting. The oracle prints `z 4` / `src 4`, and the parent and HEAD print `z 9` / `src 9`.
   The non-inheriting control `q2.rex` gives `z 9` on all three. This round extended run-time
   inheritance to `.Package~new(name)` but not to its source form.
3. **An error in a required file's `::CONSTANT` expression names the requiring file.** This predates
   the round and is not tied to INHERIT. t6b: main is `say .c~k` / `::requires "r.cls"` / `::options
   novalue syntax digits 4`, and `r.cls` is `::class c public` / `::constant k (1/3 + nv)`. The
   traceback lines and rc 215 match. The oracle says `Error 41 running DIR/r.cls line 2`, and both
   crate builds say `DIR/main.rex line 2`. The same holds through `.routine~newfile` (t8) and with
   inheriting variants (t6, t6c). This is close to the reviewed Concern 4 (a called program's
   install error names the caller's file) and may share its cause. Queue it with that one.

## Verdict

**CHANGES REQUESTED.** The four findings are closed and the new TSV rows agree with `rexxc -s`. But
the widened `find_requires` regresses `.Routine~newFile`, `.Method~newFile` and `.Method~new` with a
`::REQUIRES` of an inheriting file (new finding 1). The parent matched the oracle on those shapes.
The fix is one match arm and a witness. Findings 2 and 3 are old and go to the queue.
