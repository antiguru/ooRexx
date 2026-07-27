/* extracted from Array::test_dimension_size */
::routine main public

  a=.array~new       -- no dimension
  self~assertEquals(0, a~dimension)
  self~assertEquals(0, a~size)
  self~assertEquals(0, a~items)

  a[1]="1v"          -- single dimension
  self~assertEquals(1, a~dimension)
  self~assertEquals(1, a~size)
  self~assertEquals(1, a~items)

  a=.array~new(15)
  self~assertEquals(1, a~dimension)
  self~assertEquals(15, a~size)
  self~assertEquals(0, a~items)

  a=.array~new(1, 2, 3) -- multiple dimensions
  self~assertEquals(3, a~dimension)
  self~assertEquals(1*2*3, a~size)
  self~assertEquals(0, a~items)

  a[2,2,3]="1v"
  self~assertEquals(1, a~items)


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
