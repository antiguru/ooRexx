/* acyclic stem-through-reference shapes */
s. = 5
r = >s.
say r
say length(r)
say r || 'x'
say 1 + r
say 'p' r
say r = 5
t. = .array~of('one', 'two')
q = >t.
say q
say 'p' q
say length(q)
n = .n~new
u. = n
w = >u.
say w
say 'p' w
k = .k~new
v. = k
z = >v.
say z
say 'p' z
say length(z)
e. = 'x'
drop e.
f = >e.
say f
say 'p' f
y = 7
g = >y
h. = g
i = >h.
say i
say 'p' i
say i + 1
say h.
say 'p' h.
say h. + 1
m = .mystem~new
m[] = 'sub'
n. = m
o = >n.
say o
say 'p' o
say length(o)
say n.
say 'p' n.
p = .mystem~new
p[] = m
say p
say 'p' p
say length(p)
exit
::class n
::method makestring; return 'nmake'
::class k
::method string; return 'kstring'
::class mystem subclass stem
