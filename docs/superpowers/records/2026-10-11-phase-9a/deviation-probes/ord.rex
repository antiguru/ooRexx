d = .directory~new
do n over .array~of('ZETA','ALPHA','MANGO','BETA','QUUX','FOO','UM')
  d[n] = 'return 1'
end
call show d~supplier, 'dir'
o = .object~enhanced(d)
call show o~instanceMethods(.nil), 'enh'
o~setMethod('AAA', 'return 2')
o~setMethod('ZZZ', 'return 2')
call show o~instanceMethods(.nil), 'enh+set'
t = .table~new
do n over .array~of('UM','FOO','QUUX','BETA','MANGO','ALPHA','ZETA')
  t[n] = 'return 1'
end
call show t~supplier, 'tab'
p = .object~enhanced(t)
call show p~instanceMethods(.nil), 'enh2'
q = .object~new
q~setMethod('ZZZ', 'return 2'); q~setMethod('AAA', 'return 2'); q~setMethod('MMM', 'return 2')
call show q~instanceMethods(.nil), 'set'
exit
show: procedure
  use arg s, tag
  out = tag':'
  do while s~available; out = out s~index; s~next; end
  say out
  return
