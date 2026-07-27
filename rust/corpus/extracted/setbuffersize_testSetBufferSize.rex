/* extracted from setbuffersize::testSetBufferSize */
::routine main public
   b = .mutableBuffer~new("1234567890")
   b~setBufferSize(5)
   self~assertEquals("12345", b~string, "Buffer not truncated")
   self~assertEquals(5, b~getBufferSize, "Buffer returning wrong buffer size")

   b = .mutablebuffer~new("123")
   b~setBufferSize(0)
   self~assertEquals("", b~string, "MutableBuffer string should be empty string")
   self~assertEquals(256, b~getBufferSize, "MutableBuffer buffer size should be 256 after setting to 0")

   b = .mutablebuffer~new("My local work")
   b~setBufferSize(109873451)
   self~assertEquals("My local work", b~string, "Set buffer to large size, string should not change")
   self~assertEquals(109873451, b~getBufferSize, "Buffer size should be 109873451 after setting to 109873451")

   /* This test works (as it should.)  Deferring its inclusion while I think
    * about a mechanism to exclude some tests at run time.  MarkM

   b~setBufferSize(999999999)
   self~assertEquals("My local work", b~string, "Set buffer to large size, string should not change")
   self~assertEquals(999999999, b~getBufferSize, "Buffer size should be 999999999 after setting to 999999999")
   */

   /* This test does not work.  In StringClassUtil.cpp the get_length() function
    * uses DEFAULT_DIGITS instead of the current numeric digits:
    *
    *   value = REQUIRED_LONG(argument, DEFAULT_DIGITS, position);
    *
    * I want to wait until after 3.2.0 is relesed to discuss it with Rick. MarkM

   numeric digits 11
   b~setBufferSize(2147483647)
   self~assertEquals("My local work", b~string, "Set buffer to vey large size, string should not change")
   self~assertEquals(2147483647, b~getBufferSize, "Buffer size should be 2147483647 after setting to 2147483647")
   */

::class shim public
::method assertEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotEquals
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method assertTrue
  use arg condition
  if \condition then do
    say "FAIL expected true actual["condition"]"
    exit 1
  end
::method assertFalse
  use arg condition
  if condition then do
    say "FAIL expected false actual["condition"]"
    exit 1
  end
::method assertNull
  use arg actual
  if actual \== .nil then do
    say "FAIL expected nil actual["actual"]"
    exit 1
  end
::method assertNotNull
  use arg actual
  if actual == .nil then do
    say "FAIL expected non-nil actual nil"
    exit 1
  end
::method assertSame
  use arg expected, actual
  if \(expected == actual) then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertNotSame
  use arg expected, actual
  if expected == actual then do
    say "FAIL not-expected["expected"] actual["actual"]"
    exit 1
  end
::method expectSyntax
  use arg code
  nop
::method assertListEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
::method assertArrayEquals
  use arg expected, actual
  if expected \== actual then do
    say "FAIL expected["expected"] actual["actual"]"
    exit 1
  end
