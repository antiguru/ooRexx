# The `Literals` assertion rows need a test case for `self`

`rust/crates/rexx-exec/tests/assertions.rs`'s twelve `Literals` EXEMPT rows name
`unblocked_by: "Phase 9"`: `runDynamicSource` builds `.routine~new(name, code, package)`, whose
package argument this crate refuses (`method "NEW" of class "Routine" ... (Phase 9)`). That is
the interpreter half. The harness half: `program_for` runs a row at top level, where `self` has
none of the group's methods (`q`, `hex`, `bin`) nor the framework's `runDynamicSource`, so both
engines raise 97.1 on `self~hex` (Phase 6.1 Task 7 probe `lit3`, review probe `p8/d.rex`). The
rows pass only once Phase 9 lands and the harness runs them inside a test-case object carrying
those methods.
