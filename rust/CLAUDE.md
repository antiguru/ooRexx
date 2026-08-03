# Working in this crate tree

A clean-room Rust reimplementation of ooRexx.
Correctness is defined **differentially**: a change is right when its output matches the C++ interpreter byte for byte, not when it looks right.

Each rule below has already cost this project a session, a wrong measurement, or a shipped defect.

## The oracle

* **The C++ tree is read-only.** Never modify `interpreter/`, `samples/`, `build/`, `ootest/`.
* **Wrap every oracle run** as
  `( ulimit -v 1048576; LD_LIBRARY_PATH=/home/moritz/dev/repos/ooRexx/build/lib /home/moritz/dev/repos/ooRexx/build/bin/rexx FILE )`.
  Without the `ulimit` the interpreter requests gigabytes mid-range and is OOM-killed; that has already ended a session and taken the machine's memory with it.
* **Read stdout, stderr and exit status as three separate descriptors.** Never capture `2>&1` as one string: the trace sink and stdout interleave undefinedly by design, and comparing them together produced two false regressions.
* Run `rexx-run` from `rust/`.

## Probes

* **Run every probe from a fresh empty subdirectory you `mkdir` yourself**, never the scratchpad root.
  The scratchpad is on the oracle's **external-routine search path** and holds hundreds of stale `.rex` files, so a probe calling any unresolved name finds one and executes it.
  Measured: the same three-line program reported `44.1 rc 212` from the root and `43.1 rc 213` from a clean directory.
* **Use absolute paths for every redirect.** A relative redirect inside a subshell that has `cd`'d writes where you did not intend; that produced three false mismatches against correct code and left stray files in the repository.
* **Never instantiate `.Package~new` on a file inside the repository** -- it executes that file's prolog and has written untracked files into the tree.
* **Never run `select; when 1 = 0 then; when 2 = 2 then nop; end`** -- it segfaults the oracle (upstream SF #2018).
* **Never set `NUMERIC DIGITS` above 1000** in a probe.
* A symbol named `x` or `b` immediately followed by a quoted string parses as a hex or binary literal, so `say '['x']'` is error 15.3 rather than concatenation. Use other names.

## Gates

* **`cargo fmt --all --check`** from `rust/`.
  **Not** `cargo fmt --edition 2024 --check`: `cargo fmt` has no `--edition` flag, that spelling exits 2 before doing any work, and a task once ran it and reported formatting clean.
  `--edition 2024` belongs to the standalone `rustfmt` binary.
* **`cargo clippy --workspace --all-targets -- -D warnings`** must be clean.
* **Read every exit status unpiped, and confirm the command actually ran.** A pipeline reports the last command's status, and a misspelled flag exits non-zero before doing any work -- which reads as failure in one context and is silently skipped in another.

## Repository hygiene

* **No `unsafe`.** The workspace sets `unsafe_code = "forbid"`. If something appears to need it, stop and say so.
* **Never `git add -A`.** Stage the exact paths you changed. Never `git reset --hard`, never force-push.
* Scratch files live under the session scratchpad, **never in the repository**. Check `git status` before committing and remove anything stray.
* **Commit first, then read the hash back with `git log`**, then quote it. Hashes written from memory have gone into records wrong twice.
* Comments state the contract at the top and the reasoning at the decision point.
  Never delete a true comment to make a change easier -- but a comment that states something **false** must be corrected or removed, not hedged.
  Prefer `--` over an em-dash. There is no rule against semicolons in comments here.

## Method

Three habits, each earned the expensive way.

* **When tempted to explain why something cannot happen, run it instead.** Six confident "X cannot be reached" claims have been wrong here, every one a sound inference from a *true* premise missing a second premise nobody knew to look for. More care produces the same answer with more confidence; only running separates them.
* **A test that cannot fail is a defect.** Eight have shipped, two of them fixes for earlier such findings. For every assertion ask: what degenerate implementation satisfies this, and would deleting its subject leave it green?
* **Pair a refusal with its adjacent success.** Three times the test that made a fix *correct* was not the failing case but the neighbouring passing one, because that is what pins the rule to the property you think it is rather than to something coincidental.
