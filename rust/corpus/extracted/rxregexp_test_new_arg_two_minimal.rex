/* extracted from rxregexp::test_new_arg_two_minimal */
::routine main public
  p = .RegularExpression~new("ab+", "MINimal")
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(2, p~position) -- lazy, should have just matched "ab"

  -- keep it lazy by specifying "current")
  self~assertEquals(0, p~parse("ab+", "Current"))
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(2, p~position) -- should have just matched "ab"

  -- make it greedy by specifying "maximal")
  self~assertEquals(0, p~parse("ab+", "MAXIMAL"))
  self~assertEquals(1, p~pos("abbbbb"))
  self~assertEquals(6, p~position) -- must have matched whole string

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
