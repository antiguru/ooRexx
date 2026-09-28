-- Message~new and ~sendWith refusing their arguments.
signal on syntax
m = .message~new(.nil, 'x', 'Q')
exit
syntax: say condition('o')~code condition('o')~message; 
signal on syntax name s2
m = .message~new(.nil, 'x', 'a')
s2: say condition('o')~code condition('o')~message
signal on syntax name s3
m = .message~new(.nil, 'x', 'a', 1, 2)
s3: say condition('o')~code condition('o')~message
signal on syntax name s4
m = .message~new(.nil, 'x', 'a', 'notarr')
s4: say condition('o')~code condition('o')~message
signal on syntax name s5
m = .message~new(.nil, 'x', , 3)
s5: say condition('o')~code condition('o')~message
signal on syntax name s6
m = .message~new(.nil, 'x')~sendwith(, 'z')
s6: say condition('o')~code condition('o')~message
signal on syntax name s7
m = .message~new(.nil, 'x')~sendwith
s7: say condition('o')~code condition('o')~message
