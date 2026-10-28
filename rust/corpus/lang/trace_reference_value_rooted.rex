/* A traced reference is held while its trace lines are written. */
y = 'why'
trace i
q = >y
r = <y
call show >y
trace off
say q r~value
do i = 1 to 2
  trace i
  s = >y
  trace off
end
say s~value
exit

show:
  use arg >z
  say z
  return
