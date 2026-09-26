say Env(1)~class Env(0)~class (Env(1)~hasIndex("STDOUT")) Env(0)~hasIndex("ARRAY")
c = CallerCtx(); say c~class c~line c~package~name~right(10)
signal on syntax name s1
call Invalid
s1: say 'invalid' condition('o')~code condition('o')~message
say Types('abc', 'string') Types('abc', 'array') Types(.methods, 'nosuch') Types(.method~new('m', 'return 1'), 'Method') Types(.routine~new('r', 'return 1'), 'object')
say FindCls('array') FindCls('T') FindCls('nosuch')
say .W~new~go
exit
::requires 'reach' LIBRARY
::class T
::class W
::method go
  return self~target('hello', 2)
::method target
  return .FwdOne~new~fwdto(.Other~new)
::class FwdOne
::method fwdto external "LIBRARY reach FwdTo"
::class Other
::method fwdto
  use arg a
  return 'other got' a~class
