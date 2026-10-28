-- FAILURE and ERROR again in their own handlers
call on failure name hf
call on error name he
address system 'no_such_command_zz 2>/dev/null'
say 'after failure'
call on error name he2
k = 0
call re 5
say 'after error, k =' k
exit
re: raise error (arg(1)) return
hf:
  say 'hf' condition('c')
  address system 'no_such_command_zz 2>/dev/null'
  say 'hf returns'
  return
he:
  say 'he' condition('c') rc
  return
he2:
  k = k + 1
  say 'he2' k rc
  if k = 1 then raise error 6 return
  return
