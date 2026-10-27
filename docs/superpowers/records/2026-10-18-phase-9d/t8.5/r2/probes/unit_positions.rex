k = .kmake~new; m = .knum~new; c = .kcode~new; x = .kexit~new
n = .nstr~new; e = .array~new(2); t = .ktrace~new; d = .directory~new
l = .klabel~new
say 'cmp' ('kmake' = (>k)) ('kmake' == (>k)) ('kmakf' > (>k))
signal on syntax name m0
say 'meth' 'xkmakex'~pos(>k) 'xkmakex'~changestr(>k, '-') 'ab'~append(>k) 'kmake'~compareTo(>k) 'kmakeq'~startsWith(>k) 'kmake'~match(1, >k)
m0: signal on syntax name m1
say 'x'~pos(>n)
m1: say 'methn' condition('o')~code
signal on syntax name d1
call charout , 'do '; do (>m); call charout , 'd'; end; say
d1: signal off syntax
interpret (>c)
address system (>x); say 'address' rc
push (>k); pull v; say 'push' v
tl = >k; say 'tail' x.tl
trace value (>t); say 'trace' trace(); trace o
say 'msg' (>e)~'||'('x') (>k)~' '('y')
signal on nostring name ns
say "x" == (>d)
ns: say 'nostring' condition('d') condition('o')~additional~class~id
signal on syntax name s1
signal value (>l)
say 'not reached'
kmake: say 'signal kmake'
s1: exit
::class kmake
::method makestring; return 'kmake'
::class knum
::method makestring; return '5'
::class kcode
::method makestring; return "say 'interpreted'"
::class kexit
::method makestring; return 'exit 3'
::class ktrace
::method makestring; return 'R'
::class klabel
::method makestring; return 'KMAKE'
::class nstr
::method string; return 'nstr'
