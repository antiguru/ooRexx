### Task 8: Expression evaluation, part two — comparison and logic

**Spec:** "Expression evaluation", the comparison and logical groups, with their transcripts.

**Files:**
- Modify: `rust/crates/rexx-exec/src/eval.rs`
- Test: a `#[cfg(test)] mod tests` inside `rust/crates/rexx-exec/src/eval.rs`

- [ ] **Step 1: Write the failing tests, from the measured line**

```
' a' = 'a'  -> 1     '09'x'a' = 'a' -> 1     'a' = 'a'||'09'x -> 1
'a' = 'a '  -> 1     'a b' = 'a  b' -> 0     '' = ' '         -> 1
'01' = '1'  -> 1     ' 1 ' = 1      -> 1     'a' = 1          -> 0
'10' >> '9' -> 0     '10' > '9'     -> 1     'a' << 'a '      -> 1     '01' == '1' -> 0
```

The first row is the one that matters. An earlier draft of this plan said the string rule was "blank-pad the shorter on the right", which is wrong, and no test in the second and third rows can tell the two rules apart.

- [ ] **Step 2: Run to watch them fail**

- [ ] **Step 3: Implement all four families**

* Numeric-or-string `= \= <> >< > < >= <= \> \<`: **call `rexx-num`'s comparison and do not write a string comparison at all.**

**Step 3a comes first: amend `rexx-num` with a byte-slice entry point.** Today `compare` takes `&str` and re-parses both operands on every call. A Rexx string can hold bytes that are not valid UTF-8 (D14), so `&str` cannot carry one, and re-parsing defeats D15's cache, whose whole stated purpose is that a non-numeric string is not re-parsed on every comparison. Add an entry taking byte slices and already-decoded operands, keep the existing one, and do not duplicate `string_order`. It is the whole algorithm, string fallback included — `string_order` at `rust/crates/rexx-num/src/compare.rs:173`, ported from `RexxString::stringComp` (`StringClass.cpp:795`), which strips leading blanks *and tabs*, compares the shared prefix, and decides a leftover tail against a space. Writing a second one here means writing a divergent one.
* Strict `== \== >> << >>= <<= \>> \<<`: no padding, shorter is less.
* Logical `& | &&`: a logical value is **exactly** the one-character string `0` or `1`. Measured, `' 1 '`, `'01'`, `'1.0'` and `''` are each error 34.
* `ExprKind::Logical`, the comma list, is an AND of its parts under the same check.

- [ ] **Step 4: Verify** — every line above re-run under the oracle and pasted into the report.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/eval.rs rust/crates/rexx-exec/tests/eval_compare.rs
git commit -m "Comparison in two families, and logic that coerces nothing"
```

---

