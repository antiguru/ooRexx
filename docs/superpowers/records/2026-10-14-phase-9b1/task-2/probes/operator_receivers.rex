signal on syntax name e1
m = .mutablebuffer~new('3'); say 'mb' (m + 1)
e1: say 'mb err' condition('o')~code
signal on syntax name e2
s. = .mystr~new('3'); say 'stem' (s. + 1) (-s.)
e2: if condition('o') \= .nil then say 'stem err' condition('o')~code
signal on syntax name e3
d = .String~enhanced(.directory~new, '5'); say 'enh' (d + 1) (-d)
e3: if condition('o') \= .nil then say 'enh err' condition('o')~code
signal on syntax name e4
p = .plain~new; say 'plain' (p + 1)
e4: say 'plain err' condition('o')~code
t = .other~new('4'); say 'nooverride' (t + 1) (-t) (t * 2) (t ** 2) (t // 3) (t % 3) (t / 2) (t - 1) (\t~equals(0))
exit
::class mystr subclass string
::method '+'; if arg() = 0 then return 'pfx+'; return 'plus'
::method '-'; if arg() = 0 then return 'pfx-'; return 'minus'
::class other subclass string
::method equals; return 1
::class plain
::method string; return '7'
