/* extracted from Array::test_MULTIDIMENSIONAL_first_last_next_previous */
::routine main public
  a=.array~new(3, 3, 3) -- multiple dimensions
  f=.array~of(1,1,1) -- first
  m=.array~of(2,2,2) -- middle
  l=.array~of(3,3,3) -- last
  self~assertNull(a~first)
  self~assertNull(a~last)
  self~assertNull(a~firstItem)
  self~assertNull(a~lastItem)
  self~assertNull(a~next(1,1,1))
  self~assertNull(a~next(2,2,2))
  self~assertNull(a~next(3,3,3))
  self~assertNull(a~previous(1,1,1))
  self~assertNull(a~previous(2,2,2))
  self~assertNull(a~previous(3,3,3))
  self~assertNull(a~next(f))
  self~assertNull(a~next(m))
  self~assertNull(a~previous(l))
  self~assertNull(a~previous(m))

  a[f]="1v"
  a[l]="3v"
  self~assertEquals(f, a~first)
  self~assertEquals("1v", a~firstItem)
  self~assertEquals(l, a~last)
  self~assertEquals("3v", a~lastItem)
  self~assertEquals(l, a~next(1,1,1))
  self~assertEquals(l, a~next(2,2,2))
  self~assertEquals(f, a~previous(2,2,2))
  self~assertEquals(f, a~previous(3,3,3))
  self~assertEquals(l, a~next(f))
  self~assertEquals(l, a~next(m))
  self~assertEquals(f, a~previous(l))
  self~assertEquals(f, a~previous(m))

  a[m]="2v"
  self~assertEquals(f, a~first)
  self~assertEquals("1v", a~firstItem)
  self~assertEquals(l, a~last)
  self~assertEquals("3v", a~lastItem)
  self~assertEquals(m, a~next(1,1,1))
  self~assertEquals(l, a~next(2,2,2))
  self~assertEquals(f, a~previous(2,2,2))
  self~assertEquals(m, a~previous(3,3,3))
  self~assertEquals(m, a~next(f))
  self~assertEquals(l, a~next(m))
  self~assertEquals(m, a~previous(l))
  self~assertEquals(f, a~previous(m))


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
