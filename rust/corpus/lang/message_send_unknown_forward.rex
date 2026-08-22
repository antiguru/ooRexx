/* UNKNOWN is the search order's last step before NOMETHOD: a name the
 * receiver's behaviour does not answer is forwarded to the receiver's own
 * UNKNOWN, with the missed name as the first argument and an array of the
 * original message's arguments as the second.
 *
 * The sibling of message_send_unknown_method.rex, which is the other branch
 * of the same fork: 'abc' answers no UNKNOWN, so its miss raises 97.1 where
 * this program's receivers answer.
 *
 * Where the forward's result goes decides the exit status, so the two arms
 * are written differently on purpose. Class K's UNKNOWN returns, and its
 * rows are expression positions; class J's only says, and its rows are whole
 * clauses -- `say .j~zork(1,2)` over a method that returns nothing is
 * Error 91.999 at rc 165, because SAY demands a result.
 *
 * The array is the argument list as the send holds it, omissions included:
 * (1,,3) reaches UNKNOWN as ~items 2 and ~size 3, while a trailing omission
 * in a call's argument list is no slot at all.
 *
 * The forward is found the way every other name is, so a subclass inherits
 * its superclass's UNKNOWN and a name the subclass does answer never reaches
 * it. ~~ yields the target rather than the forward's result, and the
 * message-assignment form arrives as the name with its trailing = and the
 * assigned value as the array's one item.
 *
 * The last row's message name is longer than a handle holds, so building the
 * first of the forward's two arguments allocates and the second one has to
 * survive it. That is what makes this program the witness for the arguments
 * array's rooting: with collect_stress.rs's collect-on-every-allocation and
 * the array's root removed, a name of seven bytes or fewer never allocates
 * there and the whole subset still passes.
 *
 * Measured, rc 0.
 */

say .k~zork(1, 2)
say .k~zork
.j~zork(1, 2)
.j~zork(1, , 3)
.j~zork(1, )
.j~zork()
say .sub~m
say .sub~zork
say (.k~~zork(9))~id
.j~zork = 'v'
say .k~aVeryLongMissingMessageName(1, 2)

::class k
::method unknown class
  use arg n, a
  return 'unknown:' n 'items' a~items

::class j
::method unknown class
  use arg n, a
  say 'unknown:' n 'args' a~items 'size' a~size

::class base
::method m class
  return 'base m'
::method unknown class
  use arg n, a
  return 'base unknown:' n

::class sub subclass base
