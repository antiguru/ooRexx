/* acyclic, traced renderings */
s. = 5
r = >s.
t. = .array~of('one', 'two')
q = >t.
y = 7
g = >y
h. = g
i = >h.
m = .mystem~new
m[] = 'sub'
n. = m
o = >n.
p = .mystem~new
p[] = m
trace r
x = r
x = q
x = i
x = h.
x = o
x = n.
x = p
x = m
trace o
say 'end'
::class mystem subclass stem
