/* extracted from FUNCTION::test_simple_with */
::routine main public

  address io 'ISREDIRECTIONREQUESTED' with input using "This is a test"
  self~assertSame(.true, rc)

  -- just to see if the redirect is reported
  address io 'INPUTREDIRECTED' with input using "This is a test"
  self~assertSame(.true, rc)

  address io 'OUTPUTREDIRECTED' with input using "This is a test"
  self~assertSame(.false, rc)

  address io 'ERRORREDIRECTED' with input using "This is a test"
  self~assertSame(.false, rc)

  -- this will write a single line to the output...should not crash
  address io 'INPUTOUTPUT' with input using "This is a test"
  self~assertSame(1, rc)

  address io 'INPUTERROR' with input using "This is a test"
  self~assertSame(1, rc)

  -- using an uninitialized stem variable
  address io 'INPUTOUTPUT' with input using "This is a test" output stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)

  -- default is replace
  address io 'INPUTOUTPUT' with input using "This is another test" output stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is another test", a.1)

  -- explicit replace
  address io 'INPUTOUTPUT' with input using "This is a third test" output replace stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is a third test", a.1)

  -- and finally append
  address io 'INPUTOUTPUT' with input using "This is a fourth test" output append stem a.
  self~assertSame(1, rc)
  self~assertSame(2, a.0)
  self~assertSame("This is a third test", a.1)
  self~assertSame("This is a fourth test", a.2)

  -- and finally append with an uninitialized stem
  address io 'INPUTOUTPUT' with input using "This is a fourth test" output append stem b.
  self~assertSame(1, rc)
  self~assertSame(1, b.0)
  self~assertSame("This is a fourth test", b.1)

  -- same tests using ERROR
  address io 'INPUTERROR' with input using "This is a test" error stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is a test", a.1)

  -- default is replace
  address io 'INPUTERROR' with input using "This is another test" error stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is another test", a.1)

  -- explicit replace
  address io 'INPUTERROR' with input using "This is a third test" error replace stem a.
  self~assertSame(1, rc)
  self~assertSame(1, a.0)
  self~assertSame("This is a third test", a.1)

  -- and finally append
  address io 'INPUTERROR' with input using "This is a fourth test" error append stem a.
  self~assertSame(1, rc)
  self~assertSame(2, a.0)
  self~assertSame("This is a third test", a.1)
  self~assertSame("This is a fourth test", a.2)

  in.0 = 1
  in.1 = "This is a test"

  array = .array~new

  address io 'INPUTOUTPUT' with input stem in. output using (array)

  self~assertSame(1, rc)
  self~assertSame(1, array~items)
  self~assertSame("This is a test", array[1])

  in.1 = "This is another test"

  address io 'INPUTOUTPUT' with input stem in. output append using (array)

  self~assertSame(1, rc)
  self~assertSame(2, array~items)
  self~assertSame("This is a test", array[1])
  self~assertSame("This is another test", array[2])

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
