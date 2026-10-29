call on notready name hn
n = 0
x = linein('no_such_file_zz.txt')
say 'after notready, n =' n
exit
hn:
  n = n + 1
  say 'hn' n condition('d') condition('s')
  if n = 1 then x = linein('no_such_file_zz2.txt')
  say 'hn' n 'returns' condition('d')
  return
