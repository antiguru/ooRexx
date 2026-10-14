s.0 = 2; s.1 = .k~new; s.2 = 'a'
r = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysStemSort')
z = r~call('s.')
.local~seen = .false; s.0 = 2; s.1 = .k~new; s.2 = 'a'
call vv 's.'
.local~seen = .false; s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.'
say 'same exec' (.local~e1 == .local~e3) (.local~e1 == r) (.local~e2 == .local~e3)
exit
::routine vv external 'LIBRARY rexxutil SysStemSort'
::class k
::method string
  if .local~seen \= .true then do; .local~seen = .true
    fs = .context~stackFrames
    f = fs[2]
    say f~type '['f~name']' f~arguments~items f~executable~class~id (f~context~executable == fs[3]~executable) fs~items
    n = 1 + (.local~e1 \= .nil) + (.local~e2 \= .nil)
    .local['E'n] = f~executable
  end
  return 'b'
