/* What stays specified for a value-based value (Deviation 36): a held
   String or number keeps one identityHash through reads, an alias, ARG,
   USE ARG, a return, a stem, and storage in and retrieval from an Array, a
   Directory, an IdentityTable and a Set. A String subclass instance and an
   enhanced string are ordinary objects: the same, plus OBJECTNAME= and a
   distinct identity for each one made. Hashes are compared with ==. */
call show 'long computed ', 'a longer' || ' computed one'
call show 'short computed', 'ab' || 'c'
call show 'short literal ', 'abc'
call show 'long literal  ', 'a longer literal string'
call show 'integer 1001  ', 1000 + 1
call show 'integer 5     ', 5 + 0
call show 'decimal 1.5   ', 1.5 + 0
call show '.true         ', .true

s = .mystr~new('abc')
s~objectName = 'named sub'
call show 'subclass      ', s
say 'subclass name' s~objectName pass(s)~objectName .array~of(s)[1]~objectName
t = s; say 'subclass alias' t~objectName (t~identityHash == s~identityHash)
say 'subclass two' (.mystr~new('abc')~identityHash == .mystr~new('abc')~identityHash)
say 'subclass class' s~class~id (s == 'abc')

m = .directory~new
m['FOO'] = 'return "foo ran"'
e = .String~enhanced(m, 'abc')
e~objectName = 'named enh'
call show 'enhanced      ', e
say 'enhanced name' e~objectName pass(e)~objectName .array~of(e)[1]~objectName e~foo
f = e; say 'enhanced alias' f~objectName f~foo (f~identityHash == e~identityHash)
say 'enhanced two' (.String~enhanced(m, 'abc')~identityHash == .String~enhanced(m, 'abc')~identityHash)
exit

show: procedure
  parse arg label
  use arg , v
  h = v~identityHash
  say label paths(v, h)
  return

paths: procedure
  use arg v, h
  r = (v~identityHash == h) (arg(1)~identityHash == h)
  b = v; r = r (b~identityHash == h) (v~identityHash == v~identityHash)
  r = r (same(v)~identityHash == h) (pass(v)~identityHash == h)
  r = r (.holder~new~echo(v)~identityHash == h)
  s.1 = v; r = r (s.1~identityHash == h)
  a = .array~of(v); r = r (a[1]~identityHash == h)
  d = .directory~new; d['K'] = v; r = r (d['K']~identityHash == h)
  it = .identityTable~new; it[v] = 'x'
  r = r it~hasIndex(v) (it~allIndexes[1]~identityHash == h)
  iv = .identityTable~new; iv['K'] = v; r = r (iv['K']~identityHash == h)
  st = .set~new; st~put(v); r = r (st~allIndexes[1]~identityHash == h)
  return r

same: return arg(1)

::routine pass
  return arg(1)

::class holder
::method echo
  use arg o
  return o

::class mystr subclass string
