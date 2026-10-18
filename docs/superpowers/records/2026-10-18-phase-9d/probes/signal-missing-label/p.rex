call m
exit
m:
  signal on novalue
  signal on syntax
  a = novalue
  a = "next clause"
  syntax:
  say 'trapped' condition('c') rc '['a']'
  return
