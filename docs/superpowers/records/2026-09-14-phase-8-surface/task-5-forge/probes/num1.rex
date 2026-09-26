t = .T~new
say 'w2o' t~w2o(12) t~w2o(-5) t~w2o('999999999999999999') t~w2o('-999999999999999999')
say 'w2oA' t~w2oA(7)
say 's2o' t~s2o(0) t~s2o('999999999999999999') t~s2oA(3)
say 'i642o' t~i642o('-9223372036854775808') t~i642o('9223372036854775807') t~i642oA(1)
say 'u642o' t~u642o('18446744073709551615') t~u642oA(1)
say 'i322o' t~i322o('-2147483648') t~i322oA(5) 'u322o' t~u322o('4294967295') t~u322oA(5)
say 'ip2o' t~ip2o('-9223372036854775808') t~ip2oA(2) 'up2o' t~up2o('18446744073709551615') t~up2o('999999999999999999') t~up2oA(2)
say 'l2o' t~l2o(1) t~l2o(0) t~l2oA(1)
say 'd2o' t~d2o(1.5) t~d2o(1/3) t~d2oA(2/3) t~d2op(1/3, 4) t~d2op(1/3, 30)
numeric digits 5
say 'd2o5' t~d2o(1/3) t~d2oA('123456789')
numeric digits 20
say 'd2o20' t~d2o(1/3) t~d2oA('123456789')
say (t~w2o(12))~class t~i642o(2**62)~class
::class T
::method w2o external "LIBRARY orxmethod TestWholeNumberToObject"
::method w2oA external "LIBRARY orxmethod TestWholeNumberToObjectAlt"
::method s2o external "LIBRARY orxmethod TestStringSizeToObject"
::method s2oA external "LIBRARY orxmethod TestStringSizeToObjectAlt"
::method i642o external "LIBRARY orxmethod TestInt64ToObject"
::method i642oA external "LIBRARY orxmethod TestInt64ToObjectAlt"
::method u642o external "LIBRARY orxmethod TestUnsignedInt64ToObject"
::method u642oA external "LIBRARY orxmethod TestUnsignedInt64ToObjectAlt"
::method i322o external "LIBRARY orxmethod TestInt32ToObject"
::method i322oA external "LIBRARY orxmethod TestInt32ToObjectAlt"
::method u322o external "LIBRARY orxmethod TestUnsignedInt32ToObject"
::method u322oA external "LIBRARY orxmethod TestUnsignedInt32ToObjectAlt"
::method ip2o external "LIBRARY orxmethod TestIntptrToObject"
::method ip2oA external "LIBRARY orxmethod TestIntptrToObjectAlt"
::method up2o external "LIBRARY orxmethod TestUintptrToObject"
::method up2oA external "LIBRARY orxmethod TestUintptrToObjectAlt"
::method l2o external "LIBRARY orxmethod TestLogicalToObject"
::method l2oA external "LIBRARY orxmethod TestLogicalToObjectAlt"
::method d2o external "LIBRARY orxmethod TestDoubleToObject"
::method d2oA external "LIBRARY orxmethod TestDoubleToObjectAlt"
::method d2op external "LIBRARY orxmethod TestDoubleToObjectWithPrecision"
