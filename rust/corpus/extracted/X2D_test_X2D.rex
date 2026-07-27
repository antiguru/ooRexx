/* extracted from X2D::test_X2D */
::routine main public
    self~assertEquals(14, X2D('0E'))
    self~assertEquals(129, X2D('81'))
    self~assertEquals(3969, X2D('F81'))
    self~assertEquals(65409, X2D('FF81'))
    self~assertEquals(240 /* ASCII */, X2D('46 30'X))
    self~assertEquals(240 /* ASCII */, X2D('66 30'X))

    self~assertEquals(-127, X2D('81',2)) --
    self~assertEquals(129, X2D('81',4))
    self~assertEquals(-3967, X2D('F081',4))
    self~assertEquals(129, X2D('F081',3))
    self~assertEquals(-127, X2D('F081',2))
    self~assertEquals(1, X2D('F081',1))
    self~assertEquals(0, X2D('0031',0))

   -- new tests
    self~assertEquals(0, X2D(''))
    self~assertEquals(0, X2D('',1))
    self~assertEquals(0, X2D('',17))

-- tests for [bugs:#1424] X2D() returns incorrect result
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
