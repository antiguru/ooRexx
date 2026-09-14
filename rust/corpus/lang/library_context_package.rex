/* A Method or Routine object passed as the context of Package~new or newFile
   hands its package on as the new code's parent. A loadExternal* answer no
   directive has bound has no package: the new code still runs, and a bound
   one's package is the parent a later routine call resolves through. */
m = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos')
r = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcPower')
say 'unbound method package' show(m)
p = .Package~new('fromsource', 'say "prolog under an unbound method"', m)
say 'package answered' p~class
p = .Package~new('fromsource2', .array~of('say "prolog under an unbound routine"'), r)
say 'package answered' p~class
mf = .Method~newFile('mf.cls', m)
say 'method file answered' mf~class mf~package~class
rf = .Routine~newFile('rf.cls', r)
say 'routine file answered' rf~class rf~package~class 'call' rf~call
q = .context~package~loadPackage('pk.cls')
say 'bound method package' show(m)
p = .Package~new('fromsource3', 'call pkroutine', m)
say 'package answered' p~class
rf = .Routine~newFile('rf2.cls', m)
say 'routine file under a bound method' rf~call
exit

show: procedure
  use arg x
  p = x~package
  if p == .nil then return 'nil'
  return p~name~substr(p~name~lastpos('/') + 1)
