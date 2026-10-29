parse arg lim
call on user ping name h
n = 0
call rp 'first'
say 'done' n
exit
rp: parse arg dd; raise user ping description (dd) return
h:
  n = n + 1
  if n < lim then call rp n
  return
