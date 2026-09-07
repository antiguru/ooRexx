/* `.Method~new(name, source)` compiles a method object from source text --
 * the same compiler `Object~setMethod` reaches when its second argument is a
 * string, so the only new thing here is the class-side door to it.
 *
 * The object it answers carries no scope: a method built this way belongs to
 * no class until something installs it, and `~scope` is `.nil` until then.
 * The source may be a string or an array of source lines.
 *
 * One method object can be installed into more than one receiver, and each
 * runs it against its own receiver.
 *
 * Both arguments are required, and each missing one is reported by name.
 */

m = .Method~new('MM', 'return 7')
say 'built   ' m~class~id m~scope
say 'annotate' m~annotations~class~id

d = .Directory~new
d~setMethod('MM', m)
say 'installd' d['MM'] d~mm d~items

other = .Directory~new
other~setMethod('MM', m)
say 'reused  ' other['MM'] d['MM']

lines = .Array~of('x = 3', 'return x * 2')
fromArray = .Method~new('NN', lines)
d~setMethod('NN', fromArray)
say 'array   ' fromArray~class~id d['NN']

/* the body runs against whichever receiver holds it */
counted = .Method~new('COUNT', 'return self~items')
one = .Directory~new
one['a'] = 1
two = .Directory~new
two['a'] = 1
two['b'] = 2
one~setMethod('COUNT', counted)
two~setMethod('COUNT', counted)
say 'receiver' one['COUNT'] two['COUNT']

signal on syntax name noname
z = .Method~new()
noname:
say 'no name ' rc

signal on syntax name nosource
z = .Method~new('m')
nosource:
say 'no src  ' rc
