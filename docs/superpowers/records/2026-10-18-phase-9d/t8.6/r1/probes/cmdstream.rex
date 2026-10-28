call lineout 'in.txt', 'l1'
call lineout 'in.txt', 'l2'
address system 'cat' with input stream 'in.txt'
say 'rc' rc
call lineout 'out.txt', 'before'
address system 'echo cmd' with output append stream 'out.txt'
call lineout 'out.txt', 'after'
call lineout 'out.txt'
say charin('out.txt', 1, 100)
s = .stream~new('obj.txt'); s~lineout('o1')
address system 'echo c2' with output using (s)
s~lineout('o3'); s~close
say charin('obj.txt', 1, 100)
call lineout 'cat.txt', 'x1'
address system 'cat cat.txt'
say 'lines' lines('cat.txt') stream('cat.txt','c','query size')
