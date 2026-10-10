/* A STRING answer with no string value is .nil to a native method's numeric,
   stem and ObjectToString conversions, whose errors name the original object. */
o = .sn~new
t = .t~new
signal on syntax name s1
say t~int(o)
s1: say 'int' condition('O')~code condition('O')~message
signal on syntax name s2
say t~double(o)
s2: say 'double' condition('O')~code condition('O')~message
signal on syntax name s3
say t~size(o)
s3: say 'size' condition('O')~code condition('O')~message
signal on syntax name s4
say t~stem(o)
s4: say 'stem' condition('O')~code condition('O')~message
say 'tostring' t~tostring(o)
exit
::requires 'orxfunction' LIBRARY
::class t
::method int external "LIBRARY orxmethod TestIntArg"
::method double external "LIBRARY orxmethod TestDoubleArg"
::method size external "LIBRARY orxmethod TestSizeArg"
::method stem external "LIBRARY orxmethod TestStemArg"
::method tostring external "LIBRARY orxmethod TestObjectToString"
::class sn
::method string
  return .directory~new
