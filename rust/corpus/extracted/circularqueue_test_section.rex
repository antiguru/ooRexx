/* extracted from circularqueue::test_section */
::routine main public
  a1=.circularqueue~of(1)
  a2=a1~section(1)
  self~assertEquals(1, a2~items)
  self~assertTrue(a2~isa(.circularqueue))
  self~assertTrue(testSeq(.circularqueue~of(1), a2))

  a2=a1~section(1,1)
  self~assertEquals(1, a2~items)
  self~assertTrue(testSeq(.circularqueue~of(1), a2))

  a2=a1~section(1,5)
  self~assertEquals(1, a2~items)
  self~assertTrue(testSeq(.circularqueue~of(1), a2))

  a1=.queue~of(1,2,3,4,5)
  a2=a1~section(2)
  self~assertEquals(4, a2~items)
  self~assertTrue(testSeq(.circularqueue~of(2,3,4,5), a2))
  a2=a1~section(2,2)
  self~assertEquals(2, a2~items)
  self~assertTrue(testSeq(.circularqueue~of(2,3), a2))

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
