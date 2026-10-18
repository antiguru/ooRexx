# Task 6.6 review: parser and directives

Reviewed `ebeab2d35..61dccebe8`. The crate was built from HEAD into the session scratchpad (`rev66-target`, `cargo build --release --bin rexx --bin rexxc`, exit 0). Every probe ran from its own fresh `mktemp -d`: the oracle under the standard wrapper, the crate under `memcap 2G timeout 20`. Stdout, stderr and rc were compared separately. The probe sources were in `scratchpad/rev66/`, which is not durable, so the deciding lines are quoted below.

## Spec compliance

Compliant. All four clusters are fixed. The three corpus witnesses match the oracle byte for byte. The 22 new `parse-errors.tsv` rows agree with `rexxc -s` on both sides. The cluster's lines left `suite-9d-set.txt` (I did not re-run the suite). The perf figure reproduces. The open items are one gap in the `::CONSTANT` fix's own mechanism (finding 1) and a wrong insert in one label shape (finding 2).

## Strengths

* **Label after THEN/ELSE/OTHERWISE.** I ran 34 probes of my own beyond the implementer's 19, and all but one shape match the oracle in rc, stdout and stderr:
  * a label after THEN with and without `;`, on the same line and on the next;
  * a nested IF in ELSE (`else if 0 then nop` / `else lab: nop`);
  * labels in SELECT, SELECT CASE and WHEN THEN on the next line;
  * OTHERWISE then a label then instructions;
  * labels spelled `then:`, `else:`, `otherwise:` and `when:` after a THEN, and `else then: nop`;
  * `a.b:`, `12:`, `lab:nop` with no blank, a comment before the label, and a continuation before THEN;
  * the IF inside `do 1` and inside `loop 1`;
  * `::ROUTINE` and `::METHOD` bodies;
  * INTERPRET of the ELSE, WHEN and DO shapes, where both sides give `47.1 INTERPRET data must not contain labels; found "LAB".`
* **A label inside a DO in a THEN** (`if 1 = 1 then do` / `lab: say in` / `end`) is 47.2 on both sides, `found "LAB"`. The oracle refuses it too, so "must stay legal" does not hold for the oracle, and the crate matches the oracle.
* **The new TSV rows agree with `rexxc -s`.** I decoded each of the 22 rows and ran it through both `rexxc` binaries as `rexxc FILE -s`. rc, `Error N.M` line and line number are identical on all 22 (e.g. `R06 SAME o_rc=209 c_rc=209 line 3 | Error 47.3: ... found "THEN".`, `R17 SAME o_rc=157 c_rc=157 line 9 | Error 99.905`, `R20 SAME o_rc=0 c_rc=0`).
* **99.905 matches across 20 probes.** These covered `::method m class` with a body, `::attribute a class get|set|bare`, ABSTRACT, ATTRIBUTE, DELEGATE, EXTERNAL "LIBRARY", PRIVATE, after a `::ROUTINE`, after a `::REQUIRES`, a duplicate with CLASS, a `::method m class` ahead of a later `::class`, INTERPRET of a directive, and `::class c` / `::routine r` / `::method m class` (accepted, rc 0, since `activeClass` survives a `::ROUTINE` in `DirectiveParser`). `::constant k class 1` is 35.x (rc 235) on both sides.
* **Superclass search order matches across 29 probes.** The targets were `input`, `output`, `stdin`, `error`, `rexxinfo`, `true`, `nil`, `environment`, `endofline` and `"INPUT"` (all 99.949). Also matching:
  * INHERIT, METACLASS and MIXINCLASS INPUT;
  * a class defined later in the file;
  * a public and a non-public class from `::REQUIRES` (Array and 98.909);
  * `.environment~zz = .string` (String) and `.local~zz2 = 5` (99.949);
  * `.local~array = 5`, which leaves Array unshadowed;
  * `.local` over `.environment` (Array over List);
  * a local `::class zz5` over `.local~zz5`, and a required public class over `.local~zz7 = 5`;
  * `rexx:array`, `stem`, `rexxqueue` and `systemqueue` (98.909 on both).
* **`::CONSTANT` under `::OPTIONS` matches across 21 probes** for a single package: DIGITS, FUZZ (`1.00001 = 1.00002` under FUZZ 2), FORM ENGINEERING and SCIENTIFIC, `digits() fuzz() form()`, LOSTDIGITS SYNTAX (98.972), NOVALUE ERROR/SYNTAX (98.986), NOVALUE/ALL CONDITION (`ZZ`), ALL SYNTAX on `1/0` (42.3), NOTREADY SYNTAX (98.974), `//` under DIGITS 5 (26.12), the last of two `::OPTIONS` winning, `::OPTIONS DIGITS 4 NOVALUE SYNTAX` inside a required file, and a requiring file's options not leaking into a required file's constant.
* **The perf workaround is sound.** `environment_model()` builds native objects only (no Rexx code, no lookup, no method dispatch). The parent `directive_class` called `directory_lookup` unconditionally at this same point, and that builds the same model, so the early call restores the parent's allocation order. It does not add one. The experiment record is consistent: the libc-subtracted delta is about 0.029% in every arm, and only the whole-process figure moves. ctl to exp2 is +0.0209 to +0.0824, the 0.06% the code comment states. My spot check (`callgrind.sh -r 1 -B 0.5 -p interpret`, my build vs the pin) gives `interpret 4207371649 4116479002 -2.1603 ... -libc -4.6311`, exit 0. The implementer's is -2.1475 / -4.6312, so the head counts agree within 0.004%.
* **The moved witness lost no coverage.** `every_corpus_lang_program_parses` forbids a translation error as a corpus program, so a TSV row is the project's form for it. Row 1217 holds the file's bytes and matches `rexxc` at line 9. The runtime stderr of 99.905 matches the oracle byte for byte (probes D01-D20). `coverage.rs` and `phase-5a.txt` lost only that entry.
* **The `directive_options` claim holds.** Built at BASE in a scratch `git worktree` with its own target dir: `test result: FAILED. 23 passed; 1 failed`, with `signal-on-trap: the two sides now agree, so this row's divergence closed and the row belongs deleted / left: ("trapped\nend\n", 0) / right: ("trapped\nend\n", 0)`. HEAD gives the identical panic. The worktree and its target dir were removed afterwards.
* **Tests.** `cargo test --release -p rexx-parse` exits 0, with every binary ok (`errors` 35, `program` 28, `sourceline_oracle` 1, unit tests 260). `rexx-exec --test coverage --test refusal_sites --test rexxc` pass. `directive_options` fails only as above.

## Issues

### Critical

None.

### Important

1. **A `::CONSTANT` expression ignores `::OPTIONS NUMERIC INHERIT`.** `eval_constant_expression` calls `self.package_start(id, None)`, and with no caller, `package_start` answers `options.numeric` even when `numeric_inherit` is set. The oracle's constant inherits from the activation that installs the package:
   * `C22.rex`: `say .c~k` / `::requires "C22r.cls"` / `::options digits 4`, where `C22r.cls` is `::options numeric inherit` / `::class c public` / `::constant k (1/3)`. The oracle prints `0.3333` and the crate `0.333333333`.
   * `C23.rex`: `numeric digits 4` / `call "C23c.rex"`, where `C23c.rex` is `say .c~k digits()` / `::options numeric inherit` / `::class c` / `::constant k (1/3)`. The oracle prints `0.3333 4` and the crate `0.333333333 4`. The same holds through INTERPRET (`C25`).
   * The controls agree: an inheriting required file under a main file with no options gives `0.333333333` on both (`C27`), and so does a call into a file with no options that requires an inheriting file (`C28`).

   The parent binary prints `0.333333333` for C22 and C23 too, so this is not a regression. But it is the cluster's own cause: the constant does not start from the package's `::OPTIONS` numeric settings. The comment at the call states that it does. The fix is to pass the installing activation's settings as `caller`, as `run_loaded` already does for the body. A witness should use C22's shape. If the controller would rather queue it, the comment should name the INHERIT exception until then.

### Minor

2. **A source clause that opens with a label names the wrong token.** Example: `a: if 1 = 1 then b: nop`. `ClauseCursor::origin` searches the label-split clause list, so the clause it finds starts at IF. The oracle's `resetPosition(1)` reads the source clause, whose first token is the leading label:
   * `a: if 1 = 1 then b: nop`: the oracle gives `47.3 ... found "A".` and the crate `found "IF".`
   * `a: b: if 1 = 1 then c: nop`: `"A"` vs `"IF"`.
   * `nop` / `a: ,` / `if 1 = 1 then b: nop`: `"A"` vs `"IF"`. The line matches.
   * After a SELECT that has ended, `a: if 1 then b: nop`: `"A"` vs `"IF"`.
   * `'x': if 1 then b: nop`: the oracle gives `found "x".` and the crate `found "IF".`

   rc, line and echo match in every case. Controls that already match: `a: if ... then nop; else b: nop` (`"ELSE"`), `a: select; when 1 then b: nop; end` and `select; a: when ...`. The fix is for `origin` to step back over the label clauses that share the found clause's source clause, for example by keeping the pre-label start in `Clause`. The `misplaced_label` doc comment ("That token is the label itself unless a THEN, an ELSE or an OTHERWISE was trimmed off the clause's front, where it is the IF ...") should name the leading-label case too.
3. **The `phase-5a.txt` comment edit left a 103-column line** (line 541, `# it is written twice and the oracle runs all of it, so a rule phrased over the directive's name rather`). The base file has no line over 80 columns (`awk 'length>80'` gives 0 on `ebeab2d35`). Re-wrap that paragraph only.
4. **The implementer's Concerns stand as reported.** Each is outside the cluster tests:
   * the duplicate-member check still runs at install (`directive_10`/`_11`);
   * `::OPTIONS TRACE` on a constant;
   * a called program's install error names the caller's file. I reproduced this: `S07`/`S21` give `Error 99 running .../S07.rex line 2` where the oracle names `S07c.rex`.

   Each should go to the queued inventory if it is not there already.

## Assessment

**Task quality:** Needs a small fix.

**Reasoning:** The parser fixes match the oracle on every shape I could construct except the leading-label insert (finding 2). The class-search order and the 99.905 placement match exactly. The perf workaround restores the parent's allocation order and changes no behaviour. But the constant-expression fix leaves NUMERIC INHERIT out, and its comment claims otherwise (finding 1). The fix is one argument and one witness.

**Verdict: CHANGES REQUESTED** (finding 1. Findings 2 and 3 are cheap to take in the same round.)
