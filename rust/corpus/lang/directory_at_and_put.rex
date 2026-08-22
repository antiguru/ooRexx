/* A Directory's ~put, [] and ~at -- .environment and .local, the two
 * directories this interpreter builds.
 *
 * The index is stored and matched verbatim: the entries Setup.cpp registers
 * are uppercase because completeSystemClass upcases the name it registers,
 * not because a lookup folds case. So a lower-case index finds nothing, and
 * an index put in lower case is not found in upper.
 *
 * ~put answers no value, which is rc 0 as a whole clause. An index the
 * directory does not hold answers .nil rather than raising.
 *
 * A name put into .environment is then reachable as an environment symbol,
 * which is the whole point of the prologue's own `.environment~put(class,
 * name)` -- the entry map is what a .NAME lookup reads.
 */
say .environment['ARRAY']
say .environment~at('ARRAY')
say .environment['array']
say .environment['Array']
say .environment['no such entry']
say .environment~at(5)

d = .environment
d~put('put in upper', 'ZZTOP')
say d['ZZTOP']
say d~at('ZZTOP')
say .ZZTOP
d~put('put in lower', 'zzbot')
say d['zzbot']
say d['ZZBOT']
say .zzbot

/* Replacing an entry, and putting a non-string item. */
d~put('replaced', 'ZZTOP')
say d['ZZTOP']
d~put(.Array, 'ZZCLASS')
say d['ZZCLASS']
say .ZZCLASS
d~put(.nil, 'ZZNIL')
say d['ZZNIL']

/* An index need not be a symbol: any value with a string value is one, and
   it is stored under its own rendering. */
d~put('numbered', 5)
say d[5]
say d['5']

/* .local is a Directory too, and the same methods reach it. */
l = .local
l~put('local entry', 'ZZLOCAL')
say l['ZZLOCAL']
say l~at('ZZLOCAL')
say .ZZLOCAL
say l['no such entry']

/* The two directories are separate objects: an entry in one is not in the
   other. */
say .environment['ZZLOCAL']
say .local['ZZTOP']

/* The reflection the methods are asked through. */
say .environment~class~id
say .environment~isA(.Directory)
say .environment~hasMethod('PUT')
say .environment~hasMethod('AT')
