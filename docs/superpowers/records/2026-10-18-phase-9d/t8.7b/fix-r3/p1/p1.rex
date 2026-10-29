call on notready name hn
n = 0
x = linein('no_such_file_zz.txt')
say 'after, n =' n
exit
hn:
  n = n + 1
  say 'hn' n condition('d')
  call sub
  say 'hn' n 'returns'
  return
sub:
  x = linein('no_such_file_zz2.txt')
  say 'sub' stream('no_such_file_zz2.txt', 's')
  return
