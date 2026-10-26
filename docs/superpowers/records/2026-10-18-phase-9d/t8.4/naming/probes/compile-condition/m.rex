signal on syntax name c1
call perr.rex
c1: o = condition('o'); say o~code o~program; say (o~package == .nil) o~hasIndex('PACKAGE'); if o~package \== .nil then say o~package~name
signal on syntax name c2
call duprt.rex
c2: o = condition('o'); say o~code o~program; say (o~package == .nil) o~hasIndex('PACKAGE'); if o~package \== .nil then say o~package~name
signal on syntax name c3
r = .routine~new('xyz', .array~of('say (', 'nop'))
c3: o = condition('o'); say o~code o~program; say (o~package == .nil) o~hasIndex('PACKAGE'); if o~package \== .nil then say o~package~name
