/* Routine's own readers, and the three rows on this class that run code.
 *
 * ~call, ~callWith and ~'[]' are the only rows in this family that are not
 * readers, and each is sent TWICE to the same object -- a body that consumed
 * its program on first use would pass a one-send harness and fail here.
 *
 * WHAT THE CALL IS, and both halves are visible below rather than asserted:
 * it is a SUBROUTINE call, so PARSE SOURCE's second word inside the routine
 * is SUBROUTINE and not FUNCTION or METHOD; and it runs in the ROUTINE's own
 * package, so a second ::ROUTINE of this file resolves from inside it. A
 * routine that returns nothing answers nothing, and the 91.999 that follows
 * names the MESSAGE -- CALL or [] -- which is what says the refusal is the
 * send's and not the call's.
 *
 * THE PATH IS NEVER PRINTED, for corpus/lang/class_package.rex's reason.
 * A Routine compiled from source text is the row that separates ~package
 * from a reader of the running program: its package name is the executable's
 * own name, not this file's.
 *
 * NO ::ROUTINE EXTERNAL HERE. It is a Phase 7 gap that refuses at INSTALL
 * time, so one such directive would make every row above it unreachable on
 * this crate; the external arm of ~source and ~setSecurityManager is
 * witnessed on the Method side instead, where ::METHOD ... EXTERNAL
 * 'LIBRARY REXX ...' installs.
 */

parse source . . source

say 'A -- a ::ROUTINE reached through .ROUTINES'
r = .routines~RR
say 'class' r~class~id
say 'call-3' r~call(3)
say 'call-6-again' r~call(6)
say 'callWith' r~callWith(.Array~of(4))
say 'index' r~'[]'(5)
say 'call-no-arguments' r~call

say 'B -- what the callee sees'
d = .routines~DESCRIBE
say 'describe' d~call
say 'describe-again' d~call
say 'nested' .routines~OUTER~call(5)

say 'C -- ~source and ~package'
s = r~source
say 'source-class' s~class~id
say 'source-items' s~items
do i = 1 to s~items
  say '  <'s[i]'>'
end
p = r~package
say 'package-class' p~class~id
say 'package-name-is-the-source' (p~name == source)
say 'package-name-is-rexx' (p~name == 'REXX')

say 'D -- .Routine~new compiles, and the object it answers is a Routine'
n = .Routine~new('NEWR', 'return 42')
say 'new-class' n~class~id
say 'new-call' n~call
say 'new-call-again' n~call
ns = n~source
say 'new-source-class' ns~class~id
say 'new-source-items' ns~items
say 'new-source-1' '<'ns[1]'>'
np = n~package
say 'new-package-class' np~class~id
say 'new-package-name' np~name
say 'new-package-name-is-the-source' (np~name == source)
say 'new-annotations-class' n~annotations~class~id

say 'E -- an Array source, and a second send to what it built'
a = .Routine~new('LINES', .Array~of('use arg n', 'return n * n'))
say 'array-call' a~call(6)
say 'array-call-again' a~call(7)
say 'array-source-items' a~source~items

say 'F -- setSecurityManager answers the code object kind here too'
say 'declared' r~setSecurityManager
say 'compiled' n~setSecurityManager

say 'G -- a routine that answers nothing, and the message that reports it'
call refuses 'call-with-no-result'
call refuses 'index-with-no-result'
call refuses 'callWith-with-no-arguments'
call refuses 'callWith-a-short-list'
say 'a-string-argument-list' .routines~COUNTARGS~callWith('one')
say 'a-long-argument-list' .routines~COUNTARGS~call(1, 2, 3)
exit 0

refuses:
  use arg which
  signal on syntax name refused
  select
    when which == 'call-with-no-result' then say .routines~NORET~call
    when which == 'index-with-no-result' then say .routines~NORET~'[]'()
    when which == 'callWith-with-no-arguments' then say .routines~RR~callWith
    when which == 'callWith-a-short-list' then say .routines~RR~callWith()
    otherwise say .routines~RR~call(1, 2, 3)
  end
  say which 'WAS NOT REFUSED'
  return
refused:
  say which 'refused' rc
  return

::ROUTINE RR
  if arg() = 0 then return 'no-argument'
  use arg n
  return n * 2

::ROUTINE DESCRIBE
  parse source platform kind .
  return platform kind arg()

::ROUTINE OUTER
  use arg n
  return INNER(n) + 1

::ROUTINE INNER
  use arg n
  return n * 10

::ROUTINE COUNTARGS
  return arg()

::ROUTINE NORET
  return
