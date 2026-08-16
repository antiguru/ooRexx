/* A primitive method's argument with no string value is error 88.909, still
 * with the method's own traceback line. Phase 5a Task 5.
 *
 * .nil is the only value a program can write that has no string value: a
 * number has one, measured -- 'abc'~hasMethod(5) answers 0 rather than
 * raising. Measured, rc 168.
 */

say 'abc'~hasMethod(.nil)
