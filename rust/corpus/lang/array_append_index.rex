/* Where `append` puts things, which is NOT past the last slot.

   `ArrayClass::append` counts from `lastItem` -- the last OCCUPIED index --
   and `ArrayClass::empty` sets that field to zero.  Every case below has a
   trailing hole, and every one of them was wrong in this crate until the
   phase's own review found it, because every witness written during the phase
   appended to a DENSE array and a dense array is the one case where the two
   rules agree. */

say 'dense agrees' .Array~of('p','q','r')~append('s')

a = .Array~new(5)
say 'into an unfilled array' a~append('m') 'size' a~size 'items' a~items

b = .Array~of('p','q','r')
z = b~remove(3)
say 'over a removed last item' b~append('t') 'size' b~size 'indexes' b~allIndexes~makeString('L', ',')

c = .Array~of('p','q','r')
c~empty
say 'after empty' c~append('u') 'size' c~size 'indexes' c~allIndexes~makeString('L', ',')

d = .Array~new
d[5] = 'e'
say 'after a put past the end' d~append('x') 'size' d~size
e = .Array~new
e[5] = 'e'
y = e~remove(5)
say 'and after removing it again' e~append('w') 'size' e~size

f = .Array~new(4)
f[2] = 'm'
say 'past the last item, not the last slot' f~append('n') 'size' f~size 'indexes' f~allIndexes~makeString('L', ',')

/* `insert` counts from the same place, and shifts into slack the array
   already has rather than growing. */
g = .Array~new(4)
say 'insert into slack' g~insert('j') 'size' g~size
h = .Array~of('x','y')
say 'insert with no item still grows' h~insert 'size' h~size 'items' h~items
