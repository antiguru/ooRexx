/* extracted from VERIFY::test_VERIFY */
::routine main public
    self~assertEquals(0, VERIFY('123','1234567890'))
    self~assertEquals(2, VERIFY('1Z3','1234567890'))
    self~assertEquals(1, VERIFY('AB4T','1234567890'))
    self~assertEquals(3, VERIFY('AB4T','1234567890','M'))
    self~assertEquals(1, VERIFY('AB4T','1234567890','N'))
    self~assertEquals(4, VERIFY('1P3Q4','1234567890', ,3))
    self~assertEquals(2, VERIFY('123','',N,2))  --
    self~assertEquals(3, VERIFY('ABCDE','', ,3))  --
    self~assertEquals(6, VERIFY('AB3CD5','1234567890','M',4))


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
