say BufStr('hello')
say BufStr('')
say BufLen(8) BufLen(0)
say PtrValue(.t~new~pv) PtrValue('x') PtrValue(.t~new~np)
b = .BT~new(42)
say b~cs b~cs2
say ScopedRead(b, .BT) ScopedRead(b, .object) ScopedRead(.sub~new(7), .sub) ScopedRead(.sub~new(8), .BT)
say .sub~new(9)~cs
exit
::requires 'reach' LIBRARY
::class t
::method pv external "LIBRARY orxmethod TestPointerValue"
::method np external "LIBRARY orxmethod TestNullPointerValue"
::class BT
::method init external "LIBRARY orxmethod TestBufferInit"
::method cs external "LIBRARY reach CSelfRead"
::method cs2 external "LIBRARY orxmethod TestBufferCSelf"
::class sub subclass BT
