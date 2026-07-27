/* extracted from CONDITION::test_condition_call_bif */
::routine main public
  -- we could run real tests here, because the reverse BIF would raises NOSTRING
  -- because it receives the condition object as its only argument
  -- this is for another day; we just turn NOSTRING OFF
  signal off nostring
  call on any name reverse

  expected = ("ERROR", "exit 1")
  "exit 1"                             -- ERROR, Windows/Linux

  expected = ("FAILURE", "fails")
  trace off                            -- avoid trace output from FAILURE
  address a_failure "fails"            -- FAILURE
  trace normal

  expected = ("HALT", "raised", "halt")
  call raiseHalt                       -- HALT

  expected = ("NOTREADY", "/", "/")
  call charin "/"                      -- NOTREADY

  expected = ("USER USER_CONDITION", "raised", "user")
  call raiseUser                       -- USER USER_CONDITION


-- NOVALUE tests (not really a CONDITION BIF test, but found no better place ..
-- according to rexxref "Conditions and Condition Traps" NOVALUE is raised
-- if an uninitialized variable is used as:
-- .) A term in an expression
-- .) The name following the VAR subkeyword of a PARSE instruction
-- .) A variable reference in an EXPOSE instruction, a PROCEDURE instruction, or a DROP instruction
-- .) A method selection override specifier in a message term
-- NOVALUE is not raised for any uninitialized variables in tails in compound variables.

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
