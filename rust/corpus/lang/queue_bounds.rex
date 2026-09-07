/* A Queue's size is its ITEM count -- Setup.cpp maps `size` to
   `ArrayClass::itemsRexx` -- and its insertion index is bounded by the last
   item where an Array's extends to meet it.

   The bound is TWO-TIER and so are the errors.  `putRexx` validates the index
   against the ALLOCATED extent first (93.918 past it), then calls
   checkInsertIndex for a position inside the extent but past the last item
   (93.966).  The extent is the larger of what the queue was built with,
   floored at ooRexx's DefaultArraySize of 16, and what it now holds. */

q = .Queue~new
q~queue('a')
q~queue('b')
say 'before' q~items q~size
q~empty
say 'after empty' q~items q~size q~isEmpty
q~queue('z')
say 'and queueing starts again at the front' q~items q~size q~peek q~pull

r = .Queue~new
r~queue('a')
r~queue('b')
say 'insert at the last item is allowed' r~insert('k', 2) 'now' r~allItems~makeString('L', ',')

call bounded r, 'INSERT'
call bounded r, 'PUT'
say 'both refused'

/* Inside the extent but past the last item, against past the extent. */
e = .Queue~new
e~queue('a')
call bounded2 e, 16, 'inside the default extent'
call bounded2 e, 17, 'past the default extent'
f = .Queue~new(50)
f~queue('a')
call bounded2 f, 50, 'inside a requested extent'
call bounded2 f, 51, 'past a requested extent'
g = .Queue~new(5)
g~queue('a')
call bounded2 g, 6, 'a request below the default is floored at it'
say 'all five refused'

s = .Queue~new
s~queue('a')
s~queue('b')
s~put('n', 4)
say 'unreachable'
exit

bounded2: procedure
  use arg target, at, what
  signal on syntax name oops2
  target~put('Z', at)
  say what 'accepted'
  return
oops2:
  say what 'raised' rc
  return

bounded: procedure
  use arg target, name
  signal on syntax name oops
  v = target~send(name, 'z', 9)
  say name 'did not refuse'
  return
oops:
  say name 'past the last item raised' rc condition('C')
  return
