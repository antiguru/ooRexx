call on notready name hn
n = 0
x = linein('no_such_file_zz.txt')
say 'after, n =' n
exit
hn:
  n = n + 1
  say 'hn' n condition('d')
  signal on notready name sg
  x = linein('no_such_file_zz2.txt')
  say 'hn not reached'
  return
sg:
  say 'sg' condition('d') condition('i') sigl
  exit
