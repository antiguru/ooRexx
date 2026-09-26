t = .T~new
say t~o2s('abc') t~o2s(12) t~o2s(1/3) '['t~o2s(.array~of(1,2))']' t~o2s(.object~new) t~o2s(.nil)
say t~o2c('abc') t~o2c(1/3) t~o2c(.object~new)
say t~len('abcd') t~len('') '['t~data('xyz')']' '['t~data('')']'
say t~get('abcdef', 2, 3) t~get('abcdef', 5, 2) t~get('abcdef', 1, 6)
say t~up('abc') t~up('ABC') t~up('a1b') t~low('ABC') t~low('abc') t~low('A.b')
s = 'ABC'; say (t~up(s) == s) (t~low(s) == s)
m = t~nmb(10)
say m~class m~length t~mbl(m) t~mbc(m)
say t~smbl(m, 4) t~mbl(m) '['m~string~c2x']'
say t~smbl(m, 20) t~mbl(m)
say t~smbc(m, 30) t~mbc(m) t~smbc(m, 5) t~mbc(m)
m2 = t~setv(m, 'hello there')
say (m2 == m) m~string t~mbl(m) t~mbc(m) t~getv(m)
m3 = .mutablebuffer~new('xyz', 2)
say t~getv(m3) t~mbl(m3) t~mbc(m3) t~setv(m3, 'a longer value than two')~string t~mbc(m3)
say t~ismb(m) t~ismb('x') t~ismb(.mutablebuffer~new)
::class T
::method o2s external "LIBRARY orxmethod TestObjectToString"
::method o2c external "LIBRARY orxmethod TestObjectToCString"
::method len external "LIBRARY orxmethod TestStringLength"
::method data external "LIBRARY orxmethod TestStringData"
::method get external "LIBRARY orxmethod TestStringGet"
::method up external "LIBRARY orxmethod TestStringUpper"
::method low external "LIBRARY orxmethod TestStringLower"
::method nmb external "LIBRARY orxmethod TestNewMutableBuffer"
::method ismb external "LIBRARY orxmethod TestIsMutableBuffer"
::method mbl external "LIBRARY orxmethod TestMutableBufferLength"
::method smbl external "LIBRARY orxmethod TestSetMutableBufferLength"
::method mbc external "LIBRARY orxmethod TestMutableBufferCapacity"
::method smbc external "LIBRARY orxmethod TestSetMutableBufferCapacity"
::method setv external "LIBRARY orxmethod TestSetMutableBufferValue"
::method getv external "LIBRARY orxmethod TestGetMutableBufferValue"
