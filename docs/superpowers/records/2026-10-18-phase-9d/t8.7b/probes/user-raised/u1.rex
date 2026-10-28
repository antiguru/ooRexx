call on user ping name h
n = 0
raise user ping return
say 'after raise, n =' n
call on user pong name h2
raise user pong description 'first' return
say 'after pong'
call on halt name h3
call halter 'first'
say 'after halt'
exit
h:
  n = n + 1
  say 'handler' n condition('d')
  if n = 1 then raise user ping description 'again' return
  say 'handler' n 'returns'
  return
h2:
  say 'h2' condition('d')
  if condition('d') == 'first' then do
    raise user pong description 'second' return
    call on user pong name h2
    say 'h2 after re-enable'
  end
  return
h3:
  say 'h3' condition('d')
  if condition('d') == 'first' then raise halt description 'second' return
  say 'h3 returns' condition('d')
  return
halter:
  parse arg dd; raise halt description (dd) return
  return
