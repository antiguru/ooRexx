/* extracted from datatype::test_datatype_internal */
::routine main public
  -- numbers up to 9 digits on 32-bit resp. 18 digits on 64-bit must pass
  numeric digits .RexxInfo~internalDigits
  do digits = 1 to .RexxInfo~internalDigits
    number = 1~copies(digits)
    self~assertTrue(number~dataType("i"))
    self~assertTrue((number * 9)~dataType("i"))
    self~assertTrue((-number)~dataType("i"))
    self~assertTrue((-number * 9)~dataType("i"))
  end

  -- with a total of internalDigits + 1 digits, this number must fail
  number = 1~copies(digits)
  self~assertFalse(number~dataType("Internal whole"))
  self~assertFalse((number * 9)~dataType("Internal whole"))
  self~assertFalse((-number)~dataType("Internal whole"))
  self~assertFalse((-number * 9)~dataType("Internal whole"))


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
