call on user ping name h
n = 0
call rp 'first'
say 'after first, n =' n
exit
rp: parse arg dd; raise user ping description (dd) return
h:
  n = n + 1
  say 'handler' n condition('d')
  if n = 1 then do
    call rp 'again'
    say 'raised again'
    call on user ping name h
    say 'after call on'
  end
  say 'handler' n 'returns'
  return
