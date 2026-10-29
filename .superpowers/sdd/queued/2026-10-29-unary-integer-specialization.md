**OPEN, queued 2026-10-29 by the controller from Moritz's measurement; proposed perf round after 9d.**

# Prefix + and - on a small integer take the general decimal path

Moritz's measurement (2026-10-29), operand 12345, ns per assignment:

| Kernel | Oracle | Rust |
|---|---:|---:|
| b = a | 7.5 | 11.1 |
| b = +a, default precision | 13.2 | 65.8 |
| b = +a, explicit DIGITS 9 | 13.6 | 65.9 |
| b = +a, explicit DIGITS 20 | 13.6 | 66.1 |
| b = a + 0 | 26.3 | 16.1 |

Cause, read 2026-10-29: `Op::Prefix` (ir/drive.rs:960) calls `Interp::apply_prefix` (eval.rs:707), whose Plus/Minus arm converts the operand to a `Number` (`arith_left_operand`), runs `lostdigits_check`, computes `Number::zero().add/sub` and rebuilds the value with `self.number(...)` on every evaluation. The dyadic `Op::Arith` arm tries `arith_small_int` first under the site's hint (drive.rs:865), so `a + 0` stays on tagged integers and beats the oracle while `+a` is about five times slower than it. Fix: give `Op::Prefix` the same small-integer attempt (Plus answers the operand when it is a small integer within DIGITS; Minus negates with the range check `arith_small_int` uses), keeping the general path and its blame frame for everything else, and `\` unchanged. Witness the unchanged answers at DIGITS limits (a value with more digits than DIGITS, -0, the small-integer range edge) against the oracle; measure with callgrind on every program plus a unary kernel.
