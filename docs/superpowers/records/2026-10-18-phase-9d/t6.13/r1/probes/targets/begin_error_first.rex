s.0 = 'x'
signal on syntax
address system 'echo ran > ran.txt; echo out' with output append stem s.
say 'no' s.0 s.1
exit
syntax: say 'syntax' condition('o')~code 'ran file' (stream('ran.txt','c','query exists') \== '')
exit
