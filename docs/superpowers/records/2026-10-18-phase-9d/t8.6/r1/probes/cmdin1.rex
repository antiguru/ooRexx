'printf "p1\np2\n" > pre.txt'
address system 'cat' with input stream 'pre.txt'
say 'rc' rc
call lineout 'in.txt', 'l1'
call lineout 'in.txt', 'l2'
call lineout 'in.txt'
address system 'cat' with input stream 'in.txt'
say 'rc' rc
