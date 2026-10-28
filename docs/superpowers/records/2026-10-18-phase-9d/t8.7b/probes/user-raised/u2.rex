call on user ping name h
n = 0
call rp 'first'
say 'after first, n =' n
call on halt name hh
call rh 'one'
say 'after halt'
exit
rp: parse arg dd; raise user ping description (dd) return
rh: parse arg dd; raise halt description (dd) return
h:
  n = n + 1
  say 'handler' n condition('d')
  if n = 1 then call rp 'again'
  say 'handler' n 'returns'
  return
hh:
  say 'hh' condition('d')
  if condition('d') == 'one' then call rh 'two'
  say 'hh returns' condition('d')
  return
