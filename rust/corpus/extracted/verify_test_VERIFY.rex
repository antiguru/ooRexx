/* extracted from verify::test_VERIFY */
::routine main public
    self~assertEquals(0, '123'~VERIFY('1234567890'))
    self~assertEquals(2, '1Z3'~VERIFY('1234567890'))
    self~assertEquals(1, 'AB4T'~VERIFY('1234567890'))
    self~assertEquals(3, 'AB4T'~VERIFY('1234567890','M'))
    self~assertEquals(1, 'AB4T'~VERIFY('1234567890','N'))
    self~assertEquals(4, '1P3Q4'~VERIFY('1234567890', ,3))
    self~assertEquals(2, '123'~VERIFY('',N,2))  --
    self~assertEquals(3, 'ABCDE'~VERIFY('', ,3))  --
    self~assertEquals(6, 'AB3CD5'~VERIFY('1234567890','M',4))

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
