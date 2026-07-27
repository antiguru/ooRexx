/* extracted from Queue::test_newReptitively */
::routine main public

    holder = .set~new
    count  = 100
    do count
      q = .queue~new
      q~~queue( 'one' )~~queue( 'two' )~~queue( 'three' )
      holder~put( q )
    end

    self~assertEquals(count, holder~items, "Expected to create" count "unique queues")

    iterator = holder~supplier
    lastObj  = .queue~new
    i        = 0
    do while iterator~available
      obj = iterator~item
      self~assertEquals(.queue, obj~class, "Every new object should be instance of queue")
      self~assertEquals(3, obj~items, "Each new queue object should now have 3 items")

      self~assertNotSame(obj, lastObj, "Each new queue object should be unique")
      lastObj = obj

      i = i + 1
      iterator~next
    end

    -- Double check the logic of the test.
    self~assertEquals(count, i, "Expected to create" count "unique queues")


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
