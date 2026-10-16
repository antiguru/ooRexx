# Phase 9c Task 3 re-review (fix round 1)

Range `git diff b17078946 d5cb987d1`, code `6ada7047e`. Checks ran in a detached worktree of
`d5cb987d1` with its own release build.

**Verdict: approved. No open findings.**

1. **ARITY_OVERRIDES.** Three entries: 20.922, 27.1, 89.1. Each cited site opened in
   `InstructionParser.cpp`: `:1122` (`syntaxError(Error_Symbol_expected_qualified_call, token)`),
   `:4616` and `:4631` (`requiredEndOfClause(Error_Invalid_do_whileuntil)`, the WHILE and UNTIL arms),
   `:4393` (`syntaxError(Error_Variable_expected_USE, token)`). The inventory tests pass (8 of 8). With
   a bogus override `((10, 2), 2)` added (text has 3 placeholders), both
   `every_arity_override_exceeds_its_texts_placeholders` and
   `every_constants_arity_is_its_texts_highest_placeholder` fail.
2. **Witness.** `lang/parse_error_unused_insert.rex` on the oracle and on the release `rexx-run`
   from the same directory: stdout, stderr and rc identical (rc 0). It is listed in
   `corpus/phase-9.txt` (file `phase-9.txt` is in the corpus.rs list); its `sourceline_oracle` file
   exists and its body equals the `.rex` after the count line. `REXX_CORPUS_GATE=1` release
   corpus test: `1063 of 1063 matching`, 29 passed. Mutations: the type system rejects reverting a
   site to `error(.., [])`, so each site was changed to an empty insert (`[Vec::new()]`). Four mutants
   (89.1; 20.922 non-symbol; 20.922 nothing follows; 27.1) each differ from the oracle on exactly the
   witness line for that site and no other. Restored; worktree clean.
3. **Further shapes, oracle vs crate, identical stdout/stderr/rc.** 39 lines under trap: `use arg (a), b`,
   `a, (b)`, `strict arg (a)`, `a~b(1)`; `call ns:(x)`, `ns:"x" y`, `ns: ;`, `ns:` plus blanks, `ns: 5`
   (98.987, no 20.922), `ns:+`; 27.1 after `do i = 1 to 3 while 1 until 2`, `while..while`,
   `until..while`, `until..until`, `do 3 until..until`, `forever`, `over`, `with item` loops; 34.3 and
   35.929 controls. A second batch: `use arg 5` (31.2), `"s"` (89.1 `[s]`), `a, 5`, `~a` (35.1),
   `a b` (46.902). Untrapped rc and stderr match for all three fixed shapes.
4. **Minors.** `loops.rs` doc now names the `FOR` expression and "at `WHILE` or `UNTIL`". Added lines
   have no em-dash, ` -- ` or set sizes; `cargo fmt --all --check` exits 0.

Worktree and target dir removed.
