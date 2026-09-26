o = .V~new
say o~setv('abc', 'x1') o~getv('abc') o~getv('ABC') o~getornil('abc') o~getornil('zz')
signal on syntax name s1
say o~getv('zz')
s1: say 'unset' condition('o')~code
say o~show
r = o~ref('abc'); say r~class r~name r~value o~isref(r) o~isref('x') o~refname(r) o~refvalue(r)
o~setref(r, 'newval'); say o~show o~refvalue(r)
o~ref('1bad'); say var('result'); o~ref('a.b'); say var('result')
o~guardon('g1', 'v1'); o~guardoff('g2', 'v2'); say o~show2
b = .BT~new(3)
say b~getbuffer~class
x = 5
r2 = .W~new~ctxref('x')
say r2~class r2~name r2~value
exit
::requires 'reach' LIBRARY
::class V
::method setv external "LIBRARY orxmethod TestSetObjectVariable"
::method getv external "LIBRARY orxmethod TestGetObjectVariable"
::method getornil external "LIBRARY orxmethod TestGetObjectVariableOrNil"
::method ref external "LIBRARY orxmethod TestGetObjectVariableReference"
::method isref external "LIBRARY orxmethod TestIsVariableReference"
::method setref external "LIBRARY orxmethod TestSetVariableReferenceValue"
::method refname external "LIBRARY orxmethod TestVariableReferenceName"
::method refvalue external "LIBRARY orxmethod TestVariableReferenceValue"
::method guardon external "LIBRARY orxmethod TestSetGuardOn"
::method guardoff external "LIBRARY orxmethod TestSetGuardOff"
::method guard external "LIBRARY orxmethod TestSetGuard"
::method show
  expose abc
  return abc
::method show2
  expose g1 g2
  return g1 g2
::method show3
  expose g3 g4
  return g3 g4
::class BT
::method init external "LIBRARY orxmethod TestBufferInit"
::method getbuffer external "LIBRARY orxmethod TestGetBuffer"
::class W
::method ctxref
  use arg n
  return CtxRef(n)
