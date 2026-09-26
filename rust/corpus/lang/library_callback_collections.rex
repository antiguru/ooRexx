/* The thread table's array, directory, string table, stem and supplier
   members through orxmethod's methods: an index of zero raising, an empty
   slot or a missing index answering no object, and a stem's tail taken as
   written. */
t = .T~new
a = .array~of('a', 'b', , 'd')
say t~at(a, 1) t~at(a, 4) t~size(a) t~items(a) t~dim(a) t~dim(.array~new(2,3)) t~dim(.array~new) t~dim(.array~of(1))
say t~put(a, 'z', 7) a~size a[7] t~append(a, 'q') a~size t~appendstr(a, 'strq') a[9]
n = t~new(5); say n~class n~size n~items n~dimension
n0 = t~new(0); say n0~size n0~items
say t~of1('x')~items t~of2('x', 'y')~makestring('l', ',') t~of3(1,2,3)~makestring('l', ',') t~of4(1,2,3,4)~size
say t~of1A('x')~items t~of2A('x', 'y')~size t~of3A(1,2,3)~size t~of4A(1,2,3,4)~makestring('l', '-')
say t~isa(a) t~isa('x') t~isa(.list~new) t~isa(.array~new(2,2))
signal on syntax name s1
say t~at(a, 3)
s1: say 'at3' condition('o')~code condition('o')~message
signal on syntax name s2
say t~at(a, 99)
s2: say 'at99' condition('o')~code
signal on syntax name s3
say t~put(a, 'z', 0)
s3: say 'put0' condition('o')~code condition('o')~message
signal on syntax name s4
say t~at(a, 0)
s4: say 'at0' condition('o')~code condition('o')~message
d = t~newdir; say d~class d~items
say t~dput(d, 'v', 'k') d['k'] t~dat(d, 'k') t~isdir(d) t~isdir(.stringtable~new) t~isdir(.table~new)
signal on syntax name s5
say t~dat(d, 'missing')
s5: say 'datmiss' condition('o')~code
say t~drem(d, 'k') d~items
st = t~newst; say st~class
say t~stput(st, 'v2', 'k2') st['k2'] t~stat(st, 'k2') t~isst(st) t~isst(d) t~strem(st, 'k2') st~items
s = t~newstem('FRED.')
say s~class '['s~string']' '['s[]']'
say t~set(s, 'abc', 'v1') s['abc'] s['ABC'] '['t~get(s, 'abc')']'
say t~set(s, 'X.Y', 'v2') s['X.Y'] s['X', 'Y'] t~get(s, 'X.Y')
say t~seta(s, 3, 'three') s[3] t~geta(s, 3) s~items
say t~drop(s, 'abc') s~items t~dropa(s, 3) s~items
signal on syntax name s6
say t~get(s, 'abc')
s6: say 'missing' condition('o')~code
signal on syntax name s7
say t~geta(s, 99)
s7: say 'missinga' condition('o')~code
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
::method at external "LIBRARY orxmethod TestArrayAt"
::method put external "LIBRARY orxmethod TestArrayPut"
::method append external "LIBRARY orxmethod TestArrayAppend"
::method appendstr external "LIBRARY orxmethod TestArrayAppendString"
::method size external "LIBRARY orxmethod TestArraySize"
::method items external "LIBRARY orxmethod TestArrayItems"
::method dim external "LIBRARY orxmethod TestArrayDimension"
::method new external "LIBRARY orxmethod TestNewArray"
::method of1 external "LIBRARY orxmethod TestArrayOfOne"
::method of2 external "LIBRARY orxmethod TestArrayOfTwo"
::method of3 external "LIBRARY orxmethod TestArrayOfThree"
::method of4 external "LIBRARY orxmethod TestArrayOfFour"
::method of1A external "LIBRARY orxmethod TestArrayOfOneAlt"
::method of2A external "LIBRARY orxmethod TestArrayOfTwoAlt"
::method of3A external "LIBRARY orxmethod TestArrayOfThreeAlt"
::method of4A external "LIBRARY orxmethod TestArrayOfFourAlt"
::method isa external "LIBRARY orxmethod TestIsArray"
::method newdir external "LIBRARY orxmethod TestNewDirectory"
::method dput external "LIBRARY orxmethod TestDirectoryPut"
::method dat external "LIBRARY orxmethod TestDirectoryAt"
::method drem external "LIBRARY orxmethod TestDirectoryRemove"
::method isdir external "LIBRARY orxmethod TestIsDirectory"
::method newst external "LIBRARY orxmethod TestNewStringTable"
::method stput external "LIBRARY orxmethod TestStringTablePut"
::method stat external "LIBRARY orxmethod TestStringTableAt"
::method strem external "LIBRARY orxmethod TestStringTableRemove"
::method isst external "LIBRARY orxmethod TestIsStringTable"
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
