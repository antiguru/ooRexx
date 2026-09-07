/* The far tier of a Queue's index bound.  Inside the allocated extent but
   past the last item is 93.966 (`queue_bounds.rex` ends on that one); past
   the extent entirely is 93.918, and this program ends on that. */

e = .Queue~new
e~queue('a')
say 'a default queue holds' e~items 'and its extent is ooRexx''s DefaultArraySize'

f = .Queue~new(50)
f~queue('a')
say 'a requested extent is honoured above the default' f~items

g = .Queue~new
do i = 1 to 20
  g~queue('i'i)
end
say 'and grows to fit the items' g~items
g~put('Z', 20)
say 'so the last item is writable' g~allItems~items

g~put('Z', 21)
say 'unreachable'
