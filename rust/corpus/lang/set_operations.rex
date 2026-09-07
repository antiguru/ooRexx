/* A Set's index is its value, and its two overrides route to the index
 * bodies.  `put` takes the index from the value, an index that is given must
 * equal it, and one that does not is 93.949.
 *
 * The set operations below are the reason this file is not just a `put`
 * test.  `SetMixin` installs `union xor intersection difference subSet` at
 * the class's own scope (`RexxClasses/CoreClasses.orx:85`), each of them
 * Rexx running over the native surface, and `union` is `self~copy` followed
 * by a loop of `put`.  A `copy` that hands back the same store makes `union`
 * mutate its receiver, which is invisible until a later operation reads the
 * receiver again -- so the order here matters: `union` runs first and
 * `intersection` after it, and the two `a contents` lines either side are
 * what a witness that only checked counts would miss.
 *
 * The last send is untrapped so the 93.949 text and the frame line are
 * compared as bytes.  rc 163.
 */

s = .Set~new
s~put('a')
say 'one     ' s~items
s~put('a')
say 'again   ' s~items
s~put('b','b')
say 'indexed ' s~items
say 'lookups ' s~hasIndex('a') s~hasItem('a') s~hasIndex('z') s~hasItem('z')
say 'at      ' s['a'] s~index('a')
say 'all     ' s~allIndexes~makeString('L',',') '/' s~allItems~makeString('L',',')
say 'removeIt' s~removeItem('a') s~items
say 'of      ' .Set~of('x','y')~items .Set~of('x','x')~items
say 'makeArr ' .Set~of('p','q')~makeArray~makeString('L',',')

a = .Set~of('x','y','z')
b = .Set~of('y','z','w')
say 'a before' a~allIndexes~makeString('L',',')
say 'union   ' a~union(b)~allIndexes~makeString('L',',')
say 'a after ' a~allIndexes~makeString('L',',')
say 'isect   ' a~intersection(b)~allIndexes~makeString('L',',')
say 'diff    ' a~difference(b)~allIndexes~makeString('L',',')
say 'xor     ' a~xor(b)~allIndexes~makeString('L',',')
say 'subset  ' .Set~of('y')~subset(a) a~subset(a)
say 'disjoint' a~disjoint(b) a~disjoint(.Set~of('q'))
say 'equiv   ' a~equivalent(a) a~equivalent(b)
c = .Set~of('x')
c~putAll(b)
say 'putAll  ' c~items

/* The same aliasing, one layer down: a copy of an ordered collection has a
   store of its own too. */
q = .Queue~of('a','b')
qc = q~copy
qc~queue('z')
say 'queue   ' q~items qc~items
l = .List~of('a','b')
lc = l~copy
lc~append('z')
say 'list    ' l~items lc~items
t = .Table~new
t['k'] = 1
tc = t~copy
tc['j'] = 2
say 'table   ' t~items tc~items

s~put('c','d')
