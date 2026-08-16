/* >M>: a message send's own result line, tagged with the message name and
 * quoted -- unlike >F>, whose tag is bare.
 *
 * Four sends at three positions: inside SAY, at the right of an assignment,
 * as a whole clause, and as a whole clause in the ~~ form whose >M> shows
 * the target rather than the method's result. The argument-bearing send is
 * what puts an >A> line between the receiver's own >L> and the >M>.
 */
trace i
say 'abc'~length
zz = 'abc'~reverse
'abc'~length
'abc'~~length
say 'abc'~hasMethod('LENGTH')
exit
