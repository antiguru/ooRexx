call on notready name hn
n = 0
x = charin('no_such_file_zz.txt')
say 'after, n =' n
exit
hn:
  n = n + 1
  say 'hn' n condition('d')
  x = charin('no_such_zz2.txt')
  say 'charin' stream('no_such_zz2.txt', 's')
  x = lineout('/', 'x')
  say 'lineout' x
  x = charout('/', 'y')
  say 'charout' x
  x = linein('/')
  say 'linein dir' stream('/', 's')
  x = chars('no_such_zz3.txt')
  say 'chars' x
  x = stream('no_such_zz4.txt', 'c', 'open read')
  say 'open' x
  x = linein('no_such_zz4.txt')
  say 'hn' n 'returns'
  return
