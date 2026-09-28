t = .T~new
d = .OD~new
d['K'] = 'v'
say t~TestDirectoryAt(d, 'K')
t~TestDirectoryPut(d, 'w', 'J')
say d['J'] d['JX'] d~items
exit
::class T
::method TestDirectoryPut EXTERNAL "LIBRARY orxmethod TestDirectoryPut"
::method TestDirectoryAt EXTERNAL "LIBRARY orxmethod TestDirectoryAt"
::class OD subclass Directory
::method at
  return 'overridden' arg(1)
::method put
  use arg v, k
  self~put:super(v, k'X')
