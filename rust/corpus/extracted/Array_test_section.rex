/* extracted from Array::test_section */
::routine main public

  a1=.array~new
  a2=a1~section(2)
  self~assertTrue(a1~items=a2~items)

  a1=.array~of(1)
  a2=a1~section(2)
  self~assertTrue(a2~items=0)
  a2=a1~section(1)
  self~assertTrue(a2~items=1)
  a2=a1~section(.array~of(1))
  self~assertTrue(a2~items=1)
  a2=a1~section(1,0)
  self~assertTrue(a2~items=0)
  a2=a1~section(1,1)
  self~assertTrue(a2~items=1)
  a2=a1~section(1,5)
  self~assertTrue(a2~items=1)

  a1=.array~of(1,2,3,4,5)
  a2=a1~section(2)
  self~assertTrue(a2~items=4)
  self~assertTrue(testSeq(a1, a2, 2))
  a2=a1~section(2,2)
  self~assertTrue(a2~items=2)
  self~assertTrue(testSeq(a1, a2, 2))

  return


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
