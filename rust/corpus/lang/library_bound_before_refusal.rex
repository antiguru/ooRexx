/* A directive binds a library procedure's shared code to its package while
   the package is being translated, so a later directive of the same package
   that fails to resolve does not undo the binding. */
early = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse')
routine = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')
signal on syntax name failed
p = .context~package~loadPackage('pk.cls')
say 'pk.cls loaded'
exit

failed:
say 'loadPackage raised' condition('O')~code
say 'method answered before the failure' show(early)
say 'routine answered before the failure' show(routine)
say 'method answered after the failure' show(.Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse'))
say 'procedure after the failing directive' show(.Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos'))
exit

show: procedure
  use arg x
  p = x~package
  if p == .nil then return 'nil'
  return p~name~substr(p~name~lastpos('/') + 1)
