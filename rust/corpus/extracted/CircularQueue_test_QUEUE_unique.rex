/* extracted from CircularQueue::test_QUEUE_unique */
::routine main public
   u3=.CircularQueue~of(1,2,3)      -- queue of three elements
   a=.array~of(1,2,3)               -- test sequence #1
   self~assertTrue(testSequence(u3, a))

   u3~~queue(4,'Unique')~~queue(5,'Unique')
   a=.array~of(3,4,5)               -- test sequence #1
   self~assertTrue(testSequence(u3, a))

   u3~~queue(5,'Unique')~~queue(4,'Unique')
   a=.array~of(3,5,4)               -- test sequence #1
   self~assertTrue(testSequence(u3, a))

   -- test PUSH method ---------------------------------------------
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
