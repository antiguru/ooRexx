/* extracted from Assignments::test_+= */
::routine main public

  i1=0
  i2=0
  do i=-5 to +5 by 0.1
     i1 =i1+1.01+i
     i2+=   1.01+i
  end

  -- Note this, the arguments to the assertEquals method are:
  --
  -- self~assertEquals(expected, actual <message>)
  --
  -- Message is optional and should *only* be used if it is a message of real
  -- value.  DO NOT use messages like "subtest__01" This adds absolutely no
  -- value.  If the assert fails, the file name, the class name, the method
  -- name, and the line number of the failure are all recorded.  This is more
  -- than enough information to quickly locate *exactly* where the failure is.
  self~assertEquals(i1, i2)

  i3=0
  i3+=10
  self~assertEquals(10, i3)

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
