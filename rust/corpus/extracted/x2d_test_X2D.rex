/* extracted from x2d::test_X2D */
::routine main public
    self~assertEquals(14, '0E'~X2D())
    self~assertEquals(129, '81'~X2D())
    self~assertEquals(3969, 'F81'~X2D())
    self~assertEquals(65409, 'FF81'~X2D())
    self~assertEquals(240 /* ASCII */, '46 30'X~X2D())
    self~assertEquals(240 /* ASCII */, '66 30'X~X2D())

    self~assertEquals(-127, '81'~X2D(2)) --
    self~assertEquals(129, '81'~X2D(4))
    self~assertEquals(-3967, 'F081'~X2D(4))
    self~assertEquals(129, 'F081'~X2D(3))
    self~assertEquals(-127, 'F081'~X2D(2))
    self~assertEquals(1, 'F081'~X2D(1))
    self~assertEquals(0, '0031'~X2D(0))

   -- new tests
    self~assertEquals(0, ''~X2D())
    self~assertEquals(0, ''~X2D(1))
    self~assertEquals(0, ''~X2D(17))


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
