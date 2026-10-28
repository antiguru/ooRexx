call lineout 'in.txt', 'l1'
call lineout 'in.txt', 'l2'
say linein('in.txt')
address system 'cat' with input stream 'in.txt'
say 'rc' rc
