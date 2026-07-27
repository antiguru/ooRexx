/* extracted from Queue::test_new */
::routine main public

    -- Prefix to my messages.
    be   = "New object should be"
    have = "New object should have"

    -- Create what should be a new instance of a queue.
    q = .queue~new

    -- Test that the object is indeed an instance of .queue.
    self~assertEquals(.queue, q~class, "subtest01:" be "instance of .queue")

    -- Test that the new object is a direct subclass of .object.
    supers = q~class~superclasses

    self~assertEquals(2, supers~items, "subtest02:"  have "2 superclasses")

    self~assertEquals(.object, supers[ 1 ], "subtest03:"  have ".object as superclass")

    -- Test that the object has all the .queue methods.
    self~assertTrue(q~hasmethod( "[]"        ), "subtest04:" have '"[]" method')
    self~assertTrue(q~hasmethod( "[]="       ), "subtest05:" have '"[]=" method')
    self~assertTrue(q~hasmethod( "AT"        ), "subtest06:" have "AT method")
    self~assertTrue(q~hasmethod( "HASINDEX"  ), "subtest07:" have "HASINDEX method")
    self~assertTrue(q~hasmethod( "ITEMS"     ), "subtest08:" have "ITEMS method")
    self~assertTrue(q~hasmethod( "MAKEARRAY" ), "subtest09:" have "MAKEARRAY method")
    self~assertTrue(q~hasmethod( "PEEK"      ), "subtest10:" have "PEEK method")
    self~assertTrue(q~hasmethod( "PULL"      ), "subtest11:" have "PULL method")
    self~assertTrue(q~hasmethod( "PUSH"      ), "subtest12:" have "PUSH method")
    self~assertTrue(q~hasmethod( "PUT"       ), "subtest13:" have "PUT method")
    self~assertTrue(q~hasmethod( "QUEUE"     ), "subtest14:" have "QUEUE method")
    self~assertTrue(q~hasmethod( "REMOVE"    ), "subtest15:" have "REMOVE method")
    self~assertTrue(q~hasmethod( "SUPPLIER"  ), "subtest16:" have "SUPPLIER method")

    -- Test that the queue is created empty.
    self~assertEquals(0, q~items, "subtest17:" be "empty")

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
