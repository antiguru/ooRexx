t = .T~new
d = .OD~new
d['K'] = 'v'
say 'subclass at' t~TestDirectoryAt(d, 'K')
t~TestDirectoryPut(d, 'w', 'J')
say 'subclass put' d['J'] '|' d['JX'] '|' d~items
e = .directory~new
e~setMethod('M', 'return "meth" arg()')
say 'setmethod at' t~TestDirectoryAt(e, 'M')
e~setMethod('UNKNOWN', 'return "unk" arg(1)')
say 'unknown at' t~TestDirectoryAt(e, 'ZZ')
t~TestDirectoryPut(e, 'plain', 'M')
say 'put over method' e['M'] e~items e~hasIndex('M')
d['R'] = 'r'
say 'subclass remove' t~TestDirectoryRemove(d, 'R') d~items
g = .directory~new
g~setMethod('N', 'return "gone" arg()')
say 'setmethod remove' t~TestDirectoryRemove(g, 'N') g~hasIndex('N') g~items
s = .OS~new
s['K'] = 'sv'
say 'stringtable at' t~TestStringTableAt(s, 'K')
t~TestStringTablePut(s, 'sw', 'J')
say 'stringtable put' s['J'] '|' s['JX'] '|' s~items
f = .directory~new
f~setMethod('Q', 'return 1')
signal on syntax
say 'lowercase' t~TestDirectoryAt(f, 'q')
exit
syntax:
  say 'raised' condition('o')~code
  exit
::class T
::method TestDirectoryPut EXTERNAL "LIBRARY orxmethod TestDirectoryPut"
::method TestDirectoryAt EXTERNAL "LIBRARY orxmethod TestDirectoryAt"
::method TestDirectoryRemove EXTERNAL "LIBRARY orxmethod TestDirectoryRemove"
::method TestStringTablePut EXTERNAL "LIBRARY orxmethod TestStringTablePut"
::method TestStringTableAt EXTERNAL "LIBRARY orxmethod TestStringTableAt"
::class OD subclass Directory
::method at
  return 'overridden' arg(1)
::method put
  use arg v, k
  self~put:super(v, k'X')
::method remove
  return 'overridden remove' arg(1)
::class OS subclass StringTable
::method at
  return 'overridden' arg(1)
::method put
  use arg v, k
  self~put:super(v, k'X')
