.local~debuginput = .src~new
call on user ping name h
n = 0
call rp 'first'
say 'after' n
exit
rp: raise user ping description (arg(1)) return
h:
  n = n + 1
  say 'h' n condition('d')
  if n > 3 then return
  trace ?r
  nop
  return
::class src
::method linein
  say 'linein'
  raise user ping description 'from input' return ''
