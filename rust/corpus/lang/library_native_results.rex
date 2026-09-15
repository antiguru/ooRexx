/* What a native method or routine's result comes back as: a float or a double
   at nine digits whatever the caller's NUMERIC DIGITS, a logical_t, a CSTRING
   up to its first NUL, a size_t, and a pointer written as a string. */
t = .T~new
say 'float' t~float(1.1) t~float('1.5') t~float('1E30') t~float('1E40') t~float('-0') -
  t~float('1e-50') t~float('3.4028235E+38') t~float('3.4028236E+38')
say 'double' t~double(1.1) t~double('1E300') t~double('1e-5') t~double(1/3) -
  t~double('1e-400') t~double('nan') t~double('-infinity')
numeric digits 5
say 'digits 5' t~float(1.1) t~float(2/3) t~double(2/3) t~double(123456789)
numeric digits 20
say 'digits 20' t~float(1.1) t~double(2/3) t~double(123456789)
numeric digits
say 'routine' TestFloatArg(1.1) TestDoubleArg(2/3)
say 'logical' t~logical(1) t~logical(0) t~logical(.true) t~logical(.false)
say 'cstring' t~cstring('hello') '['t~cstring('')']' t~cstring(12) -
  t~cstring('ab'||'00'x||'cd')~length
say 'rxmath cstring ['MathLoadFuncs()']['MathDropFuncs()']'
call MathLoadFuncs
say 'call result ['result']'
say 'size_t' t~version t~level
say 'pointer' t~pointerarg(t~pointervalue) t~pointervalue~class t~nullpointer~isnull -
  t~pointervalue~isnull
say 'pointerstring' t~pointerstringarg(t~pointerstringvalue) t~nullpointerstring -
  t~pointerstringarg('0x0') (t~pointerstringvalue~left(2) == '0x')
exit
::requires 'orxfunction' LIBRARY
::requires 'rxmath' LIBRARY
::class T
::method float external "LIBRARY orxmethod TestFloatArg"
::method double external "LIBRARY orxmethod TestDoubleArg"
::method logical external "LIBRARY orxmethod TestLogicalArg"
::method cstring external "LIBRARY orxmethod TestCstringArg"
::method version external "LIBRARY orxmethod TestInterpreterVersion"
::method level external "LIBRARY orxmethod TestLanguageLevel"
::method pointervalue external "LIBRARY orxmethod TestPointerValue"
::method pointerarg external "LIBRARY orxmethod TestPointerArg"
::method nullpointer external "LIBRARY orxmethod TestNullPointerValue"
::method pointerstringvalue external "LIBRARY orxmethod TestPointerStringValue"
::method pointerstringarg external "LIBRARY orxmethod TestPointerStringArg"
::method nullpointerstring external "LIBRARY orxmethod TestNullPointerStringValue"
