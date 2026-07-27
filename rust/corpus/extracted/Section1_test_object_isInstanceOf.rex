/* extracted from Section1::test_object_isInstanceOf */
::routine main public

    obj = "abc"
    self~assertTrue(obj~isInstanceOf(.String), "abc should be instance of .String")
    self~assertTrue(obj~isInstanceOf(.Object), "abc should be instance of .Object")
    self~assertFalse(obj~isInstanceOf(.MutableBuffer), "abc should not be instance of .MutableBuffer")

    -- Note that section 5.1.1.12 is for the isA method which is declared to be
    -- an alias for isInstanceOf.
    self~assertTrue(obj~isA(.String), "abc should be instance of .String")
    self~assertTrue(obj~isA(.Object), "abc should be instance of .Object")
    self~assertFalse(obj~isA(.MutableBuffer), "abc should not be instance of .MutableBuffer")

  -- End test_object_isInstanceOf( )

  /* test_object_ObjectNameEquals( ) - - - - - - - - - - - - - - - - - - - - - - -*\

    Test the example from 5.1.1.15 (currently.)  This is the example for the
    Object class, the objectName= method of the Object class.  The following is
    the current example:

    points=.array~of("N","S","E","W")
    say points~objectName            /* (no change yet) Says: "an Array" */
    points~objectName=("compass")    /* Changes obj name POINTS to "compass"*/
    say points~objectName            /* Shows new obj name. Says: "compass" */
    say points~defaultName           /* Default is still available. */
                                     /* Says "an Array" */
    say points                       /* Says string representation of */
                                     /* points "compass" */
    say points[3]                    /* Says: "E" Points is still an array */
                                     /* of 4 items */

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
