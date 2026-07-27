/* extracted from overlay::testOverlay */
::routine main public
   b = .mutableBuffer~new("1234567890")
   self~AssertEquals(10, b~length, "Length incorrect before")
   b~overlay("abc")
   self~AssertEquals("abc4567890", b~string, "Overlay failed 1")
   b~overlay("abcABCabc")
   self~AssertEquals("abcABCabc0", b~string, "Overlay failed 2")
   b~overlay("abcABCabc",2)
   self~AssertEquals("aabcABCabc", b~string, "Overlay failed 3")
   b~overlay("abc",1,0,"+")
   self~AssertEquals("aabcABCabc", b~string, "Overlay failed 4")
   b~overlay("abc",12,4,"+")
   self~AssertEquals("aabcABCabc+abc+", b~string, "Overlay failed 4")
   self~AssertEquals(15, b~length, "Length incorrect after")
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
