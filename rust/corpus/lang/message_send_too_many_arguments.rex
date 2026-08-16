/* A primitive method handed more arguments than it declares is error 93.902,
 * with the method's own traceback line above the sending clause. Phase 5a
 * Task 5.
 *
 * The count the message names is the count the method DECLARES, not the count
 * that arrived, and the scope is the class the definition came from -- LENGTH
 * is String's. Measured, rc 163.
 */

say 'abc'~length(1)
