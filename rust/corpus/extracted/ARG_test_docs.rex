/* extracted from ARG::test_docs */
::routine main public
   -- test the rexref documentation examples
  call name1               -- call procedure without arguments
  call name2 'a', , 'b'    -- call procedure with arguments

  exit

  name1:
    self~assertSame(0, arg())
    self~assertSame("", arg(1))
    self~assertSame("", arg(2))
    self~assertSame(.false, arg(1, 'e'))
    self~assertSame(.true, arg(1, 'o'))
    self~assertSame(0, arg(1, 'a')~items)
    return

  name2:
    self~assertSame(3, arg())
    self~assertSame("a", arg(1))
    self~assertSame("", arg(2))
    self~assertSame("b", arg(3))
    self~assertSame("", arg(4))
    self~assertSame(.true, arg(1, 'e'))
    self~assertSame(.false, arg(2, 'e'))
    self~assertSame(.true, arg(3, 'e'))
    self~assertSame(.false, arg(1, 'o'))
    self~assertSame(.true, arg(2, 'o'))
    self~assertSame(.false, arg(3, 'o'))
    return

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
