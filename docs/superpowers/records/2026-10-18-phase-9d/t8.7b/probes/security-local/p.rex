p = .context~package
p~local~put(.array~of('local'), 'PKGHELD')
.routines['ADDED'] = .routine~new('a', 'return "table"')
w = .WeakReference~new(p~prolog)
p~setSecurityManager(.sm~new)
c = mk()
do i = 1 to 20
  zj = 'filler' i
end
say .pkgheld[1]
say added()
say (w~value == p~prolog) (w~value \== .nil)
say .k~v[1] c~v[1]
x = .local
exit
::routine mk
  p = .package~new('cp', .array~of('::class k public', "::constant v (.array~of('built'))"))
  return p~findClass('K')
::routine r1
  return 1
::class k
::constant v (.array~of('const'))
::class sm
::method local
  say 'mgr local'
  return 0
::method unknown
  return 0
