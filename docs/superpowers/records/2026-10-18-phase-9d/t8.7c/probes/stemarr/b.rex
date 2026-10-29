t. = .array~of('one','two')
say t.~objectName
say t.~defaultName
say t.~string
s. = 'dflt'
say s.~objectName s.~defaultName s.~string
v. = .nstr~new
say v.~objectName '|' v.~defaultName '|' v.~string
w.1 = 1
say w.~objectName '|' w.~defaultName '|' w.~string
u = .ustem~new
u[] = .array~of('zz')
say u~string
trace i
x = t.
x = u
x = s.
x = v.
::class ustem subclass stem
::class nstr
::method string; return 'NSTR'
