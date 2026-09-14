/* A library-backed EXTERNAL method reports its declaring package. A
   loadExternalMethod answer shares the library's code object for that
   procedure spelling: its package is .nil until a directive binds the
   procedure and the first binder's afterwards, for an earlier answer too. */
early = .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse')
say 'loaded before binding' show(early)
say 'unbound procedure' show(.Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Pos'))
routine = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt')
say 'routine before binding' show(routine)
p = .context~package~loadPackage('pk.cls')
say 'the same object after pk.cls binds it' show(early)
say 'the same routine after pk.cls binds it' show(routine)
say 'pk.cls method' show(.Re~method('DOPARSE'))
say 'pk.cls LIBRARY REXX method' show(.Re~method('SEP'))
say 'pk.cls routine' show(p~findRoutine('SQ'))
p2 = .context~package~loadPackage('pk2.cls')
say 'pk2.cls method' show(.Re2~method('DOPARSE'))
say 'loaded after both' show(.Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Parse'))
say 'another spelling' show(.Method~loadExternalMethod('m', 'LIBRARY rxregexp REGEXP_PARSE'))
say 'routine loaded after binding' show(.Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcSqrt'))
say 'LIBRARY REXX' show(.Method~loadExternalMethod('m', 'LIBRARY REXX file_separator'))
exit

show: procedure
  use arg x
  p = x~package
  if p == .nil then return 'nil'
  return p~name~substr(p~name~lastpos('/') + 1)
