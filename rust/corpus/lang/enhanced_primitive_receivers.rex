/* Class~enhanced on primitive, abstract and metaclass receivers: NEW sent
   to the dummy subclass takes the arguments after the methods. */
d = .stringtable~new; d["GO"] = "return self~items"
x = .array~enhanced(d, 3)
say x~go x~class~id
d = .stringtable~new
d["GO"] = "expose x; x = 3; self~setMethod('m', 'return 42', 'OBJECT'); return x self~m"
x = .string~enhanced(d, 'abc')
say x~go x~class~id
d = .directory~new
d~foo = "return 'foo' self~class~id"
a = .array~enhanced(d, 3)
say 'array:' a~items a~size a~class~id a~foo a~string
a[1] = 'x'; say 'array put:' a~items a[1]
b = a~copy; say 'copy:' b~foo b~string
s = .string~enhanced(d, 'hello')
say 'string:' s s~length s~foo s~class~id
n = .string~enhanced(d, '5')
say (n + 1) (n * 2) (n || 'x') (n~class~id)
r = n + 1
say 'derived has foo:' r~hasMethod('foo') r~class~id
say 'n:' n~string n~defaultName n~objectName datatype(n) n~datatype('W')
dd = .directory~enhanced(d)
dd~x = 1
say 'directory:' dd~x dd~items dd~foo dd~class~id dd~string
st = .stem~enhanced(d, 'dflt')
say 'stem:' st[1] st~foo st~class~id st~string
o = .object~enhanced(d)
say 'object:' o~foo o~string
say 'abstract:' .abs~enhanced(.directory~new)~string
m = .directory~new; m~dispatch = "return 'dispatched' self~id"
rc_ = .class~enhanced(m, "Remote Class")
say 'id:' rc_~id 'class:' rc_~class~id 'string:' rc_~string
say 'dispatch:' rc_~dispatch
say 'isMetaclass:' rc_~isMetaclass 'superclass:' rc_~superclass~id 'metaclass:' rc_~metaclass~id
::class abs abstract
