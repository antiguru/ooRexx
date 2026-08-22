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
 * Whose trap table decides is the running activation's, and the last three
 * routines are the three answers that follow from it. An internal CALL
 * inherits its caller's traps, so outerwins' NOMETHOD handler takes a miss
 * raised inside innerraise even though innerraise armed a SYNTAX handler of
 * its own, and innersyntax calls the same routine with no NOMETHOD armed
 * anywhere and gets that SYNTAX handler instead. A method activation inherits
 * no traps at all, so methodmiss arms both and gets the SYNTAX one: the
 * NOMETHOD condition is never raised, because nothing the running activation
 * can see would take it. A build that asked the whole activation stack
 * instead answers the NOMETHOD handler for that last row.
 *
 * Measured, rc 0.
 */

call byname
call byany
call bysyntax
call outerwins
call innersyntax
call callonany
call methodmiss
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

methodmiss:
  signal on nomethod name mmn
  signal on syntax name mms
  say 'methodmiss' .k~m
  return
mmn:
  say 'mmn reached'
  return
mms:
  say 'mms C='condition('C') 'rc=['rc']'
  return

::class plain

::class k
::method m class
  return 'inner:' .plain~zork
