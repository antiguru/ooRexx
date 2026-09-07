/* A `Stem`'s three forwarding methods.  All three turn on the same field: a
 * stem has a VALUE of its own, separate from its tails, and an unassigned
 * stem's value is its own derived name -- so `a.~length` is 2, not 0.
 *
 * `request('ARRAY')` answers the stem's `makeArray`, which for a Stem is its
 * TAILS rather than its items, walked in the tail tree's own post-order.
 * Every other class name is forwarded to the value, so `request('STRING')`
 * answers what the value would.  The name is upper-cased, and a missing one
 * is the POSITIONAL 93 rather than the named 88.
 *
 * `toDirectory` answers a Directory holding one entry per tail that HAS a
 * value: a removed tail is absent from it, as it is from `makeArray`.  The
 * order of the answer is the DIRECTORY's, not the stem's -- the names go in
 * in tail order and come back in the store's.
 *
 * `unknown` forwards the message and its arguments to the value, which is
 * how a stem answers `length`, `upper` and everything else it has no method
 * for.
 */

s = .Stem~new
s['k1'] = 'v1'
s['k2'] = 'v2'
say 'value   ['s~string']'
say 'array   ' s~request('ARRAY')~makeString('L',',')
say 'lower   ' s~request('array')~class~id
say 'string  ['s~request('STRING')']'
say 'other   ['s~request('DIRECTORY')']'

d = s~toDirectory
say 'todir   ' d~class~id d~items d~allIndexes~makeString('L',',') d~allItems~makeString('L',',')

say 'unknown ' s~length s~unknown('LENGTH', .Array~new)

/* a stem VARIABLE carries both a name and a value */
q. = 'hello'
q.x = 9
q.y = 8
say 'named   ['q.~string']' q.~length
say 'n array ' q.~request('ARRAY')~makeString('L',',')
say 'n string['q.~request('STRING')']'
qd = q.~toDirectory
say 'n todir ' qd~items qd~allIndexes~makeString('L',',') qd~allItems~makeString('L',',')

/* an unassigned stem's value is its own derived name */
a.b = 1
say 'derived ['a.~string']' a.~length a.~upper

/* a removed tail is gone from both views */
u = .Stem~new
u['a'] = 1
u['b'] = 2
u~remove('a')
say 'removed ' u~toDirectory~items u~toDirectory~allIndexes~makeString('L',',') u~request('ARRAY')~makeString('L',',')

/* the index argument is positional */
signal on syntax name noarg
z = u~request()
noarg:
say 'no arg  ' rc
