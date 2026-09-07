/* A Stem's store is the language's tails, and the collection surface has to
 * respect the one thing `Body::Stem` already models: a DROPPED tail is not
 * an absent one.
 *
 * `at` has three answers, not two.  A tail that holds something answers it; a
 * dropped tail answers its own derived name, because the tombstone is still
 * there; and a tail never assigned answers the stem's default.  `items`,
 * `hasIndex` and `index` count and find only the first kind.
 *
 * `empty` DELETES rather than drops, which is the difference the last block
 * measures: after it, a tail reads as the default and not as its derived
 * name.  Writing tombstones there would answer `U.K` where the oracle
 * answers `5`.
 *
 * `allIndexes`, `allItems`, `makeArray` and `supplier` are deliberately
 * absent: they answer tails in the stem's own table order, which is a
 * different geometry from the one the mapped classes use and which this
 * crate does not yet reproduce -- see the Task 5 report.  A witness that
 * asserted them would be asserting an order that is wrong.
 */

s. = 'dflt'
s.a = 1
s.b = 2
s.c = 3
drop s.b
obj = s.
say 'items   ' obj~items obj~isEmpty
say 'at      ' obj['A'] obj['B'] obj['ZZ']
say 'hasIndex' obj~hasIndex('A') obj~hasIndex('B') obj~hasIndex('ZZ')
say 'hasItem ' obj~hasItem(1) obj~hasItem(3) obj~hasItem(99)
say 'index   ' obj~index(3) obj~index(99)
obj['D'] = 4
say 'put     ' obj['D'] obj~items obj~hasIndex('D')
say 'remove  ' obj~remove('A') obj~items obj~hasIndex('A') obj['A']
say 'again   ' obj~remove('A')
say 'removeIt' obj~removeItem(3) obj~items obj~hasItem(3)
say 'no match' obj~removeItem(99)

/* Compound tails: the subscripts join with a dot, as a tail does. */
c.x.y = 'deep'
cobj = c.
say 'compound' cobj['X','Y'] cobj~hasIndex('X','Y') cobj~hasIndex('X')

/* empty deletes, so the default comes back. */
u. = 5
u.k = 1
uobj = u.
say 'before  ' uobj~items uobj['K']
uobj~empty
say 'after   ' uobj~items uobj~isEmpty uobj['K']

/* A stem with no default reads an absent tail as its own derived name. */
n.j = 1
nobj = n.
say 'no dflt ' nobj['ZZ'] nobj~hasIndex('ZZ')

/* Stem's argument rules are the loosest of any class in this phase: a missing
 * subscript names the stem itself rather than being an error.
 *
 * `hasItem()` and `index()` with no argument are deliberately absent.  Each
 * is a SIGSEGV on the oracle when the stem holds a tail -- it is in
 * corpus/oracle-crashes.txt -- so this crate's answers for them (0 and .nil,
 * measured against an EMPTIED stem, where the oracle survives) cannot be
 * checked differentially at all and are not asserted here.
 */
bare.a = 1
bobj = bare.
say 'bare has ' bobj~hasIndex()
say 'bare at  ' bobj~at()
say 'bare rem ' bobj~remove() bobj~items
say 'bare emp ' (bobj~empty == bobj)

signal on syntax name trapped
n = 0

next:
n = n + 1
select
  when n = 1 then say n 'answered' bobj~removeItem()
  when n = 2 then say n 'answered' bobj~items()
  when n = 3 then say n 'answered' bobj~hasIndex('NOPE')
  otherwise signal done
end
signal next

trapped:
say n 'raised' rc'.'condition('E')
signal on syntax name trapped
signal next

done:
say 'end'
