/* BITAND, BITOR and BITXOR as methods: `AddMethod("BitAnd", RexxString::bitAnd,
 * 2)` and its two siblings (memory/Setup.cpp:631-:633).
 *
 * The operation runs byte for byte over the two strings, and the SHORTER one
 * is extended with the pad -- so the longer string's tail passes through the
 * pad rather than being dropped. Each operation's default pad is its own
 * identity: 'ff'x for BITAND, '00'x for BITOR and BITXOR. That is why an
 * omitted operand leaves the receiver alone, and why supplying a pad of '00'x
 * to BITAND is not the same as omitting one.
 *
 * Answers are raw bytes, read back here through C2X.
 */

a = '13'x
b = '11'x
w = '1311'x
say a~bitAnd(b)~c2x a~bitOr(b)~c2x a~bitXor(b)~c2x
say w~bitAnd(b)~c2x w~bitOr(b)~c2x w~bitXor(b)~c2x
say b~bitAnd(w)~c2x b~bitOr(w)~c2x b~bitXor(w)~c2x
say w~bitAnd(b, '00'x)~c2x w~bitOr(b, 'ff'x)~c2x w~bitXor(b, 'ff'x)~c2x
say 'abc'~bitAnd~c2x 'abc'~bitOr~c2x 'abc'~bitXor~c2x
say ''~bitAnd(b)~c2x ''~bitOr(b)~c2x b~bitAnd('')~c2x
say 'ffff'x~bitAnd('00'x)~c2x 'ffff'x~bitAnd('00'x, '00'x)~c2x
