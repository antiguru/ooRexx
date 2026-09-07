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
 * `allIndexes`, `allItems`, `makeArray` and `supplier` answer tails in the
 * stem's own table order, which is neither insertion order nor sorted: the
 * tails live in a balanced binary tree keyed on (length, bytes) and walked in
 * POST-order.  The order therefore depends on the shape the insertions built,
 * so the two blocks at the end insert the same five tails forwards and
 * backwards and get different answers -- which is the assertion that a
 * store keeping insertion order, or one sorting the names, would fail.
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

/* The tail order, which is the tree's post-order walk. */
say 'order   ' obj~allIndexes~makeString('L',',') '/' obj~allItems~makeString('L',',')
say 'makeArr ' obj~makeArray~makeString('L',',')
osup = obj~supplier
do while osup~available
  say '  pair' osup~index osup~item
  osup~next
end

f.zebra = 1
f.apple = 2
f.mango = 3
f.q = 4
f.longkeyname = 5
say 'forward ' (f.)~allIndexes~makeString('L',',')
r.longkeyname = 5
r.q = 4
r.mango = 3
r.apple = 2
r.zebra = 1
say 'reversed' (r.)~allIndexes~makeString('L',',')

do i = 1 to 25; g.i = i; end
say 'grown   ' (g.)~allIndexes~makeString('L',',')

/* And the operations built on that order. */
u.b = 2
u.c = 3
uobj = u.
say 'union   ' obj~union(uobj)~items
say 'xor     ' obj~xor(uobj)~items
say 'isect   ' obj~intersection(uobj)~items
say 'diff    ' obj~difference(uobj)~items
say 'subset  ' uobj~subset(obj) 'disjoint' obj~disjoint(uobj)

/* The method path uses the tail EXACTLY as written; only the language path
 * upper-cases, because a symbol in `s.a` is upper-cased when it is parsed and
 * a subscript in `obj['a']` is a string.  Every line below separates the two,
 * and none of the lines above can: they all use upper-case literals, which is
 * why an implementation that upper-cased here passed all of them. */
c.a = 1
cobj2 = c.
say 'read    ' cobj2['A'] cobj2['a']
say 'hasIndex' cobj2~hasIndex('A') cobj2~hasIndex('a')
cobj2['b'] = 2
say 'written ' cobj2~allIndexes~makeString('L',',')
say 'cases   ' cobj2['b'] cobj2['B'] c.b
say 'built   ' (.Stem~new~~put('v1','k1')~~put('v2','k2'))~allIndexes~makeString('L',',')
