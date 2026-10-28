-- NOTREADY, ANY, and ERROR in their handlers
call on notready name hn
n = 0
x = linein('no_such_file_zz.txt')
say 'after notready, n =' n
call on any name ha
m = 0
call rp 'first'
say 'after any, m =' m
call on error name he
k = 0
address system 'exit 3'
say 'after error, k =' k
exit
rp: parse arg dd; raise user anyc description (dd) return
hn:
  n = n + 1
  say 'hn' n condition('c') condition('d')
  if n = 1 then x = linein('no_such_file_zz2.txt')
  say 'hn' n 'returns'
  return
ha:
  m = m + 1
  say 'ha' m condition('c') condition('d')
  if m = 1 then call rp 'again'
  return
he:
  k = k + 1
  say 'he' k condition('c') rc
  if k = 1 then address system 'exit 4'
  return
