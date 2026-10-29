.local~debuginput = .src~new
.local~reads = 0
signal on syntax name s
trace ?r
nop
say 'not reached'
exit
s:
  trace off
  say 's' condition('o')~code condition('o')~message
  exit
::class src
::method linein
  .local~reads = .local~reads + 1
  say 'linein' .local~reads
  if .local~reads > 1 then return ''
  raise syntax 98.900 array('from input') return ''
