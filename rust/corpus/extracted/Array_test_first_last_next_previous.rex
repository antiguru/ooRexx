/* extracted from Array::test_first_last_next_previous */
::routine main public

  a=.array~new       -- no dimension
  self~assertNull(a~first)
  self~assertNull(a~last)
  self~assertNull(a~firstItem)
  self~assertNull(a~lastItem)
  self~assertNull(a~next(1))
  self~assertNull(a~previous(1))

  a[1]="1v"          -- single dimension
  self~assertEquals(1, a~first)
  self~assertEquals(1, a~last)
  self~assertEquals("1v", a~firstItem)
  self~assertEquals("1v", a~lastItem)
  self~assertNull(a~next(1))
  self~assertNull(a~previous(1))


  a[15]="2v"
  self~assertEquals(1, a~first)
  self~assertEquals("1v", a~firstItem)
  self~assertEquals(15, a~last)
  self~assertEquals("2v", a~lastItem)
  self~assertEquals(15, a~next(1))
  self~assertEquals(15, a~next(.array~of(2)))
  self~assertEquals(1, a~previous(15))
  self~assertEquals(1, a~previous(2))


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
