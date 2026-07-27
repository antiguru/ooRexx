/* extracted from X2D::test_x2d_bug1424 */
::routine main public
  self~assertSame(     0, x2d(8001, 0))
  self~assertSame(     1, x2d(8001, 1))
  self~assertSame(     1, x2d(8001, 2))
  self~assertSame(     1, x2d(8001, 3))
  self~assertSame(-32767, x2d(8001, 4))
  self~assertSame( 32769, x2d(8001, 5))
  self~assertSame( 32769, x2d(8001, 6))
  self~assertSame( 32769, x2d(8001, 7))
  self~assertSame( 32769, x2d(8001, 8))
  self~assertSame( 32769, x2d(8001, 9))

  self~assertSame(    0, x2d("ffff", 0))
  self~assertSame(   -1, x2d("ffff", 1))
  self~assertSame(   -1, x2d("ffff", 2))
  self~assertSame(   -1, x2d("ffff", 3))
  self~assertSame(   -1, x2d("ffff", 4))
  self~assertSame(65535, x2d("ffff", 5))
  self~assertSame(65535, x2d("ffff", 6))
  self~assertSame(65535, x2d("ffff", 7))
  self~assertSame(65535, x2d("ffff", 8))
  self~assertSame(65535, x2d("ffff", 9))

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
