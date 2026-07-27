/* extracted from String::test_constant_methods */
::routine main public
   self~assertSame('00'x, .String~null)
   self~assertSame('0d'x, .String~cr)
   self~assertSame('0a'x, .String~nl)
   self~assertSame('09'x, .String~tab)

   self~assertSame(xrange("alnum"), .String~alnum)
   self~assertSame(xrange("alpha"), .String~alpha)
   self~assertSame(xrange("blank"), .String~blank)
   self~assertSame(xrange("cntrl"), .String~cntrl)
   self~assertSame(xrange("digit"), .String~digit)
   self~assertSame(xrange("graph"), .String~graph)
   self~assertSame(xrange("lower"), .String~lower)
   self~assertSame(xrange("print"), .String~print)
   self~assertSame(xrange("punct"), .String~punct)
   self~assertSame(xrange("space"), .String~space)
   self~assertSame(xrange("upper"), .String~upper)
   self~assertSame(xrange("xdigit"), .String~xdigit)


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
