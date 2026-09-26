t = .T~new
say 'o2w' t~o2w(12) t~o2w('999999999999999999') t~o2w(' -7 ') t~o2w('1E3') t~o2w(3.0) t~o2wA(4)
say 'o2s' t~o2s(0) t~o2s('999999999999999999') t~o2sA('5')
say 'o2i64' t~o2i64('-9223372036854775808') t~o2i64('9223372036854775807') t~o2i64A(9)
say 'o2u64' t~o2u64('18446744073709551615') t~o2u64A(9)
say 'o2i32' t~o2i32('-2147483648') t~o2i32A(9) 'o2u32' t~o2u32('4294967295') t~o2u32A(1)
say 'o2ip' t~o2ip('-9223372036854775808') t~o2ipA(1) 'o2up' t~o2up('18446744073709551615') t~o2upA(1)
say 'o2l' t~o2l(1) t~o2l(0) t~o2l(.true) t~o2lA('1')
say 'o2d' t~o2d(1.5) t~o2d('1e-5') t~o2d('nan') t~o2dA(2)
say 'str' t~nsa('hello') t~asciizalt('x') t~ns('abcdef', 3) t~newalt('abc', 0) '['t~c2o('')']'
say 'o2v' t~o2v(12, 12) t~o2v('abc', 11) t~o2v('abc', 15) t~o2v(2.5, 14) t~o2v(1, 30) t~o2v('0x10', 35)
numeric digits 20
say 'o2v20' t~o2v(1/3, 14) t~o2v(1/3, 19)
::class T
::method o2w external "LIBRARY orxmethod TestObjectToWholeNumber"
::method o2wA external "LIBRARY orxmethod TestObjectToWholeNumberAlt"
::method o2s external "LIBRARY orxmethod TestObjectToStringSize"
::method o2sA external "LIBRARY orxmethod TestObjectToStringSizeAlt"
::method o2i64 external "LIBRARY orxmethod TestObjectToInt64"
::method o2i64A external "LIBRARY orxmethod TestObjectToInt64Alt"
::method o2u64 external "LIBRARY orxmethod TestObjectToUnsignedInt64"
::method o2u64A external "LIBRARY orxmethod TestObjectToUnsignedInt64Alt"
::method o2i32 external "LIBRARY orxmethod TestObjectToInt32"
::method o2i32A external "LIBRARY orxmethod TestObjectToInt32Alt"
::method o2u32 external "LIBRARY orxmethod TestObjectToUnsignedInt32"
::method o2u32A external "LIBRARY orxmethod TestObjectToUnsignedInt32Alt"
::method o2ip external "LIBRARY orxmethod TestObjectToIntptr"
::method o2ipA external "LIBRARY orxmethod TestObjectToIntptrAlt"
::method o2up external "LIBRARY orxmethod TestObjectToUintptr"
::method o2upA external "LIBRARY orxmethod TestObjectToUintptrAlt"
::method o2l external "LIBRARY orxmethod TestObjectToLogical"
::method o2lA external "LIBRARY orxmethod TestObjectToLogicalAlt"
::method o2d external "LIBRARY orxmethod TestObjectToDouble"
::method o2dA external "LIBRARY orxmethod TestObjectToDoubleAlt"
::method nsa external "LIBRARY orxmethod TestNewStringFromAsciiz"
::method asciizalt external "LIBRARY orxmethod TestNewStringFromAsciizAlt"
::method ns external "LIBRARY orxmethod TestNewString"
::method newalt external "LIBRARY orxmethod TestNewStringAlt"
::method c2o external "LIBRARY orxmethod TestCStringToObject"
::method o2v external "LIBRARY orxmethod TestObjectToValue"
