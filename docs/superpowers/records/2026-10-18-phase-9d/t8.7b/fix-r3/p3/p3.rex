call on notready name hn
call on user u name hu
n = 0
call ru 'one'
say 'after, n =' n
exit
ru: raise user u description (arg(1)) return
hu:
  say 'hu' condition('d')
  x = linein('no_such_file_zz.txt')
  say 'hu returns'
  return
hn:
  n = n + 1
  say 'hn' n condition('d')
  return
