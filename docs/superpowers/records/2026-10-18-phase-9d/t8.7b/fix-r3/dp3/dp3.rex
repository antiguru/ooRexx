.local~debuginput = .src~new
.local~reads = 0
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
  .local~reads = .local~reads + 1
  say 'linein' .local~reads
  if .local~reads > 6 then return ''
  raise user ping description 'from input' return ''
