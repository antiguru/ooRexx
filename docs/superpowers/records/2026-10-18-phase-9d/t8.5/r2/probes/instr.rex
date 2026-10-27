m = .knum~new; c = .kcode~new; k = .kmake~new; q = .array~of('p','q')
signal on syntax name s1; do (>m); call charout , 'd'; end; say; s1: say 'after do'
signal on syntax name s2; interpret (>c); s2: say 'after interp'
push (>k); pull v; say 'push' v
queue (>q); parse pull v; say 'queue' v
numeric digits (>m); say 'digits' digits()
numeric digits 9
x = 'abc'; t = >k
say 'tail' x.t
address command (>c)
say 'rc' rc
signal value (>k)
say 'not here'
kmake: say 'at kmake'
trace value (>m)
say 'trace' trace()
trace o
r = (>k); trace i
y = r || 'z'
z = 'a' r
trace o
say d2c(>m) left(>k, 2) copies(>k, 2) reverse(>k) word(>q, 1) words(>q)
say abs(>m) format(>m, 3) (>m)~length datatype(>m) datatype(>k)
::class kmake
::method makestring; return 'kmake'
::class knum
::method makestring; return '5'
::class kcode
::method makestring; return "say 'interp'"
