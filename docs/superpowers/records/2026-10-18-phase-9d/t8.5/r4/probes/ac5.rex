/* array through a reference, and through a stem, separately */
zz = .array~of('one', 'two')
r = >zz
say r
say 'p' r
say length(r)
say r~request('STRING')
t. = zz
say t.
say 'p' t.
say length(t.)
say t.~request('STRING')
q = >t.
say q
say 'p' q
say length(q)
say q~request('STRING')
trace r
x = r
x = zz
x = t.
x = q
trace o
say 'end'
