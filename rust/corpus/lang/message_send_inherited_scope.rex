/* ~hasMethod reads the receiver's own flattened behaviour back, which is
 * what a send resolves against. Phase 5a Task 5.
 *
 * The argument is upcased before the lookup, measured: 'abc'~hasMethod('length')
 * is 1. The last two lines are the pair that makes the answer mean something
 * -- LENGTH is String's and is absent from .nil's Object behaviour, so a
 * receiver-independent answer fails one of them.
 *
 * Measured, rc 0.
 */

say 'abc'~hasMethod('LENGTH')
say 'abc'~hasMethod('length')
say 'abc'~hasMethod('NOSUCHMETHOD')
say 'abc'~hasMethod('ISNIL')
say .nil~hasMethod('STRING')
say .nil~hasMethod('LENGTH')
