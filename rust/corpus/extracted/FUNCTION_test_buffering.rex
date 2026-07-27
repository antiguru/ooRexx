/* extracted from FUNCTION::test_buffering */
::routine main public

  temp = .TemporaryTestFile~new(self, 'testinput')
  temp~create(("Line1","","Line3","","Line5"))

  -- this tests buffering on output when the same stream is used as both
  address io 'NOBLANKOUTPUT' with input stream (temp~fullName) output stream(temp~fullName)

  -- this should read 5 lines (indicated by the rc) and write 3 lines to the file, replacing the file
  array = temp~arrayIn
  self~assertSame(5, rc)
  self~assertSame(3, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line3", array[2])
  self~assertSame("Line5", array[3])

  -- now again, with append specified
  address io 'NOBLANKOUTPUT' with input stream (temp~fullName) output append stream (temp~fullName)
  array = temp~arrayIn

  self~assertSame(3, rc)
  self~assertSame(6, array~items)
  self~assertSame("Line1", array[1])
  self~assertSame("Line3", array[2])
  self~assertSame("Line5", array[3])
  self~assertSame("Line1", array[4])
  self~assertSame("Line3", array[5])
  self~assertSame("Line5", array[6])

  temp~delete

  a.0 = 3
  a.1 = "Line1"
  a.2 = ""
  a.3 = "Line3"

  -- now same variants using a stem
  address io 'NOBLANKOUTPUT' with input stem a. output stem a.
  self~assertSame(3, rc)
  self~assertSame(2, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)

  -- same as above, with append
  address io 'NOBLANKOUTPUT' with input stem a. output append stem a.
  self~assertSame(2, rc)
  self~assertSame(4, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)
  self~assertSame("Line1", a.3)
  self~assertSame("Line3", a.4)


  -- now using a stem, but with the USING keyword
  drop a.
  a.0 = 3
  a.1 = "Line1"
  a.2 = ""
  a.3 = "Line3"

  -- now same variants using a stem
  address io 'NOBLANKOUTPUT' with input using (a.) output using (a.)
  self~assertSame(3, rc)
  self~assertSame(2, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)

  -- same as above, with append
  address io 'NOBLANKOUTPUT' with input using (a.) output append using (a.)
  self~assertSame(2, rc)
  self~assertSame(4, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)
  self~assertSame("Line1", a.3)
  self~assertSame("Line3", a.4)


  -- once more, but mixing up the variantes
  drop a.
  a.0 = 3
  a.1 = "Line1"
  a.2 = ""
  a.3 = "Line3"

  -- now same variants using a stem
  address io 'NOBLANKOUTPUT' with input using (a.) output stem a.
  self~assertSame(3, rc)
  self~assertSame(2, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)

  -- same as above, with append
  address io 'NOBLANKOUTPUT' with input stem a. output append using (a.)
  self~assertSame(2, rc)
  self~assertSame(4, a.0)
  self~assertSame("Line1", a.1)
  self~assertSame("Line3", a.2)
  self~assertSame("Line1", a.3)
  self~assertSame("Line3", a.4)

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
