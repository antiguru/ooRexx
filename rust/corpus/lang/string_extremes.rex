/* MAX and MIN as methods: `AddMethod("Max", RexxString::Max, A_COUNT)` and
 * `AddMethod("Min", RexxString::Min, A_COUNT)` (memory/Setup.cpp:646-:647).
 *
 * A_COUNT, so the whole argument list reaches the body and nothing is refused
 * ahead of it. The receiver is the first value compared, so an empty list
 * answers the receiver.
 *
 * A tie keeps the EARLIER value, which is `maxMin`'s own strict comparison,
 * and the answer is that value's own rendering rather than a recomputation --
 * which is what the '5' against '5.0' rows below separate.
 *
 * Every receiver here is written as a quoted string. `5~max(,7)` -- an
 * integer literal -- is a different class's method in the oracle and is left
 * out on purpose; `integer_object`'s doc in builtin/numeric.rs records what
 * this crate can and cannot tell apart there.
 */

say '5'~max(3) '5'~max(3, 9, 7) '5'~max '5'~max(9)
say '5'~min(3) '5'~min(3, 9, 7) '5'~min '5'~min(9)
say '-2'~max('-7') '-2'~min('-7') '0'~max('-0') '0'~min('-0')
say '5'~max('5.0') '5.0'~max('5') '5'~min('5.0') '5.0'~min('5')
say '1E3'~max(999) '1E3'~min(999) '0.5'~max('0.50')
numeric digits 3
say '1.23456'~max('1.2') '1.23456'~min('1.2') '123456'~max(1)
