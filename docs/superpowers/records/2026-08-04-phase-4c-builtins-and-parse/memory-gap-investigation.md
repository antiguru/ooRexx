# Memory-gap investigation: the SIGABRT set and the 5.7 MB RSS gap

Investigation only.
No source file touched, nothing committed.
All commands below ran from a fresh subdirectory of the session scratchpad, never the scratchpad root, per the oracle's external-routine search-path hazard.
Every oracle invocation was wrapped in `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )` unless noted otherwise.
stdout, stderr and exit status were read as three separate descriptors throughout; no `2>&1`, no piped exit codes.

## Finding 1: the SIGABRT set

### The current aborting set

Built `rexx-run` in release mode from `rust/` (`cargo build --release`).
Ran the seven programs `phase-4-exclusions.txt:1603-1609` records, at `N=400000000`, under the standard `ulimit -v 1048576`.
All seven still abort today; none has been closed since that row was written.

| Program | rexx-run | oracle |
|---|---|---|
| `say length(strip(copies('a',400000000)))` | SIGABRT, rc 134 | `400000000`, rc 0 |
| `say length(reverse(copies('a',400000000)))` | SIGABRT, rc 134 | `400000000`, rc 0 |
| `say length(substr(copies('a',400000000),1,5))` | SIGABRT, rc 134 | `5`, rc 0 |
| `say length(copies('a',400000000) \|\| 'x')` | SIGABRT, rc 134 | `400000001`, rc 0 |
| `say copies('a',400000000)` | SIGABRT, rc 134 | 400000000-byte line, rc 0 |
| `if copies('a',400000000) == '' then nop` | SIGABRT, rc 134 | (no output), rc 0 |
| `parse value copies('a',400000000) with y` | SIGABRT, rc 134 | (no output), rc 0 |

Every rc 134 carried `memory allocation of N bytes failed` on stderr (`std::alloc::rust_oom`).
I independently reran the oracle on all seven at `N=400000000` and confirmed rc 0 on every one (command: the wrapped oracle invocation above, per file, exit status read unpiped).
Also reconfirmed, as a sanity check on the FIRST-cause fix already shipped: `say length(copies('a',400000000))` (no wrapping call) is rc 0, `x = copies('a',400000000)` alone (`p8.rex` in the sweep) is rc 0, and `say length(copies('a',500000000))` still raises Error 5 rc 251 rather than aborting.
None of that is new; it reproduces the FIRST- and SECOND-cause rows exactly as recorded.
The recorded seven-program list is current, not stale, and no eighth program turned up in the shapes I tried.

### The mechanism, confirmed with named call sites

`RUST_BACKTRACE=1` on each of the seven names the allocation site directly (some frames are inlined into `step_in_temps_frame`, but the source read below fills those in unambiguously):

* `builtin::string::strip` (`crates/rexx-exec/src/builtin/string.rs:564`)
* `builtin::string::reverse` (`string.rs:548`)
* `builtin::string::substr` (`string.rs:307`)
* `Interp::concat` (`crates/rexx-exec/src/eval.rs:679`)
* the `Say` arm of `Interp::step` (`crates/rexx-exec/src/run.rs:969`)
* `Interp::eval_compare` (`eval.rs:720`), reached from `eval_condition` for the `IF`
* `Interp::parse_strings`' `ParseSource::Value` arm (`crates/rexx-exec/src/parse_template.rs:469`)

All seven bottom out in `builtin::required_string` (`builtin/mod.rs:763-766`) or an equivalent inline pattern:

```
fn required_string(interp: &mut Interp, args: &[Option<ObjRef>], position: usize) -> Vec<u8> {
    let value = arg(args, position).expect(...);
    interp.to_text(value).into_owned()
}
```

`Interp::to_text` (`value.rs:128`) returns `Cow::Borrowed(bytes.as_slice())` for a `Body::Text` value: it borrows `self`, not the value.
Every one of the seven call sites needs to call back into `&mut self` immediately afterward (to allocate a result, to call `to_number`, to push a root, to run another builtin), which the borrow checker will not allow while the `Cow` is alive.
`.into_owned()` / `.to_vec()` is what ends the borrow.
**The copy is not discarded: strip reads it to find pad characters, reverse mutates it in place, substr slices it, concat appends to it, eval_compare feeds it to `compare_decoded`, SAY writes it to output, PARSE binds it to a variable.**
This is exactly the "ends a borrow" diagnosis the file records, and I confirmed it by reading every one of the seven call sites, not by inference from the backtrace alone.

`try_reserve` (the guard idiom `builtin::buffer`, `builtin/mod.rs:858-863`, already uses for a builtin's own *result* allocation) cannot be dropped in at any of these seven sites for a structural reason the file states but does not spell out: **`Cow::into_owned()` has no fallible form.** It calls `Vec::to_vec()`, which calls the *infallible* global allocator and aborts on failure by design. Guarding these sites means not calling `.into_owned()`/`.to_vec()` at all -- replacing it with a hand-written fallible copy (`try_reserve_exact` + `extend_from_slice`), which is a different, larger change than pasting `try_reserve` in, because the surrounding code was written assuming an owned `Vec<u8>` it can slice, mutate, and pass around freely.

### One shape, but at least three severities, and a growth-policy pathology the file does not name

The single mechanism above (a `Cow::Borrowed` that must become owned to release the borrow) is real and common to all seven, but it produces different numbers of *extra* live copies at different sites, and the file's own producer list undercounts two of them. Measured by watching which allocation size fails as `N` is swept down (all under `ulimit -v 1048576`):

| Program | succeeds at | first fails at | extra copies beyond the original |
|---|---|---|---|
| `reverse` | 200,000,000 | 250,000,000 | 1 (`required_string`; `text_owned` reuses it, no third copy) |
| `substr` | 200,000,000 | 250,000,000 | 1 (`required_string` copies the *whole* string to read 5 bytes of it) |
| `if ... == ''` (`eval_compare`) | 200,000,000 | 250,000,000 | 1 (`into_owned()` of the left operand; the right is the empty literal) |
| `strip` | 150,000,000 | 170,000,000 | 2 (`required_string`, then `interp.text(kept)`'s own `to_vec()` -- `strip`'s default set has nothing to trim from an all-`a` string, so `kept` is the full length) |
| `parse value` | 150,000,000 | 170,000,000 | 2 (`parse_strings`' `to_vec()`, **plus a second, unnamed copy** -- see below) |
| `say copies(...)` | 100,000,000 | 120,000,000 | 2, but with a doubling penalty -- see below |
| `copies(...) \|\| 'x'` (`concat`) | 100,000,000 | 120,000,000 | 2, but with a doubling penalty -- see below |

Two corrections to the file's own producer list, found by reading the code the backtraces pointed at:

* **PARSE has a second, unnamed copy.** The file names only `parse_strings`' source render. But `Interp::assign_targets` (`parse_template.rs:670`) does `let value = self.text(&cursor.string()[piece.clone()]);` for every bound template variable -- `self.text` (`value.rs:46`) copies via `to_vec()`, where `self.text_owned` (the sibling `reverse` already uses correctly) would move the buffer instead. For `parse value X with y`, `piece` is the whole string, so this is a full third copy, not a slice. This is why `parse value` fails at the same threshold as `strip` (two extra copies), not at `reverse`/`substr`'s threshold (one extra copy) as the file's producer list would suggest.
* **`concat`'s `||` and SAY's trailing newline both trigger `Vec` capacity doubling, not just a second copy.** `Interp::concat` builds `bytes` via `to_text(left).to_vec()` (exact capacity = left's length), then `bytes.extend_from_slice(&self.to_text(right_value))` for the one-byte right operand -- since `bytes` is already at capacity, this forces `Vec`'s amortized-growth policy to request roughly *double* the left operand's size for a one-byte append. Measured directly: `say length(copies('a',200000000) || 'x')` aborts with "memory allocation of 400000000 bytes failed" -- exactly double `N`, not `N`. The `Say` arm has the same shape: `self.out.extend_from_slice(&line)` (line.len() == N, exact fit) followed by `self.out.push(b'\n')`, and that one-byte push is what forces the doubling. Measured: `say copies('a',120000000)` fails requesting 240,000,000 bytes, and `say copies('a',150000000)` fails requesting 300,000,000 bytes -- both exactly `2N`. This is why SAY and `||` fail at roughly half the `N` that the "two/three extra copies, no doubling" sites tolerate, and it is a distinct, previously-unrecorded pathology: **the growth-policy cost, not just the copy count, is part of the shape here.**

So the honest answer to "one shape or several": **one root cause** (a `Cow` borrow of `&mut self` that must be ended before the value can be used again), but at least **three distinct severities** riding on it -- a single extra copy, a doubled extra copy from growth-on-append, and (for PARSE) an extra copy the file never named. Varying the surrounding expression, exactly as the file's own warning says to do, is what surfaces the difference; a single probe shape could not have shown any of this.

As calibration, the crate's own usable single-allocation ceiling under `ulimit -v 1048576` (no extra copies at all, `say length(copies('a',N))`) is between 465,000,000 (succeeds) and 470,000,000 (raises Error 5) -- about 70 MB less than the naive "1024 MiB minus D19's 512 MiB reservation = 512 MiB" arithmetic, from other fixed overhead (the main thread's own stack, code/data segments, allocator bookkeeping). Dividing that ceiling by each site's extra-copy count predicts the measured thresholds well: ~465M/2 ≈ 233M lands inside the observed (200M,250M] band for the one-extra-copy sites, and ~465M/3 ≈ 155M lands inside the observed (150M,170M] band for the two-extra-copy sites without doubling.

### What closing it would cost, and the qualifier the file is missing

The file already states the shape of the fix: new `Interp` methods that read and allocate in one step, so the borrow-forced copy never has to be taken at all -- a `text_from`-style helper for the slice-taking builtins, a preallocated `concat_text` (which also fixes the doubling, by reserving `left.len() + sep.len() + right.len()` up front instead of appending twice into an exactly-sized buffer), a `compare_decoded` path that does not need `&mut self` between reading each operand and reading the other, and swapping `self.text(...)` for `self.text_owned(...)` at `parse_template.rs:670` (mechanical, and the same idiom `reverse` already gets right). None of that is written yet -- I confirmed no `text_from`, `concat_text`, or similarly-shaped helper exists anywhere in `crates/rexx-exec/src`.

**The qualifier: even a fully correct fix, taking every one of the seven sites down to its theoretical minimum of one extra copy, does not make these seven programs return the oracle's answer at the file's own probe size of `N=400,000,000` under the standard `ulimit -v 1048576`.** Two live copies of 400,000,000 bytes is roughly 763 MiB, and the crate's own usable budget after D19's 512 MiB thread-stack reservation is ~465 MB (measured above) -- nowhere close. `reverse` already sits at this theoretical minimum today (`required_string` copies once, `text_owned` moves the result with no further copy) and it still aborts at 400,000,000, succeeding only up to somewhere between 200,000,000 and 250,000,000. So the THIRD cause's fix, done perfectly, raises each site's threshold (roughly doubling it, per the arithmetic above) but does **not** close the specific seven-program list at the size the file records it at -- that also needs the SECOND cause (D19's reservation) revisited, which the file already says is out of Phase 4's scope and owned by whoever reopens D19. The file states the two causes as separately owned; it does not say that the THIRD cause's owner cannot actually deliver rc 0 at `N=400,000,000` alone. That interaction is worth writing into whichever plan picks this up, so the THIRD cause's owner does not discover it only after landing a correct-looking fix that still shows red at the recorded probe size.

On the trace-witness requirement the file asks for: a wrong guard at any of these seven sites would not merely drop a trace line the way a wrong FIRST-cause guard would -- it would drop the SAY output, the STRIP result, the PARSE binding, or the comparison's own operand, because these copies are read. The existing per-builtin corpus tests already assert the *value*, not just that a trace line printed, so the witness these fixes need already exists; the risk in this fix is getting the ownership/borrow rewrite right, not silently losing a print.

## Finding 2: the 5.7 MB RSS gap

Re-measured `say length(copies('a',500000000))` at `ulimit -v 4194304` (4 GiB, both sides succeed, matching the file's own instrument choice) with `/usr/bin/time -v`, 5 runs per side, all from a fresh scratch directory:

| | rexx-run (kB) | oracle (kB) |
|---|---|---|
| run 1 | 488996 | 496100 |
| run 2 | 489092 | 496328 |
| run 3 | 490756 | 496380 |
| run 4 | 490532 | 495588 |
| run 5 | 490380 | 496096 |
| mean | 489951 | 496098 |

Mean gap: 6147 kB (~6.1 MB), rexx-run under the oracle, same direction and same rough magnitude as the recorded 5.7 MB (490692 vs 496364). Spread within each side (rexx-run ~1760 kB, oracle ~792 kB) matches the file's own note that this instrument's noise floor is under 1 MB, so the gap is still real, not noise -- exactly as the file concluded when it chose to hold this open rather than dismiss it.

To find what the gap actually is, I ran the same instrument on a program that never touches a large string -- `say 1` -- at the same `ulimit -v 4194304`, 5 runs per side:

| | rexx-run (kB) | oracle (kB) |
|---|---|---|
| run 1 | 3124 | 8424 |
| run 2 | 2812 | 8684 |
| run 3 | 2808 | 8344 |
| run 4 | 2804 | 8396 |
| run 5 | 2804 | 8380 |
| mean | 2870 | 8446 |

Mean baseline gap: 5575 kB (~5.6 MB) -- present before either interpreter does anything Rexx-specific at all. Subtracting this baseline from the big-string gap leaves a residual of 572 kB, which is *smaller* than the noise floor measured within the big-string runs themselves (792-1760 kB). **The 5.7 MB (now ~6.1 MB) gap is explained: it is almost entirely a startup/baseline RSS difference between the two processes, not anything to do with how either interpreter handles a large string.** Once the baseline is subtracted, there is no big-string-specific residual left to explain -- it does not exceed this instrument's own noise floor.

A plausible mechanism for the baseline gap, checked but not proven further: `ldd` on the oracle binary shows it links `librexx.so.4` (17.8 MB on disk), `librexxapi.so.4` (656 KB), `libstdc++.so.6` and `libm.so.6`; `rexx-run` links only `libc.so.6` and `libgcc_s.so.1`. Touching pages of a much larger shared library at load time, plus whatever C++ static initialisers and the oracle's own message-catalogue/method-table setup do at startup, is consistent with a several-MB baseline RSS difference and with the direction of the gap (oracle higher). I did not isolate this further (e.g. by sampling RSS immediately post-`dlopen` versus post-run) -- it is offered as the most likely mechanism given the evidence gathered, not as a proven cause.

Hypotheses ruled out, and how:

* **Run-to-run noise.** Ruled out both before this investigation (the file's own comparison against an ~800 kB spread) and again here (measured spread up to 1760 kB, still well under the ~6-7 MB raw gap).
* **A big-string-specific extra copy or leak in this crate.** Ruled out: subtracting a baseline measured on a program that never allocates a large string accounts for essentially all of the raw gap (residual 572 kB, below noise). If either side held or freed an extra copy specifically tied to the large string, the residual after subtracting baseline would exceed the noise floor, and it does not.
* **D19's 512 MiB thread-stack reservation as the driver.** Ruled out on the grounds the file itself already gives: RSS charges for resident pages, and an unfaulted stack reservation is not resident, so this measurement (unlike the address-space rc table) "survives the reservation being free." The baseline-gap explanation found here is a separate mechanism (shared-library/startup footprint), not a restatement of the reservation.

This downgrades the finding from "held open, unexplained" to "explained, and the explanation is not this crate's big-string handling" -- there is no leak or extra copy to chase on the large-string path specifically.

## Corrections and additions to `phase-4-exclusions.txt`

Nothing in the file's core claims for either finding turned out to be false: the seven-program list, the "ends a borrow, three copies where the oracle holds two" diagnosis, "no guard fixes any of them," and the FIRST cause's closure are all reproduced exactly as recorded, today, on this branch.

What is missing or incomplete, worth folding into the file before someone takes up either cause:

* PARSE's abort has a second, unnamed copy at `parse_template.rs:670` (`self.text` instead of `self.text_owned` for a bound template variable) beyond the one the file's producer list names.
* `concat`'s `||` and SAY's trailing newline push both suffer a `Vec` capacity-doubling penalty on top of the copy count, which is why they fail at roughly half the `N` the other five sites tolerate -- a distinct severity, not covered by "three copies where the oracle holds two."
* The THIRD cause's fix, even done perfectly (down to one unavoidable extra copy per site), cannot make the recorded seven programs succeed at `N=400,000,000` under the standard `ulimit -v 1048576` -- that also requires the SECOND cause (D19's reservation) to be revisited. The file records the two causes as separately owned but does not say they are coupled at the probe size it uses.
* Finding 2 (the 5.7 MB RSS gap) can be marked explained rather than held open: it is a startup/baseline footprint difference between the two binaries, reproduced on a program that touches no large string, and not a residual effect of the big-string path.

## Environment note

Partway through this investigation, `git status --short` briefly showed `rust/crates/rexx-exec/src/eval.rs` as modified (a one-line change to an `ExprKind` match arm's fallback, unrelated to anything above). I made no edits to that file. The modification appeared and then disappeared between two consecutive `git status` checks, consistent with the concurrently-running whole-branch review mentioned in this task's brief touching the same tree. `git status --short` is empty now; nothing was reverted or committed by this investigation.
