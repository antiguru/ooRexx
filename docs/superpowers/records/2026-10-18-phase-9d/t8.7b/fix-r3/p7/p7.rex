call on notready name hn
n = 0
x = linein('no_such_file_zz.txt')
say 'after, n =' n
exit
hn:
  n = n + 1
  say 'hn' n condition('d')
  call on notready name hn2
  x = linein('no_such_file_zz2.txt')
  say 'hn' n 'returns'
  return
hn2:
  say 'hn2' condition('d')
  return
