signal on syntax name c1
call perr.rex
c1: o = condition('o'); p = o~package; say o~code p~name p~source~items p~classes~items p~routines~items
say p~sourceline(2) '|' (p == .context~package) p~class
say o~position o~traceback~items o~stackframes~items o~stackframes[1]~type o~stackframes[1]~name
signal on syntax name c3
r = .routine~new('xyz', .array~of('nop', 'say (', 'nop'))
c3: o = condition('o'); p = o~package; say o~code p~name p~source~items o~position
signal on syntax name c4
interpret 'say ('
c4: o = condition('o'); say o~code o~program~substr(o~program~lastpos('/') + 1) (o~package == .context~package)
