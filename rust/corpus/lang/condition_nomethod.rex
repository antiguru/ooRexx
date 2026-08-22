/* The step beneath UNKNOWN: a receiver whose behaviour answers neither the
 * message nor UNKNOWN raises the NOMETHOD condition, and only a miss that
 * nothing traps becomes the 97.1 syntax error.
 *
 * The two are different answers a program can tell apart, which is what the
 * first three routines are for. A trapped NOMETHOD reports CONDITION('C') as
 * NOMETHOD, CONDITION('D') as the message name, CONDITION('E') as the null
 * string and leaves RC untouched; the same send under a SIGNAL ON SYNTAX
 * alone reports SYNTAX, no description, E as 1, and RC as 97.
 *
 * SIGNAL ON only. A CALL ON ANY trap does not take it -- there is nothing to
 * resume into once a clause has failed -- so callonany's inner routine falls
 * through to the degraded syntax error and its caller's SYNTAX handler takes
 * it.
 *
 * The offer crosses activations before the degradation happens, which is the
 * pair outerwins and innersyntax make: with a caller's NOMETHOD handler
 * armed, that handler runs even though the callee has a SYNTAX handler of its
 * own, and with no NOMETHOD armed anywhere the same callee's SYNTAX handler
 * takes the degraded error instead. A build that degraded the condition
 * before offering it to the whole stack answers the second row for both.
 *
 * Measured, rc 0.
 */

call byname
call byany
call bysyntax
call outerwins
call innersyntax
call callonany
say 'done'
exit 0

byname:
  signal on nomethod name nm
  say 'byname' 'abc'~nosuchmsg
  return
nm:
  say 'nm C='condition('C') 'D='condition('D') 'E=['condition('E')']',
      'I='condition('I') 'rc=['rc']'
  return

byany:
  signal on any name an
  say 'byany' .plain~zork
  return
an:
  say 'an C='condition('C') 'D='condition('D') 'S=['condition('S')']'
  return

bysyntax:
  signal on syntax name sy
  say 'bysyntax' 5~nosuchmsg
  return
sy:
  say 'sy C='condition('C') 'D=['condition('D')']' 'E=['condition('E')']',
      'rc=['rc']'
  return

outerwins:
  signal on nomethod name ow
  call innerraise
  return
ow:
  say 'ow C='condition('C') 'D='condition('D')
  return

innerraise:
  signal on syntax name irs
  say 'innerraise' 'abc'~nosuchmsg
  return
irs:
  say 'irs reached'
  return

innersyntax:
  call innerraise
  return

callonany:
  signal on syntax name cs
  call callonanyinner
  return
cs:
  say 'cs C='condition('C') 'rc=['rc']'
  return

callonanyinner:
  call on any name coa
  say 'callonanyinner' 'abc'~nosuchmsg
  return
coa:
  say 'coa reached'
  return

::class plain
