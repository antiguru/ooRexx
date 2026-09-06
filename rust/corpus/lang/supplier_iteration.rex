/* Supplier, which every `supplier` row in both collection phases lands on.
   `.Supplier~new(items, indexes)` takes the ITEMS first: the pairs below are
   what settles that, and getting it backwards is invisible until something
   asks for both halves of one pair.  The last send is left to escape because
   only stderr carries the error's decimal part -- `condition('O')` answers a
   Directory this crate does not build yet. */

s = .Supplier~new(.Array~of('i1', 'i2', 'i3'), .Array~of('x1', 'x2', 'x3'))
do while s~available
  say 'pair' s~index s~item
  s~next
end
say 'exhausted available' s~available

call trapped 'ITEM'
call trapped 'INDEX'
call trapped 'NEXT'

/* Mismatched lengths are accepted rather than checked at construction. */
z = .Supplier~new(.Array~of('only'), .Array~of('k1', 'k2'))
say 'mismatched available' z~available 'index' z~index 'item' z~item

/* An empty supplier is exhausted from the start, and a sparse array's
   supplier skips the holes rather than pairing them with .nil. */
e = .Supplier~new(.Array~new, .Array~new)
say 'empty available' e~available

a = .Array~new
a[1] = 'first'
a[4] = 'fourth'
as = a~supplier
do while as~available
  say 'from array' as~index as~item
  as~next
end

v = e~item
say 'unreachable' v
exit

trapped: procedure expose s
  use arg name
  signal on syntax name oops
  v = s~send(name)
  say name 'did not raise'
  return
oops:
  say name 'past the end raised' rc condition('C')
  return
