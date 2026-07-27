/* extracted from Section1::test_object_ObjectNameEquals */
::routine main public

    points=.array~of("N","S","E","W")
    self~assertTrue(points~class == .Array, "points must be an array")

    -- Need to change the example slightly to test
    self~assertSame(points~objectName, "an Array", "objectName should still be an Array")

    points~objectName=("compass")
    self~assertSame(points~objectName, "compass", "objectName should now be compass")
    self~assertSame(points~defaultName, "an Array", "default name should still be an Array")

    -- say points
    --
    -- A hard thing to translate into a test case.  points is an object,
    -- compass is an object.  The following commented out assertion fails
    -- because the two objects are *not* the same.
    /*
    self~assertSame(points, "compass", "string representation should be compass")
    */

    -- So, a little creativity.  We need a way to capture a say statement, but
    -- still keep the test automated.  So, temporarily change the destination of
    -- a "say" to a file.  Then read back the file.
    fName = "ttXXTemp.out"
    outObj = .stream~new(fName)~~command("OPEN WRITE REPLACE")
    .output~destination(outObj)

    say points

    -- Reset the destination for output
    .output~destination(.Stdout)

    -- Close the file, reopen, read the output, delete the file, do the test
    outObj~close
    inObj = .stream~new(fName)~~command("OPEN READ")
    pointsStr = inObj~linein
    inObj~close
    j = SysFileDelete(fName)

    self~assertSame(points[3], "E", "points is still an array, should be 'E'")

  -- End test_object_ObjectNameEquals( )

-- End of class: Chapter5Section1.testGroup


-- Helper class for the test_object_start test case.
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
