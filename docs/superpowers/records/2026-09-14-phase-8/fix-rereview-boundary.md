# Re-review of the fix round, boundary slice (F7, F8, F9, F2's 93.968 half, F5's `load::open`)

Range `e64202ae7..cf92ff4fb`; commits in this slice `13268f0e1` (F7), `8c839c2e9` (F8),
`94158bd73` (F9), `03ceb04df` (F2, the two 93.968 deliveries), `bab390715` (F5, `load::open`'s
refused handback). Original findings: slice A's 1.1, 1.2, 3.1, 3.2 in `final-review-a-boundary.md`.
Reviewer is read-only; a gate run is live in the worktree's `rust/target/`. Scratch under
`/tmp/claude-1000/-home-moritz-dev-repos-ooRexx-rust-rewrite/99c66dfa-1d22-4940-ab62-784c7ef57f5f/scratchpad/boundary/`.
Every finding says run or inferred; every prediction is written before its run.

Sections are appended in the order the work ran; the findings table at the end is by severity.

The findings table and "Not reached" are the last two sections.

## 0. Scratch tree (run)

`boundary/tree/rust` is `git archive cf92ff4fb rust`, with `interpreter`, `api`, `build`,
`extensions`, `ootest` (and `oodocs`, `samples`, `testbinaries`) symlinked beside it read-only;
`CARGO_TARGET_DIR=boundary/target`, `-j 4`, `--locked`. Baseline: `cargo test -p rexx-api --locked
-j 4 --no-fail-fast` exit 0; lib 15, context 14, handles 5, invoke 9, layout 18, load 10, values
38, doctests 3 (`boundary/api-tests.txt`). `cargo build --release --locked -j 4 -p rexx-exec` exit 0,
`boundary/target/release/rexx-run` sha256 prefix `2f44805543575a46`. Two more archives:
`boundary/tree-pre-f8/rust` = `13268f0e1` (= `8c839c2e9~1`) for the Miri negative control, and
`boundary/tree-mf7a/rust` = `cf92ff4fb` again, for F7's zeroing mutant.

## 1. New safe paths to UB (priority 1)

### 1.1 Reading (before any run)

`layout.rs:385-395`: `entry_type!` gives every interface-table slot the type `extern "C" fn(..)`,
a *safe* function pointer. `ffi.rs:102` declares `pub static METHOD_CONTEXT`, whose
`SetObjectVariable` and `DropObjectVariable` slots point at `set_object_variable` /
`drop_object_variable`, which call `activation_of` -> `owner_of` on whatever pointer they are
handed. `layout::Owned` (`layout.rs:370-373`) is `pub` with `pub` fields, `RexxMethodContext_`
likewise, and `lib.rs:14-19` makes `ffi` and `layout` public. So safe code outside the crate can
hand a bare `RexxMethodContext_`, or a forged `Owned` with any `owner`, to a table slot, and
`owner_of` reads past it, without a public constructor for `MethodContext` being involved. That
`dropping_stub` (`ffi.rs:386`) calls `drop_variable(context, ..)` outside an `unsafe` block is the
in-tree witness that the slot type is safe. Probe written: `tree/rust/crates/rexx-api/tests/zz_bare_context_probe.rs`
under `#![forbid(unsafe_code)]`, two tests.

**Predictions (written before the runs).**
* P1 `bare_struct_reaches_owner_of`: compiles under `forbid(unsafe_code)`; on stable the read at
  offset 24 of a 24-byte local picks up stack bytes and the `&Activation` formed from them is
  dereferenced: SIGSEGV expected, any other outcome is also UB; under Miri, an out-of-bounds read
  report at `ffi.rs:46` on an allocation of size 24.
* P2 `forged_wrapper_with_a_null_owner_reaches_owner_of`: compiles; `owner_of` answers null and
  `activation_of` dereferences it: SIGSEGV on stable, a null-dereference report under Miri.
* P3 slice A's `zz_uninit_probe.rs` copied verbatim: both tests fail to compile with E0133
  (`value_of` is now `unsafe fn`), so finding 1.1's two shapes are closed at the type level.

**Results (run).** `cargo test -p rexx-api --locked -j 4 --test zz_bare_context_probe -- --exact <name>`,
one test per process (`boundary/p-<name>.txt`):
* P1: compiled under `#![forbid(unsafe_code)]`; the process aborted, SIGABRT, from
  `crates/rexx-api/src/ffi.rs:225:14` (`activation_of`'s `&*owner_of(..)`): `misaligned pointer
  dereference: address must be a multiple of 0x8 but is 0x5558691367df` and `thread caused
  non-unwinding panic. aborting.` The eight bytes past the 24-byte local were read as a pointer
  and dereferenced; the debug build's own precondition check is what stopped it, so the mechanism
  differs from the SIGSEGV predicted and the outcome (safe code, UB reached) is the one predicted.
  Confirmed in outcome, mechanism corrected.
* P2: compiled; aborted at the same line with `null pointer dereference occurred`. Confirmed.
* P3: `cargo test -p rexx-api --test zz_uninit_probe --no-run` exit 101, `error[E0133]: call to
  unsafe function `value_of` is unsafe and requires unsafe block` at `zz_uninit_probe.rs:11:13`
  and `:18:13`, nothing else (`boundary/p-uninit-probe.txt`). Confirmed: slice A's 1.1 is closed
  at the type level. The probe file was removed from the tree again afterwards.

So the round's F8 commit message is right that no public constructor builds a `MethodContext`
from a bare struct, and wrong in what it infers: "a bare one reaching SetObjectVariable would be
the same out-of-provenance read" is exactly what `METHOD_CONTEXT.SetObjectVariable(&raw mut bare,
..)` does from safe code today. The path predates the round (the static and the safe slot types
are older than `e64202ae7`), so it is not a regression; it is the same hole as A4 reached from
the other side, and it makes the `SAFETY:` note at `ffi.rs:247-249` ("reached only through
`METHOD_CONTEXT`, which only a `Contexts` publishes") false: `METHOD_CONTEXT` is a `pub static`
that anyone calls through. Severity: Important (the same class as A3/A4, both Important). Fix
shape, not tried: type the table slots as `unsafe extern "C" fn` in `entry_type!` (the C++ side
does not care; the Rust side then needs `unsafe` to call a slot, which `dropping_stub` and any
future in-crate caller would write), or make `METHOD_CONTEXT` `pub(crate)` and hand extensions
its address only through `Contexts`. The first closes forged-`Owned` and bare-struct alike.

## 5. Forged-extension probes (priority 5, run)

Runner: a copy of the implementer's `cmp.sh` (`boundary/cmp.sh`): oracle under the standard
wrapper with `timeout 20`, both sides in the same fresh directory so the paths in `Error N running`
match, three descriptors to files. Binary `boundary/target/release/rexx-run` (`2f44805543575a46`).
Predictions written before the runs: SAME x3 on every probe below, with the deliveries the fix
report measured (argument-side 93.968 and 88.901 lineless against `f.cls`, result-side 93.968
with `main.rex line 3`).

| probe (dir under `boundary/`) | library | oracle rc | rust rc | stdout | stderr | rc |
|---|---|---|---|---|---|---|
| `p5-a_table` (slice A's `a_table.rex`, 15 steps) | `final-a/ext/libforge.so` | 0 | 0 | SAME | SAME | SAME |
| `p5-sig-arg` (`o~unknown(1)`, untrapped) | libforge | 163 | 163 | SAME | SAME: `Error 93 running .../f.cls:` no line, 93.968 | SAME |
| `p5-sig-ret` (`o~retopt`, untrapped) | libforge | 163 | 163 | SAME | SAME: `Error 93 running .../main.rex line 3:`, 93.968 | SAME |
| `p5-sig-argmissing` (`o~unknown`, untrapped) | libforge | 168 | 168 | SAME | SAME: `Error 88 running .../f.cls:` no line, 88.901 | SAME |
| `p5-f5-ver-1` (trapped first ask, then loadLibrary / loadExternalMethod / loadExternalRoutine) | `final-fix/ext/libforgever.so` | 0 | 0 | SAME: `first raised 98.982` / `second 1` / `method 1` / `routine 0` | SAME (empty) | SAME |
| `p5-f5-ver-req-2` (`::requires 'forgever' LIBRARY` in `pk.cls`, `findRoutine`, a call, `pk2.cls`'s `::routine external`) | libforgever | 0 | 0 | SAME: `loaded pk.cls` / `k 7` / `routine The NIL object` / `call raised 43.1` / `pk2 raised 90.999` | SAME | SAME |

`a_table`'s stdout, both sides: `step 1 syntax 88.901`, steps 2-5 `93.968`, `exists() 0`,
`exists("") 1`, `exists("x") 1`, `len("hello") 5`, `len(12345) 5`, `nullstr []`, `samedata 1`,
`y after setnull Y`, `step 14 syntax 88.901`, `step 15 syntax 88.909`. All as predicted. So
slice A's 3.1 (step 1) and 3.2 (step 5) are closed by running, F2's two 93.968 deliveries hold on
the fixed binary, and F5's refused library answers its methods and none of its routines
(`routine The NIL object`, `43.1`, `90.999`) as the oracle does.

## 3. Miri (priority 3, run)

Toolchain: the implementer's scratch `RUSTUP_HOME` (`final-fix/rustup-home`, nightly
1.100.0 2026-09-13, `miri 0.1.0 (4b6d04e706)`), read only; my own `XDG_CACHE_HOME=boundary/xdg-cache`
for the Miri sysroot and my own target directories. Command per step (`boundary/miri.sh`):
`MIRIFLAGS=<flags> cargo +nightly miri test --offline -j 4 <args>`; outputs `boundary/miri-<step>.txt`,
exits in `boundary/miri-status.txt`.

**Predictions (written before the runs).** M1 HEAD `--lib`: 15 of 15 ok under Stacked Borrows and
under `-Zmiri-tree-borrows` (the report's 13 were before F9 added two). M2 pre-F8 (`13268f0e1`)
`--lib` under Stacked Borrows: `a_context_we_handed_out_recovers_its_owner` reports the A4 read at
`ffi.rs:44` with the tag "created by a SharedReadWrite retag at offsets [0x0..0x18]"; the stub
test does not exist there, so nothing else; under Tree Borrows I predicted no report (a raw
pointer from a place expression is not retagged there), which would make the Tree Borrows half of
the report's "both models" claim vacuous for F8. M3 F7's mutant (`tree-mf7a`, the `Uint8` arm
returning a bare-member literal): Miri reports uninitialised memory read in `value_of` for
`a_narrow_value_is_written_over_a_zeroed_word`, `every_repr_reads_back...` stays clean. M4 the two
bare-context probes under Miri: out-of-bounds and null-reference reports respectively.

**Results.**
| step | tree | flags | result |
|---|---|---|---|
| M1 | HEAD `cf92ff4fb` | SB | 15 passed, exit 0 (`miri-m1-head-sb.txt`) |
| M1 | HEAD | TB | 15 passed, exit 0 (`miri-m1-head-tb.txt`) |
| M2 | pre-F8 `13268f0e1` | SB | exit 1: `a_context_we_handed_out_recovers_its_owner ... error: Undefined Behavior: attempting a read access using <133070> at alloc44416[0x18], but that tag does not exist in the borrow stack for this location` at `ffi.rs:44:14`, `created by a SharedReadWrite retag at offsets [0x0..0x18]` at `ffi.rs:413:26`; the other 11 tests ok (`miri-m2-pref8-sb.txt`) |
| M2 | pre-F8 | TB | **12 passed, exit 0** (`miri-m2-pref8-tb.txt`): Tree Borrows accepts the unfixed shape |
| M4 | HEAD, `bare_struct_reaches_owner_of` | SB | exit 1: `Undefined Behavior: memory access failed: attempting to access 8 bytes, but got alloc43504+0x18 which is at or beyond the end of the allocation of size 24 bytes` at `ffi.rs:46:13`, allocated at `zz_bare_context_probe.rs:13:9`, backtrace `owner_of` <- `activation_of` <- `set_object_variable` <- the test (`miri-m4-bare-sb.txt`) |
| M4 | HEAD, `forged_wrapper_with_a_null_owner...` | SB | exit 1: `Undefined Behavior: constructing invalid value of type &rexx_api::values::Activation<'_>: encountered a null reference` at `ffi.rs:225:13` (`miri-m4-forged-sb.txt`) |

M1, M2/SB and M4 as predicted. M2/TB as predicted too, and it matters: the report's "13 of 13
under both Stacked Borrows and Tree Borrows" is one witness and one pass-over-nothing, since Tree
Borrows also passes `13268f0e1`. The Stacked Borrows run is the instrument for F8; the Tree
Borrows run adds nothing about A4 and the report does not say so. Minor.

The implementer's Miri covered `--lib` only. Every callback on the *thread* table
(`whole_number_to_object`, `string_data`, `string_length`, `new_pointer`, `raise_exception0`,
`ffi.rs:266-293`) goes through `owner_of` on the thread wrapper, and the only tests that exercise
them are `tests/context.rs` and `tests/invoke.rs`, which `dlopen` `build/lib/librxregexp.so`
(`tests/context.rs:43-44`, `tests/invoke.rs:49-50`) and so cannot run under Miri. The thread
pointer is `(&raw mut self.thread).cast()` (`ffi.rs:194`, unchanged by F8), the whole wrapper, so
by reading it has the provenance `owner_of` needs; a Miri witness of my own follows in section 3.1.

## 4. The `compile_fail` doctest (priority 4, run)

**Predictions (written before the runs).** D2 stable, `tree-mf7b` with `pub unsafe fn value_of`
made `pub fn` (the only change, `diff` shown against `tree`): `cargo test -p rexx-api --doc` fails
exactly `ffi::value_of (line 54)` with `Test compiled successfully, but it's marked compile_fail`.
D3 nightly rustdoc on HEAD: 3 doctests pass, so the listed code E0133 is the one the failure
carries. D4 nightly on the mutant: the same failure as D2. S1 the F7 zeroing mutant (`tree-mf7a`)
on stable, debug and release: `a_narrow_value_is_written_over_a_zeroed_word` passes (the report's
claim that stable cannot see it).

**Results.** D2 exit 101: `ffi::value_of (line 54) - compile fail ... FAILED`, `Test compiled
successfully, but it's marked compile_fail`, the two `load.rs` doctests ok, plus an
`unused_unsafe` warning on the call site in `load.rs` (`doc-d2-stable-mutant.txt`). D3 exit 0, 3
passed (`doc-d3-nightly-head.txt`). D4 exit 101, the same failure as D2 (`doc-d4-nightly-mutant.txt`).
S1 exit 0 in both profiles (`s1-mf7a-stable{,-release}.txt`). All as predicted. The intended
reason is also witnessed on stable by P3 above: the same union literal in a test file is refused
with `E0133` and nothing else.

### 3.1 A thread-context witness of my own (run)

`boundary/tree-m5/rust` is HEAD plus a `cfg(test)` stub `ffi::numbering_stub` (reads
`(*context).threadContext`, calls that table's `WholeNumberToObject` with 42, writes the answer
into element zero, signature `RexxObjectPtr`) and `invoke::tests::a_stub_reaches_its_activation_through_the_thread_context_it_was_handed`,
which runs it through `Contexts::method` and asserts `Ok(Some(small_int(42)))`.
`boundary/tree-m5mut/rust` is the same with one line changed (`diff` shown, one hunk):
`Contexts::method` writes `threadContext = (&raw mut self.thread.context).cast()`, the field
rather than the wrapper, the thread-side twin of A4.

**Predictions (written before the runs).** `tree-m5`: the test passes under Stacked Borrows and
under Tree Borrows. `tree-m5mut`: under Stacked Borrows, `Undefined Behavior: attempting a read
access ... at alloc[0x10], but that tag does not exist in the borrow stack` at `ffi.rs:46`
(`owner_of`), the tag created by a SharedReadWrite retag at offsets `[0x0..0x10]` at the mutated
line; under Tree Borrows it passes, as the pre-F8 tree did.

## 2. Every `SAFETY:` note the round wrote or changed (priority 2, read against the code)

| where | the note's invariant | who establishes it | holds on every path? |
|---|---|---|---|
| `ffi.rs:66-69` `value_of` `# Safety` | every byte of the member `repr` names is initialised | the caller | yes as a contract: the one in-tree caller is `load.rs:182` (`grep value_of` finds no other outside `ffi.rs` itself), and `invoke.rs` no longer imports `ffi` (Q2 as ruled) |
| `ffi.rs:71-73` inside `value_of` | any initialised bytes are a valid integer, float or pointer | the language | yes |
| `load.rs:180-182` `call`'s read | element zero's word was written in full above | `load.rs:161`, which precedes the `if let Some(stub)` so it runs whether or not a stub does | yes |
| `load.rs:168-171`, `:177` `call`'s two writes | `context` holds the only borrow of a live `RexxMethodContext_`; the type is neither `Clone` nor `Sync` | `Contexts::method(&mut self)` and `MethodContext::bare(&mut _)` are the two constructors (`ffi.rs:127`, `:196`), both from an exclusive borrow; `MethodContext` derives nothing and holds a `*mut` | yes |
| `ffi.rs:33-39` `owner_of` `# Safety` | the pointer was derived from the whole `Owned`, so its provenance covers `owner`; "the guarantee is the interpreter's, which mints every context it hands out and never hands out one it did not build" | `Contexts::method`, `ffi.rs:194` and `:197`, both `&raw mut self.<wrapper>` | as a contract yes; as a description of who can reach the function, no: section 1.1 reaches it from safe code through `METHOD_CONTEXT` with a pointer nobody minted |
| `ffi.rs:42-45` inside `owner_of` | as above, plus `context` first by value | `Owned` is `#[repr(C)]` with `context` first (`layout.rs:369-373`); `the_public_struct_is_at_the_head_of_the_wrapper` pins it | yes |
| `ffi.rs:247-249` `set_object_variable` (unchanged text, but now the load-bearing one) | "reached only through `METHOD_CONTEXT`, which only a `Contexts` publishes" | nothing: `METHOD_CONTEXT` is `pub static` (`ffi.rs:102`) and its slots are safe `extern "C" fn` (`layout.rs:389-394`) | **no**, run: P1/P2 and M4 |
| `ffi.rs:348-350` `reading_stub`, `:381-384` `dropping_stub` | the context is the one `invoke::method` handed the stub, and a `Contexts` wrote its `functions` | the two tests that use them | yes for their uses (test-only) |
| `ffi.rs:470-472` the owner unit test | the pointer is the whole `Owned` cast to its `context` | the line above it | yes |
| `load.rs:454` `routine_table` "as above" | the entry's table pointers are null or terminated arrays in the mapping | the `RexxGetPackage` contract, the note it refers to | yes; the `match refused` around it reads the routine table only when the version check passed, which is what the `Refused` doc (`load.rs:63-66`) says of `LibraryPackage.cpp:232-237`, printed: `232` the check, `237` `loadRoutines` |

Citations printed for every new comment in the slice (`interpreter/execution/NativeActivation.cpp`
`190-193`, `228-229`, `325-327`, `607-612`, `672`, `720`, `855-858`, `1301-1310`;
`interpreter/package/LibraryPackage.cpp:232-237`): each lands on the construct the sentence names
(`reportSignatureError`, `descriptors[0].type = *argumentTypes` / `value_int64_t = 0`, the outer
`default:` of `processArguments`, `Error_Invalid_argument_noarg` under `!isOptional`,
`inputIndex++`, `switch (value->type)`, `valueToObject`'s `default: reportSignatureError()`,
`trapErrors = true; try { ... valueToObject }`, the version check and `loadRoutines`).

One doc sentence the round left half-true: `invoke.rs:44-49` (`method`'s `# Errors`) still says
`Failure::Signature` is answered "for ... a parameter code the table does not know"; after F9 an
unknown non-optional code with no argument is `MissingArgument` (`values.rs:952-970`, and the
test `an_unknown_parameter_code_is_an_argument_position`), and `Signature` only with an argument
or the optional bit. Minor, read.

**Results (run, `boundary/miri2.sh`, `miri-m5*.txt`).** A first shape of the probe declared a
`RexxObjectPtr` result and failed on its own assertion with `Err(Unfilled { code: 11, name:
"RexxObjectPtr", direction: FromNative })`: that row is unfilled in this slice, and the callback
had already run clean before the conversion. Reshaped (result `int`, the minted object stored
through the method table's `SetObjectVariable` and asserted on the host: `variables ==
[("N", small_int(42))]`), the diff between `tree-m5` and `tree-m5mut` is the one line at
`ffi.rs:194`:
| copy | flags | result |
|---|---|---|
| `tree-m5` | SB | ok, exit 0 |
| `tree-m5` | TB | ok, exit 0 |
| `tree-m5mut` | SB | exit 1: `Undefined Behavior: attempting a read access using <177823> at alloc55570[0x30], but that tag does not exist in the borrow stack` at `ffi.rs:46:14`, `created by a SharedReadWrite retag at offsets [0x20..0x30]` at `ffi.rs:194:45`, backtrace `owner_of::<RexxThreadContext_, Activation>` <- `activation_of` <- `whole_number_to_object` <- `numbering_stub` <- `NativeMethodEntry::call` <- `invoke::method` |
| `tree-m5mut` | TB | ok, exit 0 |
As predicted. So the thread wrapper's pointer has the provenance `owner_of` needs, the instrument
sees the shape when it is wrong, and Tree Borrows again sees nothing. This witness exists only in
scratch; the tree pins the thread-table path under Miri nowhere (its tests need `librxregexp.so`).
Minor.

## 6. Other checks (run)

* `cargo test --locked -j 4 -p rexx-core --test unsafe_sites`: 2 passed (`the_scan_reaches_the_whole_workspace`,
  `only_the_granted_module_may_say_unsafe`), so `unsafe` is still only in `ffi.rs` and `load.rs`
  (the two hits elsewhere, `layout.rs:278` a fn-pointer type and `values.rs:567` a doc sentence,
  predate the range: 1 and 1 at `e64202ae7` too). `boundary/unsafe-sites.txt`.
* `cargo test --locked -j 4 -p rexx-exec --test refusal_sites -- --test-threads=1`: 5 passed
  against the committed `refusal-sites.tsv` (`boundary/refusal-sites.txt`).
* `tests/invoke.rs` before and after the F8 rewrite: the set of `fn` names differs by exactly
  `every_repr_reads_back_the_member_the_table_wrote` and its `sample`, which moved into `ffi.rs`'s
  unit tests (10 tests before, 9 after, plus the moved one in the lib, so nothing lost). Every
  `with_context` caller now goes through `Contexts::new` (`tests/invoke.rs:198`, `:211`), as the
  commit says.
* No change to `rust/Cargo.lock`, `rust/Cargo.toml` or any crate `Cargo.toml` in the range
  (`git diff --stat` empty).
* `git diff --stat e64202ae7..cf92ff4fb -- rust/crates/rexx-api/src/layout.rs` is empty, and
  `pub static METHOD_CONTEXT` was introduced at `15e7c2797`, before the range, with the slot
  type already `extern "C" fn`: finding 1 below is pre-existing, not a regression of this round.

## 7. Per original finding

| finding | status | witness |
|---|---|---|
| A 1.1 (`value_of` safe, uninitialised read) | **closed** | P3: both of A's shapes are E0133 now; M1: `every_repr_reads_back...` and `a_narrow_value...` clean under Miri; M3: the zeroing mutant is UB under Miri and invisible on stable (S1), as the report says; D2/D3/D4: the `compile_fail` doctest fails for E0133 and for nothing else |
| A 1.2 (`owner_of` past the reference) | **closed for the path named**, and the same read stays reachable from safe code through the table: finding 1 below | M2/SB reproduces A4 on `13268f0e1` (`ffi.rs:44`, tag `[0x0..0x18]`); M1/SB clean on HEAD including `a_stub_reaches_its_activation_through_the_context_it_was_handed`; M5 covers the thread-side twin; the `SAFETY:` note now names provenance and `Contexts::method` derives both pointers from the whole wrappers (`ffi.rs:194`, `:197`); `MethodContext` has no public constructor (`bare` is `cfg(test)`, `ffi.rs:125-131`) |
| A 3.1 (unknown code, missing argument) | **closed** | `p5-a_table` step 1 `88.901` SAME x3; `p5-sig-argmissing` SAME x3, `Error 88 running .../f.cls:` lineless; the corrected `tests/values.rs` asserts the oracle's three answers |
| A 3.2 (result word with the optional bit) | **closed** | `p5-a_table` step 5 `93.968` SAME x3; `p5-sig-ret` SAME x3, `Error 93 running .../main.rex line 3:` with the line; the stub runs first (`a_result_word_carrying_the_optional_bit_is_refused_after_the_call`) |
| F2, the two 93.968 deliveries | **holds** | `p5-sig-arg` lineless against `f.cls`, `p5-sig-ret` with the line against `main.rex`, both rc 163 SAME x3; `Signature` -> `incorrect_method_signature` (lineless) and `ResultSignature` -> `incorrect_method_result_signature` (`library.rs:140-141`, `error.rs`) |
| F5, `load::open`'s refused handback | **holds** | `p5-f5-ver-1` and `p5-f5-ver-req-2` SAME x3: methods bind and run (`k 7`), no routine registers (`The NIL object`, `43.1`, `90.999`); `load.rs:452-456` reads the routine table only when the check passed |

## Findings by severity

**Critical:** none.

**Important**
1. `rust/crates/rexx-api/src/ffi.rs:102` (`pub static METHOD_CONTEXT`) with
   `src/layout.rs:389-394` (`entry_type!`, safe `extern "C" fn` slots) and `src/layout.rs:370-373`
   (`pub struct Owned` with `pub` fields): safe code outside the crate reaches `owner_of`'s
   out-of-provenance read with a bare `RexxMethodContext_`, or a null dereference with a forged
   `Owned`, by calling a table slot directly. Run: `zz_bare_context_probe.rs` under
   `#![forbid(unsafe_code)]` compiles, aborts on stable (`misaligned pointer dereference` /
   `null pointer dereference occurred` at `ffi.rs:225`), and is UB under Miri (`at or beyond the
   end of the allocation of size 24 bytes` at `ffi.rs:46`; `encountered a null reference` at
   `ffi.rs:225`). Pre-existing (introduced at `15e7c2797`), so not a regression of the round, but
   the F8 commit message's reason for having no public `MethodContext` constructor ("a bare one
   reaching SetObjectVariable would be the same out-of-provenance read") describes this open
   path as if it were closed, and the `SAFETY:` note at `ffi.rs:247-249` ("reached only through
   `METHOD_CONTEXT`, which only a `Contexts` publishes") is false. Fix shape: `unsafe extern "C"
   fn` slot types in `entry_type!` (then every in-crate caller of a slot, today only
   `dropping_stub`, says `unsafe`), or `METHOD_CONTEXT` not `pub`. Section 1.1.

**Minor**
2. `final-fix-report.md`, F8 ("13 of 13 ok under Stacked Borrows ... and 13 of 13 under
   `-Zmiri-tree-borrows`") and the commit message ("with both Stacked Borrows and Tree Borrows"):
   Tree Borrows also passes the unfixed `13268f0e1` (M2/TB, 12 passed) and the thread-side mutant
   (M5mut/TB), so only the Stacked Borrows run witnesses F8; the report presents two witnesses
   where there is one. Run. Section 3.
3. `rust/crates/rexx-api/src/ffi.rs:266-293` (the five thread-table callbacks): every one goes
   through `owner_of` on the thread wrapper, and no test the tree can run under Miri reaches
   them (`tests/context.rs` and `tests/invoke.rs` `dlopen` `librxregexp.so`). The scratch witness
   in section 3.1 passes and its mutant fails, so the code is right; the instrument is missing
   from the tree. Run. Section 3.1.
4. `rust/crates/rexx-api/src/invoke.rs:44-49` (`method`'s `# Errors`): says `Failure::Signature`
   is answered for "a parameter code the table does not know"; after F9 that is
   `MissingArgument` when the argument is absent and the code not optional
   (`values.rs:952-970`). Read. Section 2.

## Not reached

* A's finding 1 (the thread context outliving the call, `b2`/`b3`) and findings 7 and 8: out of
  the brief, not re-run.
* F2's 88.909 half and `native_argument_needs_a_string_value` (slice B's integration side), F1,
  F3, F4, F6, F10, F11: not in this slice; the corpus witnesses they added were not run here.
* The oracle's unloader at termination for a refused library (the report records it read, not
  run): not run here either.
* `refusal-sites.tsv`'s hand-measured columns for the two new 93.968 rows: the 5 `refusal_sites`
  tests pass in my copy, and `p5-sig-arg`/`p5-sig-ret` re-measure the two answers, but the
  transposition control the report describes was not repeated.
* Whether Tree Borrows with `-Zmiri-unique-is-unique` (or a future default) would see A4: only the
  default Tree Borrows flags were run.
* The `compile_fail` doctests in `load.rs` (lines 90 and 292) were run (pass on stable and
  nightly) but not read for their reason, as slice A noted.
