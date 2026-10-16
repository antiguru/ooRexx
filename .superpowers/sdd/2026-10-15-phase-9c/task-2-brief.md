### Task 2: The raising API, and the scanner and expression sites

**Files:**
- Modify: `rust/crates/rexx-inventory/build.rs` (`generate_errors`, `:49`).
- Modify: `rust/crates/rexx-parse/src/token.rs:28-73` (`ParseError`).
- Modify: `rust/crates/rexx-parse/src/error.rs` (`message()` at `:22`, `Display` at `:49`),
  `src/error/tests.rs`, `tests/errors.rs` (the `message()` asserts at `:612` and `:625`).
- Modify: `rust/crates/rexx-parse/src/scanner.rs`, `src/expr.rs` (helper `error` at `:338`).
- Modify: `rust/crates/rexx-exec/src/error.rs:1982-2005` (`From<&ParseError>`, `parse_failure`).

**Produces:**
- `rexx_inventory::errors::Msg<const N: usize>` with `pub const fn code(self) -> (u16, u16)`, and one
  generated constant per catalogue message, named `E{major}_{sub}` (for example
  `rexx_inventory::errors::E35_1: Msg<1>`), where `N` is the highest `&k` in that message's text.
- `ParseError::raise<const N: usize>(msg: Msg<N>, byte: usize, inserts: [Vec<u8>; N]) -> ParseError`.
- `ParseError::inserts(&self) -> &[Vec<u8>]`. The inserts are held behind one `Option<Box<..>>`, so a
  message with none allocates nothing. `code` and `sub` stay readable as today.
- `ParseError::message() -> Vec<u8>` renders the sub-message with its inserts as bytes, and never
  falls back to the major's text. An insert holding a non-UTF-8 byte (the oracle's 13.1 for
  `'80'x` prints the raw byte inside the quotes) reaches the output unconverted. `Display`
  (`error.rs:49`) shows it through `String::from_utf8_lossy`, for diagnostics only, and nothing
  compares `Display` output against the oracle.

- [ ] **Step 1:** Generate `Msg<N>` and the constants. A unit test in `rexx-inventory` checks, for a
      sample of constants, that `N` equals the placeholder count of the text `lookup` returns, and
      one with no placeholder is `Msg<0>`.
- [ ] **Step 2:** Add `ParseError::raise`, `inserts()` and the rendering. Every assert on
      `message()` (`tests/errors.rs:612`, `:625`, `src/error/tests.rs`) moves to a byte literal,
      and Task 1's comparison drops its `into_bytes`. Add a test pinning
      `size_of::<ParseError>()` at its new value, and record the old and new sizes in the report.
      Delete the sentence at `token.rs:40-44` that says a `u32` keeps the error two words wide,
      which the new field makes false.
- [ ] **Step 3:** `From<&ParseError> for Raised` passes the inserts as `Raised::additional`. A test in
      `rexx-exec/src/error.rs` checks that a ParseError with inserts renders the same secondary line
      through `Raised` as through `ParseError::message()`.
- [ ] **Step 4:** Convert every raise site in `scanner.rs` and `expr.rs` to `ParseError::raise`. Each
      insert is computed as the oracle's raise site computes it: cite that site in the report table
      (Rust site, oracle file:line, inserts). A site whose code is chosen at run time picks among
      constants of the same arity, or is restructured so it does.
- [ ] **Step 5:** Shrink `PENDING` by every pair now right. Run the scout C probes (`35.1` at `"("`,
      `13.1`) and the queued `2026-10-13-error-position-ampersand.md` 35.1 probe on both sides, and
      read the stdout. Per-task check, including `cargo test -p rexx-parse` and `-p rexx-inventory`.
      Commit.

