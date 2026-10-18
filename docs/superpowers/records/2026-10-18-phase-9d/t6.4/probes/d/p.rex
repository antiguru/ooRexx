call on notready name h
x = linein('/nonexistent/zz')
x = chars('/')
x = .stream~new('/')~lines
call off notready
signal on notready name s1
x = stream('/', 'c', 'open read')
s1: call dump 's1'
signal on notready name s2
x = charin('bad'||'80'x||'name')
s2: call dump 's2'
signal on notready name s3
x = .stream~new('/')~charin
s3: call dump 's3'
signal on notready name s4
x = f()
s4: call dump 's4'
exit
h: call dump 'h'; return
dump:
  o = condition('O')
  idx = o~allIndexes~sortWith(.caselessComparator~new)
  line = arg(1)':'
  do i over idx
    v = o[i]
    if i = 'STACKFRAMES' | i = 'TRACEBACK' then v = v~items
    else if i = 'PACKAGE' then v = v~name~right(5)
    else if i = 'PROGRAM' then v = v~right(5)
    else if \v~isA(.string) then v = v~class~id':'v~string
    line = line i'='v
  end
  say line
  return
f: return stream('/', 'c', 'open read')
