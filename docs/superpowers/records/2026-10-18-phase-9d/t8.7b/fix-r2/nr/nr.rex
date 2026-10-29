call on any name ha
m = 0
x = linein('no_such_file_zz.txt')
say 'after any, m =' m
call on notready name hn
n = 0
x = linein('no_such_file_zz3.txt') || linein('no_such_file_zz4.txt')
say 'after two, n =' n
exit
ha:
  m = m + 1
  say 'ha' m condition('c') condition('d')
  x = linein('no_such_file_zz2.txt')
  say 'ha' m 'returns'
  return
hn:
  n = n + 1
  say 'hn' n condition('c') condition('d')
  return
