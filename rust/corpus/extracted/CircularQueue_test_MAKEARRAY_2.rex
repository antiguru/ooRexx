/* extracted from CircularQueue::test_MAKEARRAY_2 */
::routine main public
   u0=.CircularQueue~of
   self~assertEquals(0, u0~makearray~items)

   u3=.CircularQueue~of(1,2,3)
   m=u3~makearray
   self~assertEquals(3, m~items)
   a=.array~of(1,2,3)
   self~assertTrue(testSequence(m, a))
   -- self~assertTrue(testSequence(m, .array~of(1,2,3)))

   m=u3~makearray("FIFO")
   self~assertEquals(3, m~items)
   self~assertTrue(testSequence(m, .array~of(1,2,3)))

   m=u3~makearray("FIFO")
   self~assertEquals(3, m~items)
   self~assertFalse(testSequence(m, .array~of(3,2,1)))

   m=u3~makearray("LIFO")
   self~assertEquals(3, m~items)
   self~assertFalse(testSequence(m, .array~of(1,5,3)))

   m=u3~makearray("LIFO")
   self~assertEquals(3, m~items)
   self~assertTrue(testSequence(m, .array~of(3,2,1)))



   -- test SUPPLIER method ---------------------------------------------
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
