b = .BufferTester~new(42)
say b~value b~value2
g = b~getBuffer
say g~class g~class~id g~objectName
say g~isA(.object) g~string g~hashcode~length
say .t~new~isb(g) .t~new~isb('x')
::class T
::method isb external "LIBRARY orxmethod TestIsBuffer"
::class BufferTester PUBLIC
::method Init EXTERNAL "LIBRARY orxmethod TestBufferInit"
::method value EXTERNAL "LIBRARY orxmethod TestBufferCSelf"
::method value2 EXTERNAL "LIBRARY orxmethod TestBufferObjectToCSelf"
::method getBuffer EXTERNAL "LIBRARY orxmethod TestGetBuffer"
