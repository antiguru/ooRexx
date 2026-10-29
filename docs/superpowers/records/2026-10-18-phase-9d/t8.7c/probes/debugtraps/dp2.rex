.local~debuginput = .src~new
call on user ping name h
n = 0
trace ?r
nop
say 'after' n
exit
h:
  n = n + 1
  say 'h' n condition('d')
  return
::class src
::method linein
  say 'linein'
  raise user ping description 'from input' return ''
