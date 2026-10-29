call on notready name hn
m = 0
x = linein('no_such_file_zz.txt') || rn()
say 'after a, m =' m
exit
rn: return linein('no_such_file_zz2.txt')
hn:
  m = m + 1
  say 'hn' m condition('c') condition('d')
  return
