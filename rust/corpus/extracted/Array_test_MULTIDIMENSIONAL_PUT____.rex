/* extracted from Array::test_MULTIDIMENSIONAL_PUT_[]= */
::routine main public
  a=.array~new
  idx=.array~of(1,1,1)
  a[idx]="1v"
  self~assertEquals("1v", a[1,1,1])
  self~assertEquals("1v", a~at(1,1,1))
  self~assertEquals("1v", a~at(idx))
  self~assertEquals("1v", a~"[]"(1,1,1))
  self~assertEquals("1v", a~"[]"(idx))

  idx=.array~of(1,2,1)
  a~put("2v",idx)
  self~assertEquals("2v", a[1,2,1])
  self~assertEquals("2v", a~at(1,2,1))
  self~assertEquals("2v", a~at(idx))
  self~assertEquals("2v", a~"[]"(1,2,1))
  self~assertEquals("2v", a~"[]"(idx))

  idx=.array~of(3,1,1)
  a~"[]="("3v",idx)
  self~assertEquals("3v", a[3,1,1])
  self~assertEquals("3v", a~at(3,1,1))
  self~assertEquals("3v", a~at(idx))
  self~assertEquals("3v", a~"[]"(3,1,1))
  self~assertEquals("3v", a~"[]"(idx))



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
