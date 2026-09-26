call TestSetContextVariable 'abc', 5
say abc
call TestSetContextVariable 'Low', 'x'
say low
call TestSetContextVariable 'st.', 'dflt'
say st.1 st.zz
i = 7
call TestSetContextVariable 'st.i', 'seven'
say st.7 st.i
call TestSetContextVariable 'st.i.j', 'twodim'
say st.7.j
say TestGetContextVariable('abc') TestGetContextVariable('ABC') TestGetContextVariable('st.i') TestGetContextVariable('st.7')
s = TestGetContextVariable('st.'); say s~class s[7]
signal on syntax name s1
say TestGetContextVariable('unsetvar')
s1: say 'unset' condition('o')~code
signal on syntax name s2
say TestGetContextVariable('1abc')
s2: say 'bad' condition('o')~code
call TestSetContextVariable '1abc', 5
call TestSetContextVariable 'a b', 5
call TestDropContextVariable 'abc'
say abc
call TestDropContextVariable 'st.i'
say st.7 st.1
call TestDropContextVariable 'st.'
say st.1
d = TestGetAllContextVariables()
say d~class
say d['I'] d['LOW'] d['ST.']~class d['ABC'] d~items
x. = 'q'; x.1 = 'r'
say TestResolveStemVariable('x.')[1] TestResolveStemVariable('x')[2] TestResolveStemVariable(x.)[1]
signal on syntax name s3
say TestResolveStemVariable('1bad')
s3: say 'resolve' condition('o')~code
say TestFindContextClass('array') TestFindContextClass('ARRAY') TestFindContextClass('foo')~class
exit
::requires 'orxfunction' LIBRARY
