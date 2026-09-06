/* Array's own navigation surface.  `first`/`last` answer an INDEX and
   `firstItem`/`lastItem` answer the item, which is the pair most likely to be
   implemented as one another; the array below is sparse so the two cannot
   coincide. */

a = .Array~new
a[1] = 'p'
a[3] = 'r'
a[5] = 't'

say 'first' a~first 'last' a~last
say 'firstItem' a~firstItem 'lastItem' a~lastItem
say 'next occupied' a~next(1) a~next(3) a~next(5)
say 'previous occupied' a~previous(5) a~previous(3) a~previous(1)

/* From an index that holds nothing, the walk still finds the neighbour. */
say 'next from a hole' a~next(2) a~next(4)
say 'previous from a hole' a~previous(2) a~previous(4)

/* Past the end is .nil rather than an error; index 0 is not, so the start
   case is `previous(1)` above. */
say 'next past end' a~next(9) a~next(100)

e = .Array~new
say 'empty' e~first e~last e~firstItem e~lastItem

d = .Array~of('x', 'y', 'z')
say 'dense' d~first d~last d~firstItem d~lastItem
say 'dense next' d~next(1) d~next(3)
