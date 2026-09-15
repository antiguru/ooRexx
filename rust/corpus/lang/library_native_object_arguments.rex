/* A native argument declared as an object: any object, an array or what
   converts to one, a stem or in a routine a stem's name, an instance of a
   required class, a logical value, and a pointer or its string. What a
   refusal finds is the argument's own string value, an array's included. */
t = .T~new
call try "t~object('x')"
call try "t~object(.nil)"
call try "t~object(t)~class"
call try "t~array(.array~of(1, 2))~items"
call try "t~array('abc')~items"
call try "t~array('')~items"
call try "t~array(.MA~new)~items"
call try "t~array(.list~of(1, 2))~items"
call try "t~array(.queue~of(1, 2, 3))~items"
call try "t~array(.AS~of(4, 5))~items"
x.1 = 'a'
call try "t~array(x.)~items"
call try "t~array(.directory~new)~items"
call try "t~array(.object~new)"
call try "t~array(.array~new(2, 2))"
call try "t~classarg(.string)"
call try "t~classarg(.object~class)"
call try "t~classarg(.K)"
call try "t~classarg('abc')"
call try "t~classarg()"
call try "t~logical(' 1')"
call try "t~logical(2)"
call try "t~logical('1.0')"
call try "t~logical(.object~new)"
call try "t~logical()"
call try "t~pointer('abc')"
call try "t~pointer(.nil)"
v = t~pointerstringvalue
h = substr(v, 3)
call pointer t, 'plain', v
call pointer t, 'prefix twice', '0x0x'h
call pointer t, 'upper prefix after', '0x0X'h
call pointer t, 'blank', '0x 'h
call pointer t, 'tab', '0x'||'09'x||h
call pointer t, 'newline', '0x'||'0a'x||h
call pointer t, 'junk after', v'zz'
call pointer t, 'upper digits', '0x'translate(h)
call pointer t, 'plus', '0x+'h
call pointer t, 'zeros', '0x000'h
numeric digits 40
complement = d2x(2**64 - x2d(h))
numeric digits
call pointer t, 'minus', '0x-'complement
call pointer t, 'minus prefix', '0x-0x'complement
call pointer t, 'buffer', .mutablebuffer~new(v)
call pointer t, 'zero', '0x0'
call pointer t, 'overflow', '0x1'copies('0', 17)
call pointer t, 'upper X', '0X'h
call pointer t, 'leading blank', ' 'v
call pointer t, 'g', '0xg'
call pointer t, 'prefix twice then g', '0x0xg'
call pointer t, 'prefix twice alone', '0x0x'
call pointer t, 'minus alone', '0x-'
call pointer t, 'signs', '0x--1'
call pointer t, 'nul', '0x'||'00'x||h
call pointer t, 'number', 12
call pointer t, 'string method', .S~new(v)
call pointer t, 'array', .array~of(v)
call try "t~pointerstring('zz')"
call try "t~pointerstring(.object~new)"
call try "t~stem(x.)[1]"
call try "t~stem('x')"
call try "t~stem(.object~new)"
y.7 = 'seven'
call try "TestStemArg('y')[7]"
call try "TestStemArg('Y.')[7]"
call try "TestStemArg(y.)[7]"
drop qq.
s = TestStemArg('qq')
say 'unset stem' symbol('QQ.') s~class
s[1] = 'written'
say 'written through' qq.1
call try "TestStemArg('a.b')"
call try "TestStemArg('1x')"
call try "TestStemArg('')"
call try "TestStemArg(' y')"
call try "TestStemArg('y..')"
call try "TestStemArg('.y')"
call try "TestStemArg(.array~of(1))"
call try "RxCalcSqrt(.array~of(1, 2))"
call try "RxCalcSqrt(2, .array~of(1, 2))"
call try "t~int(.array~of(1, 2))"
exit

try:
  parse arg expression
  signal on syntax name refused
  interpret 'value =' expression
  say expression 'answered' value
  return
refused:
  c = condition('O')
  say expression c~code '|' c~message
  return

pointer: procedure
  use arg t, label, text
  signal on syntax name notpointer
  say 'pointerstring' label t~pointerstringarg(text)
  return
notpointer:
  say 'pointerstring' label condition('O')~code
  return

::requires 'orxfunction' LIBRARY
::requires 'rxmath' LIBRARY
::class K
::class MA
::method makearray
  return .array~of(7, 8, 9)
::class AS subclass Array
::class S
::method init
  expose v
  use arg v
::method string
  expose v
  return v
::class T
::method object external "LIBRARY orxmethod TestObjectArg"
::method array external "LIBRARY orxmethod TestArrayArg"
::method classarg external "LIBRARY orxmethod TestClassArg"
::method logical external "LIBRARY orxmethod TestLogicalArg"
::method pointer external "LIBRARY orxmethod TestPointerArg"
::method pointerstringvalue external "LIBRARY orxmethod TestPointerStringValue"
::method pointerstringarg external "LIBRARY orxmethod TestPointerStringArg"
::method pointerstring external "LIBRARY orxmethod TestPointerStringArg"
::method stem external "LIBRARY orxmethod TestStemArg"
::method int external "LIBRARY orxmethod TestIntArg"
