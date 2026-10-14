/* A rexxutil routine that calls back into Rexx is a level of a live
   .context~stackFrames, called by its name or through a directive. */
longstemname.0 = 2; longstemname.1 = .k~new; longstemname.2 = 'a'
call SysStemSort 'longstem' || 'name.', 'D'
.local~seen = .false; s.0 = 2; s.1 = .k~new; s.2 = 'a'
call vv 's.'
r = .Routine~loadExternalRoutine('x', 'LIBRARY rexxutil SysStemSort')
say 'executable' (.local~e1 == r) (.local~e2 == r)
signal on syntax
s.0 = 2; s.1 = .j~new; s.2 = 'a'
call SysStemSort 's.'
exit
syntax:
  o = condition('o')
  say 'code' o~code
  do l over o~traceback; say 'traceback' l; end
  do f over o~stackFrames; say 'frame' f~type (f~type == 'PROGRAM' | f~name \== ''); end
  exit
::routine vv external 'LIBRARY rexxutil SysStemSort'
::class k
::method string
  if .local~seen \= .true then do
    .local~seen = .true
    fs = .context~stackFrames
    say 'items' fs~items
    f = fs[2]
    say f~type f~name (f~line == .nil) (f~invocation == .nil) (f~target == .nil)
    say 'args' f~arguments~items f~arguments[1] f~arguments[2]
    say 'context' (f~context == fs[3]~context) (f~context~executable == fs[3]~executable)
    say 'executable' f~executable~class~id f~executable~source~items
    say 'trace' f~traceLine
    say 'string' f
    .local['E' || (1 + (.local~e1 \= .nil))] = f~executable
  end
  return 'b'
::class j
::method string
  raise syntax 93.900 array('boom')
