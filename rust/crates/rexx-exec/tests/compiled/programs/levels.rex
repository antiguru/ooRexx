trace r
say trace()
interpret 'say 1+1'
call sub
x = .c~new~m
say .context~package~sourceline(1)'|'
say .context~package~name
say .context~package~sourceSize
call rr
call rt
exit
sub: say 'sub'; return
::class c
::method m
  return 'meth'
::method boom
  return 1 + 'b'
::routine rt
  say 'rt'
  x = .c~new~boom
::routine rr
  r = .context~package~findroutine('RT')
  say r~source~items
  say .context~package~findroutine('RR')~source~items
  interpret 'call rt'
