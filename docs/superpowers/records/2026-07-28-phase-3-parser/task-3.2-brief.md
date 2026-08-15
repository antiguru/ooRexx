## Task 3.2: `ProgramSource` and `SOURCELINE`

**Files:**
- Create: `rust/crates/rexx-parse/src/source.rs`
- Test: `rust/crates/rexx-parse/tests/sourceline.rs`

**Interfaces:**
- Produces: `ProgramSource::new(text: Vec<u8>) -> ProgramSource`,
  `ProgramSource::line(&self, n: usize) -> Option<&[u8]>` (1-based),
  `ProgramSource::line_count(&self) -> usize`,
  `ProgramSource::line_of(&self, byte: usize) -> usize` returning the 1-based
  physical line containing that byte. Every later task uses `line_of` for error
  reporting.

Source retention comes first because everything else holds ranges into it.

**The retained source is bytes, not a Rust `String`, and this is not a style
choice.** A Rexx source file may contain arbitrary bytes that are not valid
UTF-8. Measured: a file whose second byte sequence is a raw `FF FE` inside a
literal runs fine, `c2x` gives `FFFE` and `length` gives 2, and invalid bytes in
a comment are ignored as comment text. `String::from_utf8` would reject that
file, so a `String`-typed source rejects legal programs. `Vec<u8>` in, `&[u8]`
out, everywhere.

`SOURCELINE` returns a Rexx string, which is a byte string, so `line` returning
`&[u8]` is the faithful signature rather than a concession.

This costs almost nothing above the scanner, because the one thing that *does*
need `&str` is safe by construction: a symbol cannot contain a non-ASCII byte
(`LanguageParser::characterTable` is zero for every byte 0x80-0xFF, and `bäc = 2`
is error 13.1), so converting a symbol's bytes for interning cannot fail. Do it
with `std::str::from_utf8(...).expect(...)` and say why in the expect message,
because that invariant is the scanner's to maintain. Literal values stay raw
bytes; see Task 3.3.

Rexx has **no Unicode string semantics** to reproduce here. `length('ää')` is 4,
`substr(s,1,1)` yields the single byte `C3`, and `reverse` reverses bytes into
invalid UTF-8 — all measured. The interpreter vendors `utf8proc` for exactly one
purpose, decoding the offending sequence so error 13.1 can print a whole
character, and this phase does not reproduce parse-error text at all, so we need
no equivalent.

- [ ] **Step 1: Capture the interpreter's behaviour**

```rexx
/* probe.rex */
say sourceline()            /* the count */
say "[" || sourceline(1) || "]"
say "[" || sourceline(2) || "]"
```

Run it under `build/bin/rexx` and record the answers. Check specifically: does
`sourceline(n)` include the trailing newline? What does it return for `n`
past the end, and for `n = 0`? Do not guess these; a wrong answer here is
invisible until `TRACE` is wired up in Task 3.9.

- [ ] **Step 2: Write the failing test**

```rust
#[test]
fn sourceline_returns_lines_without_terminators() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec());
    assert_eq!(src.line_count(), 2);
    assert_eq!(src.line(1), Some("say 1"));
    assert_eq!(src.line(2), Some("say 2"));
    // Out of range is an ERROR in the interpreter, not an empty answer:
    // sourceline(0) raises 40.14 and sourceline(99) raises 40.34. Verified.
    // `line` returning None is how this crate reports that; Task 3.8 turns
    // it into the right error number. Do not let it render as "".
    assert_eq!(src.line(3), None);
    assert_eq!(src.line(0), None);
}

#[test]
fn line_of_is_one_based() {
    let src = ProgramSource::new(b"say 1\nsay 2\n".to_vec());
    assert_eq!(src.line_of(0), 1);
    assert_eq!(src.line_of(4), 1);
    assert_eq!(src.line_of(6), 2);
}

#[test]
fn source_may_hold_bytes_that_are_not_utf8() {
    // A Rexx literal may contain arbitrary bytes. Verified against the oracle:
    // a file holding a raw FF FE inside a literal runs, and c2x reports FFFE.
    // A String-typed source would refuse to construct here.
    let src = ProgramSource::new(b"s = '\xff\xfe'\n".to_vec());
    assert_eq!(src.line(1), Some(&b"s = '\xff\xfe'"[..]));
    assert_eq!(src.line_count(), 1);
}
```

- [ ] **Step 3: Run it and watch it fail**

`cargo test --offline -p rexx-parse --test sourceline`

- [ ] **Step 4: Implement**

Build a line-start index once at construction. `position` is a binary search
over it — not a scan, because error reporting calls it and Task 3.10 measures
throughput.

- [ ] **Step 5: Handle the line-terminator cases the probe found**

CRLF and a final line without a terminator both need explicit tests, with the
expected values taken from Step 1's probe rather than from intuition.

- [ ] **Step 6: Commit**

---

