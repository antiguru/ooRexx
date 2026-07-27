/* extracted from CONDITION::test_condition_none */
::routine main public
  -- "a" and "o" should return .nil
  do option over "a", "Add", "o", "object"
    self~assertSame(.nil, condition(option), "CONDITION(" || option || ") should be .nil after a CONDITION(R)")
  end
  -- all other options should return null string
  do option over "c", "cn", "d", "D ", "e", "EX", "i", "Instr", "s", "state"
    self~assertSame("", condition(option), "CONDITION(" || option || ") should be null string after a CONDITION(R)")
  end
  self~assertSame("", condition("r"))
  self~assertSame("", condition("RESET"))

-- SIGNAL trap all possible exceptions, and check CONDITION() results
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
