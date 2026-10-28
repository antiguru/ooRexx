k = .k~new
t = .table~new
t[k] = 1
say 'second put same object'
t[k] = 2
say 'at same object'
x = t[k]
say 'other object'
t[.k~new] = 3
say t~items
::class k
::method hashCode; return 'x'
::method '=='; say '  == called'; return 1
