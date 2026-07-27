/* extracted from CONDITION::test_condition_call_external */
::routine main public
  .external~raised = 0
  .external~self = self
  call on any name external

  .external~expected = ("ERROR", "exit 1")
  "exit 1"                             -- ERROR, Windows/Linux

  .external~expected = ("FAILURE", "fails")
  trace off                            -- avoid trace output from FAILURE
  address a_failure "fails"            -- FAILURE
  trace normal

  .external~expected = ("HALT", "raised", "halt")
  call raiseHalt                       -- HALT

  .external~expected = ("NOTREADY", "/", "/")
  call charin "/"                      -- NOTREADY

  .external~expected = ("USER USER_CONDITION", "raised", "user")
  call raiseUser                       -- USER USER_CONDITION

  self~assertSame(5, .external~raised) -- ANY should have been raised 5 times

  return


-- the CALL trap can also be a BIF, although there's really no known use case
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
