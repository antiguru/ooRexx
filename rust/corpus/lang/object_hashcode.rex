/* Object~hashCode: `AddMethod("HashCode", RexxObject::hashCode, 0)`
 * (memory/Setup.cpp:523), which is getHashValue() rendered as its own eight
 * bytes, little-endian.
 *
 * getHashValue is virtual. NilObject, String, Pointer, Integer, NumberString
 * and Class override it with a value; every other receiver takes the identity
 * hash, which is the complement of an address and cannot be compared across
 * processes. So the identity rows below print RELATIONS rather than values --
 * that a hash equals itself, that two objects differ, and that the width is
 * eight -- all of which hold on either side.
 *
 * The string hash accumulates `31 * h + byte` in a wrapping 64-bit register
 * with the byte SIGNED, which only shows above 0x7f.
 *
 * DateTime and TimeSpan reach this through their own Rexx-level hashCode in
 * CoreClasses.orx, which returns `timestamp~hashcode` -- a number's hash, so
 * two equal instants agree and two different ones do not.
 */

say 'null' c2x(''~hashCode)
say 'one'  c2x('a'~hashCode)
say 'abc'  c2x('abc'~hashCode)
say 'wrap' c2x('abcdefghijklmnopqrstuvwxyz0123456789'~hashCode)
say 'high' c2x('ff'x~hashCode) c2x('80'x~hashCode) c2x('7f'x~hashCode)
say 'nil'  c2x(.nil~hashCode)
say 'cls'  c2x(.String~hashCode) c2x(.Object~hashCode)
say 'num'  c2x(55~hashCode) c2x('55'~hashCode)
say 'rend' (c2x((2**40)~hashCode) == c2x('1.09951163E+12'~hashCode))
say 'wide' length('abc'~hashCode) length(.Object~new~hashCode)

d1 = .DateTime~fromIsoDate('2020-01-01T00:00:00.000000')
d2 = .DateTime~fromIsoDate('2020-01-01T00:00:00.000000')
d3 = .DateTime~fromIsoDate('2021-06-15T12:30:00.000000')
say 'dt'   c2x(d1~hashCode) (c2x(d1~hashCode) == c2x(d2~hashCode)) (c2x(d1~hashCode) == c2x(d3~hashCode))
say 'ts'   c2x(.TimeSpan~fromSeconds(5)~hashCode)

/* The identity arm, as relations only. Two buffers holding equal text hash
 * differently, which is what says the rule is the receiver's kind and not
 * whether it renders. */
o1 = .Object~new
o2 = .Object~new
b1 = .MutableBuffer~new('abc')
b2 = .MutableBuffer~new('abc')
say 'same' (c2x(o1~hashCode) == c2x(o1~hashCode))
say 'diff' (c2x(o1~hashCode) == c2x(o2~hashCode)) (c2x(b1~hashCode) == c2x(b2~hashCode))
say 'kind' (c2x(b1~hashCode) == c2x('abc'~hashCode))
