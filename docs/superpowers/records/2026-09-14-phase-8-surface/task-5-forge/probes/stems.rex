t = .T~new
s = t~newstem('FRED.')
say s~class '['s~string']' '['s[]']'
say t~set(s, 'abc', 'v1') s['abc'] s['ABC'] '['t~get(s, 'abc')']'
say t~set(s, 'X.Y', 'v2') s['X.Y'] s['X', 'Y'] t~get(s, 'X.Y')
say t~seta(s, 3, 'three') s[3] t~geta(s, 3) s~items
say t~drop(s, 'abc') s~items t~dropa(s, 3) s~items
signal on syntax name s1
say t~get(s, 'abc')
s1: say 'missing' condition('o')~code
signal on syntax name s2
say t~geta(s, 99)
s2: say 'missinga' condition('o')~code
s['k1'] = 'a'; s['k2'] = 'b'
d = t~all(s); say d~class d~items d['k1'] d['k2']
say '['t~val(s)']'
s2 = .stem~new('X'); s2[] = 'deflt'; say t~val(s2)
say t~isstem(s) t~isstem('x') t~isstem(.directory~new)
st. = 7; say t~isstem(st.)
su = t~newsup(.array~of('a','b'), .array~of(1,2))
say su~class t~avail(su) t~item(su) t~index(su) t~next(su) t~item(su) t~index(su) t~next(su) t~avail(su)
exit
::class T
::method newstem external "LIBRARY orxmethod TestNewStem"
::method set external "LIBRARY orxmethod TestSetStemElement"
::method get external "LIBRARY orxmethod TestGetStemElement"
::method drop external "LIBRARY orxmethod TestDropStemElement"
::method seta external "LIBRARY orxmethod TestSetStemArrayElement"
::method geta external "LIBRARY orxmethod TestGetStemArrayElement"
::method dropa external "LIBRARY orxmethod TestDropStemArrayElement"
::method all external "LIBRARY orxmethod TestGetAllStemElements"
::method val external "LIBRARY orxmethod TestGetStemValue"
::method isstem external "LIBRARY orxmethod TestIsStem"
::method newsup external "LIBRARY orxmethod TestNewSupplier"
::method avail external "LIBRARY orxmethod TestSupplierAvailable"
::method item external "LIBRARY orxmethod TestSupplierItem"
::method index external "LIBRARY orxmethod TestSupplierIndex"
::method next external "LIBRARY orxmethod TestSupplierNext"
