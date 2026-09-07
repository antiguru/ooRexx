/* `ARG(n,'A')`: the call's own argument list as an array, from position n on.
   The list is copied pointer for pointer, so an omitted position arrives as an
   EMPTY SLOT -- not `.nil`, and not a shortened list. */

call three 'p1', , 'p3'
call none

/* Position 1 is validated the same way for every option, and the two bad
   positions keep their order in front of the option switch. */
signal on syntax name badPosition
zz = arg(0, 'A')
say 'unreached' zz
exit 1

badPosition:
  say 'position zero' rc condition('C') condition('E')
  signal on syntax name badType
  zz = arg('nan', 'A')
  say 'unreached' zz
  exit 1

badType:
  say 'position not a number' rc condition('C') condition('E')
  exit 0

three:
  whole = arg(1, 'A')
  say 'whole' whole~size whole~items whole~dimension whole~class~id
  say 'holes' whole~hasIndex(1) whole~hasIndex(2) whole~hasIndex(3)
  say 'values' '['whole[1]']' '['whole[2]']' '['whole[3]']'
  from2 = arg(2, 'A')
  say 'from 2' from2~size from2~items from2~dimension '['from2[1]']' '['from2[2]']'
  from3 = arg(3, 'A')
  say 'from 3' from3~size from3~items from3~dimension '['from3[1]']'
  past = arg(4, 'A')
  say 'past the end' past~size past~items past~dimension past~class~id
  far = arg(99, 'A')
  say 'far past the end' far~size far~items far~dimension
  say 'count' arg() 'exists' arg(2, 'E') 'omitted' arg(2, 'O')
  return

/* With no arguments at all, position 1 still answers a SHAPED array where any
   other position answers an unshaped one: the C++ tests position 1 before the
   past-the-end arm, and the two constructors differ in exactly that. */
none:
  atOne = arg(1, 'A')
  say 'none at 1' atOne~size atOne~items atOne~dimension atOne~class~id
  atTwo = arg(2, 'A')
  say 'none at 2' atTwo~size atTwo~items atTwo~dimension atTwo~class~id
  return
