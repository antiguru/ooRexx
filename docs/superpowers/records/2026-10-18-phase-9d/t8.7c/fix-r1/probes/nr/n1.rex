/* crate only: the internal routine sets its own CALL ON NOTREADY before its NOTREADY */
call on notready name hn
m = 0
x = linein('no_such_file_zz.txt') || rn()
say 'after a, m =' m
exit
rn:
  call on notready name hn2
  y = linein('no_such_file_zz2.txt')
  say 'rn after its linein'
  return y
hn:
  m = m + 1
  say 'hn' m condition('c') condition('d')
  return
hn2:
  say 'hn2' condition('c') condition('d')
  return
