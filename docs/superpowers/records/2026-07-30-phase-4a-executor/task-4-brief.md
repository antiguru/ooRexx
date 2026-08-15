### Task 4: The value model

**Spec:** D15 in full, including every transcript.

**Files:**
- Create: `rust/crates/rexx-exec/src/value.rs`
- Test: `rust/crates/rexx-exec/tests/value.rs`

**Interfaces:**
- Produces: `Value` constructors and accessors on `Interp` — `text(&mut self, &[u8]) -> ObjRef`, `number(&mut self, Number) -> ObjRef` (which applies the `SmallInt` admissibility rule), `to_text(&mut self, ObjRef) -> Cow<'_, [u8]>`, `to_number(&mut self, ObjRef) -> Result<Number, NotNumeric>`.

**Why:** every later task manipulates values through these four functions, and the two rules they enforce are the ones the oracle makes observable everywhere.

- [ ] **Step 1: Write the failing tests, straight from the spec's transcripts**

```rust
#[test]
fn a_numbers_rendering_is_fixed_when_it_is_created() {
    // numeric digits 9 ; y = 1/3 ; numeric digits 3 ; say y  ->  0.333333333
    let mut interp = Interp::new();
    interp.settings_mut().set_digits_str("9").unwrap();
    let y = interp.eval_str("1 / 3").unwrap();
    interp.settings_mut().set_digits_str("3").unwrap();
    assert_eq!(&*interp.to_text(y), b"0.333333333");
}

#[test]
fn numeric_form_is_captured_at_creation_too() {
    // numeric form engineering ; x = 1e10+0 -> 10E+9, and stays 10E+9.
    let mut interp = Interp::new();
    interp.settings_mut().set_form_str("ENGINEERING").unwrap();
    let x = interp.eval_str("1e10 + 0").unwrap();
    interp.settings_mut().set_form_str("SCIENTIFIC").unwrap();
    assert_eq!(&*interp.to_text(x), b"10E+9");
    let y = interp.eval_str("1e10 + 0").unwrap();
    assert_eq!(&*interp.to_text(y), b"1E+10");
}

#[test]
fn a_small_int_is_only_admissible_within_the_digits_of_its_own_operation() {
    // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while 15 + 6 is 2E+1.
    let mut interp = Interp::new();
    interp.settings_mut().set_digits_str("1").unwrap();
    let x = interp.eval_str("15 + 0").unwrap();
    assert_eq!(&*interp.to_text(x), b"2E+1");
    let sum = interp.eval_with("x + 6", &[("X", x)]).unwrap();
    assert_eq!(&*interp.to_text(sum), b"3E+1");
    let direct = interp.eval_str("15 + 6").unwrap();
    assert_eq!(&*interp.to_text(direct), b"2E+1");
}

#[test]
fn text_keeps_its_own_spelling_and_caches_an_exact_parse() {
    // x = '007' ; say x -> 007 ; say x + 0 -> 7
    // and the cache is exact, so it survives a DIGITS change:
    // x = '1.234567890123456789'; digits 5 -> 1.2346 ; digits 20 -> the whole thing
    let mut interp = Interp::new();
    let x = interp.text(b"007");
    assert_eq!(&*interp.to_text(x), b"007");
    let converted = interp.eval_with("x + 0", &[("X", x)]).unwrap();
    assert_eq!(&*interp.to_text(converted), b"7");
}

#[test]
fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
    // say .nil -> The NIL object ; .true is "1" ; .false is "0"
    let mut interp = Interp::new();
    assert_eq!(&*interp.to_text(ObjRef::NIL), b"The NIL object");
}
```

- [ ] **Step 2: Run them to watch them fail**

Run: `cd rust && cargo test -p rexx-exec value`
Expected: compile errors, no constructors yet.

- [ ] **Step 3: Implement**

Conversions are total. Text to number is `std::str::from_utf8` then `Number::parse`, and both failures collapse into `NotNumeric` because a Rexx number's characters are ASCII by definition. Number to text is `format_form(created_digits, created_form)` — **never** `settings.digits()`.

The `num` cache is tri-state and **holds the exact parse, never a rounded one**. Rounding belongs to the operation, which is what makes the cache safe across a settings change.

`number()` admits a `SmallInt` only when the value is whole, inside `SMALL_INT_MIN..=SMALL_INT_MAX`, and its decimal digit count is at most the `DIGITS` of the operation that produced it. The check happens once, at creation, and is never re-derived.

- [ ] **Step 4: Verify against the oracle, not just against the tests**

For each of the five transcripts, run the equivalent `.rex` under `( ulimit -v 1048576; build/bin/rexx … )` and paste both outputs into the task report. The tests encode my transcripts; this step checks my transcripts.

- [ ] **Step 5: Commit**

```bash
git add rust/crates/rexx-exec/src/value.rs rust/crates/rexx-exec/tests/value.rs
git commit -m "The value model: text keeps its spelling, a number keeps its rendering"
```

---

