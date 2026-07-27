/* extracted from Array::test_MULTIDIMENSIONAL_AT_[]_HASINDEX */
::routine main public
  a=.array~new
  a[1,1,1]="1v"
  a[1,2,1]="2v"
  a[3,1,1]="3v"

  idx=.array~of(1,1,1)
  self~assertEquals("1v", a[1,1,1])
  self~assertEquals("1v", a~at(1,1,1))
  self~assertEquals("1v", a~at(idx))
  self~assertEquals("1v", a~"[]"(1,1,1))
  self~assertEquals("1v", a~"[]"(idx))
  self~assertTrue(a~hasindex(1,1,1))
  self~assertTrue(a~hasindex(idx))

  idx=.array~of(1,2,1)
  self~assertEquals("2v", a[1,2,1])
  self~assertEquals("2v", a~at(1,2,1))
  self~assertEquals("2v", a~at(idx))
  self~assertEquals("2v", a~"[]"(1,2,1))
  self~assertEquals("2v", a~"[]"(idx))
  self~assertTrue(a~hasindex(1,2,1))
  self~assertTrue(a~hasindex(idx))

  idx=.array~of(3,1,1)
  self~assertEquals("3v", a[3,1,1])
  self~assertEquals("3v", a~at(3,1,1))
  self~assertEquals("3v", a~at(idx))
  self~assertEquals("3v", a~"[]"(3,1,1))
  self~assertEquals("3v", a~"[]"(idx))
  self~assertTrue(a~hasindex(3,1,1))
  self~assertTrue(a~hasindex(idx))

  idx=.array~of(1,2,3)
  self~assertNull(a[1,2,3])
  self~assertNull(a~at(1,2,3))
  self~assertNull(a~at(idx))
  self~assertNull(a~"[]"(1,2,3))
  self~assertNull(a~"[]"(idx))
  self~assertFalse(a~hasindex(1,2,3))
  self~assertFalse(a~hasindex(idx))



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
