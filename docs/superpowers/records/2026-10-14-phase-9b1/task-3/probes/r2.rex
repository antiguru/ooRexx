p = .context~package
p~addRoutine('C1', .Routine~loadExternalRoutine('xyz', 'LIBRARY rexxutil SysStemSort')~copy)
s.0 = 2; s.1 = .k~new; s.2 = 'a'
call c1 's.'
.local~seen = .false; s.0 = 2; s.1 = .k~new; s.2 = 'a'
x = C1('s.')
.local~seen = .false; s.0 = 2; s.1 = .k~new; s.2 = 'a'
call vv 's.'
::routine vv external 'LIBRARY rexxutil SysStemSort'
::class k
::method string
  if .local~seen \= .true then do; .local~seen = .true
    say .context~stackFrames[2]~name; end
  return 'b'
