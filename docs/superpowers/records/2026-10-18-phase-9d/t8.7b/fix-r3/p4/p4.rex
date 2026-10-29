call on any name ha
m = 0
x = ru('one') || linein('no_such_file_zz.txt')
say 'after a, m =' m
x = linein('no_such_file_zz2.txt') || ru('two')
say 'after b, m =' m
exit
ru: raise user u description (arg(1)) return 'r'
ha:
  m = m + 1
  say 'ha' m condition('c') condition('d')
  return
