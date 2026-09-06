/* List, whose index is a HANDLE and not a position.  That is the whole of
   this program: every other ordered collection here is addressed by where an
   item sits, and a List is addressed by a token that stays with the item as
   the list is inserted into and deleted from.  An implementation that makes
   the index a subscript passes every single-element test and fails the first
   insert. */

l = .List~new
ia = l~append('a')
ib = l~append('b')
ic = l~append('c')
say 'append answers handles' ia ib ic
say 'items' l~items 'allItems' l~allItems~makeString('L', ',')
say 'allIndexes' l~allIndexes~makeString('L', ',')

say 'at a handle' l~at(ib) 'and []' l[ib]
say 'first' l~first 'last' l~last 'firstItem' l~firstItem 'lastItem' l~lastItem
say 'next' l~next(ia) 'previous' l~previous(ic)
say 'past the ends' l~next(ic) l~previous(ia)

/* Inserting does not renumber: the new item gets a FRESH handle and every
   old handle still reaches the item it always did. */
ix = l~insert('x', ia)
say 'insert answers a fresh handle' ix
say 'order' l~allItems~makeString('L', ',')
say 'old handles still reach' l~at(ia) l~at(ib) l~at(ic)

/* And removing does not renumber either. */
say 'remove answers' l~remove(ib)
say 'order' l~allItems~makeString('L', ',')
say 'ia still reaches' l~at(ia) 'hasIndex ib now' l~hasIndex(ib)

say 'index of an item is its handle' l~index('x') 'hasItem' l~hasItem('x')
say 'section' l~section(ia, 2)~allItems~makeString('L', ',')
say 'section class' l~section(ia, 2)~class~id
say 'section of none' l~section(ia, 0)~items

l~put('Z', ia)
say 'put replaces at the handle' l~at(ia) 'order' l~allItems~makeString('L', ',')
l[ia] = 'W'
say '[]= likewise' l~at(ia)

say 'insert with no index appends, answering' l~insert('tail')
say 'insert at .nil prepends, answering' l~insert('head', .nil)
say 'order' l~allItems~makeString('L', ',')
say 'removeItem answers' l~removeItem('W') 'order' l~allItems~makeString('L', ',')
say 'delete answers' l~delete(ic) 'order' l~allItems~makeString('L', ',')

say 'makeArray' l~makeArray~makeString('L', ',')
say 'supplier' l~supplier~index l~supplier~item
say 'isEmpty' l~isEmpty

/* A handle the list does not hold is not an error for the readers. */
say 'unknown handle' l~at(99) l~hasIndex(99) l~remove(99)

e = .List~new
say 'empty' e~first e~last e~firstItem e~lastItem e~items e~isEmpty
say 'empty index' e~index('x') 'hasItem' e~hasItem('x') 'allItems' e~allItems~items

l~empty
say 'after empty' l~items l~isEmpty

/* Handles are RECYCLED, and last-freed-first.  Removing 1 and then 3 makes
   the next three appends answer 3, 1 and a fresh 5 -- so the free list is a
   stack and not a queue, and not "the lowest free handle" either. */
f = .List~new
hs = .Array~new
do i = 1 to 5
  hs[i] = f~append('i'i)
end
say 'handles' f~allIndexes~makeString('L', ',')
gone1 = f~remove(hs[2])
gone2 = f~remove(hs[4])
say 'after two removals' f~allIndexes~makeString('L', ',')
say 'reused in this order' f~append('p') f~append('q') f~append('r')
say 'and the order of items is unchanged' f~allItems~makeString('L', ',')

/* The argument NUMBER in a missing-index complaint is two, whatever the
   position in the argument list is: ListClass passes ARG_TWO to validateIndex,
   requiredIndex and validateInsertionIndex alike, so a bare `~at` reports
   argument 2 even though `at` takes one argument. */
g = .List~new
call missing g, 'AT'
call missing g, 'PUT'

/* `insert` needs neither argument, and `empty` answers the receiver. */
say 'bare insert answers' .List~new~insert
say 'empty answers a' .List~new~empty~class~id

/* But it IS an error for `put`, and the number is the one `Queue~put` uses. */
l2 = .List~new
zz = l2~append('only')
l2~put('Q', 99)
say 'unreachable'
exit

missing: procedure
  use arg target, name
  signal on syntax name oops
  v = target~send(name)
  say name 'did not raise'
  return
oops:
  say name 'with no index raised' rc condition('C')
  return
