/* ABBREV and COMPARE as methods: `AddMethod("Abbrev", RexxString::abbrev, 2)`
 * and `AddMethod("Compare", RexxString::compare, 2)`
 * (memory/Setup.cpp:612, :616), `classes/StringClassMisc.cpp:75` and `:159`.
 *
 * ABBREV is case-sensitive and answers the text 1 or 0, not a boolean object,
 * so the answer goes on to arithmetic. Its length argument is a MINIMUM: an
 * abbreviation shorter than it is not one. An empty candidate abbreviates
 * anything when the minimum is zero, and nothing when it is not.
 *
 * COMPARE answers the 1-based offset of the first byte that differs, or 0 for
 * equal. The shorter operand is padded, so the answer does not depend on which
 * side is longer.
 */

p = 'Print'
say p~abbrev('Pri') p~abbrev('PRI') p~abbrev('Print') p~abbrev('Prints')
say p~abbrev('') p~abbrev('', 0) p~abbrev('', 1) p~abbrev('Pri', 3)
say p~abbrev('Pri', 4) p~abbrev('Pri', 0) ''~abbrev('') ''~abbrev('x')
say p~abbrev('Pri') + 1 datatype(p~abbrev('Pri'))
say 'abc'~compare('abc') 'abc'~compare('abd') 'abc'~compare('abcd')
say 'abcd'~compare('abc') 'abc'~compare('abc  ') 'abc  '~compare('abc')
say 'abc'~compare('abcxx', 'x') 'abcxx'~compare('abc', 'x') 'abc'~compare('abcxy', 'x')
say ''~compare('') ''~compare('  ') ''~compare('x') 'a'~compare('')
