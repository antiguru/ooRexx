-- Message~new keeps its send; ~send and ~sendWith make it.
o = .k~new
m = .message~new(o, 'add', 'i', 1, 2); say m~send
say m~send(, 10, 20)
say m~send
say m~send(.k~new('x'))
m2 = .message~new(o, 'add', 'A', .array~of(3, 4)); say m2~send
say m2~sendWith(.k~new('y'), .array~of(5, 6))
say m2~send
m3 = .message~new(o, 'noargs'); say m3~send; say m3~result m3~completed m3~hasError
m4 = .message~new(o, 'add', 'I'); say m4~send
m5 = .message~new(o, 'add', 'i', , 7); say m5~send
m6 = .message~new(o, 'boom'); signal on syntax name s1; say m6~send
s1: say 'trapped' condition('o')~code m6~hasError m6~completed
m7 = .message~new(o, 'nothing'); say m7~send~string
exit
::class k
::method init; expose tag; use arg tag = 'o'
::method add; expose tag; use arg a = 'none', b = 'none'; return tag a b
::method noargs; return 'na' arg()
::method boom; return 1/0
::method nothing; return
