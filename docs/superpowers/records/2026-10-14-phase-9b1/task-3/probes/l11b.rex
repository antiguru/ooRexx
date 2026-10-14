s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.', 'D'
say 'after' s.1 s.2
::class k
::method string
  if .local~seen \= .true then do; .local~seen = .true
    fs = .context~stackFrames
    say 'items' fs~items
    do f over fs; e = f~executable
      say '--' f~type '['f~name']' 'line['f~line']' 'inv['f~invocation~class~id']' 'tgt['f~target']'
      say '   args' f~arguments~items 'a1['f~arguments[1]']' 'a2['f~arguments[2]']'
      say '   ctx' f~context~class~id 'ctxexec' (f~context~executable == e) 'ctxname' f~context~name
      say '   exec' e~class~id e~source~items 'pkg' e~package~class~id
      say '   trace['f~traceLine']'
      say '   str['f~string']'
    end; end
  return 'b'
