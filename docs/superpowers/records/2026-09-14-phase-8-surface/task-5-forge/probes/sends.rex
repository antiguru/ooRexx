t = .T~new
say t~send('abc', 'left', .array~of(2)) t~send('abc', 'LENGTH', .array~new) t~send0('abc', 'reverse') t~send1('abc', 'pos', 'b') t~send2('abcabc', 'pos', 'c', 4)
say t~send(.array~of(1,,3), 'items', .array~new) t~send('xyz', 'substr', .array~of(2))
say t~scoped(.sub~new, 'who', .base, .array~new) t~scoped(.sub~new, 'who', .sub, .array~new) t~send0(.sub~new, 'who')
signal on syntax name s1
say t~send0('abc', 'nosuchmethod')
s1: say 'nomethod' condition('o')~code
say t~isinst('abc', .string) t~isinst(.sub~new, .base) t~isinst(.sub~new, .array) t~hasm('abc', 'left') t~hasm('abc', 'LEFT') t~hasm('abc', 'nope')
say t~args~items t~args(1,,3)~size t~args(1,,3)~items t~arg(2, 'x', 'y') t~arg(3,,'z')
signal on syntax name s2
say t~arg(2)
s2: say 'noarg' condition('o')~code
say t~msgname t~msgname2 t~getmethod~class (t~getself == t) t~getscope t~getsuper
say .sub~new~sscope .sub~new~ssuper
say t~find('array') t~find('ARRAY') t~find('t')
signal on syntax name s3
say t~find('nosuch')
s3: say 'noclass' condition('o')~code
say t~fwd('hello', 'left', .string, .array~of(2)) t~fwd(.sub~new, 'who', .base, .array~new) t~fwd(.sub~new, 'who', .sub, .array~new)
say TestGetRoutineName() TestGetArguments(1,2)~items TestGetArgument(2, 'q') TestGetRoutine()~class TestFindContextClass('T')
exit
::requires 'orxfunction' LIBRARY
::class base
::method who
  return 'base'
::class sub subclass base
::method who
  return 'sub'
::method sscope external "LIBRARY orxmethod TestGetScope"
::method ssuper external "LIBRARY orxmethod TestGetSuper"
::class T
::method send external "LIBRARY orxmethod TestSendMessage"
::method scoped external "LIBRARY orxmethod TestSendMessageScoped"
::method send0 external "LIBRARY orxmethod TestSendMessage0"
::method send1 external "LIBRARY orxmethod TestSendMessage1"
::method send2 external "LIBRARY orxmethod TestSendMessage2"
::method isinst external "LIBRARY orxmethod TestIsInstanceOf"
::method hasm external "LIBRARY orxmethod TestHasMethod"
::method args external "LIBRARY orxmethod TestGetArguments"
::method arg external "LIBRARY orxmethod TestGetArgument"
::method msgname external "LIBRARY orxmethod TestGetMessageName"
::method msgname2 external "LIBRARY orxmethod TestGetMessageName"
::method getmethod external "LIBRARY orxmethod TestGetMethod"
::method getself external "LIBRARY orxmethod TestGetSelf"
::method getscope external "LIBRARY orxmethod TestGetScope"
::method getsuper external "LIBRARY orxmethod TestGetSuper"
::method find external "LIBRARY orxmethod TestFindContextClass"
::method fwd external "LIBRARY orxmethod TestForwardMessage"
