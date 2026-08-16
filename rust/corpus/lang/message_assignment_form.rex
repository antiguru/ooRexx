/* The message-assignment form: q[1] = 2 sends []= with the assigned value as
 * its first argument. Phase 5a Task 5.
 *
 * A String answers [] and not []=, so the oracle's answer here is 97.1 naming
 * the assignment spelling. Measured, rc 159. This is what pins that the name
 * gains its = and that the form is a send at all rather than an assignment to
 * something.
 */

x = 'abc'
x[1] = 2
