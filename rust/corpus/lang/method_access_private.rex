/* PRIVATE is decided by who is sending, and the input is the caller's own
 * receiver rather than the caller's scope.
 *
 * The sending and the receiving object being the same object is allowed
 * outright, which is what answers inside (self~m from a method of the
 * declaring class) and subself (self~m from a subclass's own class method).
 * A rule written as "only the defining scope may send" answers the first and
 * refuses the second, so the two together are what pin the rule.
 *
 * subexplicit and superexplicit are the other allowing limb: a class object
 * whose own hierarchy contains the scope that defined the method, reached
 * without the two objects being the same one.
 *
 * outside, sibling and fromroutine are the refusals. A program or routine
 * frame carries no receiver at all, and a sibling class in the same package
 * is a class object the defining scope's hierarchy does not contain. Each is
 * trapped so this program can carry every row; method_access_private_refused
 * is the same refusal untrapped, for its traceback bytes.
 *
 * A refused send is not an error raised at the send: it drops the method and
 * enters the receiver's own UNKNOWN, which is what unknownforward prints. The
 * refusal only becomes a report for a receiver that answers no UNKNOWN, and
 * then it is the NOMETHOD condition first, which trapped is what refused
 * prints -- 97.2 in place of 97.1 and the same condition items.
 *
 * Measured, rc 0.
 */

say 'inside' .K~outer
say 'subself' .Sub~poke
say 'subexplicit' .Sub~pokeBase
say 'superexplicit' .Base~pokeSub
call outside
call sibling
call fromroutine
say 'unknownforward' .U~m
call bynomethod
say 'done'
exit 0

outside:
  signal on syntax name refused
  say 'outside' .K~m
  return

sibling:
  signal on syntax name refused
  say 'sibling' .S~poke
  return

fromroutine:
  signal on syntax name refused
  say 'fromroutine' r()
  return

refused:
  say 'refused rc='rc 'C='condition('C') 'D=['condition('D')']',
      'E=['condition('E')']'
  return

bynomethod:
  signal on nomethod name took
  say 'bynomethod' .K~m
  return

took:
  say 'took C='condition('C') 'D='condition('D') 'E=['condition('E')']',
      'rc=['rc']'
  return

::CLASS K
::METHOD outer CLASS
  return self~m
::METHOD m CLASS PRIVATE
  return 'inner'

::CLASS S
::METHOD poke CLASS
  return .K~m

::CLASS Base
::METHOD m CLASS PRIVATE
  return 'baseinner'
::METHOD pokeSub CLASS
  return .Sub~m

::CLASS Sub SUBCLASS Base
::METHOD poke CLASS
  return self~m
::METHOD pokeBase CLASS
  return .Base~m

::CLASS U
::METHOD m CLASS PRIVATE
  return 'never'
::METHOD unknown CLASS
  use arg name, args
  return 'unknown saw' name 'with' args~items

::ROUTINE r
  return .K~m
