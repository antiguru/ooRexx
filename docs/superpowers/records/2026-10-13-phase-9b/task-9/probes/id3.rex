p = .context~package
signal on syntax
r = p~findRoutine('rr')
x = r~call('abc')
back:
say 'copy' (r~copy == r) r~copy~call(0)
p2 = .Package~new('x', "return SysSleep(0)")
p2~addRoutine('ADDED', .Routine~loadExternalRoutine('y', 'LIBRARY rexxutil SysSleep'))
say 'added same' (p2~findRoutine('added') == r)
q = .Package~new('q', "::requires 'rexxutil' library")
say 'other package' (q~findRoutine('SysSleep') == r) (q~importedRoutines['SYSSLEEP'] == r)
say 'routines table' (p~routines['RR'] == r)
.context~package~addRoutine('ZZ', r)
call zz 0
say 'zz' result
exit
syntax:
  c = condition('o')
  say c~code
  do l over c~traceback; say l~strip; end
  signal back
::routine rr external 'LIBRARY rexxutil SysSleep'
