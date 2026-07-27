/* extracted from Section1::test_object_start */
::routine main public

    world=.WorldObject~new
    self~assertTrue(world~class == .WorldObject, "world object must be a WorldObj object")

    msg1=world~start("HELLO")
    self~assertTrue(msg1~class == .Message, "msg1 must be a Message object")

    j = SysSleep(.025)

    msg2=.message~new(world,"HELLO")~~start
    self~assertTrue(msg2~class == .Message, "msg2 must be a Message object")

    -- Now test that each message result is correct.  The result should begin
    -- with Hello world.
    res1 = msg1~result
    self~assertTrue(res1~abbrev("Hello world"), "result should begin with Hello World for msg1")
    res2 = msg2~result
    self~assertTrue(res2~abbrev("Hello world"), "result should begin with Hello World for msg2")

    -- In addition the return of the hello method has the long time appended to
    -- it.  So the message result is a string, but should not be the exact same
    -- string.
    self~assertNotSame(res1, res2, "result strings should not be exactly equal")

  -- End test_object_start( )

  /* test_object_isInstanceOf( ) - - - - - - - - - - - - - - - - - - - - - - -*\

    Test the example from 5.1.1.13 (currently.)  This is the example for the
    Object class, the isInstanceOf method of the Object class.  The following is
    the current example:

    "abc"~isInstanceOf(.string) -> 1
    "abc"~isInstanceOf(.object) -> 1
    "abc"~isInstanceOf(.mutablebuffer) -> 0

  \* - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -*/
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
