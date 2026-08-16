/* A message send resolves against the receiver's own native class and runs
 * a primitive method. Phase 5a Task 5.
 *
 * One send per value kind this crate can build: a short literal (the bytes
 * live in the handle), a long one (a heap string), a whole number, and a
 * number with a fraction. All four answer the String class -- measured,
 * 12345~class~id and (1.5)~class~id are both "String" -- so all four reach
 * the same LENGTH method through the same behaviour.
 *
 * ISNIL is defined by Object rather than by String, so the last three sends
 * resolve through the flattened cascade rather than out of the receiver's
 * own class dictionary: .nil answers Object directly (measured,
 * .nil~class~id is "Object") and 'abc' reaches the same method from String.
 *
 * Measured, rc 0.
 */

say 'abc'~length
say 'abcdefghijklmno'~length
say 12345~length
say (1.5)~length
say 'abc'~reverse
say .nil~isnil
say 'abc'~isnil
