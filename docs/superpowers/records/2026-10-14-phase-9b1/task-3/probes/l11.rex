s.0 = 2; s.1 = .k~new; s.2 = 'a'
call SysStemSort 's.'
::class k
::method string
  if .local~seen \= .true then do; .local~seen = .true
    do f over .context~stackFrames; e = f~executable
      say f~type f~name~right(12) e~class~id (e~source~items); end; end
  return 'b'
