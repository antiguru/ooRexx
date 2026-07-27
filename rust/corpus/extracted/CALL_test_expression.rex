/* extracted from CALL::test_expression */
::routine main public
  -- if name is an expression, internal labels are honored
  call (""); self~assertSame("internal", result)
  call ("label"); self~assertSame("routine", result)
  call ("Label"); self~assertSame("routine", result) -- always case-insensitive @@is this documented??
  call ("LABEL"); self~assertSame("internal", result)
  call ("arg"); self~assertSame("internal", result) -- should call internal "arg"
  call ("ARG"); self~assertSame("internal", result) -- should not call ARG built-in
  call ((,)); self~assertSame("internal", result) -- evaluates to nullstring
  -- there's no function equivalent for CALL (expression)
  return

  "": label: arg: "arg": return "internal"

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
